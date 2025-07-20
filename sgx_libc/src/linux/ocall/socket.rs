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

use crate::linux::*;
use core::cmp;
use core::ptr;
use core::slice;
use sgx_oc::linux::ocall;
use sgx_oc::linux::ocall::SockAddr;
use sgx_trts::trts::is_within_enclave;

#[no_mangle]
pub unsafe extern "C" fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
    if let Ok(fd) = ocall::socket(domain, ty, protocol) {
        fd
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn socketpair(
    domain: c_int,
    ty: c_int,
    protocol: c_int,
    sv: *mut [c_int; 2],
) -> c_int {
    if sv.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::socketpair(domain, ty, protocol, &mut *sv).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn bind(
    sockfd: c_int,
    address: *const sockaddr,
    addrlen: socklen_t,
) -> c_int {
    if address.is_null()
        || addrlen == 0
        || !is_within_enclave(address as *const u8, addrlen as usize)
    {
        set_errno(EINVAL);
        return -1;
    }

    let sockaddr = if let Ok(sockaddr) = SockAddr::try_from_addr(&*address, addrlen) {
        sockaddr
    } else {
        set_errno(EINVAL);
        return -1;
    };

    if ocall::bind(sockfd, &sockaddr).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn accept4(
    sockfd: c_int,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
    flags: c_int,
) -> c_int {
    if addr.is_null() && !addrlen.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if !addr.is_null()
        && (addrlen.is_null()
            || *addrlen == 0
            || !is_within_enclave(addrlen as *const u8, mem::size_of::<socklen_t>())
            || !is_within_enclave(addr as *const u8, *addrlen as usize))
    {
        set_errno(EINVAL);
        return -1;
    }

    if let Ok((socket, sockaddr)) = ocall::accept4(sockfd, flags) {
        let addrlen_out = sockaddr.addr_len();
        if !addr.is_null() {
            ptr::copy_nonoverlapping(
                sockaddr.as_bytes().as_ptr(),
                addr as *mut u8,
                cmp::min(addrlen_out, *addrlen as usize),
            );
            *addrlen = addrlen_out as socklen_t;
        }
        socket
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn listen(sockfd: c_int, backlog: c_int) -> c_int {
    if ocall::listen(sockfd, backlog).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn connect(
    sockfd: c_int,
    address: *const sockaddr,
    addrlen: socklen_t,
) -> c_int {
    if address.is_null()
        || addrlen == 0
        || !is_within_enclave(address as *const u8, addrlen as usize)
    {
        set_errno(EINVAL);
        return -1;
    }

    let sockaddr = if let Ok(sockaddr) = SockAddr::try_from_addr(&*address, addrlen) {
        sockaddr
    } else {
        set_errno(EINVAL);
        return -1;
    };

    if ocall::connect(sockfd, &sockaddr).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn send(
    sockfd: c_int,
    buf: *const c_void,
    len: size_t,
    flags: c_int,
) -> ssize_t {
    if buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    let buf = slice::from_raw_parts(buf as *const u8, len);
    if let Ok(rsize) = ocall::send(sockfd, buf, flags) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn sendto(
    sockfd: c_int,
    buf: *const c_void,
    len: size_t,
    flags: c_int,
    addr: *const sockaddr,
    addrlen: socklen_t,
) -> ssize_t {
    if buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }
    let buf = slice::from_raw_parts(buf as *const u8, len);

    if addr.is_null() || addrlen == 0 || !is_within_enclave(addr as *const u8, addrlen as usize) {
        set_errno(EINVAL);
        return -1;
    }

    let sockaddr = if let Ok(sockaddr) = SockAddr::try_from_addr(&*addr, addrlen) {
        sockaddr
    } else {
        set_errno(EINVAL);
        return -1;
    };

    if let Ok(rsize) = ocall::sendto(sockfd, buf, flags, &sockaddr) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn sendmsg(sockfd: c_int, msg: *const msghdr, flags: c_int) -> ssize_t {
    if msg.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    let msghdr = ocall::MsgHdr::from_raw(&*msg);
    if let Ok(rsize) = ocall::sendmsg(sockfd, &msghdr, flags) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn recv(
    sockfd: c_int,
    buf: *mut c_void,
    len: size_t,
    flags: c_int,
) -> ssize_t {
    if buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    let buf = slice::from_raw_parts_mut(buf as *mut u8, len);
    if let Ok(rsize) = ocall::recv(sockfd, buf, flags) {
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn recvfrom(
    sockfd: c_int,
    buf: *mut c_void,
    len: size_t,
    flags: c_int,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
) -> ssize_t {
    if buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }
    let buf = slice::from_raw_parts_mut(buf as *mut u8, len);

    if addr.is_null() && !addrlen.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if !addr.is_null()
        && (addrlen.is_null()
            || *addrlen == 0
            || !is_within_enclave(addrlen as *const u8, mem::size_of::<socklen_t>())
            || !is_within_enclave(addr as *const u8, *addrlen as usize))
    {
        set_errno(EINVAL);
        return -1;
    }

    if let Ok((rsize, sockaddr)) = ocall::recvfrom(sockfd, buf, flags) {
        let addrlen_out = sockaddr.addr_len();
        if !addr.is_null() {
            ptr::copy_nonoverlapping(
                sockaddr.as_bytes().as_ptr(),
                addr as *mut u8,
                cmp::min(addrlen_out, *addrlen as usize),
            );
            *addrlen = addrlen_out as socklen_t;
        }
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn recvmsg(sockfd: c_int, msg: *mut msghdr, flags: c_int) -> ssize_t {
    if msg.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    let msg = &mut *msg;
    let mut msghdr = ocall::MsgHdrMut::from_raw(msg);
    if let Ok(rsize) = ocall::recvmsg(sockfd, &mut msghdr, flags) {
        msg.msg_namelen = msghdr.name_len;
        msg.msg_controllen = msghdr.control_len;
        msg.msg_flags = msghdr.flags.bits();
        rsize as ssize_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn setsockopt(
    sockfd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: socklen_t,
) -> c_int {
    if ocall::setsockopt(sockfd, level, optname, optval, optlen).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn getsockopt(
    sockfd: c_int,
    level: c_int,
    optname: c_int,
    optval: *mut c_void,
    optlen: *mut socklen_t,
) -> c_int {
    if ocall::getsockopt(sockfd, level, optname, optval, optlen).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn getpeername(
    sockfd: c_int,
    address: *mut sockaddr,
    addrlen: *mut socklen_t,
) -> c_int {
    if address.is_null()
        || addrlen.is_null()
        || *addrlen == 0
        || !is_within_enclave(addrlen as *const u8, mem::size_of::<socklen_t>())
        || !is_within_enclave(address as *const u8, *addrlen as usize)
    {
        set_errno(EINVAL);
        return -1;
    }

    if let Ok(sockaddr) = ocall::getpeername(sockfd) {
        let addrlen_out = sockaddr.addr_len();
        ptr::copy_nonoverlapping(
            sockaddr.as_bytes().as_ptr(),
            address as *mut u8,
            cmp::min(addrlen_out, *addrlen as usize),
        );
        *addrlen = addrlen_out as socklen_t;
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn getsockname(
    sockfd: c_int,
    address: *mut sockaddr,
    addrlen: *mut socklen_t,
) -> c_int {
    if address.is_null()
        || addrlen.is_null()
        || *addrlen == 0
        || !is_within_enclave(addrlen as *const u8, mem::size_of::<socklen_t>())
        || !is_within_enclave(address as *const u8, *addrlen as usize)
    {
        set_errno(EINVAL);
        return -1;
    }

    if let Ok(sockaddr) = ocall::getsockname(sockfd) {
        let addrlen_out = sockaddr.addr_len();
        ptr::copy_nonoverlapping(
            sockaddr.as_bytes().as_ptr(),
            address as *mut u8,
            cmp::min(addrlen_out, *addrlen as usize),
        );
        *addrlen = addrlen_out as socklen_t;
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn shutdown(sockfd: c_int, how: c_int) -> c_int {
    if ocall::shutdown(sockfd, how).is_ok() {
        0
    } else {
        -1
    }
}
