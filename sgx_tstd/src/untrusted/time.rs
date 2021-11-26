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

use crate::time::{Instant, SystemTime, SystemTimeError, Duration};

pub trait InstantEx {
    fn now() -> Instant;
    fn elapsed(&self) -> Duration;
}

impl InstantEx for Instant {
    /// Returns an instant corresponding to "now".
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Instant;
    /// use std::untrusted::time::InstantEx;
    ///
    /// let now = Instant::now();
    /// ```
    fn now() -> Instant {
        Instant::_now()
    }

    /// Returns the amount of time elapsed since this instant was created.
    ///
    /// # Panics
    ///
    /// This function may panic if the current time is earlier than this
    /// instant, which is something that can happen if an `Instant` is
    /// produced synthetically.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::thread::sleep;
    /// use std::time::{Duration, Instant};
    /// use std::untrusted::time::InstantEx;
    ///
    /// let instant = Instant::now();
    /// let three_secs = Duration::from_secs(3);
    /// sleep(three_secs);
    /// assert!(instant.elapsed() >= three_secs);
    /// ```
    fn elapsed(&self) -> Duration {
        Instant::_now() - *self
    }
}

pub trait SystemTimeEx {
    fn now() -> SystemTime;
    fn elapsed(&self) -> Result<Duration, SystemTimeError>;
}

impl SystemTimeEx for SystemTime {
    /// Returns the system time corresponding to "now".
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::SystemTime;
    /// use std::untrusted::time::SystemTimeEx;
    ///
    /// let sys_time = SystemTime::now();
    /// ```
    fn now() -> SystemTime {
        SystemTime::_now()
    }

    /// Returns the difference between the clock time when this
    /// system time was created, and the current clock time.
    ///
    /// This function may fail as the underlying system clock is susceptible to
    /// drift and updates (e.g., the system clock could go backwards), so this
    /// function may not always succeed. If successful, [`Ok`]`(`[`Duration`]`)` is
    /// returned where the duration represents the amount of time elapsed from
    /// this time measurement to the current time.
    ///
    /// To measure elapsed time reliably, use [`Instant`] instead.
    ///
    /// Returns an [`Err`] if `self` is later than the current system time, and
    /// the error contains how far from the current system time `self` is.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::thread::sleep;
    /// use std::time::{Duration, SystemTime};
    /// use std::untrusted::time::SystemTimeEx;
    ///
    /// let sys_time = SystemTime::now();
    /// let one_sec = Duration::from_secs(1);
    /// sleep(one_sec);
    /// assert!(sys_time.elapsed().unwrap() >= one_sec);
    /// ```
    fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        SystemTime::_now().duration_since(*self)
    }
}
