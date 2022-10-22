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

//! Owned and borrowed Unix-like file descriptors.

#![deny(unsafe_op_in_unsafe_fn)]

use super::raw::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use crate::fmt;
use crate::fs;
use crate::marker::PhantomData;
use crate::mem::forget;
use crate::sys::cvt;
use crate::sys_common::{AsInner, FromInner, IntoInner};

/// A borrowed file descriptor.
///
/// This has a lifetime parameter to tie it to the lifetime of something that
/// owns the file descriptor.
///
/// This uses `repr(transparent)` and has the representation of a host file
/// descriptor, so it can be used in FFI in places where a file descriptor is
/// passed as an argument, it is not captured or consumed, and it never has the
/// value `-1`.
///
/// This type's `.to_owned()` implementation returns another `BorrowedFd`
/// rather than an `OwnedFd`. It just makes a trivial copy of the raw file
/// descriptor, which is then borrowed under the same lifetime.
#[derive(Copy, Clone)]
#[repr(transparent)]
#[rustc_layout_scalar_valid_range_start(0)]
// libstd/os/raw/mod.rs assures me that every libstd-supported platform has a
// 32-bit c_int. Below is -2, in two's complement, but that only works out
// because c_int is 32 bits.
#[rustc_layout_scalar_valid_range_end(0xFF_FF_FF_FE)]
pub struct BorrowedFd<'fd> {
    fd: RawFd,
    _phantom: PhantomData<&'fd OwnedFd>,
}

/// An owned file descriptor.
///
/// This closes the file descriptor on drop.
///
/// This uses `repr(transparent)` and has the representation of a host file
/// descriptor, so it can be used in FFI in places where a file descriptor is
/// passed as a consumed argument or returned as an owned value, and it never
/// has the value `-1`.
#[repr(transparent)]
#[rustc_layout_scalar_valid_range_start(0)]
// libstd/os/raw/mod.rs assures me that every libstd-supported platform has a
// 32-bit c_int. Below is -2, in two's complement, but that only works out
// because c_int is 32 bits.
#[rustc_layout_scalar_valid_range_end(0xFF_FF_FF_FE)]
pub struct OwnedFd {
    fd: RawFd,
}

impl BorrowedFd<'_> {
    /// Return a `BorrowedFd` holding the given raw file descriptor.
    ///
    /// # Safety
    ///
    /// The resource pointed to by `fd` must remain open for the duration of
    /// the returned `BorrowedFd`, and it must not have the value `-1`.
    #[inline]
    pub const unsafe fn borrow_raw(fd: RawFd) -> Self {
        assert!(fd != u32::MAX as RawFd);
        // SAFETY: we just asserted that the value is in the valid range and isn't `-1` (the only value bigger than `0xFF_FF_FF_FE` unsigned)
        unsafe { Self { fd, _phantom: PhantomData } }
    }
}

impl OwnedFd {
    /// Creates a new `OwnedFd` instance that shares the same underlying file
    /// description as the existing `OwnedFd` instance.
    pub fn try_clone(&self) -> crate::io::Result<Self> {
        self.as_fd().try_clone_to_owned()
    }
}

impl BorrowedFd<'_> {
    /// Creates a new `OwnedFd` instance that shares the same underlying file
    /// description as the existing `BorrowedFd` instance.
    pub fn try_clone_to_owned(&self) -> crate::io::Result<OwnedFd> {
        // We want to atomically duplicate this file descriptor and set the
        // CLOEXEC flag, and currently that's done via F_DUPFD_CLOEXEC. This
        // is a POSIX flag that was added to Linux in 2.6.24.
        let cmd = libc::F_DUPFD_CLOEXEC;

        // Avoid using file descriptors below 3 as they are used for stdio
        let fd = cvt(unsafe { libc::fcntl_arg1(self.as_raw_fd(), cmd, 3) })?;
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }
}

impl AsRawFd for BorrowedFd<'_> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl AsRawFd for OwnedFd {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl IntoRawFd for OwnedFd {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        let fd = self.fd;
        forget(self);
        fd
    }
}

impl FromRawFd for OwnedFd {
    /// Constructs a new instance of `Self` from the given raw file descriptor.
    ///
    /// # Safety
    ///
    /// The resource pointed to by `fd` must be open and suitable for assuming
    /// ownership. The resource must not require any cleanup other than `close`.
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        assert_ne!(fd, u32::MAX as RawFd);
        // SAFETY: we just asserted that the value is in the valid range and isn't `-1` (the only value bigger than `0xFF_FF_FF_FE` unsigned)
        unsafe { Self { fd } }
    }
}

impl Drop for OwnedFd {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            // Note that errors are ignored when closing a file descriptor. The
            // reason for this is that if an error occurs we don't actually know if
            // the file descriptor was closed or not, and if we retried (for
            // something like EINTR), we might close another valid file descriptor
            // opened after we closed ours.
            let _ = libc::close(self.fd);
        }
    }
}

impl fmt::Debug for BorrowedFd<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowedFd").field("fd", &self.fd).finish()
    }
}

impl fmt::Debug for OwnedFd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedFd").field("fd", &self.fd).finish()
    }
}

macro_rules! impl_is_terminal {
    ($($t:ty),*$(,)?) => {$(
        impl crate::sealed::Sealed for $t {}

        impl crate::io::IsTerminal for $t {
            #[inline]
            fn is_terminal(&self) -> bool {
                crate::sys::io::is_terminal(self)
            }
        }
    )*}
}

impl_is_terminal!(BorrowedFd<'_>, OwnedFd);

/// A trait to borrow the file descriptor from an underlying object.
///
/// This is only available on unix platforms and must be imported in order to
/// call the method. Windows platforms have a corresponding `AsHandle` and
/// `AsSocket` set of traits.
pub trait AsFd {
    /// Borrows the file descriptor.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// # #[cfg(any(unix, target_os = "wasi"))]
    /// # use std::os::fd::{AsFd, BorrowedFd};
    ///
    /// let mut f = File::open("foo.txt")?;
    /// # #[cfg(any(unix, target_os = "wasi"))]
    /// let borrowed_fd: BorrowedFd<'_> = f.as_fd();
    /// # Ok::<(), io::Error>(())
    /// ```
    fn as_fd(&self) -> BorrowedFd<'_>;
}

impl<T: AsFd> AsFd for &T {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        T::as_fd(self)
    }
}

impl<T: AsFd> AsFd for &mut T {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        T::as_fd(self)
    }
}

impl AsFd for BorrowedFd<'_> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        *self
    }
}

impl AsFd for OwnedFd {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        // Safety: `OwnedFd` and `BorrowedFd` have the same validity
        // invariants, and the `BorrowdFd` is bounded by the lifetime
        // of `&self`.
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl AsFd for fs::File {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.as_inner().as_fd()
    }
}

impl From<fs::File> for OwnedFd {
    #[inline]
    fn from(file: fs::File) -> OwnedFd {
        file.into_inner().into_inner().into_inner()
    }
}

impl From<OwnedFd> for fs::File {
    #[inline]
    fn from(owned_fd: OwnedFd) -> Self {
        Self::from_inner(FromInner::from_inner(FromInner::from_inner(owned_fd)))
    }
}

#[cfg(feature = "net")]
impl AsFd for crate::net::TcpStream {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.as_inner().socket().as_fd()
    }
}

#[cfg(feature = "net")]
impl From<crate::net::TcpStream> for OwnedFd {
    #[inline]
    #[allow(clippy::useless_conversion)]
    fn from(tcp_stream: crate::net::TcpStream) -> OwnedFd {
        tcp_stream.into_inner().into_socket().into_inner().into_inner().into()
    }
}

#[cfg(feature = "net")]
impl From<OwnedFd> for crate::net::TcpStream {
    #[inline]
    fn from(owned_fd: OwnedFd) -> Self {
        Self::from_inner(FromInner::from_inner(FromInner::from_inner(FromInner::from_inner(
            owned_fd,
        ))))
    }
}

#[cfg(feature = "net")]
impl AsFd for crate::net::TcpListener {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.as_inner().socket().as_fd()
    }
}

#[cfg(feature = "net")]
impl From<crate::net::TcpListener> for OwnedFd {
    #[inline]
    #[allow(clippy::useless_conversion)]
    fn from(tcp_listener: crate::net::TcpListener) -> OwnedFd {
        tcp_listener.into_inner().into_socket().into_inner().into_inner().into()
    }
}

#[cfg(feature = "net")]
impl From<OwnedFd> for crate::net::TcpListener {
    #[inline]
    fn from(owned_fd: OwnedFd) -> Self {
        Self::from_inner(FromInner::from_inner(FromInner::from_inner(FromInner::from_inner(
            owned_fd,
        ))))
    }
}

#[cfg(feature = "net")]
impl AsFd for crate::net::UdpSocket {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.as_inner().socket().as_fd()
    }
}

#[cfg(feature = "net")]
impl From<crate::net::UdpSocket> for OwnedFd {
    #[inline]
    #[allow(clippy::useless_conversion)]
    fn from(udp_socket: crate::net::UdpSocket) -> OwnedFd {
        udp_socket.into_inner().into_socket().into_inner().into_inner().into()
    }
}

#[cfg(feature = "net")]
impl From<OwnedFd> for crate::net::UdpSocket {
    #[inline]
    fn from(owned_fd: OwnedFd) -> Self {
        Self::from_inner(FromInner::from_inner(FromInner::from_inner(FromInner::from_inner(
            owned_fd,
        ))))
    }
}

/// This impl allows implementing traits that require `AsFd` on Arc.
/// ```
/// # #[cfg(any(unix, target_os = "wasi"))] mod group_cfg {
/// # #[cfg(target_os = "wasi")]
/// # use std::os::wasi::io::AsFd;
/// # #[cfg(unix)]
/// # use std::os::unix::io::AsFd;
/// use std::net::UdpSocket;
/// use std::sync::Arc;
///
/// trait MyTrait: AsFd {}
/// impl MyTrait for Arc<UdpSocket> {}
/// impl MyTrait for Box<UdpSocket> {}
/// # }
/// ```
impl<T: AsFd> AsFd for crate::sync::Arc<T> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}

impl<T: AsFd> AsFd for Box<T> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}

#[cfg(feature = "stdio")]
impl AsFd for crate::io::Stdin {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(0) }
    }
}

#[cfg(feature = "stdio")]
impl<'a> AsFd for crate::io::StdinLock<'a> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        // SAFETY: user code should not close stdin out from under the standard library
        unsafe { BorrowedFd::borrow_raw(0) }
    }
}

#[cfg(feature = "stdio")]
impl AsFd for crate::io::Stdout {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(1) }
    }
}

#[cfg(feature = "stdio")]
impl<'a> AsFd for crate::io::StdoutLock<'a> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        // SAFETY: user code should not close stdout out from under the standard library
        unsafe { BorrowedFd::borrow_raw(1) }
    }
}

#[cfg(feature = "stdio")]
impl AsFd for crate::io::Stderr {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(2) }
    }
}

#[cfg(feature = "stdio")]
impl<'a> AsFd for crate::io::StderrLock<'a> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        // SAFETY: user code should not close stderr out from under the standard library
        unsafe { BorrowedFd::borrow_raw(2) }
    }
}

mod libc {
    pub use sgx_libc::F_DUPFD_CLOEXEC;
    pub use sgx_libc::ocall::{close, fcntl_arg1};
}
