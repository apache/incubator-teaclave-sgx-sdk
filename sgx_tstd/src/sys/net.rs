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

use crate::cmp;
use crate::io::{self, BorrowedBuf, BorrowedCursor, IoSlice, IoSliceMut};
use crate::mem::MaybeUninit;
use crate::net::{Shutdown, SocketAddr};
use crate::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, RawFd};
use crate::sys::fd::FileDesc;
use crate::sys_common::net::{getsockopt, setsockopt};
use crate::sys_common::{AsInner, FromInner, IntoInner, TryIntoInner};
use crate::time::{Duration, Instant};
#[cfg(not(feature = "untrusted_time"))]
use crate::untrusted::time::InstantEx;

use sgx_oc::ocall::{PolledOk, SockAddr};
use sgx_oc::{c_int, size_t, EAI_SYSTEM, MSG_PEEK};

pub use crate::sys::{cvt_ocall, cvt_ocall_r};

pub type wrlen_t = size_t;

pub struct Socket(FileDesc);

pub fn init() {}

pub fn cvt_gai(err: c_int) -> io::Result<()> {
    if err == 0 {
        return Ok(());
    }

    if err == EAI_SYSTEM {
        return Err(io::Error::last_os_error());
    }

    let detail = libc::gai_error_string(err);
    Err(io::Error::new(
        io::ErrorKind::Uncategorized,
        &format!("failed to lookup address information: {detail}")[..],
    ))
}

impl Socket {
    pub fn new(addr: &SocketAddr, ty: c_int) -> io::Result<Socket> {
        let fam = match *addr {
            SocketAddr::V4(..) => libc::AF_INET,
            SocketAddr::V6(..) => libc::AF_INET6,
        };
        Socket::new_raw(fam, ty)
    }

    pub fn new_raw(fam: c_int, ty: c_int) -> io::Result<Socket> {
        unsafe {
            // On platforms that support it we pass the SOCK_CLOEXEC
            // flag to atomically create the socket and set it as
            // CLOEXEC. On Linux this was added in 2.6.27.
            let fd = cvt_ocall(libc::socket(fam, ty | libc::SOCK_CLOEXEC, 0))?;
            Ok(Socket(FileDesc::from_raw_fd(fd)))
        }
    }

    pub fn new_pair(fam: c_int, ty: c_int) -> io::Result<(Socket, Socket)> {
        unsafe {
            let mut fds = [0, 0];

            // Like above, set cloexec atomically
            cvt_ocall(libc::socketpair(fam, ty | libc::SOCK_CLOEXEC, 0, &mut fds))?;
            Ok((Socket(FileDesc::from_raw_fd(fds[0])), Socket(FileDesc::from_raw_fd(fds[1]))))
        }
    }

    pub fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        let addr: SockAddr = addr.to_owned().into_inner();
        loop {
            let result = unsafe { cvt_ocall(libc::connect(self.as_raw_fd(), &addr)) };
            if result.is_err() {
                let err = crate::sys::os::errno();
                match err {
                    libc::EINTR => continue,
                    libc::EISCONN => return Ok(()),
                    _ => return Err(io::Error::from_raw_os_error(err)),
                }
            }
            return Ok(());
        }
    }

    pub fn connect_timeout(&self, addr: &SocketAddr, timeout: Duration) -> io::Result<()> {
        self.set_nonblocking(true)?;
        let r = unsafe {
            let addr = addr.to_owned().into_inner();
            cvt_ocall(libc::connect(self.as_raw_fd(), &addr))
        };
        self.set_nonblocking(false)?;

        match r {
            Ok(_) => return Ok(()),
            // there's no ErrorKind for EINPROGRESS :(
            Err(ref e) if e.raw_os_error() == Some(libc::EINPROGRESS) => {}
            Err(e) => return Err(e),
        }

        let mut pollfds = [libc::pollfd { fd: self.as_raw_fd(), events: libc::POLLOUT, revents: 0 }];

        if timeout.as_secs() == 0 && timeout.subsec_nanos() == 0 {
            return Err(io::const_io_error!(
                io::ErrorKind::InvalidInput,
                "cannot set a 0 duration timeout",
            ));
        }

        let start = Instant::now();

        loop {
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Err(io::const_io_error!(io::ErrorKind::TimedOut, "connection timed out"));
            }

            let timeout = timeout - elapsed;
            let mut timeout = timeout
                .as_secs()
                .saturating_mul(1_000)
                .saturating_add(timeout.subsec_nanos() as u64 / 1_000_000);
            if timeout == 0 {
                timeout = 1;
            }

            let timeout = cmp::min(timeout, c_int::MAX as u64) as c_int;

            match unsafe { cvt_ocall(libc::poll(&mut pollfds, timeout)) } {
                Ok(polled_ok) => match polled_ok {
                    PolledOk::TimeLimitExpired => {}
                    PolledOk::ReadyFdCount(_) => {
                        // linux returns POLLOUT|POLLERR|POLLHUP for refused connections (!), so look
                        // for POLLHUP rather than read readiness
                        if pollfds[0].revents & libc::POLLHUP != 0 {
                            let e = self.take_error()?.unwrap_or_else(|| {
                            io::const_io_error!(
                                io::ErrorKind::Uncategorized,
                                "no error set after POLLHUP",
                                )
                            });
                            return Err(e);
                        }
                        return Ok(());
                    }
                }
                Err(ref e) if e.is_interrupted() => {}
                Err(e) => return Err(e),
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
    pub fn accept(&self) -> io::Result<(Socket, SockAddr)> {
        unsafe {
            let (fd, addr) = cvt_ocall_r(|| libc::accept4(self.as_raw_fd(), libc::SOCK_CLOEXEC))?;
            Ok((Socket(FileDesc::from_raw_fd(fd)), addr))
        }
    }

    pub fn duplicate(&self) -> io::Result<Socket> {
        self.0.duplicate().map(Socket)
    }

    fn recv_with_flags(&self, mut buf: BorrowedCursor<'_>, flags: c_int) -> io::Result<()> {
        let ret = cvt_ocall(unsafe {
            libc::recv(
                self.as_raw_fd(),
                MaybeUninit::slice_assume_init_mut(buf.as_mut()),
                flags,
            )
        })?;
        unsafe {
            buf.advance(ret as usize);
        }
        Ok(())
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut buf = BorrowedBuf::from(buf);
        self.recv_with_flags(buf.unfilled(), 0)?;
        Ok(buf.len())
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut buf = BorrowedBuf::from(buf);
        self.recv_with_flags(buf.unfilled(), MSG_PEEK)?;
        Ok(buf.len())
    }

    pub fn read_buf(&self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.recv_with_flags(buf, 0)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    fn recv_from_with_flags(
        &self,
        buf: &mut [u8],
        flags: c_int,
    ) -> io::Result<(usize, SocketAddr)> {
        let (n, addr) = cvt_ocall(unsafe { libc::recvfrom(self.as_raw_fd(), buf, flags) })?;
        Ok((n, addr.try_into_inner()?))
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_with_flags(buf, 0)
    }

    pub fn recv_msg(&self, msg: &mut libc::MsgHdrMut) -> io::Result<usize> {
        let n = cvt_ocall(unsafe { libc::recvmsg(self.as_raw_fd(), msg, libc::MSG_CMSG_CLOEXEC) })?;
        Ok(n as usize)
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_with_flags(buf, MSG_PEEK)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    pub fn send_msg(&self, msg: &libc::MsgHdr) -> io::Result<usize> {
        let n = cvt_ocall(unsafe { libc::sendmsg(self.as_raw_fd(), msg, 0) })?;
        Ok(n as usize)
    }

    pub fn set_timeout(&self, dur: Option<Duration>, kind: c_int) -> io::Result<()> {
        let timeout = match dur {
            Some(dur) => {
                if dur.as_secs() == 0 && dur.subsec_nanos() == 0 {
                    return Err(io::const_io_error!(
                        io::ErrorKind::InvalidInput,
                        "cannot set a 0 duration timeout",
                    ));
                }

                let secs = if dur.as_secs() > libc::time_t::MAX as u64 {
                    libc::time_t::MAX
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
            None => libc::timeval { tv_sec: 0, tv_usec: 0 },
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
        cvt_ocall(unsafe { libc::shutdown(self.as_raw_fd(), how) })?;
        Ok(())
    }

    pub fn set_linger(&self, linger: Option<Duration>) -> io::Result<()> {
        let linger = libc::linger {
            l_onoff: linger.is_some() as c_int,
            l_linger: linger.unwrap_or_default().as_secs() as c_int,
        };

        setsockopt(self, libc::SOL_SOCKET, libc::SO_LINGER, linger)
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        let val: libc::linger = getsockopt(self, libc::SOL_SOCKET, libc::SO_LINGER)?;

        Ok((val.l_onoff != 0).then(|| Duration::from_secs(val.l_linger as u64)))
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        setsockopt(self, libc::IPPROTO_TCP, libc::TCP_NODELAY, nodelay as c_int)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        let raw: c_int = getsockopt(self, libc::IPPROTO_TCP, libc::TCP_NODELAY)?;
        Ok(raw != 0)
    }

    pub fn set_quickack(&self, quickack: bool) -> io::Result<()> {
        setsockopt(self, libc::IPPROTO_TCP, libc::TCP_QUICKACK, quickack as c_int)
    }

    pub fn quickack(&self) -> io::Result<bool> {
        let raw: c_int = getsockopt(self, libc::IPPROTO_TCP, libc::TCP_QUICKACK)?;
        Ok(raw != 0)
    }

    pub fn set_passcred(&self, passcred: bool) -> io::Result<()> {
        setsockopt(self, libc::SOL_SOCKET, libc::SO_PASSCRED, passcred as c_int)
    }

    pub fn passcred(&self) -> io::Result<bool> {
        let passcred: c_int = getsockopt(self, libc::SOL_SOCKET, libc::SO_PASSCRED)?;
        Ok(passcred != 0)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        let mut nonblocking = nonblocking as c_int;
        cvt_ocall(unsafe { libc::ioctl_arg1(self.as_raw_fd(), libc::FIONBIO as u64, &mut nonblocking) }).map(drop)
    }

    pub fn set_mark(&self, mark: u32) -> io::Result<()> {
        let option = libc::SO_MARK;
        setsockopt(self, libc::SOL_SOCKET, option, mark as libc::c_int)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        let raw: c_int = getsockopt(self, libc::SOL_SOCKET, libc::SO_ERROR)?;
        if raw == 0 { Ok(None) } else { Ok(Some(io::Error::from_raw_os_error(raw))) }
    }

    // This is used by sys_common code to abstract over Windows and Unix.
    pub fn as_raw(&self) -> RawFd {
        self.as_raw_fd()
    }
}

impl AsInner<FileDesc> for Socket {
    #[inline]
    fn as_inner(&self) -> &FileDesc {
        &self.0
    }
}

impl IntoInner<FileDesc> for Socket {
    fn into_inner(self) -> FileDesc {
        self.0
    }
}

impl FromInner<FileDesc> for Socket {
    fn from_inner(file_desc: FileDesc) -> Self {
        Self(file_desc)
    }
}

impl AsFd for Socket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for Socket {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for Socket {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for Socket {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(FromRawFd::from_raw_fd(raw_fd))
    }
}

mod libc {
    pub use sgx_oc::ocall::{
        accept4, connect, gai_error_string, ioctl_arg1, poll, recv, recvfrom, recvmsg, sendmsg,
        shutdown, socket, socketpair, MsgHdr, MsgHdrMut,
    };
    pub use sgx_oc::*;
}
