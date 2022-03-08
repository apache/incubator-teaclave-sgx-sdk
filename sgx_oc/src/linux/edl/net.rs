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

s! {
    pub struct addr_info {
        pub ai_flags: c_int,
        pub ai_family: c_int,
        pub ai_socktype: c_int,
        pub ai_protocol: c_int,
        pub ai_addrlen: c_uint,
        pub ai_addr: sockaddr_storage,
    }
}

extern "C" {
    pub fn u_getaddrinfo_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        node: *const c_char,
        service: *const c_char,
        hints: *const addrinfo,
        addrinfo: *mut addr_info,
        in_count: size_t,
        out_count: *mut size_t,
    ) -> sgx_status_t;
}
