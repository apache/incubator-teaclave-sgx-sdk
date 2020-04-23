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

use sgx_types::SysError;
use sgx_trts::libc;
use core::cell::UnsafeCell;
use core::mem;
use core::ptr;
use core::fmt;
use core::ops::{Deref, DerefMut};
use alloc_crate::boxed::Box;
use crate::sys_common::poison::{self, LockResult, TryLockError, TryLockResult};
use crate::sys::rwlock as imp;

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
    pub const fn new() -> Self {
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

/// A reader-writer lock
///
/// This type of lock allows a number of readers or at most one writer at any
/// point in time. The write portion of this lock typically allows modification
/// of the underlying data (exclusive access) and the read portion of this lock
/// typically allows for read-only access (shared access).
///
/// In comparison, a [`Mutex`] does not distinguish between readers or writers
/// that acquire the lock, therefore blocking any threads waiting for the lock to
/// become available. An `RwLock` will allow any number of readers to acquire the
/// lock as long as a writer is not holding the lock.
///
/// The priority policy of the lock is dependent on the underlying operating
/// system's implementation, and this type does not guarantee that any
/// particular policy will be used.
///
/// The type parameter `T` represents the data that this lock protects. It is
/// required that `T` satisfies [`Send`] to be shared across threads and
/// [`Sync`] to allow concurrent access through readers. The RAII guards
/// returned from the locking methods implement [`Deref`] (and [`DerefMut`]
/// for the `write` methods) to allow access to the content of the lock.
///
/// # Poisoning
///
/// An `RwLock`, like [`Mutex`], will become poisoned on a panic. Note, however,
/// that an `RwLock` may only be poisoned if a panic occurs while it is locked
/// exclusively (write mode). If a panic occurs in any reader, then the lock
/// will not be poisoned.
///
pub struct SgxRwLock<T: ?Sized> {
    inner: Box<SgxThreadRwLock>,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for SgxRwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for SgxRwLock<T> {}

/// RAII structure used to release the shared read access of a lock when
/// dropped.
///
/// This structure is created by the [`read`] and [`try_read`] methods on
/// [`RwLock`].
pub struct SgxRwLockReadGuard<'a, T: ?Sized + 'a> {
    lock: &'a SgxRwLock<T>,
    poison: poison::Guard,
}

impl<T: ?Sized> !Send for SgxRwLockReadGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for SgxRwLockReadGuard<'_, T> {}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
///
/// This structure is created by the [`write`] and [`try_write`] methods
/// on [`RwLock`].
pub struct SgxRwLockWriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a SgxRwLock<T>,
    poison: poison::Guard,
}

impl<T: ?Sized> !Send for SgxRwLockWriteGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for SgxRwLockWriteGuard<'_, T> {}

impl<T> SgxRwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    pub fn new(t: T) -> SgxRwLock<T> {
        SgxRwLock {
            inner: Box::new(SgxThreadRwLock::new()),
            poison: poison::Flag::new(),
            data: UnsafeCell::new(t),
        }
    }
}

impl<T: ?Sized> SgxRwLock<T> {
    /// Locks this rwlock with shared read access, blocking the current thread
    /// until it can be acquired.
    ///
    /// The calling thread will be blocked until there are no more writers which
    /// hold the lock. There may be other readers currently inside the lock when
    /// this method returns. This method does not provide any guarantees with
    /// respect to the ordering of whether contentious readers or writers will
    /// acquire the lock first.
    ///
    /// Returns an RAII guard which will release this thread's shared access
    /// once it is dropped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the RwLock is poisoned. An RwLock
    /// is poisoned whenever a writer panics while holding an exclusive lock.
    /// The failure will occur immediately after the lock has been acquired.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    pub fn read(&self) -> LockResult<SgxRwLockReadGuard<'_, T>> {
        unsafe {
            let ret = self.inner.read();
            match ret {
                Err(libc::EAGAIN) => panic!("rwlock maximum reader count exceeded"),
                Err(libc::EDEADLK) => panic!("rwlock read lock would result in deadlock"),
                _ => SgxRwLockReadGuard::new(self),
            }
        }
    }

    /// Attempts to acquire this rwlock with shared read access.
    ///
    /// If the access could not be granted at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned which will release the shared access
    /// when it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    /// # Errors
    ///
    /// This function will return an error if the RwLock is poisoned. An RwLock
    /// is poisoned whenever a writer panics while holding an exclusive lock. An
    /// error will only be returned if the lock would have otherwise been
    /// acquired.
    pub fn try_read(&self) -> TryLockResult<SgxRwLockReadGuard<'_, T>> {
        unsafe {
            let ret = self.inner.try_read();
            match ret {
                Ok(_) => Ok(SgxRwLockReadGuard::new(self)?),
                Err(_) => Err(TryLockError::WouldBlock),
            }
        }
    }

    /// Locks this rwlock with exclusive write access, blocking the current
    /// thread until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this rwlock
    /// when dropped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the RwLock is poisoned. An RwLock
    /// is poisoned whenever a writer panics while holding an exclusive lock.
    /// An error will be returned when the lock is acquired.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    pub fn write(&self) -> LockResult<SgxRwLockWriteGuard<'_, T>> {
        unsafe {
            let ret = self.inner.write();
            match ret {
                Err(libc::EAGAIN) => panic!("rwlock maximum writer count exceeded"),
                Err(libc::EDEADLK) => panic!("rwlock write lock would result in deadlock"),
                _ => SgxRwLockWriteGuard::new(self),
            }
        }
    }

    /// Attempts to lock this rwlock with exclusive write access.
    ///
    /// If the lock could not be acquired at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned which will release the lock when
    /// it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    /// # Errors
    ///
    /// This function will return an error if the RwLock is poisoned. An RwLock
    /// is poisoned whenever a writer panics while holding an exclusive lock. An
    /// error will only be returned if the lock would have otherwise been
    /// acquired.
    pub fn try_write(&self) -> TryLockResult<SgxRwLockWriteGuard<'_, T>> {
        unsafe {
            let ret = self.inner.try_write();
            match ret {
                Ok(_) => Ok(SgxRwLockWriteGuard::new(self)?),
                Err(_) => Err(TryLockError::WouldBlock),
            }
        }
    }

    /// Determines whether the lock is poisoned.
    ///
    /// If another thread is active, the lock can still become poisoned at any
    /// time. You should not trust a `false` value for program correctness
    /// without additional synchronization.
    ///
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    /// Consumes this `RwLock`, returning the underlying data.
    ///
    /// # Errors
    ///
    /// This function will return an error if the RwLock is poisoned. An RwLock
    /// is poisoned whenever a writer panics while holding an exclusive lock. An
    /// error will only be returned if the lock would have otherwise been
    /// acquired.
    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock the inner lock.
        //
        // To get the inner value, we'd like to call `data.into_inner()`,
        // but because `RwLock` impl-s `Drop`, we can't move out of it, so
        // we'll have to destructure it manually instead.
        unsafe {
            let (inner, poison, data) = {
                let SgxRwLock { ref inner, ref poison, ref data } = self;
                (ptr::read(inner), ptr::read(poison), ptr::read(data))
            };
            mem::forget(self);
            let _ = inner.destroy();
            drop(inner);

            poison::map_result(poison.borrow(), |_| data.into_inner())
        }
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `RwLock` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    ///
    /// # Errors
    ///
    /// This function will return an error if the RwLock is poisoned. An RwLock
    /// is poisoned whenever a writer panics while holding an exclusive lock. An
    /// error will only be returned if the lock would have otherwise been
    /// acquired.
    ///
    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        // We know statically that there are no other references to `self`, so
        // there's no need to lock the inner lock.
        let data = unsafe { &mut *self.data.get() };
        poison::map_result(self.poison.borrow(), |_| data)
    }
}

unsafe impl<#[may_dangle] T: ?Sized> Drop for SgxRwLock<T> {
    fn drop(&mut self) {
        // IMPORTANT: This code needs to be kept in sync with `SgxRwLock::into_inner`.
        let result = unsafe { self.inner.destroy() };
        debug_assert_eq!(result, Ok(()), "Error when destroy an SgxMutex: {}", result.unwrap_err());
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SgxRwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_read() {
            Ok(guard) => f.debug_struct("SgxRwLock").field("data", &&*guard).finish(),
            Err(TryLockError::Poisoned(err)) => {
                f.debug_struct("SgxRwLock").field("data", &&**err.get_ref()).finish()
            }
            Err(TryLockError::WouldBlock) => {
                struct LockedPlaceholder;
                impl fmt::Debug for LockedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("<locked>")
                    }
                }

                f.debug_struct("SgxRwLock").field("data", &LockedPlaceholder).finish()
            }
        }
    }
}

impl<T: Default> Default for SgxRwLock<T> {
    /// Creates a new `SgxRwLock<T>`, with the `Default` value for T.
    fn default() -> SgxRwLock<T> {
        SgxRwLock::new(Default::default())
    }
}

impl<T> From<T> for SgxRwLock<T> {
    /// Creates a new instance of an `SgxRwLock<T>` which is unlocked.
    /// This is equivalent to [`SgxRwLock::new`].
    ///
    /// [`SgxRwLock::new`]: #method.new
    fn from(t: T) -> Self {
        SgxRwLock::new(t)
    }
}

impl<'rwlock, T: ?Sized> SgxRwLockReadGuard<'rwlock, T> {

    unsafe fn new(lock: &'rwlock SgxRwLock<T>) -> LockResult<SgxRwLockReadGuard<'rwlock, T>> {
            poison::map_result(lock.poison.borrow(), |guard| SgxRwLockReadGuard { lock, poison: guard })
    }
}

impl<'rwlock, T: ?Sized> SgxRwLockWriteGuard<'rwlock, T> {
    unsafe fn new(lock: &'rwlock SgxRwLock<T>) -> LockResult<SgxRwLockWriteGuard<'rwlock, T>> {
        poison::map_result(lock.poison.borrow(), |guard| SgxRwLockWriteGuard { lock, poison: guard })
    }
}

impl<T: fmt::Debug> fmt::Debug for SgxRwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SgxRwLockReadGuard").field("lock", &self.lock).finish()
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for SgxRwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: fmt::Debug> fmt::Debug for SgxRwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SgxRwLockWriteGuard").field("lock", &self.lock).finish()
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for SgxRwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized> Deref for SgxRwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> Deref for SgxRwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for SgxRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for SgxRwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        let result = unsafe {
            self.lock.poison.done(&self.poison);
            self.lock.inner.read_unlock()
        };
        debug_assert_eq!(result, Ok(()), "Error when unlocking an SgxRwLock: {}", result.unwrap_err());
    }
}

impl<T: ?Sized> Drop for SgxRwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        let result = unsafe {
            self.lock.poison.done(&self.poison);
            self.lock.inner.write_unlock()
        };
        debug_assert_eq!(result, Ok(()), "Error when unlocking an SgxRwLock: {}", result.unwrap_err());
    }
}