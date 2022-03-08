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
#![feature(allocator_api)]

#[cfg(all(feature = "tmsg", feature = "umsg"))]
compile_error!("feature \"tmsg\" and feature \"umsg\" cannot be enabled at the same time");

#[cfg(not(any(feature = "tmsg", feature = "umsg")))]
compile_error!("need to enable feature \"tmsg\" or feature \"umsg\"");

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate sgx_types;

#[cfg(feature = "tmsg")]
extern crate sgx_tcrypto as sgx_crypto;
#[cfg(feature = "tmsg")]
extern crate sgx_trts;
#[cfg(feature = "umsg")]
extern crate sgx_ucrypto as sgx_crypto;

#[cfg(feature = "tserialize")]
extern crate sgx_tserialize as sgx_serialize;
#[cfg(feature = "userialize")]
extern crate sgx_userialize as sgx_serialize;

mod message;
pub use message::*;
