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

use core::iter::Sum;
use core::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign};

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_MILLI: u32 = 1_000_000;
const NANOS_PER_MICRO: u32 = 1_000;
const MILLIS_PER_SEC: u64 = 1_000;
const MICROS_PER_SEC: u64 = 1_000_000;

/// A `Duration` type to represent a span of time, typically used for system
/// timeouts.
///
/// Each `Duration` is composed of a whole number of seconds and a fractional part
/// represented in nanoseconds.  If the underlying system does not support
/// nanosecond-level precision, APIs binding a system timeout will typically round up
/// the number of nanoseconds.
///
/// `Duration`s implement many common traits, including [`Add`], [`Sub`], and other
/// [`ops`] traits.
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
pub struct Duration {
    secs: u64,
    nanos: u32, // Always 0 <= nanos < NANOS_PER_SEC
}

impl Duration {
    /// Creates a new `Duration` from the specified number of whole seconds and
    /// additional nanoseconds.
    ///
    /// If the number of nanoseconds is greater than 1 billion (the number of
    /// nanoseconds in a second), then it will carry over into the seconds provided.
    ///
    /// # Panics
    ///
    /// This constructor will panic if the carry from the nanoseconds overflows
    /// the seconds counter.
    ///
    #[inline]
    pub fn new(secs: u64, nanos: u32) -> Duration {
        let secs = secs.checked_add((nanos / NANOS_PER_SEC) as u64)
            .expect("overflow in Duration::new");
        let nanos = nanos % NANOS_PER_SEC;
        Duration { secs: secs, nanos: nanos }
    }

    /// Creates a new `Duration` from the specified number of whole seconds.
    ///
    #[inline]
    pub fn from_secs(secs: u64) -> Duration {
        Duration { secs: secs, nanos: 0 }
    }

    /// Creates a new `Duration` from the specified number of milliseconds.
    ///
    #[inline]
    pub fn from_millis(millis: u64) -> Duration {
        let secs = millis / MILLIS_PER_SEC;
        let nanos = ((millis % MILLIS_PER_SEC) as u32) * NANOS_PER_MILLI;
        Duration { secs: secs, nanos: nanos }
    }

    /// Creates a new `Duration` from the specified number of microseconds.
    ///
    #[inline]
    pub fn from_micros(micros: u64) -> Duration {
        let secs = micros / MICROS_PER_SEC;
        let nanos = ((micros % MICROS_PER_SEC) as u32) * NANOS_PER_MICRO;
        Duration { secs: secs, nanos: nanos }
    }

    /// Creates a new `Duration` from the specified number of nanoseconds.
    ///
    #[inline]
    pub fn from_nanos(nanos: u64) -> Duration {
        let secs = nanos / (NANOS_PER_SEC as u64);
        let nanos = (nanos % (NANOS_PER_SEC as u64)) as u32;
        Duration { secs: secs, nanos: nanos }
    }

    /// Returns the number of _whole_ seconds contained by this `Duration`.
    ///
    /// The returned value does not include the fractional (nanosecond) part of the
    /// duration, which can be obtained using [`subsec_nanos`].
    ///
    #[inline]
    pub fn as_secs(&self) -> u64 { self.secs }

    /// Returns the fractional part of this `Duration`, in milliseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by milliseconds. The returned number always represents a
    /// fractional portion of a second (i.e. it is less than one thousand).
    ///
    #[inline]
    pub fn subsec_millis(&self) -> u32 { self.nanos / NANOS_PER_MILLI }

    /// Returns the fractional part of this `Duration`, in microseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by microseconds. The returned number always represents a
    /// fractional portion of a second (i.e. it is less than one million).
    ///
    #[inline]
    pub fn subsec_micros(&self) -> u32 { self.nanos / NANOS_PER_MICRO }

    /// Returns the fractional part of this `Duration`, in nanoseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by nanoseconds. The returned number always represents a
    /// fractional portion of a second (i.e. it is less than one billion).
    ///
    #[inline]
    pub fn subsec_nanos(&self) -> u32 { self.nanos }

    /// Checked `Duration` addition. Computes `self + other`, returning [`None`]
    /// if overflow occurred.
    ///
    #[inline]
    pub fn checked_add(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut secs) = self.secs.checked_add(rhs.secs) {
            let mut nanos = self.nanos + rhs.nanos;
            if nanos >= NANOS_PER_SEC {
                nanos -= NANOS_PER_SEC;
                if let Some(new_secs) = secs.checked_add(1) {
                    secs = new_secs;
                } else {
                    return None;
                }
            }
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration {
                secs: secs,
                nanos: nanos,
            })
        } else {
            None
        }
    }

    /// Checked `Duration` subtraction. Computes `self - other`, returning [`None`]
    /// if the result would be negative or if overflow occurred.
    ///
    #[inline]
    pub fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut secs) = self.secs.checked_sub(rhs.secs) {
            let nanos = if self.nanos >= rhs.nanos {
                self.nanos - rhs.nanos
            } else {
                if let Some(sub_secs) = secs.checked_sub(1) {
                    secs = sub_secs;
                    self.nanos + NANOS_PER_SEC - rhs.nanos
                } else {
                    return None;
                }
            };
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration { secs: secs, nanos: nanos })
        } else {
            None
        }
    }

    /// Checked `Duration` multiplication. Computes `self * other`, returning
    /// [`None`] if overflow occurred.
    ///
    #[inline]
    pub fn checked_mul(self, rhs: u32) -> Option<Duration> {
        // Multiply nanoseconds as u64, because it cannot overflow that way.
        let total_nanos = self.nanos as u64 * rhs as u64;
        let extra_secs = total_nanos / (NANOS_PER_SEC as u64);
        let nanos = (total_nanos % (NANOS_PER_SEC as u64)) as u32;
        if let Some(secs) = self.secs
            .checked_mul(rhs as u64)
            .and_then(|s| s.checked_add(extra_secs)) {
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration {
                secs: secs,
                nanos: nanos,
            })
        } else {
            None
        }
    }

    /// Checked `Duration` division. Computes `self / other`, returning [`None`]
    /// if `other == 0`.
    ///
    #[inline]
    pub fn checked_div(self, rhs: u32) -> Option<Duration> {
        if rhs != 0 {
            let secs = self.secs / (rhs as u64);
            let carry = self.secs - secs * (rhs as u64);
            let extra_nanos = carry * (NANOS_PER_SEC as u64) / (rhs as u64);
            let nanos = self.nanos / rhs + (extra_nanos as u32);
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration { secs: secs, nanos: nanos })
        } else {
            None
        }
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        self.checked_add(rhs).expect("overflow when adding durations")
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        self.checked_sub(rhs).expect("overflow when subtracting durations")
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        self.checked_mul(rhs).expect("overflow when multiplying duration by scalar")
    }
}

impl MulAssign<u32> for Duration {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

impl Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Duration {
        self.checked_div(rhs).expect("divide by zero error when dividing duration by scalar")
    }
}

impl DivAssign<u32> for Duration {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

impl Sum for Duration {
    fn sum<I: Iterator<Item=Duration>>(iter: I) -> Duration {
        iter.fold(Duration::new(0, 0), |a, b| a + b)
    }
}

impl<'a> Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item=&'a Duration>>(iter: I) -> Duration {
        iter.fold(Duration::new(0, 0), |a, b| a + *b)
    }
}