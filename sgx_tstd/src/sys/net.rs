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

#![allow(dead_code)]
use crate::io::{self, IoSlice, IoSliceMut};
use crate::net::{Shutdown, SocketAddr};
use crate::sys::fd::FileDesc;
pub use crate::sys::{cvt, cvt_ocall, cvt_ocall_r, cvt_r};
use crate::sys_common::net::{getsockopt, setsockopt};
use crate::sys_common::{AsInner, FromInner, IntoInner};
use crate::time::{Duration, Instant};
#[cfg(not(feature = "untrusted_time"))]
use crate::untrusted::time::InstantEx;
use core::cmp;
use core::convert::TryInto;
use sgx_trts::libc::{c_int, size_t, PolledOk};

pub type wrlen_t = size_t;

pub struct Socket(FileDesc);

pub fn init() {}

pub fn cvt_gai(err: c_int) -> io::Result<()> {
    if err == 0 {
        return Ok(());
    }

    // We may need to trigger a glibc workaround. See on_resolver_failure() for details.
    // on_resolver_failure();

    if err == libc::EAI_SYSTEM {
        return Err(io::Error::last_os_error());
    }

    let detail = libc::gai_error_str(err);
    Err(io::Error::new(
        io::ErrorKind::Other,
        &format!("failed to lookup address information: {}", detail)[..],
    ))
}

impl Socket {
    pub fn new(sockfd: c_int) -> io::Result<Socket> {
        let fd = FileDesc::new(sockfd);
        fd.set_cloexec()?;
        Ok(Socket(fd))
    }

    pub fn new_socket_addr_type(addr: &SocketAddr, ty: c_int) -> io::Result<Socket> {
        let fam = match *addr {
            SocketAddr::V4(..) => libc::AF_INET,
            SocketAddr::V6(..) => libc::AF_INET6,
        };
        Socket::new_raw(fam, ty)
    }

    pub fn new_raw(fam: c_int, ty: c_int) -> io::Result<Socket> {
        unsafe {
            // On linux we first attempt to pass the SOCK_CLOEXEC flag to
            // atomically create the socket and set it as CLOEXEC. Support for
            // this option, however, was added in 2.6.27, and we still support
            // 2.6.18 as a kernel, so if the returned error is EINVAL we
            // fallthrough to the fallback.
            match libc::socket(fam, ty | libc::SOCK_CLOEXEC, 0) {
                Ok(fd) => return Ok(Socket(FileDesc::new(fd))),
                Err(ref e) if e.equal_to_os_error(libc::EINVAL) => {}
                Err(e) => return cvt_ocall(Err(e)),
            }

            let fd = cvt_ocall(libc::socket(fam, ty, 0))?;
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
            match libc::socketpair(fam, ty | libc::SOCK_CLOEXEC, 0, &mut fds) {
                Ok(_) => {
                    return Ok((Socket(FileDesc::new(fds[0])), Socket(FileDesc::new(fds[1]))));
                }
                Err(ref e) if e.equal_to_os_error(libc::EINVAL) => {}
                Err(e) => return cvt_ocall(Err(e)),
            }

            cvt_ocall(libc::socketpair(fam, ty, 0, &mut fds))?;
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
            let addr = addr.to_owned().into();
            libc::connect(self.0.raw(), &addr)
        };
        self.set_nonblocking(false)?;

        match r {
            Ok(_) => return Ok(()),
            // there's no ErrorKind for EINPROGRESS :(
            Err(ref e) if e.equal_to_os_error(libc::EINPROGRESS) => {}
            Err(e) => return cvt_ocall(Err(e)),
        }

        let mut pollfds = [libc::pollfd {
            fd: self.0.raw(),
            events: libc::POLLOUT,
            revents: 0,
        }];

        if timeout.as_secs() == 0 && timeout.subsec_nanos() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot set a 0 duration timeout",
            ));
        }

        let start = Instant::now();

        loop {
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "connection timed out",
                ));
            }

            let timeout = timeout - elapsed;
            let mut timeout = timeout
                .as_secs()
                .saturating_mul(1_000)
                .saturating_add(timeout.subsec_nanos() as u64 / 1_000_000);
            if timeout == 0 {
                timeout = 1;
            }

            let timeout = cmp::min(timeout, c_int::max_value() as u64) as c_int;

            match unsafe { cvt_ocall(libc::poll(&mut pollfds, timeout))? } {
                PolledOk::TimeLimitExpired => {}
                PolledOk::ReadyDescsCount(_) => {
                    // linux returns POLLOUT|POLLERR|POLLHUP for refused connections (!), so look
                    // for POLLHUP rather than read readiness
                    if pollfds[0].revents & libc::POLLHUP != 0 {
                        let e = self.take_error()?.unwrap_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "no error set after POLLHUP")
                        });
                        return Err(e);
                    }
                    return Ok(());
                }
            }
        }
    }

    // Attention:
    // this function is a blocking function, which make an OCALL
    // and block itself **in the untrusted OS**. This is very much
    // dangerous and is misleading.
    // In SGX programming, execution is by default in enclave and
    // cannot leak information by design. Howeverm, this function
    // is not. It leaks events.
    // This function is guarded by feature `net` and should only
    // be used on demand.
    // We don't support linux kernel < 2.6.28. So we only use accept4.
    pub fn accept(&self) -> io::Result<(Socket, libc::SockAddr)> {
        let (fd, addr) =
            cvt_ocall_r(|| unsafe { libc::accept4(self.0.raw(), libc::SOCK_CLOEXEC) })?;
        Ok((Socket(FileDesc::new(fd)), addr))
    }

    pub fn raw(&self) -> c_int {
        self.0.raw()
    }

    pub fn into_raw(self) -> c_int {
        self.0.into_raw()
    }

    pub fn duplicate(&self) -> io::Result<Socket> {
        self.0.duplicate().map(Socket)
    }

    fn recv_with_flags(&self, buf: &mut [u8], flags: c_int) -> io::Result<usize> {
        let ret = cvt_ocall(unsafe { libc::recv(self.0.raw(), buf, flags) })?;
        Ok(ret as usize)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv_with_flags(buf, 0)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv_with_flags(buf, libc::MSG_PEEK)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    fn recv_from_with_flags(
        &self,
        buf: &mut [u8],
        flags: c_int,
    ) -> io::Result<(usize, SocketAddr)> {
        let (n, addr) = cvt_ocall(unsafe { libc::recvfrom(self.0.raw(), buf, flags) })?;
        Ok((n, addr.try_into()?))
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

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    pub fn set_timeout(&self, dur: Option<Duration>, kind: c_int) -> io::Result<()> {
        let timeout = match dur {
            Some(dur) => {
                if dur.as_secs() == 0 && dur.subsec_nanos() == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "cannot set a 0 duration timeout",
                    ));
                }

                let secs = if dur.as_secs() > libc::time_t::max_value() as u64 {
                    libc::time_t::max_value()
                } else {
                    dur.as_secs() as libc::time_t
                };
                let mut timeout = libc::timeval {
                    tv_sec: secs,
                    tv_usec: dur.subsec_micros() as libc::suseconds_t,
                };
                if timeout.tv_sec == 0 && timeout.tv_usec == 0 {
                    timeout.tv_usec = 1;
                }
                timeout
            }
            None => libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
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
        cvt_ocall(unsafe { libc::shutdown(self.0.raw(), how) })?;
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
        cvt_ocall(unsafe { libc::ioctl_arg1(*self.as_inner(), libc::FIONBIO, &mut nonblocking) })
            .map(drop)
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
    fn as_inner(&self) -> &c_int {
        self.0.as_inner()
    }
}

impl FromInner<c_int> for Socket {
    fn from_inner(fd: c_int) -> Socket {
        Socket(FileDesc::new(fd))
    }
}

impl IntoInner<c_int> for Socket {
    fn into_inner(self) -> c_int {
        self.0.into_raw()
    }
}

mod libc {
    pub use sgx_trts::libc::ocall::{
        accept4, connect, ioctl_arg1, poll, recv, recvfrom, shutdown, socket,
        socketpair,
    };
    pub use sgx_trts::libc::*;
}
