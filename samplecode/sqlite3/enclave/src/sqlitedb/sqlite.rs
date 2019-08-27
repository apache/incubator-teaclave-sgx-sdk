use sgx_types::*;
use std::io::{self, BufReader, Read, Write};
use std::prelude::v1::*;
use std::slice;
use std::vec::Vec;

use crate::sqlitedb::opening;
use sqlite3::{DatabaseConnection, SqliteResult};
use sqlitedb::sqlops;

pub fn start_db(existed: uint8_t) -> SqliteResult<DatabaseConnection> {
    // A sample &'static string
    let rust_raw_string = "This is a in-Enclave ";
    // An array
    println!("{}", &rust_raw_string);

    let mut conn;
    match sqlops::get_database_conn() {
        Ok(x) => {
            conn = x;
            println!("sqlite opening test:");
            opening::base_test(&mut conn, existed);
            Ok(conn)
        }
        _ => {
            panic!("connect database failed");
        }
    }
}
