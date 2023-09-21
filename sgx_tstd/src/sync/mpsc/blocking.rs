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

use crate::sync::atomic::{AtomicBool, Ordering};
use crate::sync::Arc;
use crate::thread::{self, SgxThread as Thread};
use crate::time::Instant;
#[cfg(not(feature = "untrusted_time"))]
use crate::untrusted::time::InstantEx;

/// Note about memory ordering:
/// 
/// Here woken needs to synchronize with thread, So using Acquire and
/// Release is enough. Success in CAS is safer to use AcqRel, fail in CAS
/// does not synchronize other variables, and using Relaxed can ensure the 
/// correctness of the program.
struct Inner {
    thread: Thread,
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
        let wake = self
            .inner
            .woken
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok();
        if wake {
            self.inner.thread.unpark();
        }
        wake
    }

    /// Converts to an unsafe raw pointer. Useful for storing in a pipe's state
    /// flag.
    #[inline]
    pub unsafe fn to_raw(self) -> *mut u8 {
        Arc::into_raw(self.inner) as *mut u8
    }

    /// Converts from an unsafe raw pointer. Useful for retrieving a pipe's state
    /// flag.
    #[inline]
    pub unsafe fn from_raw(signal_ptr: *mut u8) -> SignalToken {
        SignalToken { inner: Arc::from_raw(signal_ptr as *mut Inner) }
    }
}

impl WaitToken {
    pub fn wait(self) {
        while !self.inner.woken.load(Ordering::Acquire) {
            thread::park()
        }
    }

    /// Returns `true` if we wake up normally.
    pub fn wait_max_until(self, end: Instant) -> bool {
        while !self.inner.woken.load(Ordering::Acquire) {
            let now = Instant::now();
            if now >= end {
                return false;
            }
            thread::park_timeout(end - now)
        }
        true
    }
}
