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

use crate::ocall::util::*;
use libc::{self, c_int, c_void, off_t, size_t};
use std::io::Error;
use std::mem;
use std::ptr;

#[cfg(target_arch = "x86")]
const MIN_ALIGN: usize = 8;

#[cfg(target_arch = "x86_64")]
const MIN_ALIGN: usize = 16;

#[no_mangle]
pub unsafe extern "C" fn u_malloc_ocall(
    error: *mut c_int,
    size: size_t,
    align: size_t,
    zeroed: c_int,
) -> *mut c_void {
    if !align.is_power_of_two() {
        set_error(error, libc::EINVAL);
        return ptr::null_mut();
    }

    if size > usize::MAX - (align - 1) {
        set_error(error, libc::EINVAL);
        return ptr::null_mut();
    }

    let (ptr, errno) = if align <= MIN_ALIGN && align <= size {
        let out: *mut c_void = libc::malloc(size);
        if out.is_null() {
            (out, Error::last_os_error().raw_os_error().unwrap_or(0))
        } else {
            (out, 0)
        }
    } else {
        let mut out: *mut c_void = ptr::null_mut();
        let align = align.max(mem::size_of::<usize>());
        let ret = libc::posix_memalign(&mut out as *mut *mut c_void, align, size);
        if ret != 0 {
            (ptr::null_mut(), ret)
        } else if out.is_null() {
            (out, libc::ENOMEM)
        } else {
            (out, 0)
        }
    };

    if errno == 0 && !ptr.is_null() && zeroed > 0 {
        ptr.write_bytes(0_u8, size)
    }
    set_error(error, errno);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn u_free_ocall(p: *mut c_void) {
    libc::free(p)
}

#[no_mangle]
pub unsafe extern "C" fn u_mmap_ocall(
    error: *mut c_int,
    start: *mut c_void,
    length: size_t,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: off_t,
) -> *mut c_void {
    let mut errno = 0;
    let ret = libc::mmap(start, length, prot, flags, fd, offset);
    if ret as isize == -1 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_munmap_ocall(
    error: *mut c_int,
    start: *mut c_void,
    length: size_t,
) -> c_int {
    let mut errno = 0;
    let ret = libc::munmap(start, length);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_msync_ocall(
    error: *mut c_int,
    addr: *mut c_void,
    length: size_t,
    flags: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::msync(addr, length, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_mprotect_ocall(
    error: *mut c_int,
    addr: *mut c_void,
    length: size_t,
    prot: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::mprotect(addr, length, prot);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}
