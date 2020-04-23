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

#![crate_name = "backtracesampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

extern crate sgx_backtrace;
use sgx_backtrace::Backtrace;

use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::slice;
use std::io::{self, Write};
use std::backtrace::{self, PrintFormat};
use std::panic;

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

    let _ = backtrace::enable_backtrace("enclave.signed.so", PrintFormat::Short);
    panic::catch_unwind(||{
        backtrace::__rust_begin_short_backtrace(||{
            test_backtrace_1()
        })
    }).ok();

    let _ = backtrace::enable_backtrace("enclave.signed.so", PrintFormat::Full);
    panic::catch_unwind(||{
        test_backtrace_2()
    }).ok();

    println!("\nsgx_backtrace sample code:");
    let _  = sgx_backtrace::set_enclave_path("enclave.signed.so");
    foo();

    println!("\nstd::backtrace sample code:");
    foo_1();

    sgx_status_t::SGX_SUCCESS
}

#[inline(never)]
fn test_backtrace_1() {
    test_panic();
}

#[inline(never)]
fn test_backtrace_2() {
    test_panic();
}

#[inline(never)]
fn test_panic() -> !{
    panic!("enclave panicked.");
}

#[inline(never)]
fn foo() {
    bar()
}

#[inline(never)]
fn bar() {
    baz()
}

#[inline(never)]
fn baz() {
    println!("{:?}", Backtrace::new());
    raw()
}

#[inline(never)]
fn raw() {
    print()
}

#[inline(never)]
fn foo_1() {
    bar_1()
}

#[inline(never)]
fn bar_1() {
    baz_1()
}

#[inline(never)]
fn baz_1() {
    let bt = std::backtrace::Backtrace::capture();
    println!("{:#?}", bt);
    println!("{:#}", bt);
}

#[cfg(target_pointer_width = "32")]
const HEX_WIDTH: usize = 10;
#[cfg(target_pointer_width = "64")]
const HEX_WIDTH: usize = 20;

#[inline(never)]
fn print() {
    let mut cnt = 0;
    sgx_backtrace::trace(|frame| {
        let ip = frame.ip();
        print!("frame #{:<2} - {:#02$x}", cnt, ip as usize, HEX_WIDTH);
        cnt += 1;

        let mut resolved = false;
        sgx_backtrace::resolve(frame.ip(), |symbol| {
            if !resolved {
                resolved = true;
            } else {
                print!("{}", vec![" "; 7 + 2 + 3 + HEX_WIDTH].join(""));
            }

            if let Some(name) = symbol.name() {
                print!(" - {}", name);
            } else {
                print!(" - <unknown>");
            }
            if let Some(file) = symbol.filename() {
                if let Some(l) = symbol.lineno() {
                    print!("\n{:13}{:4$}@ {}:{}", "", "", file.display(), l, HEX_WIDTH);
                }
            }
            println!("");
        });
        if !resolved {
            println!(" - <no info>");
        }
        true // keep going
    });
}