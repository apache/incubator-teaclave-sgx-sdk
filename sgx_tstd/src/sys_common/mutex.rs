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

use crate::sys::mutex as imp;

use sgx_types::SysError;

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
        SgxThreadMutex(imp::SgxThreadMutex::new())
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

    pub(super) fn raw(&self) -> &imp::SgxThreadMutex {
        &self.0
    }
}

pub struct SgxMovableThreadMutex(imp::SgxMovableThreadMutex);

unsafe impl Sync for SgxMovableThreadMutex {}

impl SgxMovableThreadMutex {
    /// Creates a new mutex.
    pub fn new() -> SgxMovableThreadMutex {
        let mutex = imp::SgxMovableThreadMutex::from(imp::SgxThreadMutex::new());
        SgxMovableThreadMutex(mutex)
    }

    pub(super) fn raw(&self) -> &imp::SgxThreadMutex {
        &self.0
    }

    /// Locks the mutex blocking the current thread until it is available.
    #[inline]
    pub fn raw_lock(&self) -> SysError {
        unsafe { self.0.lock() }
    }

    /// Attempts to lock the mutex without blocking, returning whether it was
    /// successfully acquired or not.
    #[inline]
    pub fn try_lock(&self) -> SysError {
        unsafe { self.0.try_lock() }
    }

    /// Unlocks the mutex.
    ///
    /// Behavior is undefined if the current thread does not actually hold the
    /// mutex.
    #[inline]
    pub unsafe fn raw_unlock(&self) -> SysError {
        self.0.unlock()
    }
}

impl Drop for SgxMovableThreadMutex {
    fn drop(&mut self) {
        let r = unsafe { self.0.destroy() };
        debug_assert_eq!(r, Ok(()));
    }
}
