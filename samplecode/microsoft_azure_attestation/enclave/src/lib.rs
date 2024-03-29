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

#![crate_name = "maa"]
#![crate_type = "staticlib"]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_tse;
extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_tse::rsgx_create_report;
use sgx_types::*;

#[no_mangle]
pub extern "C" fn enclave_create_report(
    p_qe3_target: &sgx_target_info_t,
    p_report_data: &sgx_report_data_t,
    p_report: &mut sgx_report_t,
) -> u32 {
    match rsgx_create_report(p_qe3_target, p_report_data) {
        Ok(report) => {
            *p_report = report;
            0
        }
        Err(x) => {
            println!("rsgx_create_report failed! {:?}", x);
            x as u32
        }
    }
}
