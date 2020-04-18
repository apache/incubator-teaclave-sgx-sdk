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

#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#![allow(non_camel_case_types)]

extern crate sgx_libc as libc;

pub use self::bindings::*;

mod bindings {

    pub type backtrace_syminfo_callback = extern "C" fn(
        data: *mut libc::c_void,
        pc: libc::uintptr_t,
        symname: *const libc::c_char,
        symval: libc::uintptr_t,
        symsize: libc::uintptr_t);
    pub type backtrace_full_callback = extern "C" fn(
        data: *mut libc::c_void,
        pc: libc::uintptr_t,
        filename: *const libc::c_char,
        lineno: libc::c_int,
        function: *const libc::c_char) -> libc::c_int;
    pub type backtrace_error_callback = extern "C" fn(
        data: *mut libc::c_void,
        msg: *const libc::c_char,
        errnum: libc::c_int);
    pub enum backtrace_state {}

    extern "C" {
        pub fn backtrace_create_state(
            filename: *const libc::c_char,
            threaded: libc::c_int,
            error: backtrace_error_callback,
            data: *mut libc::c_void,
        ) -> *mut backtrace_state;
        pub fn backtrace_syminfo(
            state: *mut backtrace_state,
            addr: libc::uintptr_t,
            cb: backtrace_syminfo_callback,
            error: backtrace_error_callback,
            data: *mut libc::c_void,
        ) -> libc::c_int;
        pub fn backtrace_pcinfo(
            state: *mut backtrace_state,
            addr: libc::uintptr_t,
            cb: backtrace_full_callback,
            error: backtrace_error_callback,
            data: *mut libc::c_void,
        ) -> libc::c_int;
    }
}