// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use core::mem;
use sys::time;
use time::{Instant, SystemTime, SystemTimeError, Duration};

pub trait InstantEx {
    fn now() -> Instant;
    fn elapsed(&self) -> Duration;
}

impl InstantEx for Instant {
    /// Returns an instant corresponding to "now".
    ///
    fn now() -> Instant {
        let instant = inner::Instant(time::Instant::now());
        unsafe { mem::transmute(instant) }
    }

    /// Returns the amount of time elapsed since this instant was created.
    ///
    /// # Panics
    ///
    /// This function may panic if the current time is earlier than this
    /// instant, which is something that can happen if an `Instant` is
    /// produced synthetically.
    ///
    fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }
}

pub trait SystemTimeEx {
    fn now() -> SystemTime;
    fn elapsed(&self) -> Result<Duration, SystemTimeError>;
}

impl SystemTimeEx for SystemTime {
    /// Returns the system time corresponding to "now".
    ///
    fn now() -> SystemTime {
        let systemtime = inner::SystemTime(time::SystemTime::now());
        unsafe { mem::transmute(systemtime) }
    }

    /// Returns the amount of time elapsed since this system time was created.
    ///
    /// This function may fail as the underlying system clock is susceptible to
    /// drift and updates (e.g. the system clock could go backwards), so this
    /// function may not always succeed. If successful, [`Ok`]`(`[`Duration`]`)` is
    /// returned where the duration represents the amount of time elapsed from
    /// this time measurement to the current time.
    ///
    /// Returns an [`Err`] if `self` is later than the current system time, and
    /// the error contains how far from the current system time `self` is.
    ///
    fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        SystemTime::now().duration_since(*self)
    }
}

mod inner {
    use sys::time;
    pub struct Instant(pub time::Instant);
    pub struct SystemTime(pub time::SystemTime);
}