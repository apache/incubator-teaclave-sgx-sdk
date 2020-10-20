// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

#![crate_name = "helloworldsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

// sqlite3's symbols do not follow Rust's style conventions, suppress warnings
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::slice;
use std::io::{self, Write};

include!("./bindings.rs");

/// 
extern "C" {
    pub fn ocall_print_error(some_string: *const i8) -> sgx_status_t;
    pub fn ocall_print_string(some_string: *const i8) -> sgx_status_t;
    pub fn ocall_println_string(some_string: *const i8) -> sgx_status_t;
    pub fn callback(
        arg1: *mut ::std::os::raw::c_void,
        arg2: ::std::os::raw::c_int,
        arg3: *mut *mut ::std::os::raw::c_char,
        arg4: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}

/// A function simply invokes ocall print to print the incoming string
///
/// # Parameters
///
/// **some_string**
///
/// A pointer to the string to be printed
///
/// **len**
///
/// An unsigned int indicates the length of str
///
/// # Return value
///
/// Always returns SGX_SUCCESS
#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {

    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    // A sample &'static string
    let rust_raw_string = "This is a ";
    // An array
    let word:[u8;4] = [82, 117, 115, 116];
    // An vector
    let word_vec:Vec<u8> = vec![32, 115, 116, 114, 105, 110, 103, 33];

    // Construct a string from &'static string
    let mut hello_string = String::from(rust_raw_string);

    // Iterate on word array
    for c in word.iter() {
        hello_string.push(*c as char);
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8")
                                               .as_str();

    // Ocall to normal world for output
    println!("{}", &hello_string);

    // Opening database
    // We are certain that our string doesn't have 0 bytes in the middle,
    // so we can .expect()
    let dbname = std::ffi::CString::new("test.db").expect("CString::new failed");
    let sql = std::ffi::CString::new("CREATE TABLE COMPANY(ID INT PRIMARY KEY NOT NULL,NAME TEXT NOT NULL, AGE INT NOT NULL, ADDRESS CHAR(50), SALARY REAL);").expect("CString::new failed"); 
    let mut zErrMsg = std::ffi::CString::new("sqlite3 Execution Error").expect("CString::new failed");
    
    unsafe{
        // let mut db: *mut sqlite3 = &mut db_inner;
        let mut db: *mut sqlite3 = db_wrapper.expect("DB failed to unwrap");

        println!("Now opening database connection...");
        let res = sqlite3_open(dbname.as_ptr(), &mut db);
        if res != 0 {
            println!("SQLite error - can't open database connection: ");
            ocall_print_error(sqlite3_errmsg(db));
        }
        
        let mut z_ptr: *mut i8 = zErrMsg.into_raw();
        let mut z_ptr_ptr: *mut *mut i8 = &mut z_ptr;
        let mut empty = std::mem::MaybeUninit::uninit().assume_init();

        println!("Now quering database...");
        let res = sqlite3_exec(db, sql.as_ptr(), Some(callback), empty, z_ptr_ptr);
        if res != 0 {
            println!("SQLite query error: ");
            ocall_print_error(sqlite3_errmsg(db));
        }

        println!("Now closing database...");
        let res = sqlite3_close(db);
        if res != 0 {
            println!("SQLite error - can't close database");
            ocall_print_error(sqlite3_errmsg(db));
        }

    }

    sgx_status_t::SGX_SUCCESS
}

// static mut db: *mut sqlite3 = std::ptr::null();
// = std::mem::MaybeUninit::uninit().assume_init();
static mut _db_unused: sqlite3 = sqlite3 { _unused: [0;0] };
// static mut db: *mut sqlite3 = &mut db_inner as *mut sqlite3;
static mut db_wrapper: Option<*mut sqlite3> = None;

// struct DB {
//     db_inner: sqlite3,
//     db: *mut sqlite3,
// }

#[no_mangle]
pub extern "C" fn ecall_opendb(dbname: *const i8) -> sgx_status_t {
    unsafe{
        // let mut db: *mut sqlite3 = &mut db_inner;
        db_wrapper = Some(&mut _db_unused as *mut sqlite3);
        let mut db: *mut sqlite3 = db_wrapper.expect("DB failed to unwrap");
        // println!("ecall_opendb: {:p}", db);
        // println!("ecall_opendb: {:?}", *db);
        // println!("ecall_opendb: {:?}", (*db)._unused);


        println!("Now opening database connection...");
        let res = sqlite3_open(dbname, &mut db);
        if res != 0 {
            println!("SQLite error - can't open database connection: ");
            ocall_print_error(sqlite3_errmsg(db));
        }

        // println!("ecall_opendb: {:?}", db);
        // println!("ecall_opendb: {:?}", *db);
        // println!("ecall_opendb: {:?}", (*db)._unused);

        db_wrapper = Some(db);

        // println!("Now closing database...");
        // let res = sqlite3_close(db);
        // if res != 0 {
        //     println!("SQLite error - can't close database");
        //     ocall_print_error(sqlite3_errmsg(db));
        // }
    }

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_execute_sql(sql: *const i8) -> sgx_status_t {
    let mut zErrMsg = std::ffi::CString::new("sqlite3 Execution Error").expect("CString::new failed");
    let mut z_ptr: *mut i8 = zErrMsg.into_raw();
    let z_ptr_ptr: *mut *mut i8 = &mut z_ptr;
    
    unsafe{
        let empty = std::mem::MaybeUninit::uninit().assume_init();
        // let mut db: *mut sqlite3 = &mut db_inner;
        let db: *mut sqlite3 = db_wrapper.expect("DB failed to unwrap");
        println!("ecall_execute_sql: {:p}", db);

        println!("Now quering database...");
        let res = sqlite3_exec(db, sql, Some(callback), empty, z_ptr_ptr);
        if res != 0 {
            println!("SQLite query error: ");
            ocall_print_error(sqlite3_errmsg(db));
        }
    }

    sgx_status_t::SGX_SUCCESS
}


#[no_mangle]
pub extern "C" fn ecall_closedb() -> sgx_status_t {
    unsafe{
        // let mut db: *mut sqlite3 = &mut db_inner;  
        let db: *mut sqlite3 = db_wrapper.expect("DB failed to unwrap");
        // println!("ecall_closedb: {:?}", db);
        // println!("ecall_closedb: {:?}", *db);
        // println!("ecall_closedb: {:?}", (*db)._unused);


        println!("Now closing database...");
        let res = sqlite3_close(db);
        if res != 0 {
            println!("SQLite error - can't close database");
            ocall_print_error(sqlite3_errmsg(db));
        }
    }

    sgx_status_t::SGX_SUCCESS
}



