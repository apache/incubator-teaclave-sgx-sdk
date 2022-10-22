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

//!
//! The Intel(R) Software Guard Extensions SDK already supports mutex and conditional
//! variable synchronization mechanisms by means of the following API and data types
//! defined in the Types and Enumerations section. Some functions included in the
//! trusted Thread Synchronization library may make calls outside the enclave (OCALLs).
//! If you use any of the APIs below, you must first import the needed OCALL functions
//! from sgx_tstd.edl. Otherwise, you will get a linker error when the enclave is
//! being built; see Calling Functions outside the Enclave for additional details.
//! The table below illustrates the primitives that the Intel(R) SGX Thread
//! Synchronization library supports, as well as the OCALLs that each API function needs.
//!

use crate::sys::locks as imp;

use sgx_libc as libc;

/// An SGX-based mutual exclusion lock, meant for use in static variables.
///
/// This mutex has a const constructor ([`StaticMutex::new`]), does not
/// implement `Drop` to cleanup resources, and causes UB when used reentrantly.
///
/// This mutex does not implement poisoning.
///
/// This is a wrapper around `imp::Mutex` that does *not* call `init()` and
/// `destroy()`.
pub struct StaticMutex(imp::Mutex);

unsafe impl Sync for StaticMutex {}

impl StaticMutex {
    /// Creates a new mutex for use.
    #[inline]
    pub const fn new() -> StaticMutex {
        StaticMutex(imp::Mutex::new())
    }

    /// Calls raw_lock() and then returns an RAII guard to guarantee the mutex
    /// will be unlocked.
    ///
    /// It is undefined behaviour to call this function while locked by the
    /// same thread.
    #[inline]
    pub unsafe fn lock(&'static self) -> StaticMutexGuard {
        let r = self.0.lock();
        debug_assert_eq!(r, Ok(()));

        StaticMutexGuard(&self.0)
    }
}

#[must_use]
pub struct StaticMutexGuard(&'static imp::Mutex);

impl Drop for StaticMutexGuard {
    #[inline]
    fn drop(&mut self) {
        let r = unsafe { self.0.unlock() };
        debug_assert_eq!(r, Ok(()));
    }
}

/// An SGX-based mutual exclusion lock.
///
/// This mutex cleans up its resources in its `Drop` implementation, may safely
/// be moved (when not borrowed), and does not cause UB when used reentrantly.
///
/// This mutex does not implement poisoning.
///
/// This is either a wrapper around `LazyBox<imp::Mutex>` or `imp::Mutex`,
/// depending on the platform. It is boxed on platforms where `imp::Mutex` may
/// not be moved.
pub struct MovableMutex(imp::MovableMutex);

unsafe impl Sync for MovableMutex {}

impl MovableMutex {
    /// Creates a new mutex.
    #[inline]
    pub const fn new() -> MovableMutex {
        MovableMutex(imp::MovableMutex::new())
    }

    pub(super) fn raw(&self) -> &imp::Mutex {
        &self.0
    }

    /// Locks the mutex blocking the current thread until it is available.
    #[inline]
    pub fn raw_lock(&self) {
        let r = unsafe { self.0.lock() };
        debug_assert_eq!(r, Ok(()));
    }

    /// Attempts to lock the mutex without blocking, returning whether it was
    /// successfully acquired or not.
    #[inline]
    pub fn try_lock(&self) -> bool {
        let r = unsafe { self.0.try_lock() };
        debug_assert!(r == Err(libc::EBUSY) || r == Ok(()));
        r == Ok(())
    }

    /// Unlocks the mutex.
    ///
    /// Behavior is undefined if the current thread does not actually hold the
    /// mutex.
    #[inline]
    pub unsafe fn raw_unlock(&self) {
        let r = self.0.unlock();
        debug_assert_eq!(r, Ok(()));
    }
}
