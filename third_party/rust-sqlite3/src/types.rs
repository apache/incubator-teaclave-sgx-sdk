//! Type conversions for binding parameters and getting query results.

use super::{PreparedStatement, ResultRow, ColIx, ParamIx};
use super::{SqliteResult, SqliteErrorCode, SqliteError};
use super::ColumnType::SQLITE_NULL;
use std::prelude::v1::*;
use std::vec::Vec;

use time;

/// Values that can be bound to parameters in prepared statements.
pub trait ToSql {
    /// Bind the `ix`th parameter to this value (`self`).
    fn to_sql(&self, s: &mut PreparedStatement, ix: ParamIx) -> SqliteResult<()>;
}

/// A trait for result values from a query.
///
/// cf [sqlite3 result values][column].
///
/// *inspired by sfackler's `FromSql` (and some haskell bindings?)*
///
/// [column]: http://www.sqlite.org/c3ref/column_blob.html
///
///   - *TODO: many more implementors, including Option<T>*
pub trait FromSql: Sized {
    /// Try to extract a `Self` type value from the `col`th colum of a `ResultRow`.
    fn from_sql(row: &ResultRow, col: ColIx) -> SqliteResult<Self>;
}

impl ToSql for i32 {
    fn to_sql(&self, s: &mut PreparedStatement, ix: ParamIx) -> SqliteResult<()> {
        s.bind_int(ix, *self)
    }
}

impl FromSql for i32 {
    fn from_sql(row: &ResultRow, col: ColIx) -> SqliteResult<i32> {
        Ok(row.column_int(col))
    }
}

impl ToSql for i64 {
    fn to_sql(&self, s: &mut PreparedStatement, ix: ParamIx) -> SqliteResult<()> {
        s.bind_int64(ix, *self)
    }
}

impl FromSql for i64 {
    fn from_sql(row: &ResultRow, col: ColIx) -> SqliteResult<i64> {
        Ok(row.column_int64(col))
    }
}

impl ToSql for f64 {
    fn to_sql(&self, s: &mut PreparedStatement, ix: ParamIx) -> SqliteResult<()> {
        s.bind_double(ix, *self)
    }
}

impl FromSql for f64 {
    fn from_sql(row: &ResultRow, col: ColIx) -> SqliteResult<f64> {
        Ok(row.column_double(col))
    }
}

impl ToSql for bool {
    fn to_sql(&self, s: &mut PreparedStatement, ix: ParamIx) -> SqliteResult<()> {
        s.bind_int(ix, if *self { 1 } else { 0 })
    }
}

impl FromSql for bool {
    fn from_sql(row: &ResultRow, col: ColIx) -> SqliteResult<bool> {
        Ok(row.column_int(col) != 0)
    }
}

impl<T: ToSql + Clone> ToSql for Option<T> {
    fn to_sql(&self, s: &mut PreparedStatement, ix: ParamIx) -> SqliteResult<()> {
        match (*self).clone() {
            Some(x) => x.to_sql(s, ix),
            None => s.bind_null(ix),
        }
    }
}

impl<T: FromSql + Clone> FromSql for Option<T> {
    fn from_sql(row: &ResultRow, col: ColIx) -> SqliteResult<Option<T>> {
        match row.column_type(col) {
            SQLITE_NULL => Ok(None),
            _ => FromSql::from_sql(row, col).map(Some),
        }
    }
}

impl ToSql for String {
    fn to_sql(&self, s: &mut PreparedStatement, ix: ParamIx) -> SqliteResult<()> {
        s.bind_text(ix, (*self).as_ref())
    }
}


impl FromSql for String {
    fn from_sql(row: &ResultRow, col: ColIx) -> SqliteResult<String> {
        Ok(row.column_text(col).unwrap_or_else(String::new))
    }
}

impl<'a> ToSql for &'a [u8] {
    fn to_sql(&self, s: &mut PreparedStatement, ix: ParamIx) -> SqliteResult<()> {
        s.bind_blob(ix, *self)
    }
}

impl FromSql for Vec<u8> {
    fn from_sql(row: &ResultRow, col: ColIx) -> SqliteResult<Vec<u8>> {
        Ok(row.column_blob(col).unwrap_or_else(Vec::new))
    }
}


/// Format of sqlite date strings
///
/// From [Date And Time Functions][lang_datefunc]:
/// > The datetime() function returns "YYYY-MM-DD HH:MM:SS"
/// [lang_datefunc]: http://www.sqlite.org/lang_datefunc.html
pub static SQLITE_TIME_FMT: &'static str = "%F %T";

impl FromSql for time::Tm {
    fn from_sql(row: &ResultRow, col: ColIx) -> SqliteResult<time::Tm> {
        let txt = row.column_text(col).unwrap_or_else(String::new);
        Ok(try!(time::strptime(txt.as_ref(), SQLITE_TIME_FMT)))
    }
}

impl From<time::ParseError> for SqliteError {
    fn from(err: time::ParseError) -> SqliteError {
        SqliteError {
            kind: SqliteErrorCode::SQLITE_MISMATCH,
            desc: "Time did not match expected format",
            detail: Some(format!("{}", err)),
        }
    }
}

impl ToSql for time::Timespec {
    fn to_sql(&self, s: &mut PreparedStatement, ix: ParamIx) -> SqliteResult<()> {
        let timestr = time::at_utc(*self).strftime(SQLITE_TIME_FMT)
            .unwrap() // unit tests ensure SQLITE_TIME_FMT is ok
            .to_string();
        s.bind_text(ix, timestr.as_ref())
    }
}

impl FromSql for time::Timespec {
    /// TODO: propagate error message
    fn from_sql(row: &ResultRow, col: ColIx) -> SqliteResult<time::Timespec> {
        let tmo: SqliteResult<time::Tm> = FromSql::from_sql(row, col);
        tmo.map(|tm| tm.to_timespec())
    }
}

#[cfg(test)]
mod tests {
    use time::Tm;
    use super::super::{DatabaseConnection, SqliteResult, ResultSet};
    use super::super::ResultRowAccess;

    fn with_query<T, F>(sql: &str, mut f: F) -> SqliteResult<T>
        where F: FnMut(&mut ResultSet) -> T
    {
        let db = try!(DatabaseConnection::in_memory());
        let mut s = try!(db.prepare(sql));
        let mut rows = s.execute();
        Ok(f(&mut rows))
    }

    #[test]
    fn get_tm() {
        fn go() -> SqliteResult<()> {
            let conn = try!(DatabaseConnection::in_memory());
            let mut stmt = try!(
                conn.prepare("select datetime('2001-01-01', 'weekday 3', '3 hours')"));
            let mut results = stmt.execute();
            match results.step() {
                Ok(Some(ref mut row)) => {
                    assert_eq!(row.get::<u32, Tm>(0),
                               Tm {
                                   tm_sec: 0,
                                   tm_min: 0,
                                   tm_hour: 3,
                                   tm_mday: 3,
                                   tm_mon: 0,
                                   tm_year: 101,
                                   tm_wday: 0,
                                   tm_yday: 0,
                                   tm_isdst: 0,
                                   tm_utcoff: 0,
                                   tm_nsec: 0,
                               });
                    Ok(())
                }
                Ok(None) => panic!("no row"),
                Err(oops) => panic!("error: {:?}", oops),
            }
        }
        go().unwrap();
    }

    #[test]
    fn get_invalid_tm() {
        with_query("select 'not a time'", |results| {
                match results.step() {
                    Ok(Some(ref mut row)) => {
                        let x: SqliteResult<Tm> = row.get_opt(0u32);
                        assert!(x.is_err());
                    }
                    Ok(None) => panic!("no row"),
                    Err(oops) => panic!("error: {:?}", oops),
                };
            })
            .unwrap();
    }

    #[test]
    fn select_blob() {
        with_query("select x'ff0db0'", |results| {
                match results.step() {
                    Ok(Some(ref mut row)) => {
                        let x: SqliteResult<Vec<u8>> = row.get_opt(0u32);
                        assert_eq!(x.ok().unwrap(), [0xff, 0x0d, 0xb0].to_vec());
                    }
                    Ok(None) => panic!("no row"),
                    Err(oops) => panic!("error: {:?}", oops),
                };
            })
            .unwrap();
    }
}

// Local Variables:
// flycheck-rust-crate-root: "lib.rs"
// End:
