use crate::beans::teacher::Teacher;
use std::io::Write;
use std::prelude::v1::*;

use sqlite3::access;
use sqlite3::access::flags::Flags;
use sqlite3::{
    Access, DatabaseConnection, QueryFold, ResultRowAccess, SqliteResult, StatementUpdate,
};

pub fn get_database_conn() -> SqliteResult<DatabaseConnection> {
    let args: Vec<String> = Vec::new();
    let usage = "sqlite";

    let mut conn;

    let cli_access = {
        let ok = |flags, dbfile| {
            Some(access::ByFilename {
                flags: flags,
                filename: dbfile,
            })
        };

        let arg = |n| {
            if args.len() > n {
                Some(args[n].as_ref())
            } else {
                None
            }
        };

        match (arg(1), arg(2)) {
            (Some("-r"), Some(dbfile)) => ok(Flags::OPEN_READONLY, dbfile),
            (Some(dbfile), None) => ok(Default::default(), dbfile),
            (_, _) => {
                let dbfile = "test.db";
                ok(Default::default(), dbfile)
            }
        }
    };

    match cli_access {
        Some(a) => {
            conn = DatabaseConnection::new(a)?;
            Ok(conn)
        }
        _ => {
            lose(usage);
            panic!("create data failed");
        }
    }
}

pub fn lose(why: &str) {
    // FIXME: Set the exit status once that is stabilized
    let stderr = std::io::stderr();
    let mut stderr_lock = stderr.lock();
    stderr_lock.write_fmt(format_args!("{}", why)).unwrap()
}
