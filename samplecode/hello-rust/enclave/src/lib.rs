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

extern crate sgx_types;
extern crate sgx_trts;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_libc;

use sgx_types::*;
use std::prelude::v1::*;
//use std::backtrace::{self, PrintFormat};
use std::alloc::{Layout, set_alloc_error_hook};
use std::panic;

fn alloc_oom() {
    let mut v = vec![];
    loop {
        let array = [0u64; 16];
        v.push(Box::new(array));
    }
}

fn sample_oom_hook(_: Layout) {
    panic!("trigger oom unwind");
}

// no heap print
fn oom_write() {
    use sgx_libc::ocall::write;
    let fd = 0;
    let buf = b"Catched\0".as_ptr() as _;
    let count = 8;

    let _ = unsafe { write(fd, buf, count) };
}

#[no_mangle]
pub extern "C" fn say_something(_: *const u8, _: usize) -> sgx_status_t {
    set_alloc_error_hook(sample_oom_hook);
    for _ in 0..100 {
        // This is OK
        //if let Err(_) = panic::catch_unwind(|| {
        //    alloc_oom();
        //}){
        //    return sgx_status_t::SGX_SUCCESS;
        //};

        //// This would double fault
        //match panic::catch_unwind(|| {
        //    alloc_oom();
        //}) {
        //    Ok(s) => println!("Ok {:?}", s),
        //    Err(e) => println!("Catched {:?}", e), // heap allocation happened
        //}

        // This would not trigger double fault
        match panic::catch_unwind(|| {
            alloc_oom();
        }) {
            Ok(s) => println!("Ok {:?}", s), // never here
            Err(e) => {
                oom_write();
                return sgx_status_t::SGX_SUCCESS;
            },
        }
    }
    sgx_status_t::SGX_SUCCESS
}
