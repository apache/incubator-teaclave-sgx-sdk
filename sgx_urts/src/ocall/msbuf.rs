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
use libc::{c_int, c_void, size_t, ssize_t};
use std::ptr;

#[no_mangle]
pub unsafe extern "C" fn u_read_hostbuf_ocall(
    error: *mut c_int,
    host_buf: *const c_void,
    encl_buf: *mut c_void,
    count: size_t,
) -> ssize_t {
    if host_buf.is_null() || encl_buf.is_null() || count == 0 {
        set_error(error, libc::EINVAL);
        return -1;
    }

    ptr::copy_nonoverlapping(host_buf as *const u8, encl_buf as *mut u8, count);
    count as ssize_t
}

#[no_mangle]
pub unsafe extern "C" fn u_write_hostbuf_ocall(
    error: *mut c_int,
    host_buf: *mut c_void,
    encl_buf: *const c_void,
    count: size_t,
) -> ssize_t {
    if host_buf.is_null() || encl_buf.is_null() || count == 0 {
        set_error(error, libc::EINVAL);
        return -1;
    }

    ptr::copy_nonoverlapping(encl_buf as *const u8, host_buf as *mut u8, count);
    count as ssize_t
}
