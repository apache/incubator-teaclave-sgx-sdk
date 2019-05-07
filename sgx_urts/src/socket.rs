// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::io::Error;
use libc::{self, c_int, c_void, size_t, ssize_t, sockaddr, socklen_t, msghdr};

#[no_mangle]
pub extern "C" fn u_socket_ocall(error: *mut c_int, domain: c_int, ty: c_int, protocol: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::socket(domain, ty, protocol) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_socketpair_ocall(error: *mut c_int, domain: c_int, ty: c_int, protocol: c_int, sv: *mut c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::socketpair(domain, ty, protocol, sv) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_bind_ocall(error: * mut c_int,
                               sockfd: c_int,
                               address: * const sockaddr,
                               addrlen: socklen_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::bind(sockfd, address, addrlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
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
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_accept_ocall(error: *mut c_int,
                                 sockfd: c_int,
                                 addr: *mut sockaddr,
                                 addrlen_in: socklen_t,
                                 addrlen_out: *mut socklen_t) -> c_int {
    let mut errno = 0;
    unsafe { *addrlen_out = addrlen_in };
    let ret = unsafe { libc::accept(sockfd, addr, addrlen_out) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_accept4_ocall(error: *mut c_int,
                                  sockfd: c_int,
                                  addr: *mut sockaddr,
                                  addrlen_in: socklen_t,
                                  addrlen_out: *mut socklen_t,
                                  flags: c_int) -> c_int {
    let mut errno = 0;
    unsafe { *addrlen_out = addrlen_in };
    let ret = unsafe { libc::accept4(sockfd, addr, addrlen_out, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_connect_ocall(error: * mut c_int,
                                  sockfd: c_int,
                                  address: * const sockaddr,
                                  addrlen: socklen_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::connect(sockfd, address, addrlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_recv_ocall(error: * mut c_int,
                               sockfd: c_int,
                               buf: * mut c_void,
                               len: size_t,
                               flags: c_int) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::recv(sockfd, buf, len, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_recvfrom_ocall(error: * mut c_int,
                                   sockfd: c_int,
                                   buf: * mut c_void,
                                   len: size_t,
                                   flags: c_int,
                                   src_addr: * mut sockaddr,
                                   addrlen_in: socklen_t,
                                   addrlen_out: * mut socklen_t) -> ssize_t {
    let mut errno = 0;
    unsafe { *addrlen_out = addrlen_in };
    let ret = unsafe { libc::recvfrom(sockfd, buf, len, flags, src_addr, addrlen_out) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_recvmsg_ocall(error: * mut c_int,
                                  sockfd: c_int,
                                  msg: * mut msghdr,
                                  flags: c_int) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::recvmsg(sockfd, msg, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_send_ocall(error: * mut c_int,
                               sockfd: c_int,
                               buf: * const c_void,
                               len: size_t,
                               flags: c_int) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::send(sockfd, buf, len, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_sendto_ocall(error: * mut c_int,
                                 sockfd: c_int,
                                 buf: * const c_void,
                                 len: size_t,
                                 flags: c_int,
                                 dest_addr: * const sockaddr,
                                 addrlen: socklen_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::sendto(sockfd, buf, len, flags, dest_addr, addrlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_sendmsg_ocall(error: * mut c_int,
                                  sockfd: c_int,
                                  msg: * const msghdr,
                                  flags: c_int) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::sendmsg(sockfd, msg, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_getsockopt_ocall(error: * mut c_int,
                                     sockfd: c_int,
                                     level: c_int,
                                     optname: c_int,
                                     optval: * mut c_void,
                                     optlen_in: socklen_t,
                                     optlen_out: * mut socklen_t) -> c_int {
    let mut errno = 0;
    unsafe { *optlen_out = optlen_in };
    let ret = unsafe { libc::getsockopt(sockfd, level, optname, optval, optlen_out) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_setsockopt_ocall(error: * mut c_int,
                                     sockfd: c_int,
                                     level: c_int,
                                     optname: c_int,
                                     optval: * const c_void,
                                     optlen: socklen_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::setsockopt(sockfd, level, optname, optval, optlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_getsockname_ocall(error: * mut c_int,
                                      sockfd: c_int,
                                      address: * mut sockaddr,
                                      addrlen_in: socklen_t,
                                      addrlen_out: * mut socklen_t) -> c_int {
    let mut errno = 0;
    unsafe { *addrlen_out = addrlen_in };
    let ret = unsafe { libc::getsockname(sockfd, address, addrlen_out) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_getpeername_ocall(error: * mut c_int,
                                      sockfd: c_int,
                                      address: * mut sockaddr,
                                      addrlen_in: socklen_t,
                                      addrlen_out: * mut socklen_t) -> c_int {
    let mut errno = 0;
    unsafe { *addrlen_out = addrlen_in };
    let ret = unsafe { libc::getpeername(sockfd, address, addrlen_out) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_shutdown_ocall(error: * mut c_int, sockfd: c_int, how: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::shutdown(sockfd, how) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}