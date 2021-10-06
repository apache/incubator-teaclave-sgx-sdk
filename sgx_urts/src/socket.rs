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

use libc::{self, c_int, c_void, iovec, msghdr, size_t, sockaddr, socklen_t, ssize_t};
use std::io::Error;

#[no_mangle]
pub extern "C" fn u_socket_ocall(
    error: *mut c_int,
    domain: c_int,
    ty: c_int,
    protocol: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::socket(domain, ty, protocol) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_socketpair_ocall(
    error: *mut c_int,
    domain: c_int,
    ty: c_int,
    protocol: c_int,
    sv: *mut c_int,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::socketpair(domain, ty, protocol, sv) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_bind_ocall(
    error: *mut c_int,
    sockfd: c_int,
    address: *const sockaddr,
    addrlen: socklen_t,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::bind(sockfd, address, addrlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_listen_ocall(error: *mut c_int, sockfd: c_int, backlog: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::listen(sockfd, backlog) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_accept_ocall(
    error: *mut c_int,
    sockfd: c_int,
    addr: *mut sockaddr,
    addrlen_in: socklen_t,
    addrlen_out: *mut socklen_t,
) -> c_int {
    let mut errno = 0;
    unsafe { *addrlen_out = addrlen_in };
    let ret = unsafe { libc::accept(sockfd, addr, addrlen_out) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_accept4_ocall(
    error: *mut c_int,
    sockfd: c_int,
    addr: *mut sockaddr,
    addrlen_in: socklen_t,
    addrlen_out: *mut socklen_t,
    flags: c_int,
) -> c_int {
    let mut errno = 0;
    unsafe { *addrlen_out = addrlen_in };
    let ret = unsafe { libc::accept4(sockfd, addr, addrlen_out, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_connect_ocall(
    error: *mut c_int,
    sockfd: c_int,
    address: *const sockaddr,
    addrlen: socklen_t,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::connect(sockfd, address, addrlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_recv_ocall(
    error: *mut c_int,
    sockfd: c_int,
    buf: *mut c_void,
    len: size_t,
    flags: c_int,
) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::recv(sockfd, buf, len, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_recvfrom_ocall(
    error: *mut c_int,
    sockfd: c_int,
    buf: *mut c_void,
    len: size_t,
    flags: c_int,
    src_addr: *mut sockaddr,
    addrlen_in: socklen_t,
    addrlen_out: *mut socklen_t,
) -> ssize_t {
    let mut errno = 0;
    unsafe { *addrlen_out = addrlen_in };
    let ret = unsafe { libc::recvfrom(sockfd, buf, len, flags, src_addr, addrlen_out) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_recvmsg_ocall(
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
        if !error.is_null() {
            unsafe {
                *error = libc::EINVAL;
            }
        }
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
    let ret = unsafe { libc::recvmsg(sockfd, &mut msg, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    } else {
        unsafe {
            *msg_namelen_out = msg.msg_namelen;
            *msg_controllen_out = msg.msg_controllen;
            *msg_flags = msg.msg_flags;
        }
    }

    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_send_ocall(
    error: *mut c_int,
    sockfd: c_int,
    buf: *const c_void,
    len: size_t,
    flags: c_int,
) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::send(sockfd, buf, len, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_sendto_ocall(
    error: *mut c_int,
    sockfd: c_int,
    buf: *const c_void,
    len: size_t,
    flags: c_int,
    dest_addr: *const sockaddr,
    addrlen: socklen_t,
) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::sendto(sockfd, buf, len, flags, dest_addr, addrlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_sendmsg_ocall(
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
    let ret = unsafe { libc::sendmsg(sockfd, &msg, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_getsockopt_ocall(
    error: *mut c_int,
    sockfd: c_int,
    level: c_int,
    optname: c_int,
    optval: *mut c_void,
    optlen_in: socklen_t,
    optlen_out: *mut socklen_t,
) -> c_int {
    let mut errno = 0;
    unsafe { *optlen_out = optlen_in };
    let ret = unsafe { libc::getsockopt(sockfd, level, optname, optval, optlen_out) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_setsockopt_ocall(
    error: *mut c_int,
    sockfd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: socklen_t,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::setsockopt(sockfd, level, optname, optval, optlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_getsockname_ocall(
    error: *mut c_int,
    sockfd: c_int,
    address: *mut sockaddr,
    addrlen_in: socklen_t,
    addrlen_out: *mut socklen_t,
) -> c_int {
    let mut errno = 0;
    unsafe { *addrlen_out = addrlen_in };
    let ret = unsafe { libc::getsockname(sockfd, address, addrlen_out) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_getpeername_ocall(
    error: *mut c_int,
    sockfd: c_int,
    address: *mut sockaddr,
    addrlen_in: socklen_t,
    addrlen_out: *mut socklen_t,
) -> c_int {
    let mut errno = 0;
    unsafe { *addrlen_out = addrlen_in };
    let ret = unsafe { libc::getpeername(sockfd, address, addrlen_out) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_shutdown_ocall(error: *mut c_int, sockfd: c_int, how: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::shutdown(sockfd, how) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}
