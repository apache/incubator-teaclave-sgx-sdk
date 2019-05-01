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

//! Runtime services

use sgx_types::sgx_enclave_id_t;
use enclave;
use alloc_crate::slice;
use core::str;
// Reexport some of our utilities which are expected by other crates.
pub use panicking::{begin_panic, begin_panic_fmt, update_panic_count};
pub use sys_common::at_exit;
use sys_common::cleanup;

#[no_mangle]
pub extern "C" fn t_global_exit_ecall() {
}

#[no_mangle]
pub extern "C" fn t_global_init_ecall(id: u64, path: * const u8, len: usize) {

    enclave::set_enclave_id(id as sgx_enclave_id_t);
    let s = unsafe {
        let str_slice = slice::from_raw_parts(path, len);
        str::from_utf8_unchecked(str_slice)
    };
    enclave::set_enclave_path(s);
}

global_dtors_object! {
    GLOBAL_DTORS, global_exit = { cleanup(); }
}

