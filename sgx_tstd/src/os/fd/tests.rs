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

use sgx_test_utils::test_case;

#[test_case]
fn test_raw_fd() {
    use crate::os::unix::io::{AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, RawFd};

    let raw_fd: RawFd = crate::io::stdin().as_raw_fd();

    let stdin_as_file = unsafe { crate::fs::File::from_raw_fd(raw_fd) };
    assert_eq!(stdin_as_file.as_raw_fd(), raw_fd);
    assert_eq!(unsafe { BorrowedFd::borrow_raw(raw_fd).as_raw_fd() }, raw_fd);
    assert_eq!(stdin_as_file.into_raw_fd(), 0);
}

#[test_case]
fn test_fd() {
    use crate::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};

    let stdin = crate::io::stdin();
    let fd: BorrowedFd<'_> = stdin.as_fd();
    let raw_fd: RawFd = fd.as_raw_fd();
    let owned_fd: OwnedFd = unsafe { OwnedFd::from_raw_fd(raw_fd) };

    let stdin_as_file = crate::fs::File::from(owned_fd);

    assert_eq!(stdin_as_file.as_fd().as_raw_fd(), raw_fd);
    assert_eq!(Into::<OwnedFd>::into(stdin_as_file).into_raw_fd(), raw_fd);
}

#[test_case]
fn test_niche_optimizations() {
    use crate::mem::size_of;
    use crate::os::unix::io::{BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};

    assert_eq!(size_of::<Option<OwnedFd>>(), size_of::<RawFd>());
    assert_eq!(size_of::<Option<BorrowedFd<'static>>>(), size_of::<RawFd>());
    unsafe {
        assert_eq!(OwnedFd::from_raw_fd(RawFd::MIN).into_raw_fd(), RawFd::MIN);
        assert_eq!(OwnedFd::from_raw_fd(RawFd::MAX).into_raw_fd(), RawFd::MAX);
        assert_eq!(Some(OwnedFd::from_raw_fd(RawFd::MIN)).unwrap().into_raw_fd(), RawFd::MIN);
        assert_eq!(Some(OwnedFd::from_raw_fd(RawFd::MAX)).unwrap().into_raw_fd(), RawFd::MAX);
    }
}
