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

use crate::sys::locks as imp;

use sgx_libc as libc;

/// An SGX-based reader-writer lock., meant for use in static variables.
///
/// This rwlock has a const constructor ([`StaticRwLock::new`]), does not
/// implement `Drop` to cleanup resources.
pub struct StaticRwLock(imp::RwLock);

unsafe impl Sync for StaticRwLock {}

impl StaticRwLock {
    #[inline]
    pub const fn new() -> StaticRwLock {
        StaticRwLock(imp::RwLock::new())
    }

    /// Acquires shared access to the underlying lock, blocking the current
    /// thread to do so.
    ///
    /// The lock is automatically unlocked when the returned guard is dropped.
    #[inline]
    pub fn read(&'static self) -> StaticRwLockReadGuard {
        // Safety: All methods require static references, therefore self
        // cannot be moved between invocations.
        let r = unsafe { self.0.read() };
        debug_assert_eq!(r, Ok(()));

        StaticRwLockReadGuard(&self.0)
    }

    /// Acquires write access to the underlying lock, blocking the current thread
    /// to do so.
    ///
    /// The lock is automatically unlocked when the returned guard is dropped.
    #[inline]
    pub fn write(&'static self) -> StaticRwLockWriteGuard {
        // Safety: All methods require static references, therefore self
        // cannot be moved between invocations.
        let r = unsafe { self.0.write() };
        debug_assert_eq!(r, Ok(()));

        StaticRwLockWriteGuard(&self.0)
    }
}

#[must_use]
pub struct StaticRwLockReadGuard(&'static imp::RwLock);

impl Drop for StaticRwLockReadGuard {
    fn drop(&mut self) {
        let r = unsafe { self.0.read_unlock() };
        debug_assert_eq!(r, Ok(()));
    }
}

#[must_use]
pub struct StaticRwLockWriteGuard(&'static imp::RwLock);

impl Drop for StaticRwLockWriteGuard {
    fn drop(&mut self) {
        let r = unsafe { self.0.write_unlock() };
        debug_assert_eq!(r, Ok(()));
    }
}

/// An SGX-based reader-writer lock.
///
/// This is either a wrapper around `LazyBox<imp::RwLock>` or `imp::RwLock`,
/// depending on the platform. It is boxed on platforms where `imp::RwLock` may
/// not be moved.
pub struct MovableRwLock(imp::MovableRwLock);

impl MovableRwLock {
    /// Creates a new reader-writer lock for use.
    #[inline]
    pub const fn new() -> Self {
        Self(imp::MovableRwLock::new())
    }

    /// Acquires shared access to the underlying lock, blocking the current
    /// thread to do so.
    #[inline]
    pub fn read(&self) {
        let r = unsafe { self.0.read() };
        debug_assert_eq!(r, Ok(()));
    }

    /// Attempts to acquire shared access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    #[inline]
    pub fn try_read(&self) -> bool {
        let r = unsafe { self.0.try_read() };
        debug_assert!(r == Err(libc::EBUSY) || r == Ok(()));
        r == Ok(())
    }

    /// Acquires write access to the underlying lock, blocking the current thread
    /// to do so.
    #[inline]
    pub fn write(&self) {
        let r = unsafe { self.0.write() };
        debug_assert_eq!(r, Ok(()));
    }

    /// Attempts to acquire exclusive access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    #[inline]
    pub fn try_write(&self) -> bool {
        let r = unsafe { self.0.try_write() };
        debug_assert!(r == Err(libc::EBUSY) || r == Ok(()));
        r == Ok(())
    }

    /// Unlocks previously acquired shared access to this lock.
    ///
    /// Behavior is undefined if the current thread does not have shared access.
    #[inline]
    pub unsafe fn read_unlock(&self) {
        let r = self.0.read_unlock();
        debug_assert_eq!(r, Ok(()));
    }

    /// Unlocks previously acquired exclusive access to this lock.
    ///
    /// Behavior is undefined if the current thread does not currently have
    /// exclusive access.
    #[inline]
    pub unsafe fn write_unlock(&self) {
        let r = self.0.write_unlock();
        debug_assert_eq!(r, Ok(()));
    }
}
