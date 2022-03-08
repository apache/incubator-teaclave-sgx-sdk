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

use crate::ocall::util::*;
use libc::{self, c_int, c_uint, c_ulong, c_void, off64_t, off_t, size_t, ssize_t};
use std::io::Error;

#[no_mangle]
pub unsafe extern "C" fn u_read_ocall(
    error: *mut c_int,
    fd: c_int,
    buf: *mut c_void,
    count: size_t,
) -> ssize_t {
    let mut errno = 0;
    let ret = libc::read(fd, buf, count);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_pread64_ocall(
    error: *mut c_int,
    fd: c_int,
    buf: *mut c_void,
    count: size_t,
    offset: off64_t,
) -> ssize_t {
    let mut errno = 0;
    let ret = libc::pread64(fd, buf, count, offset);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_write_ocall(
    error: *mut c_int,
    fd: c_int,
    buf: *const c_void,
    count: size_t,
) -> ssize_t {
    let mut errno = 0;
    let ret = libc::write(fd, buf, count);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_pwrite64_ocall(
    error: *mut c_int,
    fd: c_int,
    buf: *const c_void,
    count: size_t,
    offset: off64_t,
) -> ssize_t {
    let mut errno = 0;
    let ret = libc::pwrite64(fd, buf, count, offset);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_sendfile_ocall(
    error: *mut c_int,
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut off_t,
    count: size_t,
) -> ssize_t {
    let mut errno = 0;
    let ret = libc::syscall(libc::SYS_sendfile, out_fd, in_fd, offset, count) as ssize_t;
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_copy_file_range_ocall(
    error: *mut c_int,
    fd_in: c_int,
    off_in: *mut off64_t,
    fd_out: c_int,
    off_out: *mut off64_t,
    len: size_t,
    flags: c_uint,
) -> ssize_t {
    let mut errno = 0;
    let ret = libc::syscall(
        libc::SYS_copy_file_range,
        fd_in,
        off_in,
        fd_out,
        off_out,
        len,
        flags,
    ) as ssize_t;
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_splice_ocall(
    error: *mut c_int,
    fd_in: c_int,
    off_in: *mut off64_t,
    fd_out: c_int,
    off_out: *mut off64_t,
    len: size_t,
    flags: c_uint,
) -> ssize_t {
    let mut errno = 0;
    let ret =
        libc::syscall(libc::SYS_splice, fd_in, off_in, fd_out, off_out, len, flags) as ssize_t;
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_fcntl_arg0_ocall(error: *mut c_int, fd: c_int, cmd: c_int) -> c_int {
    let mut errno = 0;
    let ret = libc::fcntl(fd, cmd);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_fcntl_arg1_ocall(
    error: *mut c_int,
    fd: c_int,
    cmd: c_int,
    arg: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::fcntl(fd, cmd, arg);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_ioctl_arg0_ocall(
    error: *mut c_int,
    fd: c_int,
    request: c_ulong,
) -> c_int {
    let mut errno = 0;
    let ret = libc::ioctl(fd, request);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_ioctl_arg1_ocall(
    error: *mut c_int,
    fd: c_int,
    request: c_ulong,
    arg: *mut c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::ioctl(fd, request, arg);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_close_ocall(error: *mut c_int, fd: c_int) -> c_int {
    let mut errno = 0;
    let ret = libc::close(fd);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_isatty_ocall(error: *mut c_int, fd: c_int) -> c_int {
    let mut errno = 0;
    let ret = libc::isatty(fd);
    if ret == 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_dup_ocall(error: *mut c_int, oldfd: c_int) -> c_int {
    let mut errno = 0;
    let ret = libc::dup(oldfd);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_eventfd_ocall(
    error: *mut c_int,
    initval: c_uint,
    flags: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::eventfd(initval, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}
