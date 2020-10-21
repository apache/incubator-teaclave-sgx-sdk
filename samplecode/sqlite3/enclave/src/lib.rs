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

/// The callback function is used for sqlite3_exec() function. It is defined in ocall_interface.c for now.
/// Other functions are standard ocall functions. Their interface are defined in .edl file. Their interface 
/// code are defined in Enclave_t and Enclave_u. Their untrusted part are implemented in app/appcpp.cpp.
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

static mut _db_unused: sqlite3 = sqlite3 { _unused: [0;0] };
static mut db_wrapper: Option<*mut sqlite3> = None;

/// A function instanciates sqlite db object
///
/// # Parameters
///
/// **dbname**
///
/// A pointer to the string of created database name
///
/// # Return value
///
/// Always returns SGX_SUCCESS
#[no_mangle]
pub extern "C" fn ecall_opendb() -> sgx_status_t {
    unsafe{
        db_wrapper = Some(&mut _db_unused as *mut sqlite3);
        let mut db: *mut sqlite3 = db_wrapper.expect("DB failed to unwrap");
        let mut dbname = std::ffi::CString::new(":memory:").expect("CString::new failed");
        let res = sqlite3_open(dbname.as_ptr() as *const ::std::os::raw::c_char, &mut db);
        if res != 0 {
            println!("SQLite error - can't open database connection: ");
            ocall_print_error(sqlite3_errmsg(db));
        }
        db_wrapper = Some(db);
    }

    sgx_status_t::SGX_SUCCESS
}

/// A function executes sql query to sqlite database
///
/// # Parameters
///
/// **sql**
///
/// A pointer to the string of executed sql query
///
/// # Return value
///
/// Always returns SGX_SUCCESS
#[no_mangle]
pub extern "C" fn ecall_execute_sql(sql: *const i8) -> sgx_status_t {
    let mut zErrMsg = std::ffi::CString::new("sqlite3 Execution Error").expect("CString::new failed");
    let mut z_ptr: *mut i8 = zErrMsg.into_raw();
    let z_ptr_ptr: *mut *mut i8 = &mut z_ptr;
    
    unsafe{
        let empty = std::mem::MaybeUninit::uninit().assume_init();
        let db: *mut sqlite3 = db_wrapper.expect("DB failed to unwrap");
        let res = sqlite3_exec(db, sql, Some(callback), empty, z_ptr_ptr);
        if res != 0 {
            println!("SQLite query error: ");
            ocall_print_error(sqlite3_errmsg(db));
        }
    }

    sgx_status_t::SGX_SUCCESS
}

/// A function close the opened sqlite database
///
/// # Return value
///
/// Always returns SGX_SUCCESS
#[no_mangle]
pub extern "C" fn ecall_closedb() -> sgx_status_t {
    unsafe{
        let db: *mut sqlite3 = db_wrapper.expect("DB failed to unwrap");
        let res = sqlite3_close(db);
        if res != 0 {
            println!("SQLite error - can't close database");
            ocall_print_error(sqlite3_errmsg(db));
        }
    }

    sgx_status_t::SGX_SUCCESS
}



