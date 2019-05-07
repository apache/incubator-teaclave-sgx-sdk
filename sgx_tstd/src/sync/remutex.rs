// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

use sgx_types::{self, SysError, sgx_thread_mutex_t};
use sync::mutex::*;
use panic::{UnwindSafe, RefUnwindSafe};
use sys_common::poison::{self, TryLockError, TryLockResult, LockResult};
use core::cell::UnsafeCell;
use core::fmt;
use core::ops::Deref;
use core::marker;
use alloc::boxed::Box;

/// The structure of sgx mutex.
pub struct SgxReentrantThreadMutex {
    lock: UnsafeCell<SgxThreadMutexInner>,
}

unsafe impl Send for SgxReentrantThreadMutex {}
unsafe impl Sync for SgxReentrantThreadMutex {}

impl SgxReentrantThreadMutex {

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
    pub const fn new() -> Self {
        SgxReentrantThreadMutex{
            lock: UnsafeCell::new(super::mutex::SgxThreadMutexInner::new(super::mutex::SgxThreadMutexControl::SGX_THREAD_MUTEX_RECURSIVE))
        }
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
        let remutex: &mut super::mutex::SgxThreadMutexInner = &mut *self.lock.get();
        remutex.lock()
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
        let remutex: &mut super::mutex::SgxThreadMutexInner = &mut *self.lock.get();
        remutex.try_lock()
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
        let remutex: &mut super::mutex::SgxThreadMutexInner = &mut *self.lock.get();
        remutex.unlock()
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
        let remutex: &mut super::mutex::SgxThreadMutexInner = &mut *self.lock.get();
        remutex.destroy()
    }
}

/// A re-entrant mutual exclusion
///
/// This mutex will block *other* threads waiting for the lock to become
/// available. The thread which has already locked the mutex can lock it
/// multiple times without blocking, preventing a common source of deadlocks.
pub struct SgxReentrantMutex<T> {
    inner: Box<SgxReentrantThreadMutex>,
    poison: poison::Flag,
    data: T,
}

unsafe impl<T: Send> Send for SgxReentrantMutex<T> {}
unsafe impl<T: Send> Sync for SgxReentrantMutex<T> {}

impl<T> UnwindSafe for SgxReentrantMutex<T> {}
impl<T> RefUnwindSafe for SgxReentrantMutex<T> {}


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
    __lock: &'a SgxReentrantMutex<T>,
    __poison: poison::Guard,
}

impl<'a, T> !marker::Send for SgxReentrantMutexGuard<'a, T> {}


impl<T> SgxReentrantMutex<T> {
    /// Creates a new reentrant mutex in an unlocked state.
    pub fn new(t: T) -> SgxReentrantMutex<T> {

        SgxReentrantMutex{
            inner: Box::new(SgxReentrantThreadMutex::new()),
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
    pub fn lock(&self) -> LockResult<SgxReentrantMutexGuard<T>> {
        unsafe { self.inner.lock(); }
        SgxReentrantMutexGuard::new(&self)
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
    pub fn try_lock(&self) -> TryLockResult<SgxReentrantMutexGuard<T>> {

        match unsafe { self.inner.try_lock() } {
            Ok(_) => Ok(SgxReentrantMutexGuard::new(&self)?),
            Err(_) => Err(TryLockError::WouldBlock),
        }
    }
}

impl<T> Drop for SgxReentrantMutex<T> {
    fn drop(&mut self) {
        // This is actually safe b/c we know that there is no further usage of
        // this mutex (it's up to the user to arrange for a mutex to get
        // dropped, that's not our job)
        unsafe { self.inner.destroy(); }
    }
}

impl<T: fmt::Debug + 'static> fmt::Debug for SgxReentrantMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Ok(guard) => f.debug_struct("ReentrantMutex").field("data", &*guard).finish(),
            Err(TryLockError::Poisoned(err)) => {
                f.debug_struct("ReentrantMutex").field("data", &**err.get_ref()).finish()
            },
            Err(TryLockError::WouldBlock) => {
                struct LockedPlaceholder;
                impl fmt::Debug for LockedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("<locked>") }
                }

                f.debug_struct("ReentrantMutex").field("data", &LockedPlaceholder).finish()
            }
        }
    }
}

impl<'mutex, T> SgxReentrantMutexGuard<'mutex, T> {
    fn new(lock: &'mutex SgxReentrantMutex<T>)
            -> LockResult<SgxReentrantMutexGuard<'mutex, T>> {
        poison::map_result(lock.poison.borrow(), |guard| {
            SgxReentrantMutexGuard {
                __lock: lock,
                __poison: guard,
            }
        })
    }
}

impl<'mutex, T> Deref for SgxReentrantMutexGuard<'mutex, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.__lock.data
    }
}

impl<'a, T> Drop for SgxReentrantMutexGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.__lock.poison.done(&self.__poison);
            self.__lock.inner.unlock();
        }
    }
}