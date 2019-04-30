// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use sgx_types::{SysError, sgx_thread_t, SGX_THREAD_T_NULL};
use sgx_trts::libc;
use panic::{UnwindSafe, RefUnwindSafe};
use thread;
use super::mutex::SgxThreadMutex;
use super::condvar::SgxThreadCondvar;
use super::spinlock::SgxThreadSpinlock;
use sys_common::poison::{self, TryLockError, TryLockResult, LockResult};
use core::cell::UnsafeCell;
use core::mem;
use core::ptr;
use core::fmt;
use core::ops::{Deref, DerefMut};
use alloc::boxed::Box;

struct RwLockInfo {
    readers_num: u32,
    writers_num: u32,
    busy: u32,
    writer_thread: sgx_thread_t,
    cond: SgxThreadCondvar,
    mutex: SgxThreadMutex,
    spinlock: SgxThreadSpinlock,
}

impl RwLockInfo {

    const fn new() -> Self {
        RwLockInfo{
            readers_num: 0,
            writers_num: 0,
            busy: 0,
            writer_thread: SGX_THREAD_T_NULL,
            cond: SgxThreadCondvar::new(),
            mutex: SgxThreadMutex::new(),
            spinlock: SgxThreadSpinlock::new(),
        }
    }

    #[allow(dead_code)]
    unsafe fn ref_busy(&mut self) -> SysError {

        let ret: SysError;
        self.spinlock.lock();
        {
            if self.busy == u32::max_value() {
                ret = Err(libc::EAGAIN);
            } else {
                self.busy += 1;
                ret = Ok(());
            }
        }
        self.spinlock.unlock();
        ret
    }

    unsafe fn deref_busy(&mut self) -> SysError {

        let ret: SysError;
        self.spinlock.lock();
        {
            if self.busy == 0 {
                ret = Err(libc::EAGAIN);
            } else {
                self.busy -= 1;
                ret = Ok(());
            }
        }
        self.spinlock.unlock();
        ret
    }
}

/// An OS-based reader-writer lock.
///
/// This structure is entirely unsafe and serves as the lowest layer of a
/// cross-platform binding of system rwlocks. It is recommended to use the
/// safer types at the top level of this crate instead of this type.
pub struct SgxThreadRwLock {
    lock: UnsafeCell<RwLockInfo>,
}

unsafe impl Send for SgxThreadRwLock {}
unsafe impl Sync for SgxThreadRwLock {}

impl SgxThreadRwLock {

    /// Creates a new reader-writer lock for use.
    pub const fn new() -> Self {
        SgxThreadRwLock { lock: UnsafeCell::new(RwLockInfo::new()) }
    }
    /// Acquires shared access to the underlying lock, blocking the current
    /// thread to do so.
    pub unsafe fn read(&self) -> SysError {

        let rwlockinfo: &mut RwLockInfo = &mut *self.lock.get();

        try!(rwlockinfo.ref_busy());

        rwlockinfo.mutex.lock();
        {
            if rwlockinfo.writer_thread == thread::rsgx_thread_self() {

                rwlockinfo.mutex.unlock();
                rwlockinfo.deref_busy();
                return Err(libc::EDEADLK);
            }

            if rwlockinfo.readers_num == u32::max_value() {

                rwlockinfo.mutex.unlock();
                rwlockinfo.deref_busy();
                return Err(libc::EAGAIN);
            }

            while rwlockinfo.writers_num > 0 {
                rwlockinfo.cond.wait(&rwlockinfo.mutex);
            }

            rwlockinfo.readers_num += 1;
        }
        rwlockinfo.mutex.unlock();

        rwlockinfo.deref_busy();

        Ok(())
    }

    /// Attempts to acquire shared access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    pub unsafe fn try_read(&self) -> SysError {

        let rwlockinfo: &mut RwLockInfo = &mut *self.lock.get();

        try!(rwlockinfo.ref_busy());

        rwlockinfo.mutex.lock();
        {
            let mut ret = Ok(());
            if rwlockinfo.writer_thread == thread::rsgx_thread_self() {
                ret = Err(libc::EDEADLK);
            }
            else if rwlockinfo.readers_num == u32::max_value() {
                ret = Err(libc::EAGAIN);
            }
            else if rwlockinfo.writers_num > 0 {
                ret = Err(libc::EBUSY);
            }

            match ret {
                Ok(_) => {},
                Err(e) => {
                    rwlockinfo.mutex.unlock();
                    rwlockinfo.deref_busy();
                    return Err(e);
                }
            }

            rwlockinfo.readers_num += 1;
        }
        rwlockinfo.mutex.unlock();

        rwlockinfo.deref_busy();

        Ok(())
    }

    /// Acquires write access to the underlying lock, blocking the current thread
    /// to do so.
    pub unsafe fn write(&self) -> SysError {

        let rwlockinfo: &mut RwLockInfo = &mut *self.lock.get();

        try!(rwlockinfo.ref_busy());

        rwlockinfo.mutex.lock();
        {
            if rwlockinfo.writer_thread == thread::rsgx_thread_self() {

                rwlockinfo.mutex.unlock();
                rwlockinfo.deref_busy();
                return Err(libc::EDEADLK);
            }

            if rwlockinfo.writers_num == u32::max_value() {

                rwlockinfo.mutex.unlock();
                rwlockinfo.deref_busy();
                return Err(libc::EAGAIN);
            }

            rwlockinfo.writers_num += 1;

            while rwlockinfo.readers_num > 0 {
                rwlockinfo.cond.wait(&rwlockinfo.mutex);
            }

            while rwlockinfo.writer_thread != SGX_THREAD_T_NULL {
                rwlockinfo.cond.wait(&rwlockinfo.mutex);
            }

            rwlockinfo.writer_thread = thread::rsgx_thread_self();
        }
        rwlockinfo.mutex.unlock();

        rwlockinfo.deref_busy();

        Ok(())
    }

    /// Attempts to acquire exclusive access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    pub unsafe fn try_write(&self) -> SysError {

        let rwlockinfo: &mut RwLockInfo = &mut *self.lock.get();

        try!(rwlockinfo.ref_busy());

        rwlockinfo.mutex.lock();
        {
            let mut ret = Ok(());
            if rwlockinfo.writer_thread == thread::rsgx_thread_self() {
                ret = Err(libc::EDEADLK);
            }
            else if rwlockinfo.writers_num == u32::max_value() {
                ret = Err(libc::EAGAIN);
            }
            else if rwlockinfo.readers_num > 0 || rwlockinfo.writer_thread != SGX_THREAD_T_NULL {
                ret = Err(libc::EBUSY);
            }

            match ret {
                Ok(_) => {},
                Err(e) => {
                    rwlockinfo.mutex.unlock();
                    rwlockinfo.deref_busy();
                    return Err(e);
                }
            }

            rwlockinfo.writers_num += 1;

            rwlockinfo.writer_thread = thread::rsgx_thread_self();
        }
        rwlockinfo.mutex.unlock();

        rwlockinfo.deref_busy();

        Ok(())
    }

    /// Unlocks previously acquired shared access to this lock.
    pub unsafe fn read_unlock(&self) -> SysError {
        self.raw_unlock()
    }

    /// Unlocks previously acquired exclusive access to this lock.
    pub unsafe fn write_unlock(&self) -> SysError {
        self.raw_unlock()
    }

    unsafe fn raw_unlock(&self) -> SysError {

        let rwlockinfo: &mut RwLockInfo = &mut *self.lock.get();

        rwlockinfo.mutex.lock();
        {
            if rwlockinfo.readers_num > 0 {
                rwlockinfo.readers_num -= 1;

                if rwlockinfo.readers_num == 0 && rwlockinfo.writers_num > 0 {
                    rwlockinfo.cond.broadcast();
                }
            } else {
                if rwlockinfo.writer_thread != thread::rsgx_thread_self() {
                    rwlockinfo.mutex.unlock();
                    return Err(libc::EPERM);
                }

                rwlockinfo.writers_num -= 1;
                rwlockinfo.writer_thread = SGX_THREAD_T_NULL;

                if rwlockinfo.busy > 0 {
                    rwlockinfo.cond.broadcast();
                }
            }
        }
        rwlockinfo.mutex.unlock();

        Ok(())
    }
    /// Destroys OS-related resources with this RWLock.
    pub unsafe fn destroy(&self) -> SysError {

        let rwlockinfo: &mut RwLockInfo = &mut *self.lock.get();

        rwlockinfo.mutex.lock();
        {
            if rwlockinfo.readers_num > 0 ||
               rwlockinfo.writers_num > 0 ||
               rwlockinfo.busy > 0 {

                rwlockinfo.spinlock.unlock();
                return Err(libc::EBUSY);
            }

            rwlockinfo.cond.destroy();
            rwlockinfo.mutex.destroy();
        }
        rwlockinfo.spinlock.unlock();

        Ok(())
    }
}

/// A reader-writer lock
///
/// This type of lock allows a number of readers or at most one writer at any
/// point in time. The write portion of this lock typically allows modification
/// of the underlying data (exclusive access) and the read portion of this lock
/// typically allows for read-only access (shared access).
///
/// The priority policy of the lock is dependent on the underlying operating
/// system's implementation, and this type does not guarantee that any
/// particular policy will be used.
///
/// The type parameter `T` represents the data that this lock protects. It is
/// required that `T` satisfies `Send` to be shared across threads and `Sync` to
/// allow concurrent access through readers. The RAII guards returned from the
/// locking methods implement `Deref` (and `DerefMut` for the `write` methods)
/// to allow access to the contained of the lock.
///
/// # Poisoning
///
/// An `RwLock`, like `Mutex`, will become poisoned on a panic. Note, however,
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

impl<T: ?Sized> UnwindSafe for SgxRwLock<T> {}
impl<T: ?Sized> RefUnwindSafe for SgxRwLock<T> {}

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
    pub fn read(&self) -> LockResult<SgxRwLockReadGuard<T>> {
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
    pub fn try_read(&self) -> TryLockResult<SgxRwLockReadGuard<T>> {
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
    pub fn write(&self) -> LockResult<SgxRwLockWriteGuard<T>> {
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
    pub fn try_write(&self) -> TryLockResult<SgxRwLockWriteGuard<T>> {
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
    /// time.  You should not trust a `false` value for program correctness
    /// without additional synchronization.
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
    pub fn into_inner(self) -> LockResult<T> where T: Sized {

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
    /// take place---the mutable borrow statically guarantees no locks exist.
    ///
    /// # Errors
    ///
    /// This function will return an error if the RwLock is poisoned. An RwLock
    /// is poisoned whenever a writer panics while holding an exclusive lock. An
    /// error will only be returned if the lock would have otherwise been
    /// acquired.
    pub fn get_mut(&mut self) -> LockResult<&mut T> {

        let data = unsafe { &mut *self.data.get() };
        poison::map_result(self.poison.borrow(), |_| data)
    }
}

unsafe impl<#[may_dangle] T: ?Sized> Drop for SgxRwLock<T> {
    fn drop(&mut self) {
        // IMPORTANT: This code needs to be kept in sync with `SgxRwLock::into_inner`.
        unsafe {
            let _ = self.inner.destroy();
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SgxRwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_read() {
            Ok(guard) => f.debug_struct("RwLock").field("data", &&*guard).finish(),
            Err(TryLockError::Poisoned(err)) => {
                f.debug_struct("RwLock").field("data", &&**err.get_ref()).finish()
            },
            Err(TryLockError::WouldBlock) => {
                struct LockedPlaceholder;
                impl fmt::Debug for LockedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("<locked>") }
                }

                f.debug_struct("RwLock").field("data", &LockedPlaceholder).finish()
            }
        }
    }
}

impl<T: Default> Default for SgxRwLock<T> {
    /// Creates a new `RwLock<T>`, with the `Default` value for T.
    fn default() -> SgxRwLock<T> {
        SgxRwLock::new(Default::default())
    }
}

impl<T> From<T> for SgxRwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    /// This is equivalent to [`RwLock::new`].
    ///
    /// [`RwLock::new`]: #method.new
    fn from(t: T) -> Self {
        SgxRwLock::new(t)
    }
}

/// RAII structure used to release the shared read access of a lock when
/// dropped.
///
/// This structure is created by the [`read`] and [`try_read`] methods on
/// [`RwLock`].
pub struct SgxRwLockReadGuard<'a, T: ?Sized + 'a> {
    __lock: &'a SgxRwLock<T>,
}

impl<'a, T: ?Sized> !Send for SgxRwLockReadGuard<'a, T> {}
unsafe impl<'a, T: ?Sized + Sync> Sync for SgxRwLockReadGuard<'a, T> {}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
///
/// This structure is created by the [`write`] and [`try_write`] methods
/// on [`RwLock`].
pub struct SgxRwLockWriteGuard<'a, T: ?Sized + 'a> {
    __lock: &'a SgxRwLock<T>,
    __poison: poison::Guard,
}

impl<'a, T: ?Sized> !Send for SgxRwLockWriteGuard<'a, T> {}
unsafe impl<'a, T: ?Sized + Sync> Sync for SgxRwLockWriteGuard<'a, T> {}

impl<'rwlock, T: ?Sized> SgxRwLockReadGuard<'rwlock, T> {
    unsafe fn new(lock: &'rwlock SgxRwLock<T>)
                  -> LockResult<SgxRwLockReadGuard<'rwlock, T>> {
        poison::map_result(lock.poison.borrow(), |_| {
            SgxRwLockReadGuard {
                __lock: lock,
            }
        })
    }
}

impl<'rwlock, T: ?Sized> SgxRwLockWriteGuard<'rwlock, T> {
    unsafe fn new(lock: &'rwlock SgxRwLock<T>)
                  -> LockResult<SgxRwLockWriteGuard<'rwlock, T>> {
        poison::map_result(lock.poison.borrow(), |guard| {
            SgxRwLockWriteGuard {
                __lock: lock,
                __poison: guard,
            }
        })
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for SgxRwLockReadGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RwLockReadGuard")
            .field("lock", &self.__lock)
            .finish()
    }
}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for SgxRwLockReadGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for SgxRwLockWriteGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RwLockWriteGuard")
            .field("lock", &self.__lock)
            .finish()
    }
}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for SgxRwLockWriteGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}
impl<'rwlock, T: ?Sized> Deref for SgxRwLockReadGuard<'rwlock, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.__lock.data.get() }
    }
}

impl<'rwlock, T: ?Sized> Deref for SgxRwLockWriteGuard<'rwlock, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.__lock.data.get() }
    }
}

impl<'rwlock, T: ?Sized> DerefMut for SgxRwLockWriteGuard<'rwlock, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.__lock.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for SgxRwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        unsafe { self.__lock.inner.read_unlock(); }
    }
}

impl<'a, T: ?Sized> Drop for SgxRwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.__lock.poison.done(&self.__poison);
        unsafe { self.__lock.inner.write_unlock(); }
    }
}