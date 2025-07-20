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

use super::sgx_status_t;
use crate::linux::x86_64::*;

extern "C" {
    pub fn u_read_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        fd: c_int,
        buf: *mut c_void,
        count: size_t,
    ) -> sgx_status_t;
    pub fn u_pread64_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        fd: c_int,
        buf: *mut c_void,
        count: size_t,
        offset: off64_t,
    ) -> sgx_status_t;
    pub fn u_write_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        fd: c_int,
        buf: *const c_void,
        count: size_t,
    ) -> sgx_status_t;
    pub fn u_pwrite64_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        fd: c_int,
        buf: *const c_void,
        count: size_t,
        offset: off64_t,
    ) -> sgx_status_t;
    pub fn u_sendfile_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        out_fd: c_int,
        in_fd: c_int,
        offset: *mut off_t,
        count: size_t,
    ) -> sgx_status_t;
    pub fn u_copy_file_range_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        fd_in: c_int,
        off_in: *mut off64_t,
        fd_out: c_int,
        off_out: *mut off64_t,
        len: size_t,
        flags: c_uint,
    ) -> sgx_status_t;
    pub fn u_splice_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        fd_in: c_int,
        off_in: *mut off64_t,
        fd_out: c_int,
        off_out: *mut off64_t,
        len: size_t,
        flags: c_uint,
    ) -> sgx_status_t;
    pub fn u_fcntl_arg0_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fd: c_int,
        cmd: c_int,
    ) -> sgx_status_t;
    pub fn u_fcntl_arg1_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fd: c_int,
        cmd: c_int,
        arg: c_int,
    ) -> sgx_status_t;
    pub fn u_ioctl_arg0_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fd: c_int,
        request: c_ulong,
    ) -> sgx_status_t;
    pub fn u_ioctl_arg1_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fd: c_int,
        request: c_ulong,
        arg: *mut c_int,
    ) -> sgx_status_t;
    pub fn u_close_ocall(result: *mut c_int, errno: *mut c_int, fd: c_int) -> sgx_status_t;
    pub fn u_isatty_ocall(result: *mut c_int, errno: *mut c_int, fd: c_int) -> sgx_status_t;
    pub fn u_dup_ocall(result: *mut c_int, errno: *mut c_int, oldfd: c_int) -> sgx_status_t;
    pub fn u_eventfd_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        initval: c_uint,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_futimens_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fd: c_int,
        times: *const [timespec; 2],
    ) -> sgx_status_t;
}
