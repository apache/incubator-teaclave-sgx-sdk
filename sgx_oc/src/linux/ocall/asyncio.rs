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

use super::*;
use core::mem;
use sgx_trts::trts::is_within_enclave;
use sgx_types::marker::ContiguousMemory;

pub unsafe fn poll(fds: &mut [pollfd], timeout: c_int) -> OCallResult<PolledOk> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let nfds = fds.len() as nfds_t;
    ensure!(nfds > 0, eos!(EINVAL));

    let status = u_poll_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fds.as_mut_ptr(),
        nfds,
        timeout,
    );

    ensure!(status.is_success(), esgx!(status));
    match result {
        1..=i32::MAX => {
            if result as nfds_t > nfds {
                Err(ecust!("Malformed return value"))
            } else {
                Ok(PolledOk::ReadyFdCount(result as u32))
            }
        }
        0 => Ok(PolledOk::TimeLimitExpired),
        _ => Err(eos!(error)),
    }
}

pub unsafe fn epoll_create1(flags: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_epoll_create1_ocall(&mut result as *mut c_int, &mut error as *mut c_int, flags);

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn epoll_create(size: c_int) -> OCallResult<c_int> {
    ensure!(size > 0, eos!(EINVAL));
    epoll_create1(0)
}

pub unsafe fn epoll_ctl(
    epfd: c_int,
    op: c_int,
    fd: c_int,
    event: &mut epoll_event,
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_epoll_ctl_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        epfd,
        op,
        fd,
        event,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn epoll_wait(
    epfd: c_int,
    events: &mut Vec<epoll_event>,
    timeout: c_int,
) -> OCallResult<PolledOk> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    ensure!(
        is_within_enclave(events as *mut _ as *mut u8, mem::size_of_val(events)),
        eos!(EINVAL)
    );
    let maxevents = events.capacity();
    ensure!((1..=i32::MAX as usize).contains(&maxevents), eos!(EINVAL));

    let status = u_epoll_wait_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        epfd,
        events.as_mut_ptr(),
        maxevents as c_uint,
        timeout,
    );

    ensure!(status.is_success(), esgx!(status));
    match result {
        1..=i32::MAX => {
            if result > maxevents as c_int {
                Err(ecust!("Malformed return value"))
            } else {
                events.set_len(result as usize);
                Ok(PolledOk::ReadyFdCount(result as u32))
            }
        }
        0 => Ok(PolledOk::TimeLimitExpired),
        _ => Err(eos!(error)),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PolledOk {
    TimeLimitExpired,
    ReadyFdCount(u32),
}

unsafe impl ContiguousMemory for PolledOk {}
