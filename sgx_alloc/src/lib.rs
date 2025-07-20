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
// under the License.

//! # liballoc crate for Rust SGX SDK
//!
//! This crate equals to the `liballoc_system` crate in Rust.
//! It connects Rust memory allocation to Intel SGX's sgx_tstd library.
//! It is essential, because we depends on Intel SGX's SDK.
//! 2018-06-22 Add liballoc components here

#![no_std]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]
#![allow(clippy::missing_safety_doc)]
#![allow(non_camel_case_types)]
#![feature(allocator_api)]
#![feature(alloc_layout_extra)]
#![feature(core_intrinsics)]
#![feature(dropck_eyepatch)]
#![feature(ptr_internals)]
#![feature(slice_ptr_get)]

extern crate alloc;

mod system;
pub use system::System;

pub mod alignalloc;
pub mod alignbox;
pub mod capi;
