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

#![crate_name = "sgxcovenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate sgx_trts;

#[cfg(feature = "cov")]
extern crate sgx_cov;

#[cfg(feature = "cov")]
pub use sgx_cov::*;

use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::io::{self, Write};
use std::slice;

fn startup_sample() {
    println!("Starting up before everything! This is just a sample, not used in sgx_cov sample! You can use it by enabling a feature of sgx_urts.");
}

fn at_exit_sample() {
    println!("Exiting after everything! This is just a sample, not used in sgx_cov sample! You can use it by enabling a feature of sgx_urts.");
}

global_ctors_object! {
    SAMPLE_GLOBAL_CTOR, sample_init_func = { startup_sample(); }
}

global_dtors_object! {
    SAMPLE_GLOBAL_DTOR, sample_exit_func = { at_exit_sample(); }
}

#[cfg(feature = "cov")]
global_dtors_object! {
    SAMPLE_SGX_COV_FINALIZE, sgx_cov_exit = {
        println!("sgx_cov finished!");
        sgx_cov::cov_writeout();
    }
}

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {

    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    // A sample &'static string
    let rust_raw_string = "This is a in-Enclave ";
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

    if hello_string.len() % 2 == 0 {
        println!("hello string len mod 2 = 0");
    } else {
        println!("hello string len mod 2 = 1");
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8")
                                               .as_str();

    // Ocall to normal world for output
    println!("{}", &hello_string);

    sgx_status_t::SGX_SUCCESS
}
