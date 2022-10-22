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

use crate::fmt;
use crate::time::Duration;

pub use self::inner::Instant;

use sgx_libc as libc;

const NSEC_PER_SEC: u64 = 1_000_000_000;
pub const UNIX_EPOCH: SystemTime = SystemTime { t: Timespec::zero() };

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[rustc_layout_scalar_valid_range_start(0)]
#[rustc_layout_scalar_valid_range_end(999_999_999)]
struct Nanoseconds(u32);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime {
    pub(in crate::sys) t: Timespec,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(in crate::sys) struct Timespec {
    tv_sec: i64,
    tv_nsec: Nanoseconds,
}

impl SystemTime {
    pub fn new(tv_sec: i64, tv_nsec: i64) -> SystemTime {
        SystemTime { t: Timespec::new(tv_sec, tv_nsec) }
    }

    pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
        self.t.sub_timespec(&other.t)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime { t: self.t.checked_add_duration(other)? })
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime { t: self.t.checked_sub_duration(other)? })
    }

    pub fn get_tup(&self) -> (i64, i64) {
        (self.t.tv_sec, self.t.tv_nsec.0 as i64)
    }
}

impl From<libc::timespec> for SystemTime {
    fn from(t: libc::timespec) -> SystemTime {
        SystemTime { t: Timespec::from(t) }
    }
}

impl fmt::Debug for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemTime")
            .field("tv_sec", &self.t.tv_sec)
            .field("tv_nsec", &self.t.tv_nsec.0)
            .finish()
    }
}

impl Timespec {
    pub const fn zero() -> Timespec {
        Timespec::new(0, 0)
    }

    const fn new(tv_sec: i64, tv_nsec: i64) -> Timespec {
        assert!(tv_nsec >= 0 && tv_nsec < NSEC_PER_SEC as i64);
        // SAFETY: The assert above checks tv_nsec is within the valid range
        Timespec { tv_sec, tv_nsec: unsafe { Nanoseconds(tv_nsec as u32) } }
    }

    pub fn sub_timespec(&self, other: &Timespec) -> Result<Duration, Duration> {
        if self >= other {
            // NOTE(eddyb) two aspects of this `if`-`else` are required for LLVM
            // to optimize it into a branchless form (see also #75545):
            //
            // 1. `self.tv_sec - other.tv_sec` shows up as a common expression
            //    in both branches, i.e. the `else` must have its `- 1`
            //    subtraction after the common one, not interleaved with it
            //    (it used to be `self.tv_sec - 1 - other.tv_sec`)
            //
            // 2. the `Duration::new` call (or any other additional complexity)
            //    is outside of the `if`-`else`, not duplicated in both branches
            //
            // Ideally this code could be rearranged such that it more
            // directly expresses the lower-cost behavior we want from it.
            let (secs, nsec) = if self.tv_nsec.0 >= other.tv_nsec.0 {
                ((self.tv_sec - other.tv_sec) as u64, self.tv_nsec.0 - other.tv_nsec.0)
            } else {
                (
                    (self.tv_sec - other.tv_sec - 1) as u64,
                    self.tv_nsec.0 + (NSEC_PER_SEC as u32) - other.tv_nsec.0,
                )
            };

            Ok(Duration::new(secs, nsec))
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
            .try_into() // <- target type would be `i64`
            .ok()
            .and_then(|secs| self.tv_sec.checked_add(secs))?;

        // Nano calculations can't overflow because nanos are <1B which fit
        // in a u32.
        let mut nsec = other.subsec_nanos() + self.tv_nsec.0;
        if nsec >= NSEC_PER_SEC as u32 {
            nsec -= NSEC_PER_SEC as u32;
            secs = secs.checked_add(1)?;
        }
        Some(Timespec::new(secs, nsec as i64))
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Timespec> {
        let mut secs = other
            .as_secs()
            .try_into() // <- target type would be `i64`
            .ok()
            .and_then(|secs| self.tv_sec.checked_sub(secs))?;

        // Similar to above, nanos can't overflow.
        let mut nsec = self.tv_nsec.0 as i32 - other.subsec_nanos() as i32;
        if nsec < 0 {
            nsec += NSEC_PER_SEC as i32;
            secs = secs.checked_sub(1)?;
        }
        Some(Timespec::new(secs, nsec as i64))
    }

    #[allow(dead_code)]
    #[allow(clippy::useless_conversion)]
    pub fn to_timespec(&self) -> Option<libc::timespec> {
        Some(libc::timespec {
            tv_sec: self.tv_sec.try_into().ok()?,
            tv_nsec: self.tv_nsec.0.try_into().ok()?,
        })
    }
}

impl From<libc::timespec> for Timespec {
    fn from(t: libc::timespec) -> Timespec {
        Timespec::new(t.tv_sec, t.tv_nsec)
    }
}

mod inner {
    use crate::fmt;
    use crate::mem::MaybeUninit;
    use crate::sys::cvt;
    use crate::time::Duration;

    use super::libc;
    use super::{SystemTime, Timespec};

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant {
        t: Timespec,
    }

    impl Instant {
        pub fn now() -> Instant {
            Instant { t: Timespec::now(libc::CLOCK_MONOTONIC) }
        }

        pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
            self.t.sub_timespec(&other.t).ok()
        }

        pub fn checked_add_duration(&self, other: &Duration) -> Option<Instant> {
            Some(Instant { t: self.t.checked_add_duration(other)? })
        }

        pub fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
            Some(Instant { t: self.t.checked_sub_duration(other)? })
        }

        pub fn get_tup(&self) -> (i64, i64) {
            (self.t.tv_sec, self.t.tv_nsec.0 as i64)
        }
    }

    impl fmt::Debug for Instant {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Instant")
                .field("tv_sec", &self.t.tv_sec)
                .field("tv_nsec", &self.t.tv_nsec.0)
                .finish()
        }
    }

    impl SystemTime {
        pub fn now() -> SystemTime {
            SystemTime { t: Timespec::now(libc::CLOCK_REALTIME) }
        }
    }

    pub type clock_t = libc::c_int;

    impl Timespec {
        pub fn now(clock: clock_t) -> Timespec {
            let mut t = MaybeUninit::uninit();
            cvt(unsafe { libc::ocall::clock_gettime(clock, t.as_mut_ptr()) }).unwrap();
            Timespec::from(unsafe { t.assume_init() })
        }
    }
}
