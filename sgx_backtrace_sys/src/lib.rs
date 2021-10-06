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
#![cfg_attr(
    all(target_env = "sgx", target_vendor = "mesalock"),
    feature(rustc_private)
)]
#![allow(non_camel_case_types)]

extern crate sgx_libc as libc;

pub use self::bindings::*;

mod bindings {
    use libc::{c_char, c_int, c_void, uintptr_t};

    pub type backtrace_syminfo_callback = extern "C" fn(
        data: *mut c_void,
        pc: uintptr_t,
        symname: *const c_char,
        symval: uintptr_t,
        symsize: uintptr_t,
    );
    pub type backtrace_full_callback = extern "C" fn(
        data: *mut c_void,
        pc: uintptr_t,
        filename: *const c_char,
        lineno: c_int,
        function: *const c_char,
    ) -> c_int;
    pub type backtrace_error_callback =
        extern "C" fn(data: *mut c_void, msg: *const c_char, errnum: c_int);
    pub enum backtrace_state {}

    extern "C" {
        pub fn backtrace_create_state(
            filename: *const c_char,
            threaded: c_int,
            error: backtrace_error_callback,
            data: *mut c_void,
        ) -> *mut backtrace_state;
        pub fn backtrace_syminfo(
            state: *mut backtrace_state,
            addr: uintptr_t,
            cb: backtrace_syminfo_callback,
            error: backtrace_error_callback,
            data: *mut c_void,
        ) -> c_int;
        pub fn backtrace_pcinfo(
            state: *mut backtrace_state,
            addr: uintptr_t,
            cb: backtrace_full_callback,
            error: backtrace_error_callback,
            data: *mut c_void,
        ) -> c_int;
    }
}
