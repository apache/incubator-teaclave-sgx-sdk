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


use alloc_crate::boxed::Box;
use core::fmt;
use core::ops::Deref;
use crate::sys_common::poison::{self, LockResult, TryLockError, TryLockResult};
use crate::sys_common::remutex as sys;

pub use crate::sys_common::remutex::SgxReentrantThreadMutex;


/// A re-entrant mutual exclusion
///
/// This mutex will block *other* threads waiting for the lock to become
/// available. The thread which has already locked the mutex can lock it
/// multiple times without blocking, preventing a common source of deadlocks.
pub struct SgxReentrantMutex<T> {
    inner: Box<sys::SgxReentrantThreadMutex>,
    poison: poison::Flag,
    data: T,
}

unsafe impl<T: Send> Send for SgxReentrantMutex<T> {}
unsafe impl<T: Send> Sync for SgxReentrantMutex<T> {}

impl<T> SgxReentrantMutex<T> {
    /// Creates a new reentrant mutex in an unlocked state.
    pub fn new(t: T) -> SgxReentrantMutex<T> {
        SgxReentrantMutex{
            inner: Box::new(sys::SgxReentrantThreadMutex::new()),
            poison: poison::Flag::new(),
            data: t,
        }
    }

    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// This function will block the caller until it is available to acquire the mutex.
    /// Upon returning, the thread is the only thread with the mutex held. When the thread
    /// calling this method already holds the lock, the call shall succeed without
    /// blocking.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return failure if the mutex would otherwise be
    /// acquired.
    pub fn lock(&self) -> LockResult<SgxReentrantMutexGuard<'_, T>> {
        unsafe {
            self.inner.lock();
            SgxReentrantMutexGuard::new(self)
        }
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned.
    ///
    /// This function does not block.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return failure if the mutex would otherwise be
    /// acquired.
    pub fn try_lock(&self) -> TryLockResult<SgxReentrantMutexGuard<'_, T>> {
        unsafe {
            match self.inner.try_lock() {
                Ok(_) => Ok(SgxReentrantMutexGuard::new(self)?),
                Err(_) => Err(TryLockError::WouldBlock),
            }
        }
    }
}

impl<T> Drop for SgxReentrantMutex<T> {
    fn drop(&mut self) {
        // This is actually safe b/c we know that there is no further usage of
        // this mutex (it's up to the user to arrange for a mutex to get
        // dropped, that's not our job)
        let result = unsafe { self.inner.destroy() };
        debug_assert_eq!(result, Ok(()), "Error when destroy an SgxReentrantMutex: {}", result.unwrap_err());
    }
}

impl<T: fmt::Debug + 'static> fmt::Debug for SgxReentrantMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_lock() {
            Ok(guard) => f.debug_struct("SgxReentrantMutex").field("data", &*guard).finish(),
            Err(TryLockError::Poisoned(err)) => {
                f.debug_struct("SgxReentrantMutex").field("data", &**err.get_ref()).finish()
            },
            Err(TryLockError::WouldBlock) => {
                struct LockedPlaceholder;
                impl fmt::Debug for LockedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("<locked>")
                    }
                }

                f.debug_struct("SgxReentrantMutex").field("data", &LockedPlaceholder).finish()
            }
        }
    }
}

/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// Deref implementation.
///
/// # Mutability
///
/// Unlike `MutexGuard`, `ReentrantMutexGuard` does not implement `DerefMut`,
/// because implementation of the trait would violate Rust’s reference aliasing
/// rules. Use interior mutability (usually `RefCell`) in order to mutate the
/// guarded data.
#[must_use]
pub struct SgxReentrantMutexGuard<'a, T: 'a> {
    // funny underscores due to how Deref currently works (it disregards field
    // privacy).
    lock: &'a SgxReentrantMutex<T>,
    poison: poison::Guard,
}

impl<T> !Send for SgxReentrantMutexGuard<'_, T> {}

impl<'mutex, T> SgxReentrantMutexGuard<'mutex, T> {
    fn new(lock: &'mutex SgxReentrantMutex<T>) -> LockResult<SgxReentrantMutexGuard<'mutex, T>> {
        poison::map_result(lock.poison.borrow(), |guard| {
            SgxReentrantMutexGuard {
                lock: lock,
                poison: guard,
            }
        })
    }
}

impl<T> Deref for SgxReentrantMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.lock.data
    }
}

impl<T> Drop for SgxReentrantMutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        let result = unsafe {
            self.lock.poison.done(&self.poison);
            self.lock.inner.unlock()
        };
        debug_assert_eq!(result, Ok(()), "Error when unlocking an SgxReentrantMutex: {}", result.unwrap_err());
    }
}

impl<T: fmt::Debug> fmt::Debug for SgxReentrantMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for SgxReentrantMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}
