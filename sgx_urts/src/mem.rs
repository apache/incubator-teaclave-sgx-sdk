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

use libc::{self, c_int, c_void, off_t, size_t};
use std::io::Error;

#[no_mangle]
pub extern "C" fn u_malloc_ocall(error: *mut c_int, size: size_t) -> *mut c_void {
    let mut errno = 0;
    let ret = unsafe { libc::malloc(size) };
    if ret.is_null() {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_free_ocall(p: *mut c_void) {
    unsafe { libc::free(p) }
}

#[no_mangle]
pub extern "C" fn u_mmap_ocall(
    error: *mut c_int,
    start: *mut c_void,
    length: size_t,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: off_t,
) -> *mut c_void {
    let mut errno = 0;
    let ret = unsafe { libc::mmap(start, length, prot, flags, fd, offset) };
    if ret as isize == -1 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_munmap_ocall(error: *mut c_int, start: *mut c_void, length: size_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::munmap(start, length) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_msync_ocall(
    error: *mut c_int,
    addr: *mut c_void,
    length: size_t,
    flags: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::msync(addr, length, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_mprotect_ocall(
    error: *mut c_int,
    addr: *mut c_void,
    length: size_t,
    prot: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::mprotect(addr, length, prot) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}
