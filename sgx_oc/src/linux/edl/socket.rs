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
    pub fn u_socket_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        domain: c_int,
        ty: c_int,
        protocol: c_int,
    ) -> sgx_status_t;
    pub fn u_socketpair_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        domain: c_int,
        ty: c_int,
        protocol: c_int,
        sv: *mut [c_int; 2],
    ) -> sgx_status_t;
    pub fn u_bind_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        address: *const sockaddr,
        addrlen: socklen_t,
    ) -> sgx_status_t;
    pub fn u_listen_ocall(
        result: *mut c_int,
        error: *mut c_int,
        sockfd: c_int,
        backlog: c_int,
    ) -> sgx_status_t;
    pub fn u_accept4_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        addr: *mut sockaddr,
        addrlen_in: socklen_t,
        addrlen_out: *mut socklen_t,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_connect_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        address: *const sockaddr,
        addrlen: socklen_t,
    ) -> sgx_status_t;
    pub fn u_send_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        sockfd: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_sendto_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        sockfd: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        addr: *const sockaddr,
        addrlen: socklen_t,
    ) -> sgx_status_t;
    pub fn u_sendmsg_ocall(
        result: *mut ssize_t,
        error: *mut c_int,
        sockfd: c_int,
        msg_name: *const c_void,
        msg_namelen: socklen_t,
        msg_iov: *const iovec,
        msg_iovlen: usize,
        msg_control: *const c_void,
        msg_controllen: usize,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_recv_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        sockfd: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_recvfrom_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        sockfd: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
        addr: *mut sockaddr,
        addrlen_in: socklen_t,
        addrlen_out: *mut socklen_t,
    ) -> sgx_status_t;
    pub fn u_recvmsg_ocall(
        result: *mut ssize_t,
        error: *mut c_int,
        sockfd: c_int,
        msg_name: *mut c_void,
        msg_namelen: socklen_t,
        msg_namelen_out: *mut socklen_t,
        msg_iov: *mut iovec,
        msg_iovlen: usize,
        msg_control: *mut c_void,
        msg_controllen: usize,
        msg_controllen_out: *mut usize,
        msg_flags: *mut c_int,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_setsockopt_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        level: c_int,
        optname: c_int,
        optval: *const c_void,
        optlen: socklen_t,
    ) -> sgx_status_t;
    pub fn u_getsockopt_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        level: c_int,
        optname: c_int,
        optval: *mut c_void,
        optlen_in: socklen_t,
        optlen_out: *mut socklen_t,
    ) -> sgx_status_t;
    pub fn u_getpeername_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        address: *mut sockaddr,
        addrlen_in: socklen_t,
        addrlen_out: *mut socklen_t,
    ) -> sgx_status_t;
    pub fn u_getsockname_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        address: *mut sockaddr,
        addrlen_in: socklen_t,
        addrlen_out: *mut socklen_t,
    ) -> sgx_status_t;
    pub fn u_shutdown_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        how: c_int,
    ) -> sgx_status_t;
}
