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

#![no_std]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]

extern crate sgx_types;

mod crypto;
pub use self::crypto::*;
