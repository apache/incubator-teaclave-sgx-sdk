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

use core::cmp;
use core::convert::{From, TryFrom};
use core::ptr;
use core::time::Duration;
use sgx_trts::error::set_errno;
use sgx_trts::tcs::TcsId;
use sgx_types::error::errno::{EINVAL, ESGX};
use sgx_types::error::{OsResult, SgxStatus};

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct TimeSpec {
    pub sec: i64,
    pub nsec: i64,
}

impl TimeSpec {
    pub fn as_duration(&self) -> Option<Duration> {
        Duration::try_from(*self).ok()
    }

    fn validate(&self) -> bool {
        self.sec >= 0 && self.nsec >= 0 && self.nsec < 1_000_000_000
    }
}

impl From<Duration> for TimeSpec {
    fn from(dur: Duration) -> TimeSpec {
        TimeSpec {
            sec: cmp::min(dur.as_secs(), i64::MAX as u64) as i64,
            nsec: dur.subsec_nanos() as i64,
        }
    }
}

impl TryFrom<TimeSpec> for Duration {
    type Error = i32;

    fn try_from(ts: TimeSpec) -> OsResult<Duration> {
        if ts.validate() {
            Ok(Duration::new(ts.sec as u64, ts.nsec as u32))
        } else {
            Err(EINVAL)
        }
    }
}

macro_rules! ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            set_errno($e);
            return Err($e);
        }
    };
}

extern "C" {
    pub fn u_thread_wait_event_ocall(
        result: *mut i32,
        error: *mut i32,
        tcs: usize,
        timeout: *const TimeSpec,
    ) -> SgxStatus;

    pub fn u_thread_set_event_ocall(result: *mut i32, error: *mut i32, tcs: usize) -> SgxStatus;

    pub fn u_thread_set_multiple_events_ocall(
        result: *mut i32,
        error: *mut i32,
        tcss: *const usize,
        total: usize,
    ) -> SgxStatus;

    pub fn u_thread_setwait_events_ocall(
        result: *mut i32,
        error: *mut i32,
        wait_tcs: usize,
        self_tcs: usize,
        timeout: *const TimeSpec,
    ) -> SgxStatus;
}

pub fn thread_wait_event(tcs: TcsId, dur: Option<Duration>) -> OsResult {
    let mut result: i32 = 0;
    let mut error: i32 = 0;

    let timeout = dur.map(|dur| dur.into());
    let timeout_ptr = timeout
        .as_ref()
        .map_or(ptr::null(), |time| time as *const TimeSpec);

    let status = unsafe {
        u_thread_wait_event_ocall(
            &mut result as *mut i32,
            &mut error as *mut i32,
            tcs.as_usize(),
            timeout_ptr,
        )
    };

    ensure!(status.is_success(), ESGX);
    ensure!(result == 0, error);
    Ok(())
}

pub fn thread_set_event(tcs: TcsId) -> OsResult {
    let mut result: i32 = 0;
    let mut error: i32 = 0;

    let status = unsafe {
        u_thread_set_event_ocall(
            &mut result as *mut i32,
            &mut error as *mut i32,
            tcs.as_usize(),
        )
    };

    ensure!(status.is_success(), ESGX);
    ensure!(result == 0, error);
    Ok(())
}

pub fn thread_set_multiple_events(tcss: &[TcsId]) -> OsResult {
    let mut result: i32 = 0;
    let mut error: i32 = 0;

    let status = unsafe {
        u_thread_set_multiple_events_ocall(
            &mut result as *mut i32,
            &mut error as *mut i32,
            tcss.as_ptr() as *const TcsId as *const usize,
            tcss.len(),
        )
    };

    ensure!(status.is_success(), ESGX);
    ensure!(result == 0, error);
    Ok(())
}

pub fn thread_setwait_events(wait_tcs: TcsId, self_tcs: TcsId, dur: Option<Duration>) -> OsResult {
    let mut result: i32 = 0;
    let mut error: i32 = 0;

    let timeout = dur.map(|dur| dur.into());
    let timeout_ptr = timeout
        .as_ref()
        .map_or(ptr::null(), |time| time as *const TimeSpec);

    let status = unsafe {
        u_thread_setwait_events_ocall(
            &mut result as *mut i32,
            &mut error as *mut i32,
            wait_tcs.as_usize(),
            self_tcs.as_usize(),
            timeout_ptr,
        )
    };

    ensure!(status.is_success(), ESGX);
    ensure!(result == 0, error);
    Ok(())
}
