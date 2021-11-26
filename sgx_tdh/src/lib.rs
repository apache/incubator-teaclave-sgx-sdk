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

//! # Diffie–Hellman (DH) Session Establishment Functions
//!
//! These functions allow an ISV to establish secure session between two enclaves using the EC DH Key exchange protocol.
//!

#![no_std]
#![cfg_attr(
    all(target_env = "sgx", target_vendor = "mesalock"),
    feature(rustc_private)
)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]

#[macro_use]
extern crate alloc;

extern crate sgx_tcrypto;
extern crate sgx_trts;
extern crate sgx_tse;
extern crate sgx_types;

mod dh;
pub use self::dh::*;

mod ecp;
