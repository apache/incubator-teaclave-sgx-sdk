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

//! # Trusted Runtime System
//!
//! The Intel(R) SGX trusted runtime system (tRTS) is a key component of the Intel(R) Software Guard Extensions SDK.
//! It provides the enclave entry point logic as well as other functions to be used by enclave developers.
//!
//! **Intel(R) Software Guard Extensions Helper Functions**
//!
//! **CustomExceptionHandling**
//!
//! # Intel(R) Software Guard Extensions Helper Functions
//!
//! The tRTS provides the helper functions for you to determine whether a given address is within or outside
//! enclave memory.
//!
//! The tRTS provides a wrapper to the RDRAND instruction to generate a true random number from hardware.
//! enclave developers should use the rsgx_read_rand function to get true random numbers.
//!
//! # CustomExceptionHandling
//!
//! The Intel(R) Software Guard Extensions SDK provides an API to allow you to register functions, or exception handlers,
//! to handle a limited set of hardware exceptions. When one of the enclave supported hardware exceptions occurs within
//! the enclave, the registered exception handlers will be called in a specific order until an exception handler reports
//! that it has handled the exception. For example, issuing a CPUID instruction inside an Enclave will result in a #UD fault
//! (Invalid Opcode Exception). ISV enclave code can call rsgx_register_exception_handler to register a function of type
//! sgx_exception_handler_t to respond to this exception. To check a list of enclave supported exceptions, see Intel(R)
//! Software Guard Extensions Programming Reference.
//!
//! **Note**
//!
//! Custom exception handling is only supported in HW mode. Although the exception handlers can be registered in simulation mode,
//! the exceptions cannot be caught and handled within the enclave.
//!
//! **Note**
//!
//! OCALLs are not allowed in the exception handler.
//!
//! **Note**
//!
//! Custom exception handing only saves general purpose registers in sgx_ exception_info_t. You should be careful when touching
//! other registers in the exception handlers.
//!
//! **Note**
//!
//! If the exception handlers can not handle the exceptions, abort() is called.
//! abort() makes the enclave unusable and generates another exception.
//!

#![no_std]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#![feature(allocator_api)]
#![feature(specialization)]
#![feature(vec_into_raw_parts)]
#![feature(rustc_attrs)]
#![allow(incomplete_features)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::derive_hash_xor_eq)]

#[cfg(target_env = "sgx")]
extern crate sgx_types;

#[cfg(target_env = "sgx")]
extern crate sgx_libc;

extern crate alloc;

#[macro_use]
mod macros;

pub mod aex;
pub mod ascii;
pub mod c_str;
pub mod cpu_feature;
pub mod cpuid;
pub mod emm;
pub mod enclave;
pub mod memchr;
pub mod memeq;
pub mod oom;
pub mod trts;
pub mod veh;

#[cfg(not(target_env = "sgx"))]
pub use sgx_libc as libc;

#[cfg(target_env = "sgx")]
pub mod libc {
    pub use sgx_libc::*;
}

pub mod error {
    pub use sgx_libc::{errno, error_string, set_errno};
}
