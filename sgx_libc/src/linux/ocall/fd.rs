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

use crate::linux::*;
use alloc::vec::Vec;
use core::mem;
use core::slice;
use sgx_oc::linux::ocall;
use sgx_trts::trts::is_within_enclave;

#[no_mangle]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    if buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    let buf = slice::from_raw_parts_mut(buf as *mut u8, count);
    if let Ok(rsize) = ocall::read(fd, buf) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn pread64(
    fd: c_int,
    buf: *mut c_void,
    count: size_t,
    offset: off64_t,
) -> ssize_t {
    if buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    let buf = slice::from_raw_parts_mut(buf as *mut u8, count);
    if let Ok(rsize) = ocall::pread64(fd, buf, offset) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn readv(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    if iov.is_null()
        || iovcnt < 0
        || !is_within_enclave(iov as *const u8, iovcnt as usize * mem::size_of::<iovec>())
    {
        set_errno(EINVAL);
        return -1;
    }

    let iov = slice::from_raw_parts(iov, iovcnt as usize);
    let mut iovec: Vec<&mut [u8]> = Vec::with_capacity(iovcnt as usize);
    for v in iov {
        if v.iov_base.is_null() {
            set_errno(EINVAL);
            return -1;
        }
        iovec.push(slice::from_raw_parts_mut(v.iov_base as *mut u8, v.iov_len));
    }

    if let Ok(rsize) = ocall::readv(fd, iovec) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn preadv64(
    fd: c_int,
    iov: *const iovec,
    iovcnt: c_int,
    offset: off64_t,
) -> ssize_t {
    if iov.is_null()
        || iovcnt < 0
        || !is_within_enclave(iov as *const u8, iovcnt as usize * mem::size_of::<iovec>())
    {
        set_errno(EINVAL);
        return -1;
    }

    let iov = slice::from_raw_parts(iov, iovcnt as usize);
    let mut iovec: Vec<&mut [u8]> = Vec::with_capacity(iovcnt as usize);
    for v in iov {
        if v.iov_base.is_null() {
            set_errno(EINVAL);
            return -1;
        }
        iovec.push(slice::from_raw_parts_mut(v.iov_base as *mut u8, v.iov_len));
    }

    if let Ok(rsize) = ocall::preadv64(fd, iovec, offset) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    if buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    let buf = slice::from_raw_parts(buf as *const u8, count);
    if let Ok(rsize) = ocall::write(fd, buf) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn pwrite64(
    fd: c_int,
    buf: *const c_void,
    count: size_t,
    offset: off64_t,
) -> ssize_t {
    if buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    let buf = slice::from_raw_parts(buf as *const u8, count);
    if let Ok(rsize) = ocall::pwrite64(fd, buf, offset) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn writev(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    if iov.is_null()
        || iovcnt < 0
        || !is_within_enclave(iov as *const u8, iovcnt as usize * mem::size_of::<iovec>())
    {
        set_errno(EINVAL);
        return -1;
    }

    let iov = slice::from_raw_parts(iov, iovcnt as usize);
    let mut iovec: Vec<&[u8]> = Vec::with_capacity(iovcnt as usize);
    for v in iov {
        if v.iov_base.is_null() {
            set_errno(EINVAL);
            return -1;
        }
        iovec.push(slice::from_raw_parts(v.iov_base as *const u8, v.iov_len));
    }

    if let Ok(rsize) = ocall::writev(fd, iovec) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn pwritev64(
    fd: c_int,
    iov: *const iovec,
    iovcnt: c_int,
    offset: off64_t,
) -> ssize_t {
    if iov.is_null()
        || iovcnt < 0
        || !is_within_enclave(iov as *const u8, iovcnt as usize * mem::size_of::<iovec>())
    {
        set_errno(EINVAL);
        return -1;
    }

    let iov = slice::from_raw_parts(iov, iovcnt as usize);
    let mut iovec: Vec<&[u8]> = Vec::with_capacity(iovcnt as usize);
    for v in iov {
        if v.iov_base.is_null() {
            set_errno(EINVAL);
            return -1;
        }
        iovec.push(slice::from_raw_parts(v.iov_base as *const u8, v.iov_len));
    }

    if let Ok(rsize) = ocall::pwritev64(fd, iovec, offset) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn sendfile(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut off_t,
    count: size_t,
) -> ssize_t {
    let offset = if !offset.is_null() {
        Some(&mut *offset)
    } else {
        None
    };

    if let Ok(rsize) = ocall::sendfile(out_fd, in_fd, offset, count) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn copy_file_range(
    fd_in: c_int,
    off_in: *mut off64_t,
    fd_out: c_int,
    off_out: *mut off64_t,
    len: size_t,
    flags: c_uint,
) -> ssize_t {
    let off_in = if !off_in.is_null() {
        Some(&mut *off_in)
    } else {
        None
    };

    let off_out = if !off_out.is_null() {
        Some(&mut *off_out)
    } else {
        None
    };

    if let Ok(rsize) = ocall::copy_file_range(fd_in, off_in, fd_out, off_out, len, flags) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn splice(
    fd_in: c_int,
    off_in: *mut off64_t,
    fd_out: c_int,
    off_out: *mut off64_t,
    len: size_t,
    flags: c_uint,
) -> ssize_t {
    let off_in = if !off_in.is_null() {
        Some(&mut *off_in)
    } else {
        None
    };

    let off_out = if !off_out.is_null() {
        Some(&mut *off_out)
    } else {
        None
    };

    if let Ok(rsize) = ocall::splice(fd_in, off_in, fd_out, off_out, len, flags) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn fcntl(fd: c_int, cmd: c_int, arg: c_long) -> c_int {
    if let Ok(ret) = ocall::fcntl(fd, cmd, arg) {
        ret
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn fcntl_arg0(fd: c_int, cmd: c_int) -> c_int {
    if let Ok(ret) = ocall::fcntl_arg0(fd, cmd) {
        ret
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn fcntl_arg1(fd: c_int, cmd: c_int, arg: c_int) -> c_int {
    if let Ok(ret) = ocall::fcntl_arg1(fd, cmd, arg) {
        ret
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn ioctl_arg0(fd: c_int, request: c_ulong) -> c_int {
    if ocall::ioctl_arg0(fd, request).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn ioctl_arg1(fd: c_int, request: c_ulong, arg: *mut c_int) -> c_int {
    if arg.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::ioctl_arg1(fd, request, &mut *arg).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn close(fd: c_int) -> c_int {
    if ocall::close(fd).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    i32::from(ocall::isatty(fd).is_ok())
}

#[no_mangle]
pub unsafe extern "C" fn dup(oldfd: c_int) -> c_int {
    if let Ok(fd) = ocall::dup(oldfd) {
        fd
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn eventfd(initval: c_uint, flags: c_int) -> c_int {
    if let Ok(fd) = ocall::eventfd(initval, flags) {
        fd
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn futimens(fd: c_int, times: *const [timespec; 2]) -> c_int {
    if times.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::futimens(fd, &*times).is_ok() {
        0
    } else {
        -1
    }
}
