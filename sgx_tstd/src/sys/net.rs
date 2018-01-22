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

#![allow(dead_code)]

use sgx_trts::libc::{c_int, size_t, c_void};
use core::mem;
use io;
use net::{SocketAddr, Shutdown};
use sys::fd::FileDesc;
use sys_common::{AsInner, FromInner, IntoInner};
use sys_common::net::{getsockopt, setsockopt, sockaddr_to_addr};
use time::Duration;

pub use sys::{cvt, cvt_r};

pub type wrlen_t = size_t;

pub struct Socket(FileDesc);

impl Socket {

    pub fn new(sockfd: c_int) -> io::Result<Socket> {
        let fd = FileDesc::new(sockfd);
        fd.set_cloexec()?;
        Ok(Socket(fd))
    }

    pub fn raw(&self) -> c_int { self.0.raw() }

    pub fn into_raw(self) -> c_int { self.0.into_raw() }

    pub fn duplicate(&self) -> io::Result<Socket> {
        self.0.duplicate().map(Socket)
    }

    fn recv_with_flags(&self, buf: &mut [u8], flags: c_int) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::recv(self.0.raw(),
                       buf.as_mut_ptr() as *mut c_void,
                       buf.len(),
                       flags)
        })?;
        Ok(ret as usize)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv_with_flags(buf, 0)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv_with_flags(buf, libc::MSG_PEEK)
    }

    fn recv_from_with_flags(&self, buf: &mut [u8], flags: c_int)
                            -> io::Result<(usize, SocketAddr)> {
        let mut storage: libc::sockaddr_storage = unsafe { mem::zeroed() };
        let mut addrlen = mem::size_of_val(&storage) as libc::socklen_t;

        let n = cvt(unsafe {
            libc::recvfrom(self.0.raw(),
                        buf.as_mut_ptr() as *mut c_void,
                        buf.len(),
                        flags,
                        &mut storage as *mut _ as *mut _,
                        &mut addrlen)
        })?;
        Ok((n as usize, sockaddr_to_addr(&storage, addrlen as usize)?))
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_with_flags(buf, 0)
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_with_flags(buf, libc::MSG_PEEK)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn set_timeout(&self, dur: Option<Duration>, kind: c_int) -> io::Result<()> {
        let timeout = match dur {
            Some(dur) => {
                if dur.as_secs() == 0 && dur.subsec_nanos() == 0 {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                              "cannot set a 0 duration timeout"));
                }

                let secs = if dur.as_secs() > libc::time_t::max_value() as u64 {
                    libc::time_t::max_value()
                } else {
                    dur.as_secs() as libc::time_t
                };
                let mut timeout = libc::timeval {
                    tv_sec: secs,
                    tv_usec: (dur.subsec_nanos() / 1000) as libc::suseconds_t,
                };
                if timeout.tv_sec == 0 && timeout.tv_usec == 0 {
                    timeout.tv_usec = 1;
                }
                timeout
            }
            None => {
                libc::timeval {
                    tv_sec: 0,
                    tv_usec: 0,
                }
            }
        };
        setsockopt(self, libc::SOL_SOCKET, kind, timeout)
    }

    pub fn timeout(&self, kind: c_int) -> io::Result<Option<Duration>> {
        let raw: libc::timeval = getsockopt(self, libc::SOL_SOCKET, kind)?;
        if raw.tv_sec == 0 && raw.tv_usec == 0 {
            Ok(None)
        } else {
            let sec = raw.tv_sec as u64;
            let nsec = (raw.tv_usec as u32) * 1000;
            Ok(Some(Duration::new(sec, nsec)))
        }
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        let how = match how {
            Shutdown::Write => libc::SHUT_WR,
            Shutdown::Read => libc::SHUT_RD,
            Shutdown::Both => libc::SHUT_RDWR,
        };
        cvt(unsafe { libc::shutdown(self.0.raw(), how) })?;
        Ok(())
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        setsockopt(self, libc::IPPROTO_TCP, libc::TCP_NODELAY, nodelay as c_int)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        let raw: c_int = getsockopt(self, libc::IPPROTO_TCP, libc::TCP_NODELAY)?;
        Ok(raw != 0)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        let mut nonblocking = nonblocking as c_int;
        cvt(unsafe { libc::ioctl(*self.as_inner(), libc::FIONBIO, &mut nonblocking) }).map(|_| ())
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        let raw: c_int = getsockopt(self, libc::SOL_SOCKET, libc::SO_ERROR)?;
        if raw == 0 {
            Ok(None)
        } else {
            Ok(Some(io::Error::from_raw_os_error(raw as i32)))
        }
    }
}

impl AsInner<c_int> for Socket {
    fn as_inner(&self) -> &c_int { self.0.as_inner() }
}

impl FromInner<c_int> for Socket {
    fn from_inner(fd: c_int) -> Socket { Socket(FileDesc::new(fd)) }
}

impl IntoInner<c_int> for Socket {
    fn into_inner(self) -> c_int { self.0.into_raw() }
}

mod libc {
    use sgx_types::sgx_status_t;
    use io;
    pub use sgx_trts::libc::*;

    extern "C" {
        pub fn u_net_recv_ocall(result: * mut ssize_t,
                                errno: * mut c_int,
                                sockfd: c_int,
                                buf: * mut c_void,
                                len: size_t,
                                flags: c_int) -> sgx_status_t;

        pub fn u_net_recvfrom_ocall(result: * mut ssize_t,
                                    errno: * mut c_int,
                                    sockfd: c_int,
                                    buf: * mut c_void,
                                    len: size_t,
                                    flags: c_int,
                                    addr: * mut sockaddr,
                                    _in_addrlen: socklen_t,
                                    addrlen: * mut socklen_t) -> sgx_status_t;

        pub fn u_net_shutdown_ocall(result: * mut c_int,
                                    errno: * mut c_int,
                                    sockfd: c_int,
                                    how: c_int) -> sgx_status_t;

        pub fn u_net_ioctl_ocall(result: * mut c_int,
                                 errno: * mut c_int,
                                 fd: c_int,
                                 request: c_int,
                                 arg: * mut c_int) -> sgx_status_t;

    }

    pub unsafe fn recv(sockfd: c_int, buf: * mut c_void, len: size_t, flags: c_int) -> ssize_t {
        
        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let status = u_net_recv_ocall(&mut result as * mut ssize_t,
                                      &mut error as * mut c_int,
                                      sockfd,
                                      buf,
                                      len,
                                      flags);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn recvfrom(sockfd: c_int,
                           buf: * mut c_void,
                           len: size_t,
                           flags: c_int,
                           addr: * mut sockaddr,
                           addrlen: * mut socklen_t) -> ssize_t {

        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let in_addrlen: socklen_t = if !addrlen.is_null() {
            *addrlen
        } else {
            0
        };

        let status = u_net_recvfrom_ocall(&mut result as * mut ssize_t,
                                          &mut error as * mut c_int,
                                          sockfd,
                                          buf,
                                          len,
                                          flags,
                                          addr,
                                          in_addrlen,
                                          addrlen);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn shutdown(sockfd: c_int, how: c_int) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_net_shutdown_ocall(&mut result as * mut c_int,
                                          &mut error as * mut c_int,
                                          sockfd,
                                          how);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn ioctl(fd: c_int, request: c_int, arg: * mut c_int) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_net_ioctl_ocall(&mut result as * mut c_int,
                                       &mut error as * mut c_int,
                                       fd,
                                       request,
                                       arg);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }
}