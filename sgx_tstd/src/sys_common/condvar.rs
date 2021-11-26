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

use core::time::Duration;
use crate::sys::condvar as imp;
use crate::sys::mutex as mutex_imp;
use crate::sys_common::mutex::{SgxMovableThreadMutex, SgxThreadMutex};

use sgx_types::SysError;

mod check;

pub struct SgxThreadCondvar(imp::SgxThreadCondvar);

unsafe impl Send for SgxThreadCondvar {}
unsafe impl Sync for SgxThreadCondvar {}

impl SgxThreadCondvar {
    pub const fn new() -> SgxThreadCondvar {
        SgxThreadCondvar(imp::SgxThreadCondvar::new())
    }

    #[inline]
    pub unsafe fn wait(&self, mutex: &SgxThreadMutex) -> SysError {
        self.0.wait(mutex.raw())
    }

    #[inline]
    pub unsafe fn wait_timeout(&self, mutex: &SgxThreadMutex, dur: Duration) -> SysError {
        self.0.wait_timeout(mutex.raw(), dur)
    }

    #[inline]
    pub unsafe fn signal(&self) -> SysError {
        self.0.signal()
    }

    #[inline]
    pub unsafe fn broadcast(&self) -> SysError {
        self.0.broadcast()
    }

    #[inline]
    pub unsafe fn notify_one(&self) -> SysError {
        self.signal()
    }

    #[inline]
    pub unsafe fn notify_all(&self) -> SysError {
        self.broadcast()
    }

    #[inline]
    pub unsafe fn destroy(&self) -> SysError {
        self.0.destroy()
    }
}

type CondvarCheck = <mutex_imp::SgxMovableThreadMutex as check::CondvarCheck>::Check;

/// An OS-based condition variable.
pub struct SgxMovableThreadCondvar {
    inner: imp::SgxMovableThreadCondvar,
    check: CondvarCheck,
}

impl SgxMovableThreadCondvar {
    /// Creates a new condition variable for use.
    pub fn new() -> SgxMovableThreadCondvar {
        let c = imp::SgxMovableThreadCondvar::from(imp::SgxThreadCondvar::new());
        SgxMovableThreadCondvar { inner: c, check: CondvarCheck::new() }
    }

    #[inline]
    pub unsafe fn signal(&self) -> SysError {
        self.inner.signal()
    }

    #[inline]
    pub unsafe fn broadcast(&self) -> SysError {
        self.inner.broadcast()
    }

    /// Signals one waiter on this condition variable to wake up.
    #[inline]
    pub unsafe fn notify_one(&self) -> SysError {
        self.signal()
    }

    /// Awakens all current waiters on this condition variable.
    #[inline]
    pub unsafe fn notify_all(&self) -> SysError {
        self.broadcast()
    }

    /// Waits for a signal on the specified mutex.
    ///
    /// Behavior is undefined if the mutex is not locked by the current thread.
    ///
    /// May panic if used with more than one mutex.
    #[inline]
    pub unsafe fn wait(&self, mutex: &SgxMovableThreadMutex) -> SysError {
        self.check.verify(mutex);
        self.inner.wait(mutex.raw())
    }

    /// Waits for a signal on the specified mutex with a timeout duration
    /// specified by `dur` (a relative time into the future).
    ///
    /// Behavior is undefined if the mutex is not locked by the current thread.
    ///
    /// May panic if used with more than one mutex.
    #[inline]
    pub unsafe fn wait_timeout(&self, mutex: &SgxMovableThreadMutex, dur: Duration) -> SysError {
        self.check.verify(mutex);
        self.inner.wait_timeout(mutex.raw(), dur)
    }
}

impl Drop for SgxMovableThreadCondvar {
    fn drop(&mut self) {
        let r = unsafe { self.inner.destroy() };
        debug_assert_eq!(r, Ok(()));
    }
}
