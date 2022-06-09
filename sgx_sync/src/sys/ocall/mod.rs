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

use core::cmp::{self, Ordering};
use core::convert::{From, TryFrom};
use core::hash::{Hash, Hasher};
use core::ptr;
use core::time::Duration;
use sgx_trts::error::set_errno;
use sgx_trts::tcs::TcsId;
use sgx_types::error::errno::{EINVAL, ESGX};
use sgx_types::error::{OsResult, SgxStatus};
use sgx_types::types::{c_long, time_t, timespec};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Timespec {
    t: timespec,
}

const NANOS_PER_SEC: u64 = 1_000_000_000;

impl Timespec {
    pub fn new(secs: u64, nanos: u32) -> Timespec {
        let secs = secs
            .try_into()
            .ok()
            .and_then(|secs: i64| secs.checked_add((nanos / NANOS_PER_SEC as u32) as i64))
            .unwrap_or(i64::MAX);

        let nanos = nanos % NANOS_PER_SEC as u32;
        Timespec {
            t: timespec {
                tv_sec: secs,
                tv_nsec: nanos as _,
            },
        }
    }

    pub fn sub_timespec(&self, other: &Timespec) -> Result<Duration, Duration> {
        if self >= other {
            let (secs, nanos) = if self.t.tv_nsec >= other.t.tv_nsec {
                (
                    (self.t.tv_sec - other.t.tv_sec) as u64,
                    (self.t.tv_nsec - other.t.tv_nsec) as u32,
                )
            } else {
                (
                    (self.t.tv_sec - other.t.tv_sec - 1) as u64,
                    self.t.tv_nsec as u32 + (NANOS_PER_SEC as u32) - other.t.tv_nsec as u32,
                )
            };

            Ok(Duration::new(secs, nanos))
        } else {
            match other.sub_timespec(self) {
                Ok(d) => Err(d),
                Err(d) => Ok(d),
            }
        }
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<Timespec> {
        let mut secs = other
            .as_secs()
            .try_into()
            .ok()
            .and_then(|secs| self.t.tv_sec.checked_add(secs))?;

        let mut nanos = other.subsec_nanos() + self.t.tv_nsec as u32;
        if nanos >= NANOS_PER_SEC as u32 {
            nanos -= NANOS_PER_SEC as u32;
            secs = secs.checked_add(1)?;
        }
        Some(Timespec {
            t: timespec {
                tv_sec: secs,
                tv_nsec: nanos as _,
            },
        })
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Timespec> {
        let mut secs = other
            .as_secs()
            .try_into()
            .ok()
            .and_then(|secs| self.t.tv_sec.checked_sub(secs))?;

        let mut nanos = self.t.tv_nsec as i32 - other.subsec_nanos() as i32;
        if nanos < 0 {
            nanos += NANOS_PER_SEC as i32;
            secs = secs.checked_sub(1)?;
        }
        Some(Timespec {
            t: timespec {
                tv_sec: secs,
                tv_nsec: nanos as _,
            },
        })
    }

    #[inline]
    pub const fn secs(&self) -> i64 {
        self.t.tv_sec
    }

    #[inline]
    pub const fn subsec_nanos(&self) -> i64 {
        self.t.tv_nsec
    }

    #[inline]
    pub const fn zero() -> Timespec {
        Timespec {
            t: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
        }
    }

    #[inline]
    pub const fn max() -> Timespec {
        Timespec {
            t: timespec {
                tv_sec: <time_t>::MAX,
                tv_nsec: (NANOS_PER_SEC - 1) as c_long,
            },
        }
    }

    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.t.tv_sec == 0 && self.t.tv_nsec == 0
    }

    #[inline]
    pub fn to_duration(self) -> Option<Duration> {
        Duration::try_from(self).ok()
    }
}

impl From<Duration> for Timespec {
    fn from(dur: Duration) -> Timespec {
        Timespec {
            t: timespec {
                tv_sec: cmp::min(dur.as_secs(), <time_t>::MAX as u64) as time_t,
                tv_nsec: dur.subsec_nanos() as c_long,
            },
        }
    }
}

impl TryFrom<timespec> for Timespec {
    type Error = i32;

    #[inline]
    fn try_from(ts: timespec) -> OsResult<Timespec> {
        (ts.tv_sec, ts.tv_nsec).try_into()
    }
}

impl TryFrom<(i64, i64)> for Timespec {
    type Error = i32;

    fn try_from(ts: (i64, i64)) -> OsResult<Timespec> {
        if ts.0 >= 0 && ts.1 >= 0 {
            let secs =
                ts.0.checked_add(ts.1 / NANOS_PER_SEC as i64)
                    .ok_or(EINVAL)?;
            let nanos = ts.1 % NANOS_PER_SEC as i64;

            Ok(Timespec {
                t: timespec {
                    tv_sec: secs,
                    tv_nsec: nanos as _,
                },
            })
        } else {
            Err(EINVAL)
        }
    }
}

impl TryFrom<Timespec> for Duration {
    type Error = i32;

    fn try_from(ts: Timespec) -> OsResult<Duration> {
        if ts.t.tv_sec >= 0 && ts.t.tv_nsec >= 0 && ts.t.tv_nsec < NANOS_PER_SEC as c_long {
            Ok(Duration::new(ts.t.tv_sec as u64, ts.t.tv_nsec as u32))
        } else {
            Err(EINVAL)
        }
    }
}

impl PartialEq for Timespec {
    fn eq(&self, other: &Timespec) -> bool {
        self.t.tv_sec == other.t.tv_sec && self.t.tv_nsec == other.t.tv_nsec
    }
}

impl Eq for Timespec {}

impl PartialOrd for Timespec {
    fn partial_cmp(&self, other: &Timespec) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timespec {
    fn cmp(&self, other: &Timespec) -> Ordering {
        let me = (self.t.tv_sec, self.t.tv_nsec);
        let other = (other.t.tv_sec, other.t.tv_nsec);
        me.cmp(&other)
    }
}

impl Hash for Timespec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.t.tv_sec.hash(state);
        self.t.tv_nsec.hash(state);
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

#[derive(Clone, Copy, Debug)]
pub struct Timeout {
    ts: Timespec,
    clockid: u32,
    absolute_time: bool,
}

impl Timeout {
    pub fn new(ts: Timespec, clockid: u32, absolute_time: bool) -> Timeout {
        Timeout {
            ts,
            clockid,
            absolute_time,
        }
    }
}

extern "C" {
    pub fn u_thread_wait_event_ocall(
        result: *mut i32,
        error: *mut i32,
        tcs: usize,
        timeout: *const timespec,
        clockid: i32,
        absolute_time: i32,
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
        timeout: *const timespec,
        clockid: i32,
        absolute_time: i32,
    ) -> SgxStatus;
}

pub fn thread_wait_event(tcs: TcsId, dur: Option<Duration>) -> OsResult {
    let mut result: i32 = 0;
    let mut error: i32 = 0;

    let ts: Option<Timespec> = dur.map(|dur| dur.into());
    let ts_ptr = ts
        .as_ref()
        .map_or(ptr::null(), |ts| &ts.t as *const timespec);

    let status = unsafe {
        u_thread_wait_event_ocall(
            &mut result as *mut i32,
            &mut error as *mut i32,
            tcs.as_usize(),
            ts_ptr,
            0,
            0,
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

    let ts: Option<Timespec> = dur.map(|dur| dur.into());
    let ts_ptr = ts
        .as_ref()
        .map_or(ptr::null(), |time| &time.t as *const timespec);

    let status = unsafe {
        u_thread_setwait_events_ocall(
            &mut result as *mut i32,
            &mut error as *mut i32,
            wait_tcs.as_usize(),
            self_tcs.as_usize(),
            ts_ptr,
            0,
            0,
        )
    };

    ensure!(status.is_success(), ESGX);
    ensure!(result == 0, error);
    Ok(())
}

pub fn thread_wait_event_ex(tcs: TcsId, timeout: Option<Timeout>) -> OsResult {
    let mut result: i32 = 0;
    let mut error: i32 = 0;

    let (ts_ptr, clockid, absolute_time) =
        timeout.as_ref().map_or((ptr::null(), 0, 0), |timeout| {
            let ts_ptr = &timeout.ts.t as *const timespec;
            let clockid = timeout.clockid as i32;
            let absolute_time = if timeout.absolute_time { 1 } else { 0 };
            (ts_ptr, clockid, absolute_time)
        });

    let status = unsafe {
        u_thread_wait_event_ocall(
            &mut result as *mut i32,
            &mut error as *mut i32,
            tcs.as_usize(),
            ts_ptr,
            clockid,
            absolute_time,
        )
    };

    ensure!(status.is_success(), ESGX);
    ensure!(result == 0, error);
    Ok(())
}
