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

#![no_std]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]

extern crate sgx_no_tstd;
extern crate sgx_types;

use sgx_types::error::SgxStatus;

extern "C" {
    pub fn ocall_empty() -> SgxStatus;
    pub fn ocall_empty_switchless() -> SgxStatus;
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn ecall_repeat_ocalls(nrepeats: u64, use_switchless: i32) {
    if use_switchless == 0 {
        for _ in 0..nrepeats {
            ocall_empty();
        }
    } else {
        for _ in 0..nrepeats {
            ocall_empty_switchless();
        }
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn ecall_empty() {}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn ecall_empty_switchless() {}
