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
use libc::{self, c_int, c_void, iovec, msghdr, size_t, sockaddr, socklen_t, ssize_t};
use std::io::Error;

#[no_mangle]
pub unsafe extern "C" fn u_socket_ocall(
    error: *mut c_int,
    domain: c_int,
    ty: c_int,
    protocol: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::socket(domain, ty, protocol);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_socketpair_ocall(
    error: *mut c_int,
    domain: c_int,
    ty: c_int,
    protocol: c_int,
    sv: *mut [c_int; 2],
) -> c_int {
    let mut errno = 0;
    let ret = libc::socketpair(domain, ty, protocol, sv as *mut c_int);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_bind_ocall(
    error: *mut c_int,
    sockfd: c_int,
    address: *const sockaddr,
    addrlen: socklen_t,
) -> c_int {
    let mut errno = 0;
    let ret = libc::bind(sockfd, address, addrlen);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_listen_ocall(error: *mut c_int, sockfd: c_int, backlog: c_int) -> c_int {
    let mut errno = 0;
    let ret = libc::listen(sockfd, backlog);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_accept4_ocall(
    error: *mut c_int,
    sockfd: c_int,
    addr: *mut sockaddr,
    addrlen_in: socklen_t,
    addrlen_out: *mut socklen_t,
    flags: c_int,
) -> c_int {
    let mut errno = 0;
    *addrlen_out = addrlen_in;
    let ret = libc::accept4(sockfd, addr, addrlen_out, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_connect_ocall(
    error: *mut c_int,
    sockfd: c_int,
    address: *const sockaddr,
    addrlen: socklen_t,
) -> c_int {
    let mut errno = 0;
    let ret = libc::connect(sockfd, address, addrlen);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_send_ocall(
    error: *mut c_int,
    sockfd: c_int,
    buf: *const c_void,
    len: size_t,
    flags: c_int,
) -> ssize_t {
    let mut errno = 0;
    let ret = libc::send(sockfd, buf, len, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_sendto_ocall(
    error: *mut c_int,
    sockfd: c_int,
    buf: *const c_void,
    len: size_t,
    flags: c_int,
    addr: *const sockaddr,
    addrlen: socklen_t,
) -> ssize_t {
    let mut errno = 0;
    let ret = libc::sendto(sockfd, buf, len, flags, addr, addrlen);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_sendmsg_ocall(
    error: *mut c_int,
    sockfd: c_int,
    msg_name: *mut c_void,
    msg_namelen: socklen_t,
    msg_iov: *mut iovec,
    msg_iovlen: usize,
    msg_control: *mut c_void,
    msg_controllen: usize,
    flags: c_int,
) -> ssize_t {
    let mut errno = 0;
    let msg = msghdr {
        msg_name,
        msg_namelen,
        msg_iov,
        msg_iovlen,
        msg_control,
        msg_controllen,
        msg_flags: 0,
    };
    let ret = libc::sendmsg(sockfd, &msg, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_recv_ocall(
    error: *mut c_int,
    sockfd: c_int,
    buf: *mut c_void,
    len: size_t,
    flags: c_int,
) -> ssize_t {
    let mut errno = 0;
    let ret = libc::recv(sockfd, buf, len, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_recvfrom_ocall(
    error: *mut c_int,
    sockfd: c_int,
    buf: *mut c_void,
    len: size_t,
    flags: c_int,
    addr: *mut sockaddr,
    addrlen_in: socklen_t,
    addrlen_out: *mut socklen_t,
) -> ssize_t {
    let mut errno = 0;
    *addrlen_out = addrlen_in;
    let ret = libc::recvfrom(sockfd, buf, len, flags, addr, addrlen_out);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_recvmsg_ocall(
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
) -> ssize_t {
    if msg_namelen_out.is_null() || msg_controllen_out.is_null() || msg_flags.is_null() {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let mut errno = 0;
    let mut msg = msghdr {
        msg_name,
        msg_namelen,
        msg_iov,
        msg_iovlen,
        msg_control,
        msg_controllen,
        msg_flags: 0,
    };
    let ret = libc::recvmsg(sockfd, &mut msg, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    } else {
        *msg_namelen_out = msg.msg_namelen;
        *msg_controllen_out = msg.msg_controllen;
        *msg_flags = msg.msg_flags;
    }

    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_setsockopt_ocall(
    error: *mut c_int,
    sockfd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: socklen_t,
) -> c_int {
    let mut errno = 0;
    let ret = libc::setsockopt(sockfd, level, optname, optval, optlen);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_getsockopt_ocall(
    error: *mut c_int,
    sockfd: c_int,
    level: c_int,
    optname: c_int,
    optval: *mut c_void,
    optlen_in: socklen_t,
    optlen_out: *mut socklen_t,
) -> c_int {
    let mut errno = 0;
    *optlen_out = optlen_in;
    let ret = libc::getsockopt(sockfd, level, optname, optval, optlen_out);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_getpeername_ocall(
    error: *mut c_int,
    sockfd: c_int,
    address: *mut sockaddr,
    addrlen_in: socklen_t,
    addrlen_out: *mut socklen_t,
) -> c_int {
    let mut errno = 0;
    *addrlen_out = addrlen_in;
    let ret = libc::getpeername(sockfd, address, addrlen_out);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_getsockname_ocall(
    error: *mut c_int,
    sockfd: c_int,
    address: *mut sockaddr,
    addrlen_in: socklen_t,
    addrlen_out: *mut socklen_t,
) -> c_int {
    let mut errno = 0;
    *addrlen_out = addrlen_in;
    let ret = libc::getsockname(sockfd, address, addrlen_out);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_shutdown_ocall(error: *mut c_int, sockfd: c_int, how: c_int) -> c_int {
    let mut errno = 0;
    let ret = libc::shutdown(sockfd, how);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}
