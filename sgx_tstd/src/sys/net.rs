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

#![allow(dead_code)]

use sgx_trts::libc::{c_int, size_t, c_void};
use core::mem;
use core::cmp;
use io;
use net::{SocketAddr, Shutdown};
use sys::fd::FileDesc;
use sys_common::{AsInner, FromInner, IntoInner};
use sys_common::net::{getsockopt, setsockopt, sockaddr_to_addr};
use time::{Duration, Instant};
use untrusted::time::InstantEx;
pub use sys::{cvt, cvt_r};

pub type wrlen_t = size_t;

pub struct Socket(FileDesc);

impl Socket {

    pub fn new(sockfd: c_int) -> io::Result<Socket> {
        let fd = FileDesc::new(sockfd);
        fd.set_cloexec()?;
        Ok(Socket(fd))
    }

    // YU: Added to support net
    //     TcpListener::bind -> Socket::new
    //     I renamed new as new_socket_addr_type for better naming
    pub fn new_socket_addr_type(addr: &SocketAddr, ty: c_int) -> io::Result<Socket> {
        let fam = match *addr {
            SocketAddr::V4(..) => libc::AF_INET,
            SocketAddr::V6(..) => libc::AF_INET6,
        };
        Socket::new_raw(fam, ty)
    }

    // YU: Added to support net
    //     TcpListener::bind -> Socket::new -> Socket::new_raw
    //     Naming seems good
    pub fn new_raw(fam: c_int, ty: c_int) -> io::Result<Socket> {
        unsafe {
            // On linux we first attempt to pass the SOCK_CLOEXEC flag to
            // atomically create the socket and set it as CLOEXEC. Support for
            // this option, however, was added in 2.6.27, and we still support
            // 2.6.18 as a kernel, so if the returned error is EINVAL we
            // fallthrough to the fallback.
            match cvt(libc::socket(fam, ty | libc::SOCK_CLOEXEC, 0)) {
                Ok(fd) => return Ok(Socket(FileDesc::new(fd))),
                Err(ref e) if e.raw_os_error() == Some(libc::EINVAL) => {}
                Err(e) => return Err(e),
            }

            let fd = cvt(libc::socket(fam, ty, 0))?;
            let fd = FileDesc::new(fd);
            fd.set_cloexec()?;
            let socket = Socket(fd);

            Ok(socket)
        }
    }

    pub fn new_pair(fam: c_int, ty: c_int) -> io::Result<(Socket, Socket)> {
        unsafe {
            let mut fds = [0, 0];

            // Like above, see if we can set cloexec atomically
            match cvt(libc::socketpair(fam, ty | libc::SOCK_CLOEXEC, 0, fds.as_mut_ptr())) {
                Ok(_) => {
                    return Ok((Socket(FileDesc::new(fds[0])), Socket(FileDesc::new(fds[1]))));
                }
                Err(ref e) if e.raw_os_error() == Some(libc::EINVAL) => {},
                Err(e) => return Err(e),
            }

            cvt(libc::socketpair(fam, ty, 0, fds.as_mut_ptr()))?;
            let a = FileDesc::new(fds[0]);
            let b = FileDesc::new(fds[1]);
            a.set_cloexec()?;
            b.set_cloexec()?;
            Ok((Socket(a), Socket(b)))
        }
    }

    pub fn connect_timeout(&self, addr: &SocketAddr, timeout: Duration) -> io::Result<()> {
        self.set_nonblocking(true)?;
        let r = unsafe {
            let (addrp, len) = addr.into_inner();
            cvt(libc::connect(self.0.raw(), addrp, len))
        };
        self.set_nonblocking(false)?;

        match r {
            Ok(_) => return Ok(()),
            // there's no ErrorKind for EINPROGRESS :(
            Err(ref e) if e.raw_os_error() == Some(libc::EINPROGRESS) => {}
            Err(e) => return Err(e),
        }

        let mut pollfd = libc::pollfd {
            fd: self.0.raw(),
            events: libc::POLLOUT,
            revents: 0,
        };

        if timeout.as_secs() == 0 && timeout.subsec_nanos() == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                      "cannot set a 0 duration timeout"));
        }

        let start = Instant::now();

        loop {
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "connection timed out"));
            }

            let timeout = timeout - elapsed;
            let mut timeout = timeout.as_secs()
                .saturating_mul(1_000)
                .saturating_add(timeout.subsec_nanos() as u64 / 1_000_000);
            if timeout == 0 {
                timeout = 1;
            }

            let timeout = cmp::min(timeout, c_int::max_value() as u64) as c_int;

            match unsafe { libc::poll(&mut pollfd, 1, timeout) } {
                -1 => {
                    let err = io::Error::last_os_error();
                    if err.kind() != io::ErrorKind::Interrupted {
                        return Err(err);
                    }
                }
                0 => {}
                _ => {
                    // linux returns POLLOUT|POLLERR|POLLHUP for refused connections (!), so look
                    // for POLLHUP rather than read readiness
                    if pollfd.revents & libc::POLLHUP != 0 {
                        let e = self.take_error()?
                            .unwrap_or_else(|| {
                                io::Error::new(io::ErrorKind::Other, "no error set after POLLHUP")
                            });
                        return Err(e);
                    }

                    return Ok(());
                }
            }
        }
    }

    // YU: Added to support net
    //     TcpListener::accept -> Socket::accept
    //     Naming seems good
    //     We don't support linux kernel < 2.6.28.
    //     So we only use accept4
    // Attention:
    //     this function is a blocking function, which make an OCALL
    //     and block itself **in the untrusted OS**. This is very much
    //     dangerous and is misleading.
    //     In SGX programming, execution is by default in enclave and
    //     cannot leak information by design. Howeverm, this function
    //     is not. It leaks events.
    //     This function is guarded by feature `net2` and should only
    //     be used on demand.
    pub fn accept(&self, storage: *mut libc::sockaddr, len: *mut libc::socklen_t) -> io::Result<Socket> {
        let res = cvt_r(|| unsafe {
            libc::accept4(self.0.raw(), storage, len, libc::SOCK_CLOEXEC)
        });
        match res {
            Ok(fd) => Ok(Socket(FileDesc::new(fd))),
            Err(e) => Err(e),
        }
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
        cvt(unsafe { libc::ioctl_arg1(*self.as_inner(), libc::FIONBIO, &mut nonblocking) }).map(|_| ())
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
    pub use sgx_trts::libc::*;
    pub use sgx_trts::libc::ocall::{socket, socketpair, connect, accept4, recv, recvfrom, shutdown,
                                    ioctl_arg1, poll};
}