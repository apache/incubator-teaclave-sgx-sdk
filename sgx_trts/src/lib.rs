// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

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

#![feature(allocator_api)]
#![feature(asm)]
#![feature(lang_items)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(overflowing_literals)]
#![allow(non_snake_case)]

extern crate alloc;
extern crate sgx_types;
extern crate sgx_libc;

#[macro_use]
mod macros;

pub mod veh;
pub mod trts;
pub mod enclave;
pub mod memeq;
pub mod oom;
pub mod error;
pub mod libc;
pub mod memchr;
pub mod ascii;
pub mod c_str;
