//! `rust-sqlite3` is a rustic binding to the [sqlite3 API][].
//!
//! [sqlite3 API]: http://www.sqlite.org/c3ref/intro.html
//!
//! Three layers of API are provided:
//!
//!  - `mod ffi` provides exhaustive, though unsafe, [bindgen] bindings for `libsqlite.h`.
//!  - `mod core` provides a minimal safe interface to the basic sqlite3 API.
//!  - `mod types` provides `ToSql`/`FromSql` traits, and the library provides
//!     convenient `query()` and `update()` APIs.
//!
//! [bindgen]: https://github.com/crabtw/rust-bindgen
//!
//! The following example demonstrates opening a database, executing
//! DDL, and using the high-level `query()` and `update()` API. Note the
//! use of `Result` and `try!()` for error handling.
//!
//! ```rust
//! extern crate sgx_untrusted_time as time;
//! extern crate sqlite3;
//!
//! use time::Timespec;
//!
//! use sqlite3::{
//!     DatabaseConnection,
//!     Query,
//!     ResultRow,
//!     ResultRowAccess,
//!     SqliteResult,
//!     StatementUpdate,
//! };
//!
//! #[derive(Debug)]
//! struct Person {
//!     id: i32,
//!     name: String,
//!     time_created: Timespec,
//!     // TODO: data: Option<Vec<u8>>
//! }
//!
//! pub fn main() {
//!     match io() {
//!         Ok(ppl) => println!("Found people: {:?}", ppl),
//!         Err(oops) => panic!(oops)
//!     }
//! }
//!
//! fn io() -> SqliteResult<Vec<Person>> {
//!     let mut conn = try!(DatabaseConnection::in_memory());
//!
//!     try!(conn.exec("CREATE TABLE person (
//!                  id              SERIAL PRIMARY KEY,
//!                  name            VARCHAR NOT NULL,
//!                  time_created    TIMESTAMP NOT NULL
//!                )"));
//!
//!     let me = Person {
//!         id: 0,
//!         name: format!("Dan"),
//!         time_created: time::get_time(),
//!     };
//!     {
//!         let mut tx = try!(conn.prepare("INSERT INTO person (name, time_created)
//!                            VALUES ($1, $2)"));
//!         let changes = try!(tx.update(&[&me.name, &me.time_created]));
//!         assert_eq!(changes, 1);
//!     }
//!
//!     let mut stmt = try!(conn.prepare("SELECT id, name, time_created FROM person"));
//!
//!     let to_person = |row: &mut ResultRow| Ok(
//!         Person {
//!             id: row.get("id"),
//!             name: row.get("name"),
//!             time_created: row.get(2)
//!         });
//!     let ppl = try!(stmt.query(&[], to_person));
//!     ppl.collect()
//! }
//! ```

#![crate_name = "sqlite3"]
#![crate_type = "lib"]
#![warn(missing_docs)]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_libc as libc;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate enum_primitive;

extern crate sgx_untrusted_time as time;

use std::error::Error;
use std::fmt::Display;
use std::fmt;
use std::prelude::v1::*;

pub use coresql::Access;
pub use coresql::{DatabaseConnection, PreparedStatement, ResultSet, ResultRow};
pub use coresql::{ColIx, ParamIx};
pub use types::{FromSql, ToSql};

use self::SqliteErrorCode::SQLITE_MISUSE;

pub mod coresql;
pub mod types;

/// bindgen-bindings to libsqlite3
#[allow(non_camel_case_types, non_snake_case)]
#[allow(dead_code)]
#[allow(missing_docs)]
#[allow(missing_copy_implementations)]  // until I figure out rust-bindgen #89
#[cfg_attr(rustfmt, rustfmt_skip)]
pub mod ffi;

pub mod access;

/// Mix in `update()` convenience function.
pub trait StatementUpdate {
    /// Execute a statement after binding any parameters.
    fn update(&mut self, values: &[&ToSql]) -> SqliteResult<u64>;
}


impl StatementUpdate for coresql::PreparedStatement {
    /// Execute a statement after binding any parameters.
    ///
    /// When the statement is done, The [number of rows
    /// modified][changes] is reported.
    ///
    /// Fail with `Err(SQLITE_MISUSE)` in case the statement results
    /// in any any rows (e.g. a `SELECT` rather than `INSERT` or
    /// `UPDATE`).
    ///
    /// [changes]: http://www.sqlite.org/c3ref/changes.html
    fn update(&mut self, values: &[&ToSql]) -> SqliteResult<u64> {
        let check = {
            try!(bind_values(self, values));
            let mut results = self.execute();
            match try!(results.step()) {
                None => Ok(()),
                Some(_row) => {
                    Err(SqliteError {
                        kind: SQLITE_MISUSE,
                        desc: "unexpected SQLITE_ROW from update",
                        detail: None,
                    })
                }
            }
        };
        check.map(|_ok| self.changes())
    }
}


/// Mix in `query_each()` convenience function.
pub trait QueryEach<F>
    where F: FnMut(&mut ResultRow) -> SqliteResult<()>
{
    /// Process rows from a query after binding parameters.
    fn query_each(&mut self, values: &[&ToSql], each_row: &mut F) -> SqliteResult<()>;
}

impl<F> QueryEach<F> for coresql::PreparedStatement
    where F: FnMut(&mut ResultRow) -> SqliteResult<()>
{
    /// Process rows from a query after binding parameters.
    ///
    /// For call `each_row(row)` for each resulting step,
    /// exiting on `Err`.
    fn query_each(&mut self, values: &[&ToSql], each_row: &mut F) -> SqliteResult<()> {
        try!(bind_values(self, values));
        let mut results = self.execute();
        loop {
            match try!(results.step()) {
                None => break,
                Some(ref mut row) => try!(each_row(row)),
            }
        }
        Ok(())
    }
}


/// Mix in `query_fold()` convenience function.
pub trait QueryFold<F, A>
    where F: Fn(&mut ResultRow, A) -> SqliteResult<A>
{
    /// Fold rows from a query after binding parameters.
    fn query_fold(&mut self, values: &[&ToSql], init: A, each_row: F) -> SqliteResult<A>;
}


impl<F, A> QueryFold<F, A> for coresql::PreparedStatement
    where F: Fn(&mut ResultRow, A) -> SqliteResult<A>
{
    /// Fold rows from a query after binding parameters.
    fn query_fold(&mut self, values: &[&ToSql], init: A, f: F) -> SqliteResult<A> {
        try!(bind_values(self, values));
        let mut results = self.execute();
        let mut accum = init;
        loop {
            match try!(results.step()) {
                None => break,
                Some(ref mut row) => accum = try!(f(row, accum)),
            }
        }
        Ok(accum)
    }
}


/// Mix in `query()` convenience function.
pub trait Query<F, T>
    where F: FnMut(&mut ResultRow) -> SqliteResult<T>
{
    /// Iterate over query results after binding parameters.
    ///
    /// Each of the `values` is bound to the statement (using `to_sql`)
    /// and the statement is executed.
    ///
    /// Returns an iterator over rows transformed by `txform`,
    /// which computes a value for each row (or an error).
    fn query<'stmt>(&'stmt mut self,
                    values: &[&ToSql],
                    txform: F)
                    -> SqliteResult<QueryResults<'stmt, T, F>>;
}

impl<F, T> Query<F, T> for coresql::PreparedStatement
    where F: FnMut(&mut ResultRow) -> SqliteResult<T>
{
    fn query<'stmt>(&'stmt mut self,
                    values: &[&ToSql],
                    txform: F)
                    -> SqliteResult<QueryResults<'stmt, T, F>> {
        try!(bind_values(self, values));
        let results = self.execute();
        Ok(QueryResults {
            results: results,
            txform: txform,
        })
    }
}

/// An iterator over transformed query results
pub struct QueryResults<'stmt, T, F>
    where F: FnMut(&mut ResultRow) -> SqliteResult<T>
{
    results: coresql::ResultSet<'stmt>,
    txform: F,
}

impl<'stmt, T, F> Iterator for QueryResults<'stmt, T, F>
    where F: FnMut(&mut ResultRow) -> SqliteResult<T>
{
    type Item = SqliteResult<T>;

    fn next(&mut self) -> Option<SqliteResult<T>> {
        match self.results.step() {
            Ok(None) => None,
            Ok(Some(ref mut row)) => Some((self.txform)(row)),
            Err(e) => Some(Err(e)),
        }
    }
}


fn bind_values(s: &mut PreparedStatement, values: &[&ToSql]) -> SqliteResult<()> {
    for (ix, v) in values.iter().enumerate() {
        let p = ix as ParamIx + 1;
        try!(v.to_sql(s, p));
    }
    Ok(())
}


/// Access result columns of a row by name or numeric index.
pub trait ResultRowAccess {
    /// Get `T` type result value from `idx`th column of a row.
    ///
    /// # Panic
    ///
    /// Panics if there is no such column or value.
    fn get<I: RowIndex + Display + Clone, T: FromSql>(&mut self, idx: I) -> T;

    /// Try to get `T` type result value from `idx`th column of a row.
    fn get_opt<I: RowIndex + Display + Clone, T: FromSql>(&mut self, idx: I) -> SqliteResult<T>;
}

impl<'res, 'row> ResultRowAccess for coresql::ResultRow<'res, 'row> {
    fn get<I: RowIndex + Display + Clone, T: FromSql>(&mut self, idx: I) -> T {
        match self.get_opt(idx.clone()) {
            Ok(ok) => ok,
            Err(err) => {
                panic!("the column '{}' was not found in the result row ({})",
                       idx,
                       err)
            }
        }
    }

    fn get_opt<I: RowIndex + Display + Clone, T: FromSql>(&mut self, idx: I) -> SqliteResult<T> {
        match idx.idx(self) {
            Some(idx) => FromSql::from_sql(self, idx),
            None => {
                Err(SqliteError {
                    kind: SQLITE_MISUSE,
                    desc: "no such row name/number",
                    detail: Some(format!("{}", idx)),
                })
            }
        }
    }
}

/// A trait implemented by types that can index into columns of a row.
///
/// *inspired by sfackler's [RowIndex][]*
/// [RowIndex]: http://www.rust-ci.org/sfackler/rust-postgres/doc/postgres/trait.RowIndex.html
pub trait RowIndex {
    /// Try to convert `self` to an index into a row.
    fn idx(&self, row: &mut ResultRow) -> Option<ColIx>;
}

impl RowIndex for ColIx {
    /// Index into a row directly by uint.
    fn idx(&self, _row: &mut ResultRow) -> Option<ColIx> {
        Some(*self)
    }
}

impl RowIndex for &'static str {
    /// Index into a row by column name.
    ///
    /// *TODO: figure out how to use lifetime of row rather than
    /// `static`.*
    fn idx(&self, row: &mut ResultRow) -> Option<ColIx> {
        let mut ixs = 0..row.column_count();
        ixs.find(|ix| row.with_column_name(*ix, false, |name| name == *self))
    }
}


/// The type used for returning and propagating sqlite3 errors.
#[must_use]
pub type SqliteResult<T> = Result<T, SqliteError>;

/// Result codes for errors.
///
/// cf. [sqlite3 result codes][codes].
///
/// Note `SQLITE_OK` is not included; we use `Ok(...)` instead.
///
/// Likewise, in place of `SQLITE_ROW` and `SQLITE_DONE`, we return
/// `Some(...)` or `None` from `ResultSet::next()`.
///
/// [codes]: http://www.sqlite.org/c3ref/c_abort.html
enum_from_primitive! {
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    #[allow(non_camel_case_types)]
    #[allow(missing_docs)]
    pub enum SqliteErrorCode {
        SQLITE_ERROR     =  1,
        SQLITE_INTERNAL  =  2,
        SQLITE_PERM      =  3,
        SQLITE_ABORT     =  4,
        SQLITE_BUSY      =  5,
        SQLITE_LOCKED    =  6,
        SQLITE_NOMEM     =  7,
        SQLITE_READONLY  =  8,
        SQLITE_INTERRUPT =  9,
        SQLITE_IOERR     = 10,
        SQLITE_CORRUPT   = 11,
        SQLITE_NOTFOUND  = 12,
        SQLITE_FULL      = 13,
        SQLITE_CANTOPEN  = 14,
        SQLITE_PROTOCOL  = 15,
        SQLITE_EMPTY     = 16,
        SQLITE_SCHEMA    = 17,
        SQLITE_TOOBIG    = 18,
        SQLITE_CONSTRAINT= 19,
        SQLITE_MISMATCH  = 20,
        SQLITE_MISUSE    = 21,
        SQLITE_NOLFS     = 22,
        SQLITE_AUTH      = 23,
        SQLITE_FORMAT    = 24,
        SQLITE_RANGE     = 25,
        SQLITE_NOTADB    = 26
    }
}

/// Error results
#[derive(Debug, PartialEq, Eq)]
pub struct SqliteError {
    /// kind of error, by code
    pub kind: SqliteErrorCode,
    /// static error description
    pub desc: &'static str,
    /// dynamic detail (optional)
    pub detail: Option<String>,
}

impl Display for SqliteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.detail {
            Some(ref x) => {
                f.write_fmt(format_args!("{} [{}] (code: {})", self.desc, x, self.kind as u32))
            }
            None => f.write_fmt(format_args!("{} (code: {})", self.desc, self.kind as u32)),
        }
    }
}

impl SqliteError {
    /// Get a detailed description of the error
    pub fn detail(&self) -> Option<String> {
        self.detail.clone()
    }
}

impl Error for SqliteError {
    fn description(&self) -> &str {
        self.desc
    }
    fn cause(&self) -> Option<&Error> {
        None
    }
}


/// Fundamental Datatypes
enum_from_primitive! {
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    #[allow(non_camel_case_types)]
    #[allow(missing_docs)]
    pub enum ColumnType {
        SQLITE_INTEGER = 1,
        SQLITE_FLOAT   = 2,
        SQLITE_TEXT    = 3,
        SQLITE_BLOB    = 4,
        SQLITE_NULL    = 5
    }
}

#[cfg(test)]
mod bind_tests {
    use super::{DatabaseConnection, ResultSet};
    use super::ResultRowAccess;
    use super::SqliteResult;

    #[test]
    fn bind_fun() {
        fn go() -> SqliteResult<()> {
            let mut database = try!(DatabaseConnection::in_memory());

            try!(database.exec("
                BEGIN;
                CREATE TABLE test (id int, name text, address text);
                INSERT INTO test (id, name, address)
                VALUES (1, 'John Doe', '123 w Pine');
                COMMIT;"));

            {
                let mut tx = try!(database.prepare(
                    "INSERT INTO test (id, name, address) VALUES (?, ?, ?)"));
                assert_eq!(tx.bind_parameter_count(), 3);
                try!(tx.bind_int(1, 2));
                try!(tx.bind_text(2, "Jane Doe"));
                try!(tx.bind_text(3, "345 e Walnut"));
                let mut results = tx.execute();
                assert!(results.step().ok().unwrap().is_none());
            }
            assert_eq!(database.changes(), 1);

            let mut q = try!(database.prepare("select * from test order by id"));
            let mut rows = q.execute();
            match rows.step() {
                Ok(Some(ref mut row)) => {
                    assert_eq!(row.get::<u32, i32>(0), 1);
                    // TODO let name = q.get_text(1);
                    // assert_eq!(name.as_slice(), "John Doe");
                }
                _ => panic!(),
            }

            match rows.step() {
                Ok(Some(ref mut row)) => {
                    assert_eq!(row.get::<u32, i32>(0), 2);
                    // TODO let addr = q.get_text(2);
                    // assert_eq!(addr.as_slice(), "345 e Walnut");
                }
                _ => panic!(),
            }
            Ok(())
        }
        match go() {
            Ok(_) => (),
            Err(e) => panic!("oops! {:?}", e),
        }
    }

    fn with_query<T, F>(sql: &str, mut f: F) -> SqliteResult<T>
        where F: FnMut(&mut ResultSet) -> T
    {
        let db = try!(DatabaseConnection::in_memory());
        let mut s = try!(db.prepare(sql));
        let mut rows = s.execute();
        let x = f(&mut rows);
        return Ok(x);
    }

    #[test]
    fn named_rowindex() {
        fn go() -> SqliteResult<(u32, i32)> {
            let mut count = 0;
            let mut sum = 0i32;

            with_query("select 1 as col1
                       union all
                       select 2",
                       |rows| {
                loop {
                    match rows.step() {
                        Ok(Some(ref mut row)) => {
                            count += 1;
                            sum += row.column_int(0);
                        }
                        _ => break,
                    }
                }
                (count, sum)
            })
        }
        assert_eq!(go(), Ok((2, 3)))
    }

    #[test]
    fn err_with_detail() {
        let io = || {
            let mut conn = try!(DatabaseConnection::in_memory());
            conn.exec("CREATE gobbledygook")
        };

        let go = || match io() {
            Ok(_) => panic!(),
            Err(oops) => format!("{:?}: {}: {}", oops.kind, oops.desc, oops.detail.unwrap()),
        };

        let expected = "SQLITE_ERROR: sqlite3_exec: near \"gobbledygook\": syntax error";
        assert_eq!(go(), expected.to_string())
    }
}
