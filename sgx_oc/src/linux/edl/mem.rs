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
    pub fn u_malloc_ocall(
        result: *mut *mut c_void,
        error: *mut c_int,
        size: size_t,
        align: size_t,
        zeroed: c_int,
    ) -> sgx_status_t;
    pub fn u_free_ocall(p: *mut c_void) -> sgx_status_t;
    pub fn u_mmap_ocall(
        result: *mut *mut c_void,
        error: *mut c_int,
        start: *mut c_void,
        length: size_t,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: off_t,
    ) -> sgx_status_t;
    pub fn u_munmap_ocall(
        result: *mut c_int,
        error: *mut c_int,
        start: *mut c_void,
        length: size_t,
    ) -> sgx_status_t;
    pub fn u_msync_ocall(
        result: *mut c_int,
        error: *mut c_int,
        addr: *mut c_void,
        length: size_t,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_mprotect_ocall(
        result: *mut c_int,
        error: *mut c_int,
        addr: *mut c_void,
        length: size_t,
        prot: c_int,
    ) -> sgx_status_t;
}
