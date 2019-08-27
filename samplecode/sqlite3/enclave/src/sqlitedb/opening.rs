use sgx_types::*;
use std::default::Default;
use std::prelude::v1::*;
use std::vec::Vec;

use crate::sqlitedb::sqlops;
use crate::sqlitedb::{studentdao, teacherdao};
use sqlite3::access;
use sqlite3::access::flags::Flags;
use sqlite3::{
    Access, DatabaseConnection, QueryFold, ResultRowAccess, SqliteResult, StatementUpdate,
};
use sqlitedb::sqlops::lose;
use std::untrusted::fs::File;

use crate::beans::teacher::Teacher;

pub fn base_test(conn: &mut DatabaseConnection, existed: uint8_t) {
    let mut exist_flag = false;
    let mut number = 1;
    if (existed == 1) {
        exist_flag = true
    }

    teacherdao::base_teacher_ops(conn, &exist_flag);
    studentdao::base_student_ops(conn, &exist_flag);
}
