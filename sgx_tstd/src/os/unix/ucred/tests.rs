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

use crate::os::unix::net::UnixStream;
use libc::{getgid, getpid, getuid};

use sgx_test_utils::test_case;

#[test_case]
fn test_socket_pair() {
    // Create two connected sockets and get their peer credentials. They should be equal.
    let (sock_a, sock_b) = UnixStream::pair().unwrap();
    let (cred_a, cred_b) = (sock_a.peer_cred().unwrap(), sock_b.peer_cred().unwrap());
    assert_eq!(cred_a, cred_b);

    // Check that the UID and GIDs match up.
    let uid = unsafe { getuid().unwrap() };
    let gid = unsafe { getgid().unwrap() };
    assert_eq!(cred_a.uid, uid);
    assert_eq!(cred_a.gid, gid);
}

#[test_case]
fn test_socket_pair_pids() {
    // Create two connected sockets and get their peer credentials.
    let (sock_a, sock_b) = UnixStream::pair().unwrap();
    let (cred_a, cred_b) = (sock_a.peer_cred().unwrap(), sock_b.peer_cred().unwrap());

    // On supported platforms (see the cfg above), the credentials should always include the PID.
    let pid = unsafe { getpid().unwrap() };
    assert_eq!(cred_a.pid, Some(pid));
    assert_eq!(cred_b.pid, Some(pid));
}

mod libc {
    pub use sgx_oc::ocall::{getgid, getpid, getuid};
}
