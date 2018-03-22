// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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
use libc::{self, c_int, c_void, size_t, ssize_t, c_ulong, sockaddr, socklen_t};

#[no_mangle]
pub extern "C" fn u_net_bind_ocall(error: * mut c_int,
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
pub extern "C" fn u_net_connect_ocall(error: * mut c_int,
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
pub extern "C" fn u_net_recv_ocall(error: * mut c_int,
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
pub extern "C" fn u_net_recvfrom_ocall(error: * mut c_int,
                                       sockfd: c_int,
                                       buf: * mut c_void,
                                       len: size_t,
                                       flags: c_int,
                                       src_addr: * mut sockaddr,
                                       _in_addrlen: socklen_t,
                                       addrlen: * mut socklen_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::recvfrom(sockfd, buf, len, flags, src_addr, addrlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_net_send_ocall(error: * mut c_int,
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
pub extern "C" fn u_net_sendto_ocall(error: * mut c_int,
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
pub extern "C" fn u_net_getsockopt_ocall(error: * mut c_int,
                                         sockfd: c_int,
                                         level: c_int,
                                         optname: c_int,
                                         optval: * mut c_void,
                                         _in_optlen: socklen_t,
                                         optlen: * mut socklen_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::getsockopt(sockfd, level, optname, optval, optlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_net_setsockopt_ocall(error: * mut c_int,
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
pub extern "C" fn u_net_getsockname_ocall(error: * mut c_int,
                                          sockfd: c_int,
                                          address: * mut sockaddr,
                                          _in_addrlen: socklen_t,
                                          addrlen: * mut socklen_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::getsockname(sockfd, address, addrlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_net_getpeername_ocall(error: * mut c_int,
                                          sockfd: c_int,
                                          address: * mut sockaddr,
                                          _in_addrlen: socklen_t,
                                          addrlen: * mut socklen_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::getpeername(sockfd, address, addrlen) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_net_shutdown_ocall(error: * mut c_int, sockfd: c_int, how: c_int) -> c_int {

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

#[no_mangle]
pub extern "C" fn u_net_ioctl_ocall(error: * mut c_int, fd: c_int, request: c_int, arg: * mut c_int) -> c_int {

    let mut errno = 0;
    let ret = unsafe { libc::ioctl(fd, request as c_ulong, arg) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}