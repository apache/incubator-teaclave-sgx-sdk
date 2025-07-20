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

use crate::pin::Pin;
use crate::sync::atomic::AtomicU32;
use crate::sync::atomic::Ordering::{Acquire, Release};
use crate::sys::futex::{futex_wait, futex_wake};
use crate::time::Duration;

const PARKED: u32 = u32::MAX;
const EMPTY: u32 = 0;
const NOTIFIED: u32 = 1;

pub struct Parker {
    state: AtomicU32,
}

// Notes about memory ordering:
//
// Memory ordering is only relevant for the relative ordering of operations
// between different variables. Even Ordering::Relaxed guarantees a
// monotonic/consistent order when looking at just a single atomic variable.
//
// So, since this parker is just a single atomic variable, we only need to look
// at the ordering guarantees we need to provide to the 'outside world'.
//
// The only memory ordering guarantee that parking and unparking provide, is
// that things which happened before unpark() are visible on the thread
// returning from park() afterwards. Otherwise, it was effectively unparked
// before unpark() was called while still consuming the 'token'.
//
// In other words, unpark() needs to synchronize with the part of park() that
// consumes the token and returns.
//
// This is done with a release-acquire synchronization, by using
// Ordering::Release when writing NOTIFIED (the 'token') in unpark(), and using
// Ordering::Acquire when checking for this state in park().
impl Parker {
    /// Construct the futex parker. The UNIX parker implementation
    /// requires this to happen in-place.
    #[allow(clippy::new_ret_no_self)]
    pub unsafe fn new_in_place(parker: *mut Parker) {
        parker.write(Self { state: AtomicU32::new(EMPTY) });
    }

    // Assumes this is only called by the thread that owns the Parker,
    // which means that `self.state != PARKED`.
    pub unsafe fn park(self: Pin<&Self>) {
        // Change NOTIFIED=>EMPTY or EMPTY=>PARKED, and directly return in the
        // first case.
        if self.state.fetch_sub(1, Acquire) == NOTIFIED {
            return;
        }
        loop {
            // Wait for something to happen, assuming it's still set to PARKED.
            futex_wait(&self.state, PARKED, None);
            // Change NOTIFIED=>EMPTY and return in that case.
            if self.state.compare_exchange(NOTIFIED, EMPTY, Acquire, Acquire).is_ok() {
                return;
            } else {
                // Spurious wake up. We loop to try again.
            }
        }
    }

    // Assumes this is only called by the thread that owns the Parker,
    // which means that `self.state != PARKED`. This implementation doesn't
    // require `Pin`, but other implementations do.
    #[allow(clippy::if_same_then_else)]
    pub unsafe fn park_timeout(self: Pin<&Self>, timeout: Duration) {
        // Change NOTIFIED=>EMPTY or EMPTY=>PARKED, and directly return in the
        // first case.
        if self.state.fetch_sub(1, Acquire) == NOTIFIED {
            return;
        }
        // Wait for something to happen, assuming it's still set to PARKED.
        futex_wait(&self.state, PARKED, Some(timeout));
        // This is not just a store, because we need to establish a
        // release-acquire ordering with unpark().
        if self.state.swap(EMPTY, Acquire) == NOTIFIED {
            // Woke up because of unpark().
        } else {
            // Timeout or spurious wake up.
            // We return either way, because we can't easily tell if it was the
            // timeout or not.
        }
    }

    // This implementation doesn't require `Pin`, but other implementations do.
    #[inline]
    pub fn unpark(self: Pin<&Self>) {
        // Change PARKED=>NOTIFIED, EMPTY=>NOTIFIED, or NOTIFIED=>NOTIFIED, and
        // wake the thread in the first case.
        //
        // Note that even NOTIFIED=>NOTIFIED results in a write. This is on
        // purpose, to make sure every unpark() has a release-acquire ordering
        // with park().
        if self.state.swap(NOTIFIED, Release) == PARKED {
            futex_wake(&self.state);
        }
    }
}
