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
#![allow(clippy::derive_hash_xor_eq)]
#![allow(clippy::missing_safety_doc)]
#![allow(incomplete_features)]
#![feature(rustc_attrs)]
#![feature(specialization)]
#![feature(toowned_clone_into)]
#![feature(vec_into_raw_parts)]
#![cfg_attr(feature = "unit_test", allow(clippy::useless_format))]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

extern crate sgx_types;

pub mod ascii;
pub mod c_str;
pub mod memchr;
