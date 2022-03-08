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

use super::sgx_status_t;
use crate::linux::x86_64::*;

extern "C" {
    pub fn u_getuid_ocall(result: *mut uid_t) -> sgx_status_t;
    pub fn u_getgid_ocall(result: *mut gid_t) -> sgx_status_t;
    pub fn u_env_ocall(
        result: *mut ssize_t,
        error: *mut c_int,
        buf: *mut c_uchar,
        bufsz: size_t,
    ) -> sgx_status_t;
    pub fn u_args_ocall(
        result: *mut ssize_t,
        error: *mut c_int,
        buf: *mut c_uchar,
        bufsz: size_t,
    ) -> sgx_status_t;
    pub fn u_getcwd_ocall(
        result: *mut c_int,
        error: *mut c_int,
        buf: *mut c_char,
        bufsz: size_t,
    ) -> sgx_status_t;
    pub fn u_chdir_ocall(result: *mut c_int, error: *mut c_int, dir: *const c_char)
        -> sgx_status_t;
}
