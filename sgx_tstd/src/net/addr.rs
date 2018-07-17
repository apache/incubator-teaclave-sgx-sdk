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

use sgx_trts::libc as c;
use core::fmt;
use core::hash;
use core::mem;
use core::option;
use core::iter;
use alloc_crate::vec;
use alloc_crate::slice;
use io;
use net::{ntoh, hton, IpAddr, Ipv4Addr, Ipv6Addr};
use sys_common::{FromInner, AsInner, IntoInner};

/// An internet socket address, either IPv4 or IPv6.
///
/// Internet socket addresses consist of an [IP address], a 16-bit port number, as well
/// as possibly some version-dependent additional information. See [`SocketAddrV4`]'s and
/// [`SocketAddrV6`]'s respective documentation for more details.
///
/// The size of a `SocketAddr` instance may vary depending on the target operating
/// system.
///
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SocketAddr {
    /// An IPv4 socket address.
    V4(SocketAddrV4),
    /// An IPv6 socket address.
    V6(SocketAddrV6),
}

/// An IPv4 socket address.
///
/// IPv4 socket addresses consist of an [IPv4 address] and a 16-bit port number, as
/// stated in [IETF RFC 793].
///
/// See [`SocketAddr`] for a type encompassing both IPv4 and IPv6 socket addresses.
///
#[derive(Copy)]
pub struct SocketAddrV4 { inner: c::sockaddr_in }

/// An IPv6 socket address.
///
/// IPv6 socket addresses consist of an [Ipv6 address], a 16-bit port number, as well
/// as fields containing the traffic class, the flow label, and a scope identifier
/// (see [IETF RFC 2553, Section 3.3] for more details).
///
/// See [`SocketAddr`] for a type encompassing both IPv4 and IPv6 socket addresses.
///
/// The size of a `SocketAddrV6` struct may vary depending on the target operating
/// system.
///
#[derive(Copy)]
pub struct SocketAddrV6 { inner: c::sockaddr_in6 }

impl SocketAddr {
    /// Creates a new socket address from an [IP address] and a port number.
    ///
    pub fn new(ip: IpAddr, port: u16) -> SocketAddr {
        match ip {
            IpAddr::V4(a) => SocketAddr::V4(SocketAddrV4::new(a, port)),
            IpAddr::V6(a) => SocketAddr::V6(SocketAddrV6::new(a, port, 0, 0)),
        }
    }

    /// Returns the IP address associated with this socket address.
    ///
    pub fn ip(&self) -> IpAddr {
        match *self {
            SocketAddr::V4(ref a) => IpAddr::V4(*a.ip()),
            SocketAddr::V6(ref a) => IpAddr::V6(*a.ip()),
        }
    }

    /// Changes the IP address associated with this socket address.
    ///
    pub fn set_ip(&mut self, new_ip: IpAddr) {
        // `match (*self, new_ip)` would have us mutate a copy of self only to throw it away.
        match (self, new_ip) {
            (&mut SocketAddr::V4(ref mut a), IpAddr::V4(new_ip)) => a.set_ip(new_ip),
            (&mut SocketAddr::V6(ref mut a), IpAddr::V6(new_ip)) => a.set_ip(new_ip),
            (self_, new_ip) => *self_ = Self::new(new_ip, self_.port()),
        }
    }

    /// Returns the port number associated with this socket address.
    ///
    pub fn port(&self) -> u16 {
        match *self {
            SocketAddr::V4(ref a) => a.port(),
            SocketAddr::V6(ref a) => a.port(),
        }
    }

    /// Changes the port number associated with this socket address.
    ///
    pub fn set_port(&mut self, new_port: u16) {
        match *self {
            SocketAddr::V4(ref mut a) => a.set_port(new_port),
            SocketAddr::V6(ref mut a) => a.set_port(new_port),
        }
    }

    /// Returns [`true`] if the [IP address] in this `SocketAddr` is an
    /// [IPv4 address], and [`false`] otherwise.
    ///
    pub fn is_ipv4(&self) -> bool {
        match *self {
            SocketAddr::V4(_) => true,
            SocketAddr::V6(_) => false,
        }
    }

    /// Returns [`true`] if the [IP address] in this `SocketAddr` is an
    /// [IPv6 address], and [`false`] otherwise.
    ///
    pub fn is_ipv6(&self) -> bool {
        match *self {
            SocketAddr::V4(_) => false,
            SocketAddr::V6(_) => true,
        }
    }
}

impl SocketAddrV4 {
    /// Creates a new socket address from an [IPv4 address] and a port number.
    ///
    pub fn new(ip: Ipv4Addr, port: u16) -> SocketAddrV4 {
        SocketAddrV4 {
            inner: c::sockaddr_in {
                sin_family: c::AF_INET as c::sa_family_t,
                sin_port: hton(port),
                sin_addr: *ip.as_inner(),
                .. unsafe { mem::zeroed() }
            },
        }
    }

    /// Returns the IP address associated with this socket address.
    ///
    pub fn ip(&self) -> &Ipv4Addr {
        unsafe {
            &*(&self.inner.sin_addr as *const c::in_addr as *const Ipv4Addr)
        }
    }

    /// Changes the IP address associated with this socket address.
    ///
    pub fn set_ip(&mut self, new_ip: Ipv4Addr) {
        self.inner.sin_addr = *new_ip.as_inner()
    }

    /// Returns the port number associated with this socket address.
    ///
    pub fn port(&self) -> u16 {
        ntoh(self.inner.sin_port)
    }

    /// Changes the port number associated with this socket address.
    ///
    pub fn set_port(&mut self, new_port: u16) {
        self.inner.sin_port = hton(new_port);
    }
}

impl SocketAddrV6 {
    /// Creates a new socket address from an [IPv6 address], a 16-bit port number,
    /// and the `flowinfo` and `scope_id` fields.
    ///
    /// For more information on the meaning and layout of the `flowinfo` and `scope_id`
    /// parameters, see [IETF RFC 2553, Section 3.3].
    ///
    pub fn new(ip: Ipv6Addr, port: u16, flowinfo: u32, scope_id: u32)
               -> SocketAddrV6 {
        SocketAddrV6 {
            inner: c::sockaddr_in6 {
                sin6_family: c::AF_INET6 as c::sa_family_t,
                sin6_port: hton(port),
                sin6_addr: *ip.as_inner(),
                sin6_flowinfo: flowinfo,
                sin6_scope_id: scope_id,
                .. unsafe { mem::zeroed() }
            },
        }
    }

    /// Returns the IP address associated with this socket address.
    ///
    pub fn ip(&self) -> &Ipv6Addr {
        unsafe {
            &*(&self.inner.sin6_addr as *const c::in6_addr as *const Ipv6Addr)
        }
    }

    /// Changes the IP address associated with this socket address.
    ///
    pub fn set_ip(&mut self, new_ip: Ipv6Addr) {
        self.inner.sin6_addr = *new_ip.as_inner()
    }

    /// Returns the port number associated with this socket address.
    ///
    pub fn port(&self) -> u16 {
        ntoh(self.inner.sin6_port)
    }

    /// Changes the port number associated with this socket address.
    ///
    pub fn set_port(&mut self, new_port: u16) {
        self.inner.sin6_port = hton(new_port);
    }

    /// Returns the flow information associated with this address.
    ///
    /// This information corresponds to the `sin6_flowinfo` field in C's `netinet/in.h`,
    /// as specified in [IETF RFC 2553, Section 3.3].
    /// It combines information about the flow label and the traffic class as specified
    /// in [IETF RFC 2460], respectively [Section 6] and [Section 7].
    ///
    pub fn flowinfo(&self) -> u32 {
        self.inner.sin6_flowinfo
    }

    /// Changes the flow information associated with this socket address.
    ///
    /// See the [`flowinfo`] method's documentation for more details.
    ///
    /// [`flowinfo`]: #method.flowinfo
    ///
    pub fn set_flowinfo(&mut self, new_flowinfo: u32) {
        self.inner.sin6_flowinfo = new_flowinfo;
    }

    /// Returns the scope ID associated with this address.
    ///
    /// This information corresponds to the `sin6_scope_id` field in C's `netinet/in.h`,
    /// as specified in [IETF RFC 2553, Section 3.3].
    ///
    pub fn scope_id(&self) -> u32 {
        self.inner.sin6_scope_id
    }

    /// Change the scope ID associated with this socket address.
    ///
    /// See the [`scope_id`] method's documentation for more details.
    ///
    /// [`scope_id`]: #method.scope_id
    ///
    pub fn set_scope_id(&mut self, new_scope_id: u32) {
        self.inner.sin6_scope_id = new_scope_id;
    }
}

impl FromInner<c::sockaddr_in> for SocketAddrV4 {
    fn from_inner(addr: c::sockaddr_in) -> SocketAddrV4 {
        SocketAddrV4 { inner: addr }
    }
}

impl FromInner<c::sockaddr_in6> for SocketAddrV6 {
    fn from_inner(addr: c::sockaddr_in6) -> SocketAddrV6 {
        SocketAddrV6 { inner: addr }
    }
}

impl From<SocketAddrV4> for SocketAddr {
    fn from(sock4: SocketAddrV4) -> SocketAddr {
        SocketAddr::V4(sock4)
    }
}

impl From<SocketAddrV6> for SocketAddr {
    fn from(sock6: SocketAddrV6) -> SocketAddr {
        SocketAddr::V6(sock6)
    }
}

impl<I: Into<IpAddr>> From<(I, u16)> for SocketAddr {
    fn from(pieces: (I, u16)) -> SocketAddr {
        SocketAddr::new(pieces.0.into(), pieces.1)
    }
}

impl<'a> IntoInner<(*const c::sockaddr, c::socklen_t)> for &'a SocketAddr {
    fn into_inner(self) -> (*const c::sockaddr, c::socklen_t) {
        match *self {
            SocketAddr::V4(ref a) => {
                (a as *const _ as *const _, mem::size_of_val(a) as c::socklen_t)
            }
            SocketAddr::V6(ref a) => {
                (a as *const _ as *const _, mem::size_of_val(a) as c::socklen_t)
            }
        }
    }
}

impl fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SocketAddr::V4(ref a) => a.fmt(f),
            SocketAddr::V6(ref a) => a.fmt(f),
        }
    }
}

impl fmt::Display for SocketAddrV4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.ip(), self.port())
    }
}

impl fmt::Debug for SocketAddrV4 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl fmt::Display for SocketAddrV6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]:{}", self.ip(), self.port())
    }
}

impl fmt::Debug for SocketAddrV6 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl Clone for SocketAddrV4 {
    fn clone(&self) -> SocketAddrV4 { *self }
}

impl Clone for SocketAddrV6 {
    fn clone(&self) -> SocketAddrV6 { *self }
}

impl PartialEq for SocketAddrV4 {
    fn eq(&self, other: &SocketAddrV4) -> bool {
        self.inner.sin_port == other.inner.sin_port &&
            self.inner.sin_addr.s_addr == other.inner.sin_addr.s_addr
    }
}

impl PartialEq for SocketAddrV6 {
    fn eq(&self, other: &SocketAddrV6) -> bool {
        self.inner.sin6_port == other.inner.sin6_port &&
            self.inner.sin6_addr.s6_addr == other.inner.sin6_addr.s6_addr &&
            self.inner.sin6_flowinfo == other.inner.sin6_flowinfo &&
            self.inner.sin6_scope_id == other.inner.sin6_scope_id
    }
}

impl Eq for SocketAddrV4 {}

impl Eq for SocketAddrV6 {}

impl hash::Hash for SocketAddrV4 {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        (self.inner.sin_port, self.inner.sin_addr.s_addr).hash(s)
    }
}

impl hash::Hash for SocketAddrV6 {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        (self.inner.sin6_port, &self.inner.sin6_addr.s6_addr,
         self.inner.sin6_flowinfo, self.inner.sin6_scope_id).hash(s)
    }
}

/// A trait for objects which can be converted or resolved to one or more
/// [`SocketAddr`] values.
///
/// This trait is used for generic address resolution when constructing network
/// objects.  By default it is implemented for the following types:
///
///  * [`SocketAddr`]: [`to_socket_addrs`] is the identity function.
///
///  * [`SocketAddrV4`], [`SocketAddrV6`], `(`[`IpAddr`]`, `[`u16`]`)`,
///    `(`[`Ipv4Addr`]`, `[`u16`]`)`, `(`[`Ipv6Addr`]`, `[`u16`]`)`:
///    [`to_socket_addrs`] constructs a [`SocketAddr`] trivially.
///
///  * `(`[`&str`]`, `[`u16`]`)`: the string should be either a string representation
///    of an [`IpAddr`] address as expected by [`FromStr`] implementation or a host
///    name.
///
///  * [`&str`]: the string should be either a string representation of a
///    [`SocketAddr`] as expected by its [`FromStr`] implementation or a string like
///    `<host_name>:<port>` pair where `<port>` is a [`u16`] value.
///
/// This trait allows constructing network objects like [`TcpStream`] or
/// [`UdpSocket`] easily with values of various types for the bind/connection
/// address. It is needed because sometimes one type is more appropriate than
/// the other: for simple uses a string like `"localhost:12345"` is much nicer
/// than manual construction of the corresponding [`SocketAddr`], but sometimes
/// [`SocketAddr`] value is *the* main source of the address, and converting it to
/// some other type (e.g. a string) just for it to be converted back to
/// [`SocketAddr`] in constructor methods is pointless.
///
/// Addresses returned by the operating system that are not IP addresses are
/// silently ignored.
///
pub trait ToSocketAddrs {
    /// Returned iterator over socket addresses which this type may correspond
    /// to.
    type Iter: Iterator<Item=SocketAddr>;

    /// Converts this object to an iterator of resolved `SocketAddr`s.
    ///
    /// The returned iterator may not actually yield any values depending on the
    /// outcome of any resolution performed.
    ///
    /// Note that this function may block the current thread while resolution is
    /// performed.
    fn to_socket_addrs(&self) -> io::Result<Self::Iter>;
}

impl ToSocketAddrs for SocketAddr {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        Ok(Some(*self).into_iter())
    }
}

impl ToSocketAddrs for SocketAddrV4 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V4(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for SocketAddrV6 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V6(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        match ip {
            IpAddr::V4(ref a) => (*a, port).to_socket_addrs(),
            IpAddr::V6(ref a) => (*a, port).to_socket_addrs(),
        }
    }
}

impl ToSocketAddrs for (Ipv4Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV4::new(ip, port).to_socket_addrs()
    }
}

impl ToSocketAddrs for (Ipv6Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV6::new(ip, port, 0, 0).to_socket_addrs()
    }
}

impl<'a> ToSocketAddrs for (&'a str, u16) {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        let (host, port) = *self;

        // try to parse the host as a regular IP address first
        if let Ok(addr) = host.parse::<Ipv4Addr>() {
            let addr = SocketAddrV4::new(addr, port);
            return Ok(vec![SocketAddr::V4(addr)].into_iter())
        }
        if let Ok(addr) = host.parse::<Ipv6Addr>() {
            let addr = SocketAddrV6::new(addr, port, 0, 0);
            return Ok(vec![SocketAddr::V6(addr)].into_iter())
        }

        Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid socket address"))
    }
}

// accepts strings like 'localhost:12345'
#[allow(unused_variables)]
impl ToSocketAddrs for str {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        // try to parse as a regular SocketAddr first
        if let Some(addr) = self.parse().ok() {
            return Ok(vec![addr].into_iter());
        }

        macro_rules! try_opt {
            ($e:expr, $msg:expr) => (
                match $e {
                    Some(r) => r,
                    None => return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                                      $msg)),
                }
            )
        }

        // split the string by ':' and convert the second part to u16
        let mut parts_iter = self.rsplitn(2, ':');
        let port_str = try_opt!(parts_iter.next(), "invalid socket address");
        let host = try_opt!(parts_iter.next(), "invalid socket address");
        let port: u16 = try_opt!(port_str.parse().ok(), "invalid port value");
        Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid socket address"))
    }
}

impl<'a> ToSocketAddrs for &'a [SocketAddr] {
    type Iter = iter::Cloned<slice::Iter<'a, SocketAddr>>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(self.iter().cloned())
    }
}

impl<'a, T: ToSocketAddrs + ?Sized> ToSocketAddrs for &'a T {
    type Iter = T::Iter;
    fn to_socket_addrs(&self) -> io::Result<T::Iter> {
        (**self).to_socket_addrs()
    }
}

impl ToSocketAddrs for String {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        (&**self).to_socket_addrs()
    }
}