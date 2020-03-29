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

//! # Trusted SE Library
//!
//! The library provides functions for getting specific keys and for creating and verifying an enclave report.
//!

#![no_std]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#![allow(non_camel_case_types)]

extern crate sgx_types;

mod se;
pub use self::se::*;
