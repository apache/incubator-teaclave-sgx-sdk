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

//! Generic support for building blocking abstractions.

use core::sync::atomic::{AtomicBool, Ordering};
use core::mem;
use alloc_crate::sync::Arc;
use crate::thread::{self, SgxThread};
use crate::time::Instant;
#[cfg(not(feature = "untrusted_time"))]
use crate::untrusted::time::InstantEx;

struct Inner {
    thread: SgxThread,
    woken: AtomicBool,
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

#[derive(Clone)]
pub struct SignalToken {
    inner: Arc<Inner>,
}

pub struct WaitToken {
    inner: Arc<Inner>,
}

impl !Send for WaitToken {}

impl !Sync for WaitToken {}

pub fn tokens() -> (WaitToken, SignalToken) {
    let inner = Arc::new(Inner { thread: thread::current(), woken: AtomicBool::new(false) });
    let wait_token = WaitToken { inner: inner.clone() };
    let signal_token = SignalToken { inner };
    (wait_token, signal_token)
}

impl SignalToken {
    pub fn signal(&self) -> bool {
        let wake = !self.inner.woken.compare_and_swap(false, true, Ordering::SeqCst);
        if wake {
            self.inner.thread.unpark();
        }
        wake
    }

    /// Converts to an unsafe usize value. Useful for storing in a pipe's state
    /// flag.
    #[inline]
    pub unsafe fn cast_to_usize(self) -> usize {
        mem::transmute(self.inner)
    }

    /// Converts from an unsafe usize value. Useful for retrieving a pipe's state
    /// flag.
    #[inline]
    pub unsafe fn cast_from_usize(signal_ptr: usize) -> SignalToken {
        SignalToken { inner: mem::transmute(signal_ptr) }
    }
}

impl WaitToken {
    pub fn wait(self) {
        while !self.inner.woken.load(Ordering::SeqCst) {
            thread::park()
        }
    }

    /// Returns `true` if we wake up normally.
    pub fn wait_max_until(self, end: Instant) -> bool {
        while !self.inner.woken.load(Ordering::SeqCst) {
            let now = Instant::now();
            if now >= end {
                return false;
            }
            thread::park_timeout(end - now)
        }
        true
    }
}
