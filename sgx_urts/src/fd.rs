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

use std::io::Error;
use libc::{self, c_int, c_void, size_t, ssize_t, off64_t, c_ulong, iovec};

#[no_mangle]
pub extern "C" fn u_read_ocall(error: * mut c_int,
                               fd: c_int,
                               buf: * mut c_void,
                               count: size_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::read(fd, buf, count) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_pread64_ocall(error: * mut c_int,
                                  fd: c_int,
                                  buf: * mut c_void,
                                  count: size_t,
                                  offset: off64_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::pread64(fd, buf, count, offset) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_readv_ocall(error: * mut c_int,
                                fd: c_int,
                                iov: * const iovec,
                                iovcnt: c_int) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::readv(fd, iov, iovcnt) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_preadv64_ocall(error: * mut c_int,
                                   fd: c_int,
                                   iov: * const iovec,
                                   iovcnt: c_int,
                                   offset: off64_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::preadv64(fd, iov, iovcnt, offset) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_write_ocall(error: * mut c_int,
                                fd: c_int,
                                buf: * const c_void,
                                count: size_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::write(fd, buf, count) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_pwrite64_ocall(error: * mut c_int,
                                   fd: c_int,
                                   buf: * const c_void,
                                   count: size_t,
                                   offset: off64_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::pwrite64(fd, buf, count, offset) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_writev_ocall(error: * mut c_int,
                                 fd: c_int,
                                 iov: * const iovec,
                                 iovcnt: c_int) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::writev(fd, iov, iovcnt) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_pwritev64_ocall(error: * mut c_int,
                                    fd: c_int,
                                    iov: * const iovec,
                                    iovcnt: c_int,
                                    offset: off64_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::pwritev64(fd, iov, iovcnt, offset) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fcntl_arg0_ocall(error: * mut c_int,
                                     fd: c_int,
                                     cmd: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::fcntl(fd, cmd) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fcntl_arg1_ocall(error: * mut c_int,
                                     fd: c_int,
                                     cmd: c_int,
                                     arg: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::fcntl(fd, cmd, arg) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_ioctl_arg0_ocall(error: * mut c_int,
                                     fd: c_int,
                                     request: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::ioctl(fd, request as c_ulong) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_ioctl_arg1_ocall(error: * mut c_int,
                                     fd: c_int,
                                     request: c_int,
                                     arg: * const c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::ioctl(fd, request as c_ulong, arg) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_close_ocall(error: * mut c_int, fd: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::close(fd) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}