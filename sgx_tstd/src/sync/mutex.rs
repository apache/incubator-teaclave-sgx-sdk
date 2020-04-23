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
use sgx_types::SysError;
use core::cell::UnsafeCell;
use core::mem;
use core::ptr;
use core::fmt;
use core::ops::{Deref, DerefMut};
use alloc_crate::boxed::Box;
use crate::sys_common::poison::{self, LockResult, TryLockError, TryLockResult};
use crate::sys::mutex as imp;

/// The structure of sgx mutex.
pub struct SgxThreadMutex(imp::SgxThreadMutex);

unsafe impl Send for SgxThreadMutex {}
unsafe impl Sync for SgxThreadMutex {}

impl SgxThreadMutex {
    ///
    /// The function initializes a trusted mutex object within the enclave.
    ///
    /// # Description
    ///
    /// When a thread creates a mutex within an enclave, sgx_thread_mutex_
    /// init simply initializes the various fields of the mutex object to indicate that
    /// the mutex is available. rsgx_thread_mutex_init creates a non-recursive
    /// mutex. The results of using a mutex in a lock or unlock operation before it has
    /// been fully initialized (for example, the function call to rsgx_thread_mutex_
    /// init returns) are undefined. To avoid race conditions in the initialization of a
    /// trusted mutex, it is recommended statically initializing the mutex with the
    /// macro SGX_THREAD_MUTEX_INITIALIZER, SGX_THREAD_NON_RECURSIVE_MUTEX_INITIALIZER ,
    /// of, or SGX_THREAD_RECURSIVE_MUTEX_INITIALIZER instead.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tstdc.a
    ///
    /// # Return value
    ///
    /// The trusted mutex object to be initialized.
    ///
    pub const fn new() -> SgxThreadMutex {
        SgxThreadMutex(imp::SgxThreadMutex::new(imp::SgxThreadMutexControl::SGX_THREAD_MUTEX_NONRECURSIVE))
    }

    ///
    /// The function locks a trusted mutex object within an enclave.
    ///
    /// # Description
    ///
    /// To acquire a mutex, a thread first needs to acquire the corresponding spin
    /// lock. After the spin lock is acquired, the thread checks whether the mutex is
    /// available. If the queue is empty or the thread is at the head of the queue the
    /// thread will now become the owner of the mutex. To confirm its ownership, the
    /// thread updates the refcount and owner fields. If the mutex is not available, the
    /// thread searches the queue. If the thread is already in the queue, but not at the
    /// head, it means that the thread has previously tried to lock the mutex, but it
    /// did not succeed and had to wait outside the enclave and it has been
    /// awakened unexpectedly. When this happens, the thread makes an OCALL and
    /// simply goes back to sleep. If the thread is trying to lock the mutex for the first
    /// time, it will update the waiting queue and make an OCALL to get suspended.
    /// Note that threads release the spin lock after acquiring the mutex or before
    /// leaving the enclave.
    ///
    /// **Note**
    ///
    /// A thread should not exit an enclave returning from a root ECALL after acquiring
    /// the ownership of a mutex. Do not split the critical section protected by a
    /// mutex across root ECALLs.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tstdc.a
    ///
    /// # Errors
    ///
    /// **EINVAL**
    ///
    /// The trusted mutex object is invalid.
    ///
    #[inline]
    pub unsafe fn lock(&self) -> SysError {
        self.0.lock()
    }

    ///
    /// The function tries to lock a trusted mutex object within an enclave.
    ///
    /// # Description
    ///
    /// A thread may check the status of the mutex, which implies acquiring the spin
    /// lock and verifying that the mutex is available and that the queue is empty or
    /// the thread is at the head of the queue. When this happens, the thread
    /// acquires the mutex, releases the spin lock and returns 0. Otherwise, the
    /// thread releases the spin lock and returns EINVAL/EBUSY. The thread is not suspended
    /// in this case.
    ///
    /// **Note**
    ///
    /// A thread should not exit an enclave returning from a root ECALL after acquiring
    /// the ownership of a mutex. Do not split the critical section protected by a
    /// mutex across root ECALLs.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tstdc.a
    ///
    /// # Errors
    ///
    /// **EINVAL**
    ///
    /// The trusted mutex object is invalid.
    ///
    /// **EBUSY**
    ///
    /// The mutex is locked by another thread or has pending threads to acquire the mutex
    ///
    #[inline]
    pub unsafe fn try_lock(&self) -> SysError {
        self.0.try_lock()
    }

    ///
    /// The function unlocks a trusted mutex object within an enclave.
    ///
    /// # Description
    ///
    /// Before a thread releases a mutex, it has to verify it is the owner of the mutex. If
    /// that is the case, the thread decreases the refcount by 1 and then may either
    /// continue normal execution or wakeup the first thread in the queue. Note that
    /// to ensure the state of the mutex remains consistent, the thread that is
    /// awakened by the thread releasing the mutex will then try to acquire the
    /// mutex almost as in the initial call to the rsgx_thread_mutex_lock routine.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tstdc.a
    ///
    /// # Errors
    ///
    /// **EINVAL**
    ///
    /// The trusted mutex object is invalid or it is not locked by any thread.
    ///
    /// **EPERM**
    ///
    /// The mutex is locked by another thread.
    ///
    #[inline]
    pub unsafe fn unlock(&self) -> SysError {
        self.0.unlock()
    }

    ///
    /// The function destroys a trusted mutex object within an enclave.
    ///
    /// # Description
    ///
    /// rsgx_thread_mutex_destroy resets the mutex, which brings it to its initial
    /// status. In this process, certain fields are checked to prevent releasing a mutex
    /// that is still owned by a thread or on which threads are still waiting.
    ///
    /// **Note**
    ///
    /// Locking or unlocking a mutex after it has been destroyed results in undefined
    /// behavior. After a mutex is destroyed, it must be re-created before it can be
    /// used again.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tstdc.a
    ///
    /// # Errors
    ///
    /// **EINVAL**
    ///
    /// The trusted mutex object is invalid.
    ///
    /// **EBUSY**
    ///
    /// The mutex is locked by another thread or has pending threads to acquire the mutex.
    ///
    #[inline]
    pub unsafe fn destroy(&self) -> SysError {
        self.0.destroy()
    }
}

pub fn raw(mutex: &SgxThreadMutex) -> &imp::SgxThreadMutex { &mutex.0 }

/// A mutual exclusion primitive useful for protecting shared data
///
/// This mutex will block threads waiting for the lock to become available. The
/// mutex can also be statically initialized or created via a [`new`]
/// constructor. Each mutex has a type parameter which represents the data that
/// it is protecting. The data can only be accessed through the RAII guards
/// returned from [`lock`] and [`try_lock`], which guarantees that the data is only
/// ever accessed when the mutex is locked.
///
/// # Poisoning
///
/// The mutexes in this module implement a strategy called "poisoning" where a
/// mutex is considered poisoned whenever a thread panics while holding the
/// mutex. Once a mutex is poisoned, all other threads are unable to access the
/// data by default as it is likely tainted (some invariant is not being
/// upheld).
///
/// For a mutex, this means that the [`lock`] and [`try_lock`] methods return a
/// [`Result`] which indicates whether a mutex has been poisoned or not. Most
/// usage of a mutex will simply [`unwrap()`] these results, propagating panics
/// among threads to ensure that a possibly invalid invariant is not witnessed.
///
/// A poisoned mutex, however, does not prevent all access to the underlying
/// data. The [`PoisonError`] type has an [`into_inner`] method which will return
/// the guard that would have otherwise been returned on a successful lock. This
/// allows access to the data, despite the lock being poisoned.
///
/// [`new`]: #method.new
/// [`lock`]: #method.lock
/// [`try_lock`]: #method.try_lock
/// [`Result`]: ../../std/result/enum.Result.html
/// [`unwrap()`]: ../../std/result/enum.Result.html#method.unwrap
/// [`PoisonError`]: ../../std/sync/struct.PoisonError.html
/// [`into_inner`]: ../../std/sync/struct.PoisonError.html#method.into_inner
pub struct SgxMutex<T: ?Sized> {
    inner: Box<SgxThreadMutex>,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

// these are the only places where `T: Send` matters; all other
// functionality works fine on a single thread.
unsafe impl<T: ?Sized + Send> Send for SgxMutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for SgxMutex<T> {}

///
/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// [`Deref`] and [`DerefMut`] implementations.
///
/// This structure is created by the [`lock`] and [`try_lock`] methods on
/// [`Mutex`].
///
/// [`Deref`]: ../../std/ops/trait.Deref.html
/// [`DerefMut`]: ../../std/ops/trait.DerefMut.html
/// [`lock`]: struct.Mutex.html#method.lock
/// [`try_lock`]: struct.Mutex.html#method.try_lock
/// [`Mutex`]: struct.Mutex.html
pub struct SgxMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a SgxMutex<T>,
    poison: poison::Guard,
}

impl<T: ?Sized> !Send for SgxMutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for SgxMutexGuard<'_, T> {}

impl<T> SgxMutex<T> {
    ///
    /// Creates a new mutex in an unlocked state ready for use.
    ///
    pub fn new(t: T) -> SgxMutex<T> {
        SgxMutex{
            inner: Box::new(SgxThreadMutex::new()),
            poison: poison::Flag::new(),
            data: UnsafeCell::new(t),
        }
    }
}

impl<T: ?Sized> SgxMutex<T> {
    ///
    /// The function locks a trusted mutex object within an enclave.
    ///
    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// This function will block the local thread until it is available to acquire
    /// the mutex. Upon returning, the thread is the only thread with the lock
    /// held. An RAII guard is returned to allow scoped unlock of the lock. When
    /// the guard goes out of scope, the mutex will be unlocked.
    ///
    /// The exact behavior on locking a mutex in the thread which already holds
    /// the lock is left unspecified. However, this function will not return on
    /// the second call (it might panic or deadlock, for example).
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error once the mutex is acquired.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by
    /// the current thread.
    pub fn lock(&self) -> LockResult<SgxMutexGuard<'_, T>> {
        unsafe {
            self.inner.lock();
            SgxMutexGuard::new(self)
        }
    }

    ///
    /// The function tries to lock a trusted mutex object within an enclave.
    ///
    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then [`Err`] is returned.
    /// Otherwise, an RAII guard is returned. The lock will be unlocked when the
    /// guard is dropped.
    ///
    /// This function does not block.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return failure if the mutex would otherwise be
    /// acquired.
    ///
    /// [`Err`]: ../../std/result/enum.Result.html#variant.Err
    pub fn try_lock(&self) -> TryLockResult<SgxMutexGuard<'_, T>> {
        unsafe {
            match self.inner.try_lock() {
                Ok(_) => Ok(SgxMutexGuard::new(self)?),
                Err(_) => Err(TryLockError::WouldBlock),
            }
        }
    }

    /// Determines whether the mutex is poisoned.
    ///
    /// If another thread is active, the mutex can still become poisoned at any
    /// time. You should not trust a `false` value for program correctness
    /// without additional synchronization.
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    /// Consumes this mutex, returning the underlying data.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error instead.
    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        unsafe {
            let (inner, poison, data) = {
                let SgxMutex {ref inner, ref poison, ref data } = self;
                (ptr::read(inner), ptr::read(poison), ptr::read(data))
            };
            mem::forget(self);
            inner.destroy();
            drop(inner);

            poison::map_result(poison.borrow(), |_| data.into_inner())
        }
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error instead.
    ///
    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = unsafe { &mut *self.data.get() };
        poison::map_result(self.poison.borrow(), |_| data)
    }
}

unsafe impl<#[may_dangle] T: ?Sized> Drop for SgxMutex<T> {
    fn drop(&mut self) {
        // IMPORTANT: This code must be kept in sync with `SgxMutex::into_inner`.
        let result = unsafe { self.inner.destroy() };
        debug_assert_eq!(result, Ok(()), "Error when destroy an SgxMutex: {}", result.unwrap_err());
    }
}

impl<T> From<T> for SgxMutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    /// This is equivalent to [`Mutex::new`].
    ///
    /// [`Mutex::new`]: ../../std/sync/struct.Mutex.html#method.new
    fn from(t: T) -> Self {
        SgxMutex::new(t)
    }
}

impl<T: ?Sized + Default> Default for SgxMutex<T> {
    /// Creates a `SgxMutex<T>`, with the `Default` value for T.
    fn default() -> SgxMutex<T> {
        SgxMutex::new(Default::default())
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SgxMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_lock() {
            Ok(guard) => f.debug_struct("SgxMutex").field("data", &&*guard).finish(),
            Err(TryLockError::Poisoned(err)) => {
                f.debug_struct("SgxMutex").field("data", &&**err.get_ref()).finish()
            },
            Err(TryLockError::WouldBlock) => {
                struct LockedPlaceholder;
                impl fmt::Debug for LockedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("<locked>")
                    }
                }

                f.debug_struct("SgxMutex").field("data", &LockedPlaceholder).finish()
            }
        }
    }
}

impl<'mutex, T: ?Sized> SgxMutexGuard<'mutex, T> {
    unsafe fn new(lock: &'mutex SgxMutex<T>) -> LockResult<SgxMutexGuard<'mutex, T>> {
        poison::map_result(lock.poison.borrow(), |guard| SgxMutexGuard { lock, poison: guard })
    }
}

impl<T: ?Sized> Deref for SgxMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for SgxMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for SgxMutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        let result = unsafe {
            self.lock.poison.done(&self.poison);
            self.lock.inner.unlock()
        };
        debug_assert_eq!(result, Ok(()), "Error when unlocking an SgxMutex: {}", result.unwrap_err());
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SgxMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for SgxMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

pub fn guard_lock<'a, T: ?Sized>(guard: &SgxMutexGuard<'a, T>) -> &'a SgxThreadMutex {
    &guard.lock.inner
}

pub fn guard_poison<'a, T: ?Sized>(guard: &SgxMutexGuard<'a, T>) -> &'a poison::Flag {
    &guard.lock.poison
}
