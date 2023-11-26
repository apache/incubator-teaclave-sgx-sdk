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
#![feature(extract_if)]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

extern crate sgx_crypto;
extern crate sgx_dcap_ra_msg;
extern crate sgx_dcap_tvl;
extern crate sgx_sync;
extern crate sgx_trts;
extern crate sgx_tse;
#[macro_use]
extern crate sgx_types;

mod ecall;
mod session;
pub use ecall::*;
pub use session::*;

pub use sgx_dcap_tvl::QveReportInfo;

#[cfg(feature = "capi")]
pub mod capi;
