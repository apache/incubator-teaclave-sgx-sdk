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
    pub fn u_poll_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fds: *mut pollfd,
        nfds: nfds_t,
        timeout: c_int,
    ) -> sgx_status_t;
    pub fn u_epoll_create1_ocall(
        result: *mut c_int,
        error: *mut c_int,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_epoll_ctl_ocall(
        result: *mut c_int,
        error: *mut c_int,
        epfd: c_int,
        op: c_int,
        fd: c_int,
        event: *mut epoll_event,
    ) -> sgx_status_t;
    pub fn u_epoll_wait_ocall(
        result: *mut c_int,
        error: *mut c_int,
        epfd: c_int,
        events: *mut epoll_event,
        maxevents: c_uint,
        timeout: c_int,
    ) -> sgx_status_t;
}
