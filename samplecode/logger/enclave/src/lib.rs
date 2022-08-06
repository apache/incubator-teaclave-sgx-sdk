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

extern crate sgx_libc;
extern crate sgx_types;

use log::{debug, error, info, trace};
use sgx_types::error::SgxStatus;
use std::slice;

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn output_log(some_string: *const u8, some_len: usize) -> SgxStatus {
    env_logger::init();

    debug!("this is a debug {}.", "message");
    error!("this is printed by default.");

    let str_slice = slice::from_raw_parts(some_string, some_len);
    info!("{}", String::from_utf8(str_slice.to_vec()).unwrap());
    trace!("{}", "This is a in-Enclave Rust string.");

    SgxStatus::Success
}
