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

//! # Cryptography Library
//!
//! The Intel(R) Software Guard Extensions SDK includes a trusted cryptography library named sgx_tcrypto.
//! It includes the cryptographic functions used by other trusted libraries included in the SDK
//!

#![cfg_attr(feature = "tcrypto", no_std)]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]
#![feature(specialization)]
#![allow(clippy::missing_safety_doc)]
#![allow(incomplete_features)]

#[cfg(all(feature = "tcrypto", feature = "ucrypto"))]
compile_error!("feature \"tcrypto\" and feature \"ucrypto\" cannot be enabled at the same time");

#[cfg(not(any(feature = "tcrypto", feature = "ucrypto")))]
compile_error!("need to enable feature \"tcrypto\" or feature \"ucrypto\"");

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate sgx_types;
extern crate sgx_crypto_sys;

#[cfg(feature = "tcrypto")]
extern crate sgx_trts;

#[cfg(feature = "tserialize")]
extern crate sgx_tserialize as sgx_serialize;
#[cfg(feature = "userialize")]
extern crate sgx_userialize as sgx_serialize;

pub mod aes;
pub mod ecc;
pub mod mac;
pub mod rsa;
pub mod sha;
mod sm;
pub use sm::*;
