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

//! Unix-specific extensions to general I/O primitives

use sgx_trts::libc;
#[cfg(feature = "untrusted_fs")]
use crate::fs;
#[cfg(not(feature = "untrusted_fs"))]
use crate::untrusted::fs;
use crate::os::raw;
use crate::sys;
use crate::io;
use crate::sys_common::{AsInner, FromInner, IntoInner};

/// Raw file descriptors.
pub type RawFd = raw::c_int;

/// A trait to extract the raw unix file descriptor from an underlying
/// object.
///
/// This is only available on unix platforms and must be imported in order
/// to call the method. Windows platforms have a corresponding `AsRawHandle`
/// and `AsRawSocket` set of traits.
pub trait AsRawFd {
    /// Extracts the raw file descriptor.
    ///
    /// This method does **not** pass ownership of the raw file descriptor
    /// to the caller. The descriptor is only guaranteed to be valid while
    /// the original object has not yet been destroyed.
    fn as_raw_fd(&self) -> RawFd;
}

/// A trait to express the ability to construct an object from a raw file
/// descriptor.
pub trait FromRawFd {
    /// Constructs a new instance of `Self` from the given raw file
    /// descriptor.
    ///
    /// This function **consumes ownership** of the specified file
    /// descriptor. The returned object will take responsibility for closing
    /// it when the object goes out of scope.
    ///
    /// This function is also unsafe as the primitives currently returned
    /// have the contract that they are the sole owner of the file
    /// descriptor they are wrapping. Usage of this function could
    /// accidentally allow violating this contract which can cause memory
    /// unsafety in code that relies on it being true.
    unsafe fn from_raw_fd(fd: RawFd) -> Self;
}

/// A trait to express the ability to consume an object and acquire ownership of
/// its raw file descriptor.
pub trait IntoRawFd {
    /// Consumes this object, returning the raw underlying file descriptor.
    ///
    /// This function **transfers ownership** of the underlying file descriptor
    /// to the caller. Callers are then the unique owners of the file descriptor
    /// and must close the descriptor once it's no longer needed.
    fn into_raw_fd(self) -> RawFd;
}

impl AsRawFd for fs::File {
    fn as_raw_fd(&self) -> RawFd {
        self.as_inner().fd().raw()
    }
}

impl FromRawFd for fs::File {
    unsafe fn from_raw_fd(fd: RawFd) -> fs::File {
        fs::File::from_inner(sys::fs::File::from_inner(fd))
    }
}

impl IntoRawFd for fs::File {
    fn into_raw_fd(self) -> RawFd {
        self.into_inner().into_fd().into_raw()
    }
}

#[cfg(feature = "stdio")]
impl AsRawFd for io::Stdin {
    fn as_raw_fd(&self) -> RawFd {
        libc::STDIN_FILENO
    }
}

#[cfg(feature = "stdio")]
impl AsRawFd for io::Stdout {
    fn as_raw_fd(&self) -> RawFd {
        libc::STDOUT_FILENO
    }
}

#[cfg(feature = "stdio")]
impl AsRawFd for io::Stderr {
    fn as_raw_fd(&self) -> RawFd {
        libc::STDERR_FILENO
    }
}

#[cfg(feature = "stdio")]
impl<'a> AsRawFd for io::StdinLock<'a> {
    fn as_raw_fd(&self) -> RawFd {
        libc::STDIN_FILENO
    }
}

#[cfg(feature = "stdio")]
impl<'a> AsRawFd for io::StdoutLock<'a> {
    fn as_raw_fd(&self) -> RawFd {
        libc::STDOUT_FILENO
    }
}

#[cfg(feature = "stdio")]
impl<'a> AsRawFd for io::StderrLock<'a> {
    fn as_raw_fd(&self) -> RawFd {
        libc::STDERR_FILENO
    }
}
