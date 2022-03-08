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
#![allow(unused_imports)]

#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_types;

use sgx_types::error::SgxStatus;
#[macro_use]
extern crate sgx_unit_test;
use sgx_unit_test::{run_bench_cases, run_test_cases, sgx_test_utils};

///# Safety
#[no_mangle]
pub unsafe extern "C" fn run_test_cases() -> SgxStatus {
    run_test_cases!();
    SgxStatus::Success
}

///# Safety
#[no_mangle]
pub unsafe extern "C" fn run_bench_cases(freq: u64) -> SgxStatus {
    run_bench_cases!(freq);
    SgxStatus::Success
}
