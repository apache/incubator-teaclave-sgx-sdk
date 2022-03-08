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

#![cfg_attr(not(target_vendor = "teaclave"), no_std)]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]

#[cfg(not(target_vendor = "teaclave"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_types;

use sgx_types::error::SgxStatus;
use std::backtrace::PrintFormat;
use std::panic;

///# Safety
#[no_mangle]
pub unsafe extern "C" fn backtrace() -> SgxStatus {
    let _ = std::backtrace::enable_backtrace(PrintFormat::Short);
    panic::catch_unwind(|| std::backtrace::__rust_begin_short_backtrace(test_backtrace_1)).ok();

    let _ = std::backtrace::enable_backtrace(PrintFormat::Full);
    panic::catch_unwind(test_backtrace_2).ok();

    println!("\nsgx_backtrace sample code:");
    foo();

    println!("\nstd::backtrace sample code:");
    foo_1();

    SgxStatus::Success
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
fn test_panic() -> ! {
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
    println!("{:?}", sgx_backtrace::Backtrace::new());
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
            println!();
        });
        if !resolved {
            println!(" - <no info>");
        }
        true // keep going
    });
}
