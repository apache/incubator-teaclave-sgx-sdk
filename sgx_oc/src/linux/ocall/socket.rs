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

use super::*;
use core::convert::From;
use core::mem;
use sgx_types::marker::ContiguousMemory;

pub unsafe fn socket(domain: c_int, ty: c_int, protocol: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_socket_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        domain,
        ty,
        protocol,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn socketpair(
    domain: c_int,
    ty: c_int,
    protocol: c_int,
    sv: &mut [c_int; 2],
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_socketpair_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        domain,
        ty,
        protocol,
        sv as *mut [c_int; 2],
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn bind(sockfd: c_int, sock_addr: &SockAddr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let address = sock_addr.as_bytes();

    let status = u_bind_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        address.as_ptr() as *const sockaddr,
        address.len() as socklen_t,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn listen(sockfd: c_int, backlog: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_listen_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        backlog,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn accept4(sockfd: c_int, flags: c_int) -> OCallResult<(c_int, SockAddr)> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let mut storage: sockaddr_storage = mem::zeroed();
    let len_in = mem::size_of_val(&storage) as socklen_t;

    let mut len_out: socklen_t = 0;
    let status = u_accept4_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        &mut storage as *mut _ as *mut _,
        len_in, // This additional arg is just for EDL
        &mut len_out as *mut socklen_t,
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    ensure!(
        len_out <= len_in,
        ecust!("Caller should alloc enough addr buffer.")
    );

    let addr = SockAddr::try_from_storage(storage, len_out)?;
    Ok((result, addr))
}

pub unsafe fn connect(sockfd: c_int, address: &SockAddr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let address = address.as_bytes();

    let status = u_connect_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        address.as_ptr() as *const sockaddr,
        address.len() as socklen_t,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn send(sockfd: c_int, buf: &[u8], flags: c_int) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let bufsz = buf.len();
    let host_buf = HostBuffer::from_enclave_slice(buf)?;

    let status = u_send_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        sockfd,
        host_buf.as_ptr() as *const c_void,
        host_buf.len(),
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nsent = result as usize;
    ensure!(nsent <= bufsz, ecust!("Malformed return size"));
    Ok(nsent)
}

pub unsafe fn sendto(
    sockfd: c_int,
    buf: &[u8],
    flags: c_int,
    sock_addr: &SockAddr,
) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let bufsz = buf.len();
    let host_buf = HostBuffer::from_enclave_slice(buf)?;
    let addr = sock_addr.as_bytes();

    let status = u_sendto_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        sockfd,
        host_buf.as_ptr() as *const c_void,
        host_buf.len(),
        flags,
        addr.as_ptr() as *const sockaddr,
        addr.len() as socklen_t,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nsent = result as usize;
    ensure!(nsent <= bufsz, ecust!("Malformed return size"));

    Ok(nsent)
}

pub unsafe fn recv(sockfd: c_int, buf: &mut [u8], flags: c_int) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    check_trusted_enclave_buffer(buf)?;
    let bufsz = buf.len();
    let mut host_buf = HostBuffer::alloc_zeroed(bufsz)?;

    let status = u_recv_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        sockfd,
        host_buf.as_mut_ptr() as *mut c_void,
        host_buf.len(),
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nrecv = result as usize;
    ensure!(nrecv <= bufsz, ecust!("Malformed return size"));
    host_buf.to_enclave_slice(&mut buf[..nrecv])
}

pub unsafe fn recvfrom(
    sockfd: c_int,
    buf: &mut [u8],
    flags: c_int,
) -> OCallResult<(size_t, SockAddr)> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let mut storage: sockaddr_storage = mem::zeroed();
    let len_in = mem::size_of_val(&storage) as socklen_t;
    let mut len_out: socklen_t = 0;

    check_trusted_enclave_buffer(buf)?;
    let bufsz = buf.len();
    let mut host_buf = HostBuffer::alloc_zeroed(bufsz)?;

    let status = u_recvfrom_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        sockfd,
        host_buf.as_mut_ptr() as *mut c_void,
        host_buf.len(),
        flags,
        &mut storage as *mut sockaddr_storage as *mut sockaddr,
        len_in,
        &mut len_out as *mut socklen_t,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    ensure!(
        len_out <= len_in,
        ecust!("Caller should alloc enough addr buffer.")
    );

    let nrecv = result as usize;
    ensure!(nrecv <= bufsz, ecust!("Malformed return value"));

    host_buf.to_enclave_slice(&mut buf[..nrecv])?;
    let addr = SockAddr::try_from_storage(storage, len_out)?;
    Ok((nrecv, addr))
}

pub unsafe fn setsockopt(
    sockfd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: socklen_t,
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_setsockopt_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        level,
        optname,
        optval,
        optlen,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn getsockopt(
    sockfd: c_int,
    level: c_int,
    optname: c_int,
    optval: *mut c_void,
    optlen: *mut socklen_t,
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let len_in: socklen_t = if !optlen.is_null() { *optlen } else { 0 };
    let mut len_out: socklen_t = 0;

    let status = u_getsockopt_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        level,
        optname,
        optval,
        len_in,
        &mut len_out as *mut socklen_t,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    ensure!(
        len_out <= len_in,
        ecust!("Caller should alloc enough opt buffer.")
    );
    if !optlen.is_null() {
        *optlen = len_out;
    }
    Ok(())
}

pub unsafe fn getpeername(sockfd: c_int) -> OCallResult<SockAddr> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let mut storage: sockaddr_storage = mem::zeroed();
    let len_in = mem::size_of_val(&storage) as socklen_t;
    let mut len_out: socklen_t = 0;

    let status = u_getpeername_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        &mut storage as *mut sockaddr_storage as *mut sockaddr,
        len_in,
        &mut len_out as *mut socklen_t,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    ensure!(
        len_out <= len_in,
        ecust!("Caller should alloc enough addr buffer.")
    );
    SockAddr::try_from_storage(storage, len_out)
}

pub unsafe fn getsockname(sockfd: c_int) -> OCallResult<SockAddr> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let mut storage: sockaddr_storage = mem::zeroed();
    let len_in = mem::size_of_val(&storage) as socklen_t;
    let mut len_out: socklen_t = 0;

    let status = u_getsockname_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        &mut storage as *mut sockaddr_storage as *mut sockaddr,
        len_in,
        &mut len_out as *mut socklen_t,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    ensure!(
        len_out <= len_in,
        ecust!("Caller should alloc enough addr buffer.")
    );

    SockAddr::try_from_storage(storage, len_out)
}

pub unsafe fn shutdown(sockfd: c_int, how: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_shutdown_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        how,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

#[cfg_attr(feature = "extra_traits", derive(Debug))]
#[derive(Clone, Copy)]
pub enum SockAddr {
    IN4(sockaddr_in),
    IN6(sockaddr_in6),
    UN((sockaddr_un, socklen_t)),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddressKind {
    V4,
    V6,
    Pathname,
    Abstract,
    Unnamed,
}

unsafe impl ContiguousMemory for AddressKind {}
unsafe impl ContiguousMemory for SockAddr {}

impl SockAddr {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            match self {
                SockAddr::IN4(addr) => slice::from_raw_parts(
                    addr as *const _ as *const u8,
                    core::mem::size_of::<sockaddr_in>(),
                ),
                SockAddr::IN6(addr) => slice::from_raw_parts(
                    addr as *const _ as *const u8,
                    core::mem::size_of::<sockaddr_in6>(),
                ),
                SockAddr::UN((addr, len)) => {
                    slice::from_raw_parts(addr as *const _ as *const u8, *len as usize)
                }
            }
        }
    }

    pub fn kind(&self) -> AddressKind {
        match self {
            SockAddr::IN4(_) => AddressKind::V4,
            SockAddr::IN6(_) => AddressKind::V6,
            SockAddr::UN((addr, len)) => {
                if *len == 0 {
                    AddressKind::Unnamed
                } else if addr.sun_path[0] == 0 {
                    AddressKind::Abstract
                } else {
                    AddressKind::Pathname
                }
            }
        }
    }

    pub fn addr_len(&self) -> usize {
        match self {
            SockAddr::IN4(_) => core::mem::size_of::<sockaddr_in>(),
            SockAddr::IN6(_) => core::mem::size_of::<sockaddr_in6>(),
            SockAddr::UN((_, len)) => *len as usize,
        }
    }

    pub unsafe fn try_from_addr(addr: &sockaddr, len: socklen_t) -> OCallResult<SockAddr> {
        match addr.sa_family as i32 {
            AF_INET => {
                ensure!(
                    len as usize >= core::mem::size_of::<sockaddr_in>(),
                    ecust!("Malformed addr_len")
                );
                let sockaddr: *const sockaddr_in = addr as *const _ as *const sockaddr_in;
                Ok(SockAddr::from(*sockaddr))
            }
            AF_INET6 => {
                ensure!(
                    len as usize >= core::mem::size_of::<sockaddr_in6>(),
                    ecust!("Malformed addr_len")
                );
                let sockaddr: *const sockaddr_in6 = addr as *const _ as *const sockaddr_in6;
                Ok(SockAddr::from(*sockaddr))
            }
            AF_UNIX => {
                let sockaddr: *const sockaddr_un = addr as *const _ as *const sockaddr_un;
                SockAddr::try_from_un(*sockaddr, len)
            }
            _ => {
                bail!(ecust!("Unsupported family info"));
            }
        }
    }

    pub unsafe fn try_from_storage(
        storage: sockaddr_storage,
        len: socklen_t,
    ) -> OCallResult<SockAddr> {
        match storage.ss_family as i32 {
            AF_INET => {
                ensure!(
                    len as usize == core::mem::size_of::<sockaddr_in>(),
                    ecust!("Malformed addr_len")
                );
                let sockaddr: *const sockaddr_in = &storage as *const _ as *const sockaddr_in;
                Ok(SockAddr::from(*sockaddr))
            }
            AF_INET6 => {
                ensure!(
                    len as usize == core::mem::size_of::<sockaddr_in6>(),
                    ecust!("Malformed addr_len")
                );
                let sockaddr: *const sockaddr_in6 = &storage as *const _ as *const sockaddr_in6;
                Ok(SockAddr::from(*sockaddr))
            }
            AF_UNIX => {
                let sockaddr: *const sockaddr_un = &storage as *const _ as *const sockaddr_un;
                SockAddr::try_from_un(*sockaddr, len)
            }
            0 if len == 0 => {
                let sockaddr: *const sockaddr_un = &storage as *const _ as *const sockaddr_un;
                Ok(SockAddr::UN((*sockaddr, len)))
            }
            _ => {
                bail!(ecust!("Unsupported family info"));
            }
        }
    }

    pub unsafe fn try_from_un(un: sockaddr_un, len: socklen_t) -> OCallResult<SockAddr> {
        let let_out = if len == 0 && un.sun_family == 0 {
            0
        } else if un.sun_family as i32 != AF_UNIX {
            bail!(ecust!("Unsupported family info"));
        } else {
            ensure!(
                len as usize <= core::mem::size_of::<sockaddr_un>(),
                ecust!("Malformed addr_len")
            );
            len
        };
        Ok(SockAddr::UN((un, let_out)))
    }
}

impl From<sockaddr_in> for SockAddr {
    fn from(sockaddr: sockaddr_in) -> SockAddr {
        assert!(sockaddr.sin_family as i32 == AF_INET);
        SockAddr::IN4(sockaddr)
    }
}

impl From<sockaddr_in6> for SockAddr {
    fn from(sockaddr: sockaddr_in6) -> SockAddr {
        assert!(sockaddr.sin6_family as i32 == AF_INET6);
        SockAddr::IN6(sockaddr)
    }
}
