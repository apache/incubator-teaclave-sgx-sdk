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

use crate::io::prelude::*;

use sgx_trts::libc::c_int;
use core::fmt;
use crate::io::{self, Initializer, IoSlice, IoSliceMut};
use crate::net::{Shutdown, SocketAddr, ToSocketAddrs};
use crate::sys_common::net as net_imp;
use crate::sys_common::{AsInner, FromInner, IntoInner};
use crate::time::Duration;

/// A TCP stream between a local and a remote socket.
///
/// After creating a `TcpStream` by either [`connect`]ing to a remote host or
/// [`accept`]ing a connection on a [`TcpListener`], data can be transmitted
/// by [reading] and [writing] to it.
///
/// The connection will be closed when the value is dropped. The reading and writing
/// portions of the connection can also be shut down individually with the [`shutdown`]
/// method.
///
/// The Transmission Control Protocol is specified in [IETF RFC 793].
///
/// [`accept`]: ../../std/net/struct.TcpListener.html#method.accept
/// [`connect`]: #method.connect
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
/// [reading]: ../../std/io/trait.Read.html
/// [`shutdown`]: #method.shutdown
/// [`TcpListener`]: ../../std/net/struct.TcpListener.html
/// [writing]: ../../std/io/trait.Write.html
///
pub struct TcpStream(net_imp::TcpStream);

/// A TCP socket server, listening for connections.
///
/// After creating a `TcpListener` by [`bind`]ing it to a socket address, it listens
/// for incoming TCP connections. These can be accepted by calling [`accept`] or by
/// iterating over the [`Incoming`] iterator returned by [`incoming`][`TcpListener::incoming`].
///
/// The socket will be closed when the value is dropped.
///
/// The Transmission Control Protocol is specified in [IETF RFC 793].
///
/// [`accept`]: #method.accept
/// [`bind`]: #method.bind
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
/// [`Incoming`]: ../../std/net/struct.Incoming.html
/// [`TcpListener::incoming`]: #method.incoming
///
pub struct TcpListener(net_imp::TcpListener);

/// An iterator that infinitely [`accept`]s connections on a [`TcpListener`].
///
/// This `struct` is created by the [`incoming`] method on [`TcpListener`].
/// See its documentation for more.
///
/// [`accept`]: ../../std/net/struct.TcpListener.html#method.accept
/// [`incoming`]: ../../std/net/struct.TcpListener.html#method.incoming
/// [`TcpListener`]: ../../std/net/struct.TcpListener.html
#[derive(Debug)]
pub struct Incoming<'a> {
    listener: &'a TcpListener,
}

impl TcpStream {
    pub fn new(sockfd: c_int) -> io::Result<TcpStream> {
        net_imp::TcpStream::new(sockfd).map(TcpStream)
    }

    pub fn new_v4() -> io::Result<TcpStream> {
        net_imp::TcpStream::new_v4().map(TcpStream)
    }

    pub fn new_v6() -> io::Result<TcpStream> {
        net_imp::TcpStream::new_v6().map(TcpStream)
    }

    pub fn raw(&self) -> c_int {
        self.0.raw()
    }

    pub fn into_raw(self) -> c_int {
        self.0.into_raw()
    }

    /// Opens a TCP connection to a remote host.
    ///
    /// `addr` is an address of the remote host. Anything which implements
    /// [`ToSocketAddrs`] trait can be supplied for the address; see this trait
    /// documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until a connection is successful. If none of
    /// the addresses result in a successful connection, the error returned from
    /// the last connection attempt (the last address) is returned.
    ///
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        super::each_addr(addr, net_imp::TcpStream::connect).map(TcpStream)
    }

    /// Opens a TCP connection to a remote host.
    ///
    /// `addr` is an address of the remote host. Anything which implements
    /// [`ToSocketAddrs`] trait can be supplied for the address; see this trait
    /// documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until a connection is successful. If none of
    /// the addresses result in a successful connection, the error returned from
    /// the last connection attempt (the last address) is returned.
    ///
    pub fn connect_socket<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        super::each_addr(addr, |addr| self.0.connect_socket(addr))
    }
    
    /// Opens a TCP connection to a remote host with a timeout.
    ///
    /// Unlike `connect`, `connect_timeout` takes a single [`SocketAddr`] since
    /// timeout must be applied to individual addresses.
    ///
    /// It is an error to pass a zero `Duration` to this function.
    ///
    /// Unlike other methods on `TcpStream`, this does not correspond to a
    /// single system call. It instead calls `connect` in nonblocking mode and
    /// then uses an OS-specific mechanism to await the completion of the
    /// connection request.
    ///
    /// [`SocketAddr`]: ../../std/net/enum.SocketAddr.html
    pub fn connect_timeout(addr: &SocketAddr, timeout: Duration) -> io::Result<TcpStream> {
        net_imp::TcpStream::connect_timeout(addr, timeout).map(TcpStream)
    }

    /// Opens a TCP connection to a remote host with a timeout.
    ///
    /// Unlike `connect_socket`, `connect_socket_timeout` takes a single [`SocketAddr`] since
    /// timeout must be applied to individual addresses.
    ///
    /// It is an error to pass a zero `Duration` to this function.
    ///
    /// Unlike other methods on `TcpStream`, this does not correspond to a
    /// single system call. It instead calls `connect` in nonblocking mode and
    /// then uses an OS-specific mechanism to await the completion of the
    /// connection request.
    ///
    pub fn connect_socket_timeout(&self, addr: &SocketAddr, timeout: Duration) -> io::Result<()> {
        self.0.connect_socket_timeout(addr, timeout)
    }

    /// Returns the socket address of the remote peer of this TCP connection.
    ///
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    /// Returns the socket address of the local half of this TCP connection.
    ///
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O on the specified
    /// portions to return immediately with an appropriate value (see the
    /// documentation of [`Shutdown`]).
    ///
    /// [`Shutdown`]: ../../std/net/enum.Shutdown.html
    ///
    /// # Platform-specific behavior
    ///
    /// Calling this function multiple times may result in different behavior,
    /// depending on the operating system. On Linux, the second call will
    /// return `Ok(())`, but on macOS, it will return `ErrorKind::NotConnected`.
    /// This may change in the future.
    ///
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.0.shutdown(how)
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned `TcpStream` is a reference to the same stream that this
    /// object references. Both handles will read and write the same stream of
    /// data, and options set on one stream will be propagated to the other
    /// stream.
    ///
    pub fn try_clone(&self) -> io::Result<TcpStream> {
        self.0.duplicate().map(TcpStream)
    }

    /// Sets the read timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`read`] calls will block
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a read times out as
    /// a result of setting this option. For example Unix typically returns an
    /// error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    /// [`Err`]: ../../std/result/enum.Result.html#variant.Err
    /// [`read`]: ../../std/io/trait.Read.html#tymethod.read
    /// [`WouldBlock`]: ../../std/io/enum.ErrorKind.html#variant.WouldBlock
    /// [`TimedOut`]: ../../std/io/enum.ErrorKind.html#variant.TimedOut
    /// [`Duration`]: ../../std/time/struct.Duration.html
    ///
    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(dur)
    }

    /// Sets the write timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`write`] calls will block
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a write times out
    /// as a result of setting this option. For example Unix typically returns
    /// an error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    /// [`Err`]: ../../std/result/enum.Result.html#variant.Err
    /// [`write`]: ../../std/io/trait.Write.html#tymethod.write
    /// [`Duration`]: ../../std/time/struct.Duration.html
    /// [`WouldBlock`]: ../../std/io/enum.ErrorKind.html#variant.WouldBlock
    /// [`TimedOut`]: ../../std/io/enum.ErrorKind.html#variant.TimedOut
    ///
    pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(dur)
    }

    /// Returns the read timeout of this socket.
    ///
    /// If the timeout is [`None`], then [`read`] calls will block indefinitely.
    ///
    /// # Platform-specific behavior
    ///
    /// Some platforms do not provide access to the current timeout.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    /// [`read`]: ../../std/io/trait.Read.html#tymethod.read
    ///
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.read_timeout()
    }

    /// Returns the write timeout of this socket.
    ///
    /// If the timeout is [`None`], then [`write`] calls will block indefinitely.
    ///
    /// # Platform-specific behavior
    ///
    /// Some platforms do not provide access to the current timeout.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    /// [`write`]: ../../std/io/trait.Write.html#tymethod.write
    ///
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.write_timeout()
    }

    /// Receives data on the socket from the remote address to which it is
    /// connected, without removing that data from the queue. On success,
    /// returns the number of bytes peeked.
    ///
    /// Successive calls return the same data. This is accomplished by passing
    /// `MSG_PEEK` as a flag to the underlying `recv` system call.
    ///
    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.peek(buf)
    }

    /// Sets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// If set, this option disables the Nagle algorithm. This means that
    /// segments are always sent as soon as possible, even if there is only a
    /// small amount of data. When not set, data is buffered until there is a
    /// sufficient amount to send out, thereby avoiding the frequent sending of
    /// small packets.
    ///
    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.0.set_nodelay(nodelay)
    }

    /// Gets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// For more information about this option, see [`set_nodelay`][link].
    ///
    /// [link]: #method.set_nodelay
    ///
    pub fn nodelay(&self) -> io::Result<bool> {
        self.0.nodelay()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`set_ttl`][link].
    ///
    /// [link]: #method.set_ttl
    ///
    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    /// Gets the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, clearing
    /// the field in the process. This can be useful for checking errors between
    /// calls.
    ///
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    ///
    /// This will result in `read`, `write`, `recv` and `send` operations
    /// becoming nonblocking, i.e., immediately returning from their calls.
    /// If the IO operation is successful, `Ok` is returned and no further
    /// action is required. If the IO operation could not be completed and needs
    /// to be retried, an error with kind [`io::ErrorKind::WouldBlock`] is
    /// returned.
    ///
    /// On Unix platforms, calling this method corresponds to calling `fcntl`
    /// `FIONBIO`. On Windows calling this method corresponds to calling
    /// `ioctlsocket` `FIONBIO`.
    ///
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for &TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

impl Write for &TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsInner<net_imp::TcpStream> for TcpStream {
    fn as_inner(&self) -> &net_imp::TcpStream {
        &self.0
    }
}

impl FromInner<net_imp::TcpStream> for TcpStream {
    fn from_inner(inner: net_imp::TcpStream) -> TcpStream {
        TcpStream(inner)
    }
}

impl IntoInner<net_imp::TcpStream> for TcpStream {
    fn into_inner(self) -> net_imp::TcpStream {
        self.0
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TcpListener {
    pub fn new(sockfd: c_int) -> io::Result<TcpListener> {
        net_imp::TcpListener::new(sockfd).map(TcpListener)
    }

    pub fn new_v4() -> io::Result<TcpListener> {
        net_imp::TcpListener::new_v4().map(TcpListener)
    }

    pub fn new_v6() -> io::Result<TcpListener> {
        net_imp::TcpListener::new_v6().map(TcpListener)
    }

    pub fn raw(&self) -> c_int {
        self.0.raw()
    }

    pub fn into_raw(self) -> c_int {
        self.0.into_raw()
    }

    /// Creates a new `TcpListener` which will be bound to the specified
    /// address.
    ///
    /// The returned listener is ready for accepting connections.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port
    /// to this listener. The port allocated can be queried via the
    /// [`local_addr`] method.
    ///
    /// The address type can be any implementor of [`ToSocketAddrs`] trait. See
    /// its documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `bind` will be attempted with
    /// each of the addresses until one succeeds and returns the listener. If
    /// none of the addresses succeed in creating a listener, the error returned
    /// from the last attempt (the last address) is returned.
    ///
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        super::each_addr(addr, net_imp::TcpListener::bind).map(TcpListener)
    }

    /// TcpListener will be bound to the specified address.
    ///
    /// The returned listener is ready for accepting connections.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port
    /// to this listener. The port allocated can be queried via the
    /// [`local_addr`] method.
    ///
    /// The address type can be any implementor of [`ToSocketAddrs`] trait. See
    /// its documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `bind` will be attempted with
    /// each of the addresses until one succeeds and returns the listener. If
    /// none of the addresses succeed in creating a listener, the error returned
    /// from the last attempt (the last address) is returned.
    ///
    pub fn bind_socket<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        super::each_addr(addr, |addr| self.0.bind_socket(addr))
    }

    /// Returns the local socket address of this listener.
    ///
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned [`TcpListener`] is a reference to the same socket that this
    /// object references. Both handles can be used to accept incoming
    /// connections and options set on one listener will affect the other.
    ///
    /// [`TcpListener`]: ../../std/net/struct.TcpListener.html
    ///
    pub fn try_clone(&self) -> io::Result<TcpListener> {
        self.0.duplicate().map(TcpListener)
    }

    /// Accept a new incoming connection from this listener.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, the corresponding [`TcpStream`] and the
    /// remote peer's address will be returned.
    ///
    /// [`TcpStream`]: ../../std/net/struct.TcpStream.html
    ///
    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        self.0.accept().map(|(a, b)| (TcpStream(a), b))
    }

    /// Returns an iterator over the connections being received on this
    /// listener.
    ///
    /// The returned iterator will never return [`None`] and will also not yield
    /// the peer's [`SocketAddr`] structure. Iterating over it is equivalent to
    /// calling [`accept`] in a loop.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    /// [`SocketAddr`]: ../../std/net/enum.SocketAddr.html
    /// [`accept`]: #method.accept
    ///
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`set_ttl`][link].
    ///
    /// [link]: #method.set_ttl
    ///
    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    /// This option can only be set before the socket is bound
    pub fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        self.0.set_only_v6(only_v6)
    }

    /// This option can only be set before the socket is bound
    pub fn only_v6(&self) -> io::Result<bool> {
        self.0.only_v6()
    }

    /// Gets the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, clearing
    /// the field in the process. This can be useful for checking errors between
    /// calls.
    ///
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    ///
    /// This will result in the `accept` operation becoming nonblocking,
    /// i.e., immediately returning from their calls. If the IO operation is
    /// successful, `Ok` is returned and no further action is required. If the
    /// IO operation could not be completed and needs to be retried, an error
    /// with kind [`io::ErrorKind::WouldBlock`] is returned.
    ///
    /// On Unix platforms, calling this method corresponds to calling `fcntl`
    /// `FIONBIO`. On Windows calling this method corresponds to calling
    /// `ioctlsocket` `FIONBIO`.
    ///
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }
}

impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<TcpStream>;
    fn next(&mut self) -> Option<io::Result<TcpStream>> {
        Some(self.listener.accept().map(|p| p.0))
    }
}

impl AsInner<net_imp::TcpListener> for TcpListener {
    fn as_inner(&self) -> &net_imp::TcpListener {
        &self.0
    }
}

impl FromInner<net_imp::TcpListener> for TcpListener {
    fn from_inner(inner: net_imp::TcpListener) -> TcpListener {
        TcpListener(inner)
    }
}

impl IntoInner<net_imp::TcpListener> for TcpListener {
    fn into_inner(self) -> net_imp::TcpListener {
        self.0
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}