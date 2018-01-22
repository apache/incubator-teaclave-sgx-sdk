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

use sgx_trts::libc::{c_int, c_uint, c_void};
use core::cmp;
use core::fmt;
use core::mem;
use io::{self, Error, ErrorKind};
use net::{SocketAddr, Shutdown, Ipv4Addr, Ipv6Addr};
use sys::net::{cvt, cvt_r, Socket, wrlen_t};
use sys_common::{AsInner, FromInner, IntoInner};
use time::Duration;

////////////////////////////////////////////////////////////////////////////////
// sockaddr and misc bindings
////////////////////////////////////////////////////////////////////////////////

pub fn setsockopt<T>(sock: &Socket, opt: c_int, val: c_int,
                     payload: T) -> io::Result<()> {
    unsafe {
        let payload = &payload as *const T as *const c_void;
        cvt(c::setsockopt(*sock.as_inner(), opt, val, payload,
                          mem::size_of::<T>() as c::socklen_t))?;
        Ok(())
    }
}

pub fn getsockopt<T: Copy>(sock: &Socket, opt: c_int,
                       val: c_int) -> io::Result<T> {
    unsafe {
        let mut slot: T = mem::zeroed();
        let mut len = mem::size_of::<T>() as c::socklen_t;
        cvt(c::getsockopt(*sock.as_inner(), opt, val,
                          &mut slot as *mut _ as *mut _,
                          &mut len))?;
        assert_eq!(len as usize, mem::size_of::<T>());
        Ok(slot)
    }
}

fn sockname<F>(f: F) -> io::Result<SocketAddr>
    where F: FnOnce(*mut c::sockaddr, *mut c::socklen_t) -> c_int
{
    unsafe {
        let mut storage: c::sockaddr_storage = mem::zeroed();
        let mut len = mem::size_of_val(&storage) as c::socklen_t;
        cvt(f(&mut storage as *mut _ as *mut _, &mut len))?;
        sockaddr_to_addr(&storage, len as usize)
    }
}

pub fn sockaddr_to_addr(storage: &c::sockaddr_storage,
                    len: usize) -> io::Result<SocketAddr> {
    match storage.ss_family as c_int {
        c::AF_INET => {
            assert!(len as usize >= mem::size_of::<c::sockaddr_in>());
            Ok(SocketAddr::V4(FromInner::from_inner(unsafe {
                *(storage as *const _ as *const c::sockaddr_in)
            })))
        }
        c::AF_INET6 => {
            assert!(len as usize >= mem::size_of::<c::sockaddr_in6>());
            Ok(SocketAddr::V6(FromInner::from_inner(unsafe {
                *(storage as *const _ as *const c::sockaddr_in6)
            })))
        }
        _ => {
            Err(Error::new(ErrorKind::InvalidInput, "invalid argument"))
        }
    }
}

fn to_ipv6mr_interface(value: u32) -> c_uint {
    value as c_uint
}

////////////////////////////////////////////////////////////////////////////////
// TCP streams
////////////////////////////////////////////////////////////////////////////////

pub struct TcpStream {
    inner: Socket,
}

impl TcpStream {

    pub fn new(sockfd: c_int) -> io::Result<TcpStream> {
        let sock = Socket::new(sockfd)?;
        Ok(TcpStream { inner: sock })
    }

    pub fn raw(&self) -> c_int { self.inner.raw() }

    pub fn into_raw(self) -> c_int { self.inner.into_raw() } 
    
    pub fn socket(&self) -> &Socket { &self.inner }

    pub fn into_socket(self) -> Socket { self.inner }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.inner.set_timeout(dur, c::SO_RCVTIMEO)
    }

    pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.inner.set_timeout(dur, c::SO_SNDTIMEO)
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.inner.timeout(c::SO_RCVTIMEO)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.inner.timeout(c::SO_SNDTIMEO)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.peek(buf)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let len = cmp::min(buf.len(), <wrlen_t>::max_value() as usize) as wrlen_t;
        let ret = cvt(unsafe {
            c::send(*self.inner.as_inner(),
                    buf.as_ptr() as *const c_void,
                    len,
                    c::MSG_NOSIGNAL)
        })?;
        Ok(ret as usize)
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        sockname(|buf, len| unsafe {
            c::getpeername(*self.inner.as_inner(), buf, len)
        })
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        sockname(|buf, len| unsafe {
            c::getsockname(*self.inner.as_inner(), buf, len)
        })
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.inner.shutdown(how)
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        self.inner.duplicate().map(|s| TcpStream { inner: s })
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.inner.set_nodelay(nodelay)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        self.inner.nodelay()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        setsockopt(&self.inner, c::IPPROTO_IP, c::IP_TTL, ttl as c_int)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        let raw: c_int = getsockopt(&self.inner, c::IPPROTO_IP, c::IP_TTL)?;
        Ok(raw as u32)
    }

    pub fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        setsockopt(&self.inner, c::IPPROTO_IPV6, c::IPV6_V6ONLY, only_v6 as c_int)
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        let raw: c_int = getsockopt(&self.inner, c::IPPROTO_IPV6, c::IPV6_V6ONLY)?;
        Ok(raw != 0)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.inner.take_error()
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.inner.set_nonblocking(nonblocking)
    }
}

impl FromInner<Socket> for TcpStream {
    fn from_inner(socket: Socket) -> TcpStream {
        TcpStream { inner: socket }
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = f.debug_struct("TcpStream");

        if let Ok(addr) = self.socket_addr() {
            res.field("addr", &addr);
        }

        if let Ok(peer) = self.peer_addr() {
            res.field("peer", &peer);
        }

        let name = if cfg!(windows) {"socket"} else {"fd"};
        res.field(name, &self.inner.as_inner())
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////
// UDP
////////////////////////////////////////////////////////////////////////////////

pub struct UdpSocket {
    inner: Socket,
}

impl UdpSocket {

    pub fn new(sockfd: c_int) -> io::Result<UdpSocket> {
        let sock = Socket::new(sockfd)?;
        Ok(UdpSocket { inner: sock })
    }

    pub fn raw(&self) -> c_int { self.inner.raw() }

    pub fn into_raw(self) -> c_int { self.inner.into_raw() } 

    pub fn bind(&self, addr: &SocketAddr) -> io::Result<()> {
        let (addrp, len) = addr.into_inner();
        cvt(unsafe { c::bind(*self.inner.as_inner(), addrp, len as _) }).map(|_| ())
    }

    pub fn socket(&self) -> &Socket { &self.inner }

    pub fn into_socket(self) -> Socket { self.inner }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        sockname(|buf, len| unsafe {
            c::getsockname(*self.inner.as_inner(), buf, len)
        })
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.inner.recv_from(buf)
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.inner.peek_from(buf)
    }

    pub fn send_to(&self, buf: &[u8], dst: &SocketAddr) -> io::Result<usize> {
        let len = cmp::min(buf.len(), <wrlen_t>::max_value() as usize) as wrlen_t;
        let (dstp, dstlen) = dst.into_inner();
        let ret = cvt(unsafe {
            c::sendto(*self.inner.as_inner(),
                      buf.as_ptr() as *const c_void, len,
                      c::MSG_NOSIGNAL, dstp, dstlen)
        })?;
        Ok(ret as usize)
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        self.inner.duplicate().map(|s| UdpSocket { inner: s })
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.inner.set_timeout(dur, c::SO_RCVTIMEO)
    }

    pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.inner.set_timeout(dur, c::SO_SNDTIMEO)
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.inner.timeout(c::SO_RCVTIMEO)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.inner.timeout(c::SO_SNDTIMEO)
    }

    pub fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
        setsockopt(&self.inner, c::SOL_SOCKET, c::SO_BROADCAST, broadcast as c_int)
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        let raw: c_int = getsockopt(&self.inner, c::SOL_SOCKET, c::SO_BROADCAST)?;
        Ok(raw != 0)
    }

    pub fn set_multicast_loop_v4(&self, multicast_loop_v4: bool) -> io::Result<()> {
        setsockopt(&self.inner, c::IPPROTO_IP, c::IP_MULTICAST_LOOP, multicast_loop_v4 as c_int)
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        let raw: c_int = getsockopt(&self.inner, c::IPPROTO_IP, c::IP_MULTICAST_LOOP)?;
        Ok(raw != 0)
    }

    pub fn set_multicast_ttl_v4(&self, multicast_ttl_v4: u32) -> io::Result<()> {
        setsockopt(&self.inner, c::IPPROTO_IP, c::IP_MULTICAST_TTL, multicast_ttl_v4 as c_int)
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        let raw: c_int = getsockopt(&self.inner, c::IPPROTO_IP, c::IP_MULTICAST_TTL)?;
        Ok(raw as u32)
    }

    pub fn set_multicast_loop_v6(&self, multicast_loop_v6: bool) -> io::Result<()> {
        setsockopt(&self.inner, c::IPPROTO_IPV6, c::IPV6_MULTICAST_LOOP, multicast_loop_v6 as c_int)
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        let raw: c_int = getsockopt(&self.inner, c::IPPROTO_IPV6, c::IPV6_MULTICAST_LOOP)?;
        Ok(raw != 0)
    }

    pub fn join_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr)
                         -> io::Result<()> {
        let mreq = c::ip_mreq {
            imr_multiaddr: *multiaddr.as_inner(),
            imr_interface: *interface.as_inner(),
        };
        setsockopt(&self.inner, c::IPPROTO_IP, c::IP_ADD_MEMBERSHIP, mreq)
    }

    pub fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32)
                         -> io::Result<()> {
        let mreq = c::ipv6_mreq {
            ipv6mr_multiaddr: *multiaddr.as_inner(),
            ipv6mr_interface: to_ipv6mr_interface(interface),
        };
        setsockopt(&self.inner, c::IPPROTO_IPV6, c::IPV6_ADD_MEMBERSHIP, mreq)
    }

    pub fn leave_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr)
                          -> io::Result<()> {
        let mreq = c::ip_mreq {
            imr_multiaddr: *multiaddr.as_inner(),
            imr_interface: *interface.as_inner(),
        };
        setsockopt(&self.inner, c::IPPROTO_IP, c::IP_DROP_MEMBERSHIP, mreq)
    }

    pub fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32)
                          -> io::Result<()> {
        let mreq = c::ipv6_mreq {
            ipv6mr_multiaddr: *multiaddr.as_inner(),
            ipv6mr_interface: to_ipv6mr_interface(interface),
        };
        setsockopt(&self.inner, c::IPPROTO_IPV6, c::IPV6_DROP_MEMBERSHIP, mreq)
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        setsockopt(&self.inner, c::IPPROTO_IP, c::IP_TTL, ttl as c_int)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        let raw: c_int = getsockopt(&self.inner, c::IPPROTO_IP, c::IP_TTL)?;
        Ok(raw as u32)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.inner.take_error()
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.inner.set_nonblocking(nonblocking)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.peek(buf)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let len = cmp::min(buf.len(), <wrlen_t>::max_value() as usize) as wrlen_t;
        let ret = cvt(unsafe {
            c::send(*self.inner.as_inner(),
                    buf.as_ptr() as *const c_void,
                    len,
                    c::MSG_NOSIGNAL)
        })?;
        Ok(ret as usize)
    }

    pub fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        let (addrp, len) = addr.into_inner();
        cvt_r(|| unsafe { c::connect(*self.inner.as_inner(), addrp, len) }).map(|_| ())
    }
}

impl FromInner<Socket> for UdpSocket {
    fn from_inner(socket: Socket) -> UdpSocket {
        UdpSocket { inner: socket }
    }
}

impl fmt::Debug for UdpSocket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = f.debug_struct("UdpSocket");

        if let Ok(addr) = self.socket_addr() {
            res.field("addr", &addr);
        }

        let name = if cfg!(windows) {"socket"} else {"fd"};
        res.field(name, &self.inner.as_inner())
            .finish()
    }
}

mod c {
    use sgx_types::sgx_status_t;
    use io;
    pub use sgx_trts::libc::*;

    extern "C" {

        pub fn u_net_bind_ocall(result: * mut c_int,
                                errno: * mut c_int,
                                sockfd: c_int,
                                address: * const sockaddr,
                                addrlen: socklen_t) -> sgx_status_t;

        pub fn u_net_connect_ocall(result: * mut c_int,
                                   errno: * mut c_int,
                                   sockfd: c_int,
                                   address: * const sockaddr,
                                   addrlen: socklen_t) -> sgx_status_t;

        pub fn u_net_setsockopt_ocall(result: * mut c_int,
                                      errno: * mut c_int,
                                      sockfd: c_int,
                                      level: c_int,
                                      optname: c_int,
                                      optval: * const c_void,
                                      optlen: socklen_t) -> sgx_status_t;

        pub fn u_net_getsockopt_ocall(result: * mut c_int,
                                      errno: * mut c_int,
                                      sockfd: c_int,
                                      level: c_int,
                                      optname: c_int,
                                      optval: * mut c_void,
                                      _in_optlen: socklen_t,
                                      optlen: * mut socklen_t) -> sgx_status_t;

        pub fn u_net_send_ocall(result: * mut ssize_t,
                                errno: * mut c_int,
                                sockfd: c_int,
                                buf: * const c_void,
                                len: size_t,
                                flags: c_int) -> sgx_status_t;

        pub fn u_net_sendto_ocall(result: * mut ssize_t,
                                  errno: * mut c_int,
                                  sockfd: c_int,
                                  buf: * const c_void,
                                  len: size_t,
                                  flags: c_int,
                                  addr: * const sockaddr,
                                  addrlen: socklen_t) -> sgx_status_t;

        pub fn u_net_getpeername_ocall(result: * mut c_int,
                                       errno: * mut c_int,
                                       sockfd: c_int,
                                       address: * mut sockaddr,
                                       _in_addrlen: socklen_t,
                                       addrlen: * mut socklen_t) -> sgx_status_t;

        pub fn u_net_getsockname_ocall(result: * mut c_int,
                                       errno: * mut c_int,
                                       sockfd: c_int,
                                       address: * mut sockaddr,
                                       _in_addrlen: socklen_t,
                                       addrlen: * mut socklen_t) -> sgx_status_t;
    }

    pub unsafe fn bind(sockfd: c_int, address: * const sockaddr, addrlen: socklen_t) -> c_int {
        
        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_net_bind_ocall(&mut result as * mut c_int,
                                      &mut error as * mut c_int,
                                      sockfd,
                                      address,
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

    pub unsafe fn connect(sockfd: c_int, address: * const sockaddr, addrlen: socklen_t) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_net_connect_ocall(&mut result as * mut c_int,
                                         &mut error as * mut c_int,
                                         sockfd,
                                         address,
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

    pub unsafe fn setsockopt(sockfd: c_int,
                             level: c_int,
                             optname: c_int,
                             optval: * const c_void,
                             optlen: socklen_t) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_net_setsockopt_ocall(&mut result as * mut c_int,
                                            &mut error as * mut c_int,
                                            sockfd,
                                            level,
                                            optname,
                                            optval,
                                            optlen);

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

    pub unsafe fn getsockopt(sockfd: c_int,
                             level: c_int,
                             optname: c_int,
                             optval: * mut c_void,
                             optlen: * mut socklen_t) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let in_optlen: socklen_t = if !optlen.is_null() {
            *optlen
        } else {
            0
        };
        let status = u_net_getsockopt_ocall(&mut result as * mut c_int,
                                            &mut error as * mut c_int,
                                            sockfd,
                                            level,
                                            optname,
                                            optval,
                                            in_optlen,
                                            optlen);

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

    pub unsafe fn send(sockfd: c_int, buf: * const c_void, len: size_t, flags: c_int) -> ssize_t {

        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let status = u_net_send_ocall(&mut result as * mut ssize_t,
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

    pub unsafe fn sendto(sockfd: c_int,
                         buf: * const c_void,
                         len: size_t,
                         flags: c_int,
                         addr: * const sockaddr,
                         addrlen: socklen_t) -> ssize_t {

        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let status = u_net_sendto_ocall(&mut result as * mut ssize_t,
                                        &mut error as * mut c_int,
                                        sockfd,
                                        buf,
                                        len,
                                        flags,
                                        addr,
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

    pub unsafe fn getpeername(sockfd: c_int, address: * mut sockaddr, addrlen: * mut socklen_t) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let in_addrlen: socklen_t = if !addrlen.is_null() {
            *addrlen
        } else {
            0
        };
        let status = u_net_getpeername_ocall(&mut result as * mut c_int,
                                             &mut error as * mut c_int,
                                             sockfd,
                                             address,
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

    pub unsafe fn getsockname(sockfd: c_int, address: * mut sockaddr, addrlen: * mut socklen_t) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let in_addrlen: socklen_t = if !addrlen.is_null() {
            *addrlen
        } else {
            0
        };
        let status = u_net_getsockname_ocall(&mut result as * mut c_int,
                                             &mut error as * mut c_int,
                                             sockfd,
                                             address,
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
}