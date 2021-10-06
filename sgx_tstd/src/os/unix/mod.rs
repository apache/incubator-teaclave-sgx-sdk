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

//! Platform-specific extensions to `std` for Unix platforms.
//!
//! Provides access to platform-level information on Unix platforms, and
//! exposes Unix-specific functions that would otherwise be inappropriate as
//! part of the core `std` library.
//!
//! It exposes more ways to deal with platform-specific strings (`OsStr`,
//! `OsString`), allows to set permissions more granularly, extract low-level
//! file descriptors from files and sockets, and has platform-specific helpers
//! for spawning processes.
//!
//! # Examples
//!
//! ```no_run
//! use std::fs::File;
//! use std::os::unix::prelude::*;
//!
//! fn main() -> std::io::Result<()> {
//!     let f = File::create("foo.txt")?;
//!     let fd = f.as_raw_fd();
//!
//!     // use fd with native unix bindings
//!
//!     Ok(())
//! }
//! ```

pub mod ffi;
pub mod fs;
pub mod io;
#[cfg(feature = "net")]
pub mod net;
pub mod raw;
#[cfg(feature = "thread")]
pub mod thread;
#[cfg(feature = "net")]
pub mod ucred;

/// A prelude for conveniently writing platform-specific code.
///
/// Includes all extension traits, and some important type definitions.
pub mod prelude {
    #[doc(no_inline)]
    pub use super::ffi::{OsStrExt, OsStringExt};
    #[doc(no_inline)]
    pub use super::fs::DirEntryExt;
    #[doc(no_inline)]
    pub use super::fs::FileExt;
    #[doc(no_inline)]
    pub use super::fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt};
    #[doc(no_inline)]
    pub use super::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
    #[doc(no_inline)]
    #[cfg(feature = "thread")]
    pub use super::thread::JoinHandleExt;
}
