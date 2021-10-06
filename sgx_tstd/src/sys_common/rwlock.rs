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

use crate::sys::rwlock as imp;
use sgx_types::SysError;

/// An OS-based reader-writer lock.
///
/// This structure is entirely unsafe and serves as the lowest layer of a
/// cross-platform binding of system rwlocks. It is recommended to use the
/// safer types at the top level of this crate instead of this type.
pub struct SgxThreadRwLock(imp::SgxThreadRwLock);

unsafe impl Send for SgxThreadRwLock {}
unsafe impl Sync for SgxThreadRwLock {}

impl SgxThreadRwLock {

    /// Creates a new reader-writer lock for use.
    pub const fn new() -> SgxThreadRwLock {
        SgxThreadRwLock(imp::SgxThreadRwLock::new())
    }

    /// Acquires shared access to the underlying lock, blocking the current
    /// thread to do so.
    #[inline]
    pub unsafe fn read(&self) -> SysError {
        self.0.read()
    }

    /// Attempts to acquire shared access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    #[inline]
    pub unsafe fn try_read(&self) -> SysError {
        self.0.try_read()
    }

    /// Acquires write access to the underlying lock, blocking the current thread
    /// to do so.
    #[inline]
    pub unsafe fn write(&self) -> SysError {
        self.0.write()
    }

    /// Attempts to acquire exclusive access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    #[inline]
    pub unsafe fn try_write(&self) -> SysError {
        self.0.try_write()
    }

    /// Unlocks previously acquired shared access to this lock.
    #[inline]
    pub unsafe fn read_unlock(&self) -> SysError {
        self.0.read_unlock()
    }

    /// Unlocks previously acquired exclusive access to this lock.
    #[inline]
    pub unsafe fn write_unlock(&self) -> SysError {
        self.0.write_unlock()
    }

    /// Destroys OS-related resources with this RWLock.
    #[inline]
    pub unsafe fn destroy(&self) -> SysError {
        self.0.destroy()
    }
}

pub struct SgxMovableThreadRwLock(imp::SgxMovableThreadRwLock);

unsafe impl Sync for SgxMovableThreadRwLock {}

impl SgxMovableThreadRwLock {
    /// Creates a new reader-writer lock for use.
    pub fn new() -> SgxMovableThreadRwLock {
        SgxMovableThreadRwLock(imp::SgxMovableThreadRwLock::from(imp::SgxThreadRwLock::new()))
    }

    /// Acquires shared access to the underlying lock, blocking the current
    /// thread to do so.
    #[inline]
    pub fn read(&self) -> SysError {
        unsafe { self.0.read() }
    }

    /// Attempts to acquire shared access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    #[inline]
    pub fn try_read(&self) -> SysError {
        unsafe { self.0.try_read() }
    }

    /// Acquires write access to the underlying lock, blocking the current thread
    /// to do so.
    ///
    /// Behavior is undefined if the rwlock has been moved between this and any
    /// previous method call.
    #[inline]
    pub fn write(&self) -> SysError {
        unsafe { self.0.write() }
    }

    /// Attempts to acquire exclusive access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    ///
    /// Behavior is undefined if the rwlock has been moved between this and any
    /// previous method call.
    #[inline]
    pub fn try_write(&self) -> SysError {
        unsafe { self.0.try_write() }
    }

    /// Unlocks previously acquired shared access to this lock.
    ///
    /// Behavior is undefined if the current thread does not have shared access.
    #[inline]
    pub unsafe fn read_unlock(&self) -> SysError {
        self.0.read_unlock()
    }

    /// Unlocks previously acquired exclusive access to this lock.
    ///
    /// Behavior is undefined if the current thread does not currently have
    /// exclusive access.
    #[inline]
    pub unsafe fn write_unlock(&self) -> SysError {
        self.0.write_unlock()
    }
}

impl Drop for SgxMovableThreadRwLock {
    fn drop(&mut self) {
        let r = unsafe { self.0.destroy() };
        debug_assert_eq!(r, Ok(()));
    }
}
