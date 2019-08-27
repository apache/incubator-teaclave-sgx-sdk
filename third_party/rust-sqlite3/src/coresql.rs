//! A minimal safe interface to sqlite3's basic API.
//!
//! The basic sqlite3 API is discussed in the [sqlite intro][intro].
//! To go beyond that, use the (unsafe) `ffi` module directly.
//!
//! [intro]: http://www.sqlite.org/cintro.html
//!
//! ```rust
//! extern crate sqlite3;
//!
//! use sqlite3::{
//!     DatabaseConnection,
//!     SqliteResult,
//! };
//!
//! fn convenience_exec() -> SqliteResult<DatabaseConnection> {
//!     let mut conn = try!(DatabaseConnection::in_memory());
//!
//!     try!(conn.exec("
//!        create table items (
//!                    id integer,
//!                    description varchar(40),
//!                    price integer
//!                    )"));
//!
//!     Ok(conn)
//! }
//!
//! fn typical_usage(conn: &mut DatabaseConnection) -> SqliteResult<String> {
//!     {
//!         let mut stmt = try!(conn.prepare(
//!             "insert into items (id, description, price)
//!            values (1, 'stuff', 10)"));
//!         let mut results = stmt.execute();
//!         match try!(results.step()) {
//!             None => (),
//!             Some(_) => panic!("row from insert?!")
//!         };
//!     }
//!     assert_eq!(conn.changes(), 1);
//!     assert_eq!(conn.last_insert_rowid(), 1);
//!     {
//!         let mut stmt = try!(conn.prepare(
//!             "select * from items"));
//!         let mut results = stmt.execute();
//!         match results.step() {
//!             Ok(Some(ref mut row1)) => {
//!                 let id = row1.column_int(0);
//!                 let desc_opt = row1.column_text(1).expect("desc_opt should be non-null");
//!                 let price = row1.column_int(2);
//!
//!                 assert_eq!(id, 1);
//!                 assert_eq!(desc_opt, format!("stuff"));
//!                 assert_eq!(price, 10);
//!
//!                 Ok(format!("row: {}, {}, {}", id, desc_opt, price))
//!             },
//!             Err(oops) => panic!(oops),
//!             Ok(None) => panic!("where did our row go?")
//!         }
//!     }
//! }
//!
//! pub fn main() {
//!     match convenience_exec() {
//!         Ok(ref mut db) => {
//!             match typical_usage(db) {
//!                 Ok(txt) => println!("item: {}", txt),
//!                 Err(oops) => {
//!                     panic!("error: {:?} msg: {}", oops,
//!                            db.errmsg())
//!                 }
//!             }
//!         },
//!         Err(oops) => panic!(oops)
//!     }
//! }
//! ```
//!
//! The `DatabaseConnection` and `PreparedStatment` structures are
//! memory-safe versions of the sqlite3 connection and prepared
//! statement structures. A `PreparedStatement` maintains mutable,
//! and hence exclusive, reference to the database connection.
//! Note the use of blocks avoid borrowing the connection more
//! than once at a time.
//!
//! In addition:
//!
//!   - `ResultSet` represents, as a rust lifetime, all of the steps
//!     of one execution of a statement. (*Ideally, it would be an
//!     Iterator over `ResultRow`s, but the `Iterator::next()`
//!     function has no lifetime parameter.*) Use of mutable
//!     references ensures that its lifetime is subsumed by the
//!     statement lifetime.  Its destructor resets the statement.
//!
//!   - `ResultRow` is a lifetime for access to the columns of one row.
//!

use enum_primitive::FromPrimitive;
use libc::{c_int, c_char};
use std::ffi as std_ffi;
use std::mem;
use std::ptr;
use std::slice;
use std::str;
use std::ffi::CStr;
use std::rc::Rc;
use std::prelude::v1::*;
use std::vec::Vec;
use time::Duration;

use self::SqliteOk::SQLITE_OK;
use self::Step::{SQLITE_ROW, SQLITE_DONE};

pub use super::{SqliteError, SqliteErrorCode, SqliteResult};

pub use super::ColumnType;
pub use super::ColumnType::SQLITE_NULL;

use ffi; // TODO: move to sqlite3-sys crate


/// Successful result
///
/// Use `SQLITE_OK as c_int` to decode return values from mod ffi.
/// See SqliteResult, SqliteError for typical return code handling.
enum_from_primitive! {
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    #[allow(non_camel_case_types)]
    #[allow(missing_docs)]
    pub enum SqliteOk {
        SQLITE_OK = 0
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Eq)]
    #[allow(non_camel_case_types)]
    // TODO: use, test this
    enum SqliteLogLevel {
        SQLITE_NOTICE    = 27,
        SQLITE_WARNING   = 28,
    }
}

struct Database {
    // not pub so that nothing outside this module
    // interferes with the lifetime
    handle: *mut ffi::sqlite3,
}
impl Drop for Database {
    /// Release resources associated with connection.
    ///
    /// # Failure
    ///
    /// Fails if "the database connection is associated with
    /// unfinalized prepared statements or unfinished sqlite3_backup
    /// objects"[1] which the Rust memory model ensures is impossible
    /// (barring bugs in the use of unsafe blocks in the implementation
    /// of this library).
    ///
    /// [1]: http://www.sqlite.org/c3ref/close.html
    fn drop(&mut self) {
        // sqlite3_close_v2 is for gced languages.
        let ok = unsafe { ffi::sqlite3_close(self.handle) };
        assert_eq!(ok, SQLITE_OK as c_int);
    }
}

/// A connection to a sqlite3 database.
pub struct DatabaseConnection {
    db: Rc<Database>,

    // whether to copy errmsg() to error detail
    detailed: bool,
}



/// Authorization to connect to database.
pub trait Access {
    /// Open a database connection.
    ///
    /// Whether or not an error occurs, allocate a handle and update
    /// db to point to it.  return `SQLITE_OK as c_int` or set the
    /// `errmsg` of the db handle and return a relevant result code.
    unsafe fn open(self, db: *mut *mut ffi::sqlite3) -> c_int;
}


// why isn't this in std::option?
fn maybe<T>(choice: bool, x: T) -> Option<T> {
    if choice { Some(x) } else { None }
}

use std::ffi::NulError;
impl From<NulError> for SqliteError {
    fn from(_: NulError) -> SqliteError {
        SqliteError {
            kind: SqliteErrorCode::SQLITE_MISUSE,
            desc: "Sql string contained an internal 0 byte",
            detail: None,
        }
    }
}

impl DatabaseConnection {
    /// Given explicit access to a database, attempt to connect to it.
    ///
    /// Note `SqliteError` code is accompanied by (copy) of `sqlite3_errmsg()`.
    pub fn new<A: Access>(access: A) -> SqliteResult<DatabaseConnection> {
        let mut db = ptr::null_mut();
        let result = unsafe { access.open(&mut db) };
        match decode_result(result, "sqlite3_open_v2", Some(db)) {
            Ok(()) => Ok(DatabaseConnection {
                db: Rc::new(Database { handle: db}),
                detailed: true,
            }),
            Err(err) => {
                // "Whether or not an error occurs when it is opened,
                // resources associated with the database connection
                // handle should be released by passing it to
                // sqlite3_close() when it is no longer required."
                unsafe { ffi::sqlite3_close(db) };

                Err(err)
            }
        }
    }

    /// Opt out of copies of error message details.
    pub fn ignore_detail(&mut self) {
        self.detailed = false;
    }


    /// Create connection to an in-memory database.
    ///
    ///  - TODO: integrate sqlite3_errmsg()
    pub fn in_memory() -> SqliteResult<DatabaseConnection> {
        struct InMemory;
        impl Access for InMemory {
            unsafe fn open(self, db: *mut *mut ffi::sqlite3) -> c_int {
                let c_memory = str_charstar(":memory:");
                ffi::sqlite3_open(c_memory.as_ptr(), db)
            }
        }
        DatabaseConnection::new(InMemory)
    }

    /// Prepare/compile an SQL statement.
    pub fn prepare(&self, sql: &str) -> SqliteResult<PreparedStatement> {
        match self.prepare_with_offset(sql) {
            Ok((cur, _)) => Ok(cur),
            Err(e) => Err(e),
        }
    }

    /// Prepare/compile an SQL statement and give offset to remaining text.
    ///
    /// *TODO: give caller a safe way to use the offset. Perhaps
    /// return a &'x str?*
    pub fn prepare_with_offset(&self, sql: &str) -> SqliteResult<(PreparedStatement, usize)> {
        let mut stmt = ptr::null_mut();
        let mut tail = ptr::null();
        let z_sql = str_charstar(sql);
        let n_byte = sql.len() as c_int;
        let r = unsafe {
            ffi::sqlite3_prepare_v2(self.db.handle, z_sql.as_ptr(), n_byte, &mut stmt, &mut tail)
        };
        match decode_result(r,
                            "sqlite3_prepare_v2",
                            maybe(self.detailed, self.db.handle)) {
            Ok(()) => {
                let ps = PreparedStatement {
                    stmt: stmt,
                    db: self.db.clone(),
                    detailed: self.detailed };
                let offset = tail as usize - z_sql.as_ptr() as usize;
                Ok((ps, offset))
            }
            Err(code) => Err(code),
        }
    }

    /// Return a copy of the latest error message.
    ///
    /// Return `""` in case of ill-formed utf-8 or null.
    ///
    /// *TODO: represent error state in types: "If a prior API call
    /// failed but the most recent API call succeeded, the return
    /// value from sqlite3_errcode() is undefined."*
    ///
    /// cf `ffi::sqlite3_errmsg`.
    pub fn errmsg(&mut self) -> String {
        DatabaseConnection::_errmsg(self.db.handle)
    }

    fn _errmsg(db: *mut ffi::sqlite3) -> String {
        let errmsg = unsafe { ffi::sqlite3_errmsg(db) };
        // returning Option<String> doesn't seem worthwhile.
        charstar_str(&(errmsg)).unwrap_or("").to_string()
    }

    /// One-Step Query Execution Interface
    ///
    /// cf [sqlite3_exec][exec]
    /// [exec]: http://www.sqlite.org/c3ref/exec.html
    ///
    ///  - TODO: callback support?
    ///  - TODO: errmsg support
    pub fn exec(&mut self, sql: &str) -> SqliteResult<()> {
        let c_sql = try!(std_ffi::CString::new(sql.as_bytes()));
        let result = unsafe {
            ffi::sqlite3_exec(self.db.handle,
                              c_sql.as_ptr(),
                              None,
                              ptr::null_mut(),
                              ptr::null_mut())
        };
        decode_result(result, "sqlite3_exec", maybe(self.detailed, self.db.handle))
    }

    /// Return the number of database rows that were changed or
    /// inserted or deleted by the most recently completed SQL
    /// statement.
    ///
    /// cf `sqlite3_changes`.
    pub fn changes(&self) -> u64 {
        let dbh = self.db.handle;
        let count = unsafe { ffi::sqlite3_changes(dbh) };
        count as u64
    }

    /// Set a busy timeout and clear any previously set handler.
    /// If duration is zero or negative, turns off busy handler.
    pub fn busy_timeout(&mut self, d: Duration) -> SqliteResult<()> {
        let ms = d.num_milliseconds() as i32;
        let result = unsafe { ffi::sqlite3_busy_timeout(self.db.handle, ms) };
        decode_result(result,
                      "sqlite3_busy_timeout",
                      maybe(self.detailed, self.db.handle))
    }

    /// Return the rowid of the most recent successful INSERT into
    /// a rowid table or virtual table.
    ///
    /// cf `sqlite3_last_insert_rowid`
    pub fn last_insert_rowid(&self) -> i64 {
        unsafe { ffi::sqlite3_last_insert_rowid(self.db.handle) }
    }

    /// Expose the underlying `sqlite3` struct pointer for use
    /// with the `ffi` module.
    pub unsafe fn expose(&mut self) -> *mut ffi::sqlite3 {
        self.db.handle
    }
}


/// Convert from sqlite3 API utf8 to rust str.
fn charstar_str(utf_bytes: &*const c_char) -> Option<&str> {
    if utf_bytes.is_null() {
        return None;
    }
    let c_str = unsafe { CStr::from_ptr(*utf_bytes) };

    Some(unsafe { str::from_utf8_unchecked(c_str.to_bytes()) })
}

/// Convenience function to get a `CString` from a str
#[inline(always)]
pub fn str_charstar(s: &str) -> std_ffi::CString {
    std_ffi::CString::new(s.as_bytes()).unwrap_or(std_ffi::CString::new("").unwrap())
}

/// A prepared statement.
pub struct PreparedStatement {
    db: Rc<Database>,
    stmt: *mut ffi::sqlite3_stmt,
    detailed: bool,
}

impl Drop for PreparedStatement {
    fn drop(&mut self) {
        unsafe {

            // We ignore the return code from finalize because:

            // "If If the most recent evaluation of statement S
            // failed, then sqlite3_finalize(S) returns the
            // appropriate error code"

            // "The sqlite3_finalize(S) routine can be called at any
            // point during the life cycle of prepared statement S"

            ffi::sqlite3_finalize(self.stmt);
        }
    }
}


/// Type for picking out a bind parameter.
/// 1-indexed
pub type ParamIx = u16;

impl PreparedStatement {
    /// Begin executing a statement.
    ///
    /// An sqlite "row" only lasts until the next call to
    /// `ffi::sqlite3_step()`, so `ResultSet` has a corresponding
    /// lifetime constraint, which prevents it `ResultSet` from
    /// implementing the `Iterator` trait. See the `Query` trait
    /// for and `Iterator` over query results.
    pub fn execute(&mut self) -> ResultSet {
        ResultSet { statement: self }
    }
}

/// A compiled prepared statement that may take parameters.
/// **Note:** "The leftmost SQL parameter has an index of 1."[1]
///
/// [1]: http://www.sqlite.org/c3ref/bind_blob.html
impl PreparedStatement {
    /// Opt out of copies of error message details.
    pub fn ignore_detail(&mut self) {
        self.detailed = false;
    }


    fn detail_db(&mut self) -> Option<*mut ffi::sqlite3> {
        if self.detailed {
            let db = unsafe { ffi::sqlite3_db_handle(self.stmt) };
            Some(db)
        } else {
            None
        }
    }

    fn get_detail(&mut self) -> Option<String> {
        self.detail_db().map(DatabaseConnection::_errmsg)
    }

    /// Bind null to a statement parameter.
    pub fn bind_null(&mut self, i: ParamIx) -> SqliteResult<()> {
        let ix = i as c_int;
        let r = unsafe { ffi::sqlite3_bind_null(self.stmt, ix) };
        decode_result(r, "sqlite3_bind_null", self.detail_db())
    }

    /// Bind an int to a statement parameter.
    pub fn bind_int(&mut self, i: ParamIx, value: i32) -> SqliteResult<()> {
        let ix = i as c_int;
        let r = unsafe { ffi::sqlite3_bind_int(self.stmt, ix, value) };
        decode_result(r, "sqlite3_bind_int", self.detail_db())
    }

    /// Bind an int64 to a statement parameter.
    pub fn bind_int64(&mut self, i: ParamIx, value: i64) -> SqliteResult<()> {
        let ix = i as c_int;
        let r = unsafe { ffi::sqlite3_bind_int64(self.stmt, ix, value) };
        decode_result(r, "sqlite3_bind_int64", self.detail_db())
    }

    /// Bind a double to a statement parameter.
    pub fn bind_double(&mut self, i: ParamIx, value: f64) -> SqliteResult<()> {
        let ix = i as c_int;
        let r = unsafe { ffi::sqlite3_bind_double(self.stmt, ix, value) };
        decode_result(r, "sqlite3_bind_double", self.detail_db())
    }

    /// Bind a (copy of a) str to a statement parameter.
    ///
    /// *TODO: support binding without copying strings, blobs*
    pub fn bind_text(&mut self, i: ParamIx, value: &str) -> SqliteResult<()> {
        let ix = i as c_int;
        // SQLITE_TRANSIENT => SQLite makes a copy
        let transient = unsafe { mem::transmute(-1 as isize) };
        let c_value = str_charstar(value);
        let len = value.len() as c_int;
        let r = unsafe { ffi::sqlite3_bind_text(self.stmt, ix, c_value.as_ptr(), len, transient) };
        decode_result(r, "sqlite3_bind_text", self.detail_db())
    }

    /// Bind a (copy of a) byte sequence to a statement parameter.
    ///
    /// *TODO: support binding without copying strings, blobs*
    pub fn bind_blob(&mut self, i: ParamIx, value: &[u8]) -> SqliteResult<()> {
        let ix = i as c_int;
        // SQLITE_TRANSIENT => SQLite makes a copy
        let transient = unsafe { mem::transmute(-1 as isize) };
        let len = value.len() as c_int;
        // from &[u8] to &[i8]
        let val = unsafe { mem::transmute(value.as_ptr()) };
        let r = unsafe { ffi::sqlite3_bind_blob(self.stmt, ix, val, len, transient) };
        decode_result(r, "sqlite3_bind_blob", self.detail_db())
    }

    /// Clear all parameter bindings.
    pub fn clear_bindings(&mut self) {
        // We ignore the return value, since no return codes are documented.
        unsafe { ffi::sqlite3_clear_bindings(self.stmt) };
    }

    /// Return the number of SQL parameters.
    /// If parameters of the ?NNN form are used, there may be gaps in the list.
    pub fn bind_parameter_count(&mut self) -> ParamIx {
        let count = unsafe { ffi::sqlite3_bind_parameter_count(self.stmt) };
        count as ParamIx
    }

    /// Expose the underlying `sqlite3_stmt` struct pointer for use
    /// with the `ffi` module.
    pub unsafe fn expose(&mut self) -> *mut ffi::sqlite3_stmt {
        self.stmt
    }

    /// Return the number of database rows that were changed or
    /// inserted or deleted by this statement if it is the most
    /// recently run on its database connection.
    ///
    /// cf `sqlite3_changes`.
    pub fn changes(&self) -> u64 {
        let dbh = self.db.handle;
        let count = unsafe { ffi::sqlite3_changes(dbh) };
        count as u64
    }
}


/// Results of executing a `prepare()`d statement.
pub struct ResultSet<'res> {
    statement: &'res mut PreparedStatement,
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Eq)]
    #[allow(non_camel_case_types)]
    enum Step {
        SQLITE_ROW       = 100,
        SQLITE_DONE      = 101,
    }
}

impl<'res> Drop for ResultSet<'res> {
    fn drop(&mut self) {

        // We ignore the return code from reset because it has already
        // been reported:
        //
        // "If the most recent call to sqlite3_step(S) for the prepared
        // statement S indicated an error, then sqlite3_reset(S)
        // returns an appropriate error code."
        unsafe { ffi::sqlite3_reset(self.statement.stmt) };
    }
}


impl<'res: 'row, 'row> ResultSet<'res> {
    /// Execute the next step of a prepared statement.
    pub fn step(&'row mut self) -> SqliteResult<Option<ResultRow<'res, 'row>>> {
        let result = unsafe { ffi::sqlite3_step(self.statement.stmt) };
        match Step::from_i32(result) {
            Some(SQLITE_ROW) => Ok(Some(ResultRow { rows: self })),
            Some(SQLITE_DONE) => Ok(None),
            None => Err(error_result(result, "step", self.statement.get_detail())),
        }
    }
}


/// Access to columns of a row.
pub struct ResultRow<'res: 'row, 'row> {
    rows: &'row mut ResultSet<'res>,
}

/// Column index for accessing parts of a row.
pub type ColIx = u32;

/// Access to one row (step) of a result.
///
/// Note "These routines attempt to convert the value where appropriate."[1]
/// and "The value returned by `sqlite3_column_type()` is only
/// meaningful if no type conversions have occurred as described
/// below. After a type conversion, the value returned by
/// `sqlite3_column_type()` is undefined."[1]
///
/// [1]: http://www.sqlite.org/c3ref/column_blob.html
impl<'res, 'row> ResultRow<'res, 'row> {
    /// cf `sqlite3_column_count`
    ///
    /// *TODO: consider returning Option<uint>
    /// "This routine returns 0 if pStmt is an SQL statement that does
    /// not return data (for example an UPDATE)."*
    pub fn column_count(&self) -> ColIx {
        let stmt = self.rows.statement.stmt;
        let result = unsafe { ffi::sqlite3_column_count(stmt) };
        result as ColIx
    }

    /// Look up a column name and compute some function of it.
    ///
    /// Return `default` if there is no column `i`
    ///
    /// cf `sqlite_column_name`
    pub fn with_column_name<T, F: Fn(&str) -> T>(&mut self, i: ColIx, default: T, f: F) -> T {
        let stmt = self.rows.statement.stmt;
        let n = i as c_int;
        let result = unsafe { ffi::sqlite3_column_name(stmt, n) };
        match charstar_str(&result) {
            Some(name) => f(name),
            None => default,
        }
    }

    /// Look up the type of a column.
    ///
    /// Return `SQLITE_NULL` if there is no such `col`.
    pub fn column_type(&self, col: ColIx) -> ColumnType {
        let stmt = self.rows.statement.stmt;
        let i_col = col as c_int;
        let result = unsafe { ffi::sqlite3_column_type(stmt, i_col) };
        // fail on out-of-range result instead?
        ColumnType::from_i32(result).unwrap_or(SQLITE_NULL)
    }

    /// Get `int` value of a column.
    pub fn column_int(&self, col: ColIx) -> i32 {
        let stmt = self.rows.statement.stmt;
        let i_col = col as c_int;
        unsafe { ffi::sqlite3_column_int(stmt, i_col) }
    }

    /// Get `int64` value of a column.
    pub fn column_int64(&self, col: ColIx) -> i64 {
        let stmt = self.rows.statement.stmt;
        let i_col = col as c_int;
        unsafe { ffi::sqlite3_column_int64(stmt, i_col) }
    }

    /// Get `f64` (aka double) value of a column.
    pub fn column_double(&self, col: ColIx) -> f64 {
        let stmt = self.rows.statement.stmt;
        let i_col = col as c_int;
        unsafe { ffi::sqlite3_column_double(stmt, i_col) }
    }

    /// Get `Option<String>` (aka text) value of a column.
    pub fn column_text(&self, col: ColIx) -> Option<String> {
        self.column_str(col).map(|s| s.to_string())
    }

    /// Get `Option<&str>` (aka text) value of a column.
    pub fn column_str(&self, col: ColIx) -> Option<&str> {
        self.column_slice(col).and_then(|slice| str::from_utf8(slice).ok())
    }

    /// Get `Option<Vec<u8>>` (aka blob) value of a column.
    pub fn column_blob(&self, col: ColIx) -> Option<Vec<u8>> {
        self.column_slice(col).map(|bs| bs.to_vec())
    }

    /// Get `Option<&[u8]>` (aka blob) value of a column.
    pub fn column_slice(&self, col: ColIx) -> Option<&[u8]> {
        let stmt = self.rows.statement.stmt;
        let i_col = col as c_int;
        let bs = unsafe { ffi::sqlite3_column_blob(stmt, i_col) } as *const ::libc::c_uchar;
        if bs.is_null() {
            return None;
        }
        let len = unsafe { ffi::sqlite3_column_bytes(stmt, i_col) } as usize;
        Some(unsafe { slice::from_raw_parts(bs, len) })
    }
}


/// Decode SQLite result as `SqliteResult`.
///
/// Note the use of the `Result<T, E>` pattern to distinguish errors in
/// the type system.
///
/// # Panic
///
/// Panics if result is not a SQLITE error code.
pub fn decode_result(result: c_int,
                     desc: &'static str,
                     detail_db: Option<*mut ffi::sqlite3>)
                     -> SqliteResult<()> {
    if result == SQLITE_OK as c_int {
        Ok(())
    } else {
        let detail = detail_db.map(DatabaseConnection::_errmsg);
        Err(error_result(result, desc, detail))
    }
}


fn error_result(result: c_int, desc: &'static str, detail: Option<String>) -> SqliteError {
    SqliteError {
        kind: SqliteErrorCode::from_i32(result).unwrap(),
        desc: desc,
        detail: detail,
    }
}


#[cfg(test)]
mod test_opening {
    use super::{DatabaseConnection, SqliteResult};
    use time::Duration;

    #[test]
    fn db_construct_typechecks() {
        assert!(DatabaseConnection::in_memory().is_ok())
    }

    #[test]
    fn db_busy_timeout() {
        fn go() -> SqliteResult<()> {
            let mut db = try!(DatabaseConnection::in_memory());
            db.busy_timeout(Duration::seconds(2))
        }
        go().unwrap();
    }

    // TODO: _v2 with flags
}


#[cfg(test)]
mod tests {
    use super::{DatabaseConnection, SqliteResult, ResultSet};
    use std::str;

    #[test]
    fn stmt_new_types() {
        fn go() -> SqliteResult<()> {
            let db = try!(DatabaseConnection::in_memory());
            let res = db.prepare("select 1 + 1").map(|_s| ());
            res
        }
        go().unwrap();
    }


    fn with_query<T, F>(sql: &str, mut f: F) -> SqliteResult<T>
        where F: FnMut(&mut ResultSet) -> T
    {
        let db = try!(DatabaseConnection::in_memory());
        let mut s = try!(db.prepare(sql));
        let mut rows = s.execute();
        Ok(f(&mut rows))
    }

    #[test]
    fn query_two_rows() {
        fn go() -> SqliteResult<(u32, i32)> {
            let mut count = 0;
            let mut sum = 0i32;

            with_query("select 1
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
    fn query_null_string() {
        with_query("select null", |rows| {
                match rows.step() {
                    Ok(Some(ref mut row)) => {
                        assert_eq!(row.column_text(0), None);
                    }
                    _ => {
                        panic!("Expected a row");
                    }
                }
            })
            .unwrap();
    }

    #[test]
    fn detailed_errors() {
        let go = || -> SqliteResult<()> {
            let db = try!(DatabaseConnection::in_memory());
            try!(db.prepare("select bogus"));
            Ok(())
        };
        let err = go().err().unwrap();
        assert_eq!(err.detail(), Some("no such column: bogus".to_string()))
    }

    #[test]
    fn no_alloc_errors_db() {
        let go = || {
            let mut db = try!(DatabaseConnection::in_memory());
            db.ignore_detail();
            try!(db.prepare("select bogus"));
            Ok(())
        };
        let x: SqliteResult<()> = go();
        let err = x.err().unwrap();
        assert_eq!(err.detail(), None)
    }

    #[test]
    fn no_alloc_errors_stmt() {
        let db = DatabaseConnection::in_memory().unwrap();
        let mut stmt = db.prepare("select 1").unwrap();
        stmt.ignore_detail();
        let oops = stmt.bind_text(3, "abc");
        assert_eq!(oops.err().unwrap().detail(), None)
    }

    #[test]
    fn non_utf8_str() {
        let mut stmt =
            DatabaseConnection::in_memory().unwrap().prepare("SELECT x'4546FF'").unwrap();
        let mut rows = stmt.execute();
        let row = rows.step().unwrap().unwrap();
        assert_eq!(row.column_str(0), None);
        assert!(str::from_utf8(&[0x45u8, 0x46, 0xff]).is_err());
    }

}

// Local Variables:
// flycheck-rust-crate-root: "lib.rs"
// End:
