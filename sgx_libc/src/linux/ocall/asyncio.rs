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
// under the License.

use crate::linux::*;
use alloc::vec::Vec;
use core::mem::ManuallyDrop;
use core::slice;
use sgx_oc::linux::ocall;
use sgx_oc::linux::ocall::PolledOk;

#[no_mangle]
pub unsafe extern "C" fn poll(fds: *mut pollfd, nfds: nfds_t, timeout: c_int) -> c_int {
    if fds.is_null() || nfds == 0 {
        set_errno(EINVAL);
        return -1;
    }

    let fds = slice::from_raw_parts_mut(fds, nfds as usize);
    if let Ok(result) = ocall::poll(fds, timeout) {
        match result {
            PolledOk::ReadyFdCount(n) => n as i32,
            PolledOk::TimeLimitExpired => 0,
        }
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn epoll_create1(flags: c_int) -> c_int {
    if let Ok(fd) = ocall::epoll_create1(flags) {
        fd
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn epoll_create(size: c_int) -> c_int {
    if let Ok(fd) = ocall::epoll_create(size) {
        fd
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn epoll_ctl(
    epfd: c_int,
    op: c_int,
    fd: c_int,
    event: *mut epoll_event,
) -> c_int {
    if event.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::epoll_ctl(epfd, op, fd, &mut *event).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn epoll_wait(
    epfd: c_int,
    events: *mut epoll_event,
    maxevents: c_int,
    timeout: c_int,
) -> c_int {
    if events.is_null() || maxevents <= 0 {
        set_errno(EINVAL);
        return -1;
    }

    let mut events = ManuallyDrop::new(Vec::from_raw_parts(events, 0, maxevents as usize));
    if let Ok(result) = ocall::epoll_wait(epfd, &mut events, timeout) {
        match result {
            PolledOk::ReadyFdCount(n) => n as i32,
            PolledOk::TimeLimitExpired => 0,
        }
    } else {
        -1
    }
}
