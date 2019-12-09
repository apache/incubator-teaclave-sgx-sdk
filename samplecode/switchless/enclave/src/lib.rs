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

#![crate_name = "switchlessenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
extern crate sgx_tstd as std;

use sgx_types::*;

extern "C"{
    // OCALLS
    pub fn ocall_empty () -> sgx_status_t;
    pub fn ocall_empty_switchless () -> sgx_status_t;
}

#[no_mangle]
pub extern "C"
fn ecall_repeat_ocalls(nrepeats : u64, use_switchless : i32) {

    if use_switchless == 0 {
        for _ in 0..nrepeats {
            unsafe {ocall_empty();}
        }
    }
    else {
        for _ in 0..nrepeats {
            unsafe {ocall_empty_switchless();}
        }
    }
}
#[no_mangle]
pub extern "C"
fn ecall_empty(){
}
#[no_mangle]
pub extern "C"
fn ecall_empty_switchless() {
}
