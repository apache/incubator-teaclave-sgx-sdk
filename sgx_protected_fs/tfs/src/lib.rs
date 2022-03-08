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

#![cfg_attr(all(feature = "tfs", not(target_vendor = "teaclave")), no_std)]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]
#![feature(exact_size_is_empty)]
#![feature(dropck_eyepatch)]

#[cfg(all(feature = "tfs", feature = "ufs"))]
compile_error!("feature \"tfs\" and feature \"ufs\" cannot be enabled at the same time");

#[cfg(all(feature = "tfs", not(target_vendor = "teaclave")))]
#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate sgx_types;

#[cfg(feature = "tfs")]
extern crate sgx_rsrvmm;
#[cfg(feature = "tfs")]
extern crate sgx_trts;

#[cfg(feature = "tfs")]
extern crate sgx_tcrypto as sgx_crypto;
#[cfg(feature = "ufs")]
extern crate sgx_ucrypto as sgx_crypto;

#[cfg(feature = "tfs")]
extern crate sgx_trand as sgx_rand;
#[cfg(feature = "ufs")]
extern crate sgx_urand as sgx_rand;

#[cfg(feature = "ufs")]
extern crate sgx_uprotected_fs;

mod fs;
#[macro_use]
mod sys;

pub use fs::*;

#[cfg(feature = "capi")]
pub mod capi;
