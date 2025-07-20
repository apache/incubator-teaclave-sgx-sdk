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

use crate::enclave::{args, env};
use crate::ocall::util::*;
use libc::{self, c_char, c_int, c_uchar, gid_t, size_t, ssize_t, uid_t};
use std::io::Error;
use std::ptr;

#[no_mangle]
pub unsafe extern "C" fn u_getuid_ocall() -> uid_t {
    libc::getuid()
}

#[no_mangle]
pub unsafe extern "C" fn u_getgid_ocall() -> gid_t {
    libc::getgid()
}

#[no_mangle]
pub unsafe extern "C" fn u_env_ocall(
    error: *mut c_int,
    buf: *mut c_uchar,
    bufsz: size_t,
) -> ssize_t {
    if bufsz == 0 || buf.is_null() {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let mut errno = 0;
    let env = env();
    let sn = env.len();
    let ret = if bufsz >= sn {
        ptr::copy_nonoverlapping(env.as_ptr(), buf, sn);
        sn as ssize_t
    } else {
        errno = libc::ERANGE;
        -1
    };

    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_args_ocall(
    error: *mut c_int,
    buf: *mut c_uchar,
    bufsz: size_t,
) -> ssize_t {
    if bufsz == 0 || buf.is_null() {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let mut errno = 0;
    let args = args();
    let sn = args.len();
    let ret = if bufsz >= sn {
        ptr::copy_nonoverlapping(args.as_ptr(), buf, sn);
        sn as ssize_t
    } else {
        errno = libc::ERANGE;
        -1
    };

    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_getcwd_ocall(
    error: *mut c_int,
    buf: *mut c_char,
    bufsz: size_t,
) -> c_int {
    if bufsz == 0 || buf.is_null() {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let mut errno = 0;
    let ret = libc::getcwd(buf, bufsz);
    let ret = if ret.is_null() {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
        -1
    } else {
        0
    };
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_chdir_ocall(error: *mut c_int, dir: *const c_char) -> c_int {
    let mut errno = 0;
    let ret = libc::chdir(dir);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}
