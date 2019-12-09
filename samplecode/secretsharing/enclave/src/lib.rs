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

#![crate_name = "secretsharingenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_types::*;
use std::vec::Vec;
use std::slice;
use std::io::{self, Write};

extern crate threshold_secret_sharing as tss;

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

    const PRIME: i64 = 1300477;

    let ref tss = tss::shamir::ShamirSecretSharing {
        threshold: 9,
        share_count: 20,
        prime: PRIME,  // any large enough prime will do
    };

    let secret: i64 = 23242;
    println!("Secret generated is {}", secret);
    let all_shares = tss.share(secret);

    let reconstruct_share_count = 10;
    assert!(reconstruct_share_count >= tss.reconstruct_limit());

    let indices: Vec<usize> = (0..reconstruct_share_count).collect();
    let shares: &[i64] = &all_shares[0..reconstruct_share_count];
    let recovered_secret = tss.reconstruct(&indices, shares);

    println!("The recovered secret is {}", recovered_secret);
    assert_eq!(recovered_secret, secret);


    sgx_status_t::SGX_SUCCESS
}
