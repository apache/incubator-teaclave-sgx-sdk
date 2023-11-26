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

//! Linux and Android-specific extensions to socket addresses.

use crate::os::unix::net::SocketAddr;
use crate::sealed::Sealed;

/// Platform-specific extensions to [`SocketAddr`].
pub trait SocketAddrExt: Sealed {
    /// Creates a Unix socket address in the abstract namespace.
    ///
    /// The abstract namespace is a Linux-specific extension that allows Unix
    /// sockets to be bound without creating an entry in the filesystem.
    /// Abstract sockets are unaffected by filesystem layout or permissions,
    /// and no cleanup is necessary when the socket is closed.
    ///
    /// An abstract socket address name may contain any bytes, including zero.
    ///
    /// # Errors
    ///
    /// Returns an error if the name is longer than `SUN_LEN - 1`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::os::unix::net::{UnixListener, SocketAddr};
    /// use std::os::linux::net::SocketAddrExt;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let addr = SocketAddr::from_abstract_name(b"hidden")?;
    ///     let listener = match UnixListener::bind_addr(&addr) {
    ///         Ok(sock) => sock,
    ///         Err(err) => {
    ///             println!("Couldn't bind: {err:?}");
    ///             return Err(err);
    ///         }
    ///     };
    ///     Ok(())
    /// }
    /// ```
    fn from_abstract_name<N>(name: N) -> crate::io::Result<SocketAddr>
    where
        N: AsRef<[u8]>;

    /// Returns the contents of this address if it is in the abstract namespace.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::os::unix::net::{UnixListener, SocketAddr};
    /// use std::os::linux::net::SocketAddrExt;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let name = b"hidden";
    ///     let name_addr = SocketAddr::from_abstract_name(name)?;
    ///     let socket = UnixListener::bind_addr(&name_addr)?;
    ///     let local_addr = socket.local_addr().expect("Couldn't get local address");
    ///     assert_eq!(local_addr.as_abstract_name(), Some(&name[..]));
    ///     Ok(())
    /// }
    /// ```
    fn as_abstract_name(&self) -> Option<&[u8]>;
}
