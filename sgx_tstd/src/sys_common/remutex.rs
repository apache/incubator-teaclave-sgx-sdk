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

use crate::fmt;
use crate::ops::Deref;
use crate::panic::{RefUnwindSafe, UnwindSafe};
use crate::sys::locks as sys;

/// A re-entrant mutual exclusion
///
/// This mutex will block *other* threads waiting for the lock to become
/// available. The thread which has already locked the mutex can lock it
/// multiple times without blocking, preventing a common source of deadlocks.
pub struct ReentrantMutex<T> {
    inner: sys::MovableReentrantMutex,
    data: T,
}

unsafe impl<T: Send> Send for ReentrantMutex<T> {}
unsafe impl<T: Send> Sync for ReentrantMutex<T> {}

impl<T> UnwindSafe for ReentrantMutex<T> {}
impl<T> RefUnwindSafe for ReentrantMutex<T> {}

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
#[must_use = "if unused the ReentrantMutex will immediately unlock"]
pub struct ReentrantMutexGuard<'a, T: 'a> {
    lock: &'a ReentrantMutex<T>,
}

impl<T> !Send for ReentrantMutexGuard<'_, T> {}

impl<T> ReentrantMutex<T> {
    /// Creates a new reentrant mutex in an unlocked state.
    ///
    pub const fn new(t: T) -> ReentrantMutex<T> {
        ReentrantMutex {
            inner: sys::MovableReentrantMutex::new(),
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
    pub fn lock(&self) -> ReentrantMutexGuard<'_, T> {
        unsafe {
            let _ = self.inner.lock();
        }
        ReentrantMutexGuard { lock: self }
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
    pub fn try_lock(&self) -> Option<ReentrantMutexGuard<'_, T>> {
        if unsafe { self.inner.try_lock().is_ok() } {
            Some(ReentrantMutexGuard { lock: self })
        } else {
            None
        }
    }
}

impl<T: fmt::Debug + 'static> fmt::Debug for ReentrantMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => f
                .debug_struct("ReentrantMutex")
                .field("data", &*guard)
                .finish(),
            None => {
                struct LockedPlaceholder;
                impl fmt::Debug for LockedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("<locked>")
                    }
                }

                f.debug_struct("ReentrantMutex")
                    .field("data", &LockedPlaceholder)
                    .finish()
            }
        }
    }
}

impl<T> Deref for ReentrantMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.lock.data
    }
}

impl<T> Drop for ReentrantMutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let _ = self.lock.inner.unlock();
        }
    }
}
