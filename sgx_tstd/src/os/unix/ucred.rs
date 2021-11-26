
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

//! Unix peer credentials.

// NOTE: Code in this file is heavily based on work done in PR 13 from the tokio-uds repository on
//       GitHub.
//
//       For reference, the link is here: https://github.com/tokio-rs/tokio-uds/pull/13
//       Credit to Martin Habovštiak (GitHub username Kixunil) and contributors for this work.

use sgx_libc::{gid_t, pid_t, uid_t};

/// Credentials for a UNIX process for credentials passing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UCred {
    /// The UID part of the peer credential. This is the effective UID of the process at the domain
    /// socket's endpoint.
    pub uid: uid_t,
    /// The GID part of the peer credential. This is the effective GID of the process at the domain
    /// socket's endpoint.
    pub gid: gid_t,
    /// The PID part of the peer credential. This field is optional because the PID part of the
    /// peer credentials is not supported on every platform. On platforms where the mechanism to
    /// discover the PID exists, this field will be populated to the PID of the process at the
    /// domain socket's endpoint. Otherwise, it will be set to None.
    pub pid: Option<pid_t>,
}

pub use self::impl_linux::peer_cred;

pub mod impl_linux {
    use super::UCred;
    use crate::os::unix::io::AsRawFd;
    use crate::os::unix::net::UnixStream;
    use crate::{io, mem};
    use sgx_libc::{c_void, socklen_t, ucred, SOL_SOCKET, SO_PEERCRED};
    use sgx_libc::ocall::getsockopt;

    pub fn peer_cred(socket: &UnixStream) -> io::Result<UCred> {
        let ucred_size = mem::size_of::<ucred>();

        // Trivial sanity checks.
        assert!(mem::size_of::<u32>() <= mem::size_of::<usize>());
        assert!(ucred_size <= u32::MAX as usize);

        let mut ucred_size = ucred_size as socklen_t;
        let mut ucred: ucred = ucred { pid: 1, uid: 1, gid: 1 };

        unsafe {
            let ret = getsockopt(
                socket.as_raw_fd(),
                SOL_SOCKET,
                SO_PEERCRED,
                &mut ucred as *mut ucred as *mut c_void,
                &mut ucred_size,
            );

            if ret == 0 && ucred_size as usize == mem::size_of::<ucred>() {
                Ok(UCred { uid: ucred.uid, gid: ucred.gid, pid: Some(ucred.pid) })
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}
