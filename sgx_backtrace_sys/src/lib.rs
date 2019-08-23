// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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
        pub fn backtrace_create_state(filename: *const libc::c_char,
                                      threaded: libc::c_int,
                                      error: backtrace_error_callback,
                                      data: *mut libc::c_void) -> *mut backtrace_state;
        pub fn backtrace_syminfo(state: *mut backtrace_state,
                                 addr: libc::uintptr_t,
                                 cb: backtrace_syminfo_callback,
                                 error: backtrace_error_callback,
                                 data: *mut libc::c_void) -> libc::c_int;
        pub fn backtrace_pcinfo(state: *mut backtrace_state,
                                addr: libc::uintptr_t,
                                cb: backtrace_full_callback,
                                error: backtrace_error_callback,
                                data: *mut libc::c_void) -> libc::c_int;
    }
}