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

use std::io::Error;
use libc::{self, c_int, c_void, size_t, off_t};

#[no_mangle]
pub extern "C" fn u_malloc_ocall(error: * mut c_int, size: size_t) -> * mut c_void {
    let mut errno = 0;
    let ret = unsafe { libc::malloc(size) };
    if ret.is_null() {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_free_ocall(p: * mut c_void) {
    unsafe { libc::free(p) }
}

#[no_mangle]
pub extern "C" fn u_mmap_ocall(error: * mut c_int,
                               start: * mut c_void,
                               length: size_t,
                               prot: c_int,
                               flags: c_int,
                               fd: c_int,
                               offset: off_t) -> * mut c_void {
    let mut errno = 0;
    let ret = unsafe { libc::mmap(start, length, prot, flags, fd, offset) };
    if ret as isize == -1 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_munmap_ocall(error: * mut c_int,
                                 start: * mut c_void,
                                 length: size_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::munmap(start, length) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_msync_ocall(error: * mut c_int,
                                addr: * mut c_void,
                                length: size_t,
                                flags: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::msync(addr, length, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_mprotect_ocall(error: * mut c_int,
                                   addr: * mut c_void,
                                   length: size_t,
                                   prot: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::mprotect(addr, length, prot) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}