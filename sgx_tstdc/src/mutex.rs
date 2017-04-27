// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

//!
//! The Intel(R) Software Guard Extensions SDK already supports mutex and conditional 
//! variable synchronization mechanisms by means of the following APIand data types 
//! defined in the Types and Enumerations section. Some functions included in the 
//! trusted Thread Synchronization library may make calls outside the enclave (OCALLs).
//! If you use any of the APIs below, you must first import the needed OCALL functions
//! from sgx_tstdc.edl. Otherwise, you will get a linker error when the enclave is
//! being built; see Calling Functions outside the Enclave for additional details. 
//! The table below illustrates the primitives that the Intel(R) SGX Thread 
//! Synchronization library supports, as well as the OCALLs that each API function needs.
//!
use sgx_types::*;
use core::cell::UnsafeCell;
use core::mem;
use core::ptr;
use core::ops::{Deref, DerefMut};
#[cfg(not(feature = "use_std"))]
use core::marker;
#[cfg(not(feature = "use_std"))]
use alloc::boxed::Box;


pub unsafe fn raw_mutex(lock: &sgx_thread_mutex_t) -> * mut sgx_thread_mutex_t {
    lock as *const _ as *mut _
}

#[allow(dead_code)]
fn rsgx_thread_mutex_init(mutex: &sgx_thread_mutex_t, unused: &sgx_thread_mutexattr_t) -> sys_error_t {

    unsafe { sgx_thread_mutex_init(raw_mutex(mutex), unused as * const sgx_thread_mutexattr_t) }
}

fn rsgx_thread_mutex_destroy(mutex: &sgx_thread_mutex_t) -> sys_error_t {
    
    unsafe { sgx_thread_mutex_destroy(raw_mutex(mutex)) }
}

fn rsgx_thread_mutex_lock(mutex: &sgx_thread_mutex_t) -> sys_error_t {
    
    unsafe { sgx_thread_mutex_lock(raw_mutex(mutex)) }
}

fn rsgx_thread_mutex_trylock(mutex: &sgx_thread_mutex_t) -> sys_error_t {
    
    unsafe { sgx_thread_mutex_trylock(raw_mutex(mutex)) }
}

fn rsgx_thread_mutex_unlock(mutex: &sgx_thread_mutex_t) -> sys_error_t {
    
    unsafe { sgx_thread_mutex_unlock(raw_mutex(mutex)) }
}

/// The structure of sgx mutex.
pub struct SgxThreadMutex {
    lock: sgx_thread_mutex_t,
}

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
    pub fn new() -> Self {
        SgxThreadMutex{lock: SGX_THREAD_NONRECURSIVE_MUTEX_INITIALIZER}
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
    pub fn lock(&self) -> SysError {

        let ret = rsgx_thread_mutex_lock(&self.lock);
        if ret == 0 { Ok(()) } else { Err(ret) }
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
    pub fn trylock(&self) -> SysError {

        let ret = rsgx_thread_mutex_trylock(&self.lock);
        if ret == 0 { Ok(()) } else { Err(ret) }
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
    pub fn unlock(&self) -> SysError {

        let ret = rsgx_thread_mutex_unlock(&self.lock);
        if ret == 0 { Ok(()) } else { Err(ret) }
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
    pub fn destory(&self) -> SysError {

        let ret = rsgx_thread_mutex_destroy(&self.lock);
        if ret == 0 { Ok(()) } else { Err(ret) }
    }

    /// Get the pointer of sgx_thread_mutex_t in SgxThreadMutex.
    pub fn get_raw(&self) -> &sgx_thread_mutex_t {
        &self.lock
    }
}

/// The structure wrapped of SgxThreadMutex.
pub struct SgxMutex<T: ?Sized> {
    inner: Box<SgxThreadMutex>,
    data: UnsafeCell<T>
}

unsafe impl<T: ?Sized + Send> Send for SgxMutex<T> { }
unsafe impl<T: ?Sized + Send> Sync for SgxMutex<T> { }

impl<T> SgxMutex<T> {
    ///
    /// Creates a new mutex in an unlocked state ready for use.
    ///
    pub fn new(t: T) -> SgxMutex<T> {
        SgxMutex{
            inner: Box::new(SgxThreadMutex::new()),
            data: UnsafeCell::new(t),
        }
    }
}

impl<T: ?Sized> SgxMutex<T> {

    ///
    /// The function locks a trusted mutex object within an enclave.
    ///
    /// An RAII guard is returned to allow scoped unlock of the lock. When
    /// the guard goes out of scope, the mutex will be unlocked.
    ///
    pub fn lock(&self) -> SysResult<SgxMutexGuard<T>> {
        self.inner.lock().map(|_| SgxMutexGuard::new(self))
    }

    ///
    /// The function tries to lock a trusted mutex object within an enclave.
    ///
    /// If the lock could not be acquired at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned. The lock will be unlocked when the
    /// guard is dropped.
    ///
    /// This function does not block.
    ///
    pub fn try_lock(&self) -> SysResult<SgxMutexGuard<T>> {
        self.inner.trylock().map(|_| SgxMutexGuard::new(self))
    }

    /// Consumes this mutex, returning the underlying data.
    pub fn into_inner(self) -> SysResult<T> where T: Sized {

        unsafe {
            let (inner, data) = {
                let SgxMutex {ref inner, ref data } = self;
                (ptr::read(inner), ptr::read(data))
            };
            mem::forget(self);
            let result = inner.destory();
            drop(inner);
            result.map(|_| data.into_inner())
        }
    }

    /// Returns a mutable reference to the underlying data.
    pub fn get_mut(&mut self) -> SysResult<&mut T> {
      
        let data = unsafe { &mut *self.data.get() };
        Ok(data)
    }
}

impl<T: ?Sized> Drop for SgxMutex<T> {
    fn drop(&mut self) {
       let _ = self.inner.destory();
    }
}

impl<T: ?Sized + Default> Default for SgxMutex<T> {
    fn default() -> SgxMutex<T> {
        SgxMutex::new(Default::default())
    }
}

///
/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// Deref and DerefMut implementations.
///
/// This structure is created by the lock and try_lock methods on Mutex.
///
pub struct SgxMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a SgxMutex<T>,
}

#[cfg(not(feature = "use_std"))]
impl<'a, T: ?Sized> !marker::Send for SgxMutexGuard<'a, T> {}

impl<'mutex, T: ?Sized> SgxMutexGuard<'mutex, T> {

    fn new(lock: &'mutex SgxMutex<T>) -> SgxMutexGuard<'mutex, T> {
        SgxMutexGuard {
            lock: lock,
        }
    }
}

impl<'mutex, T: ?Sized> Deref for SgxMutexGuard<'mutex, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'mutex, T: ?Sized> DerefMut for SgxMutexGuard<'mutex, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for SgxMutexGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
       let _ = self.lock.inner.unlock();
    }
}

pub fn guard_lock<'a, T: ?Sized>(guard: &SgxMutexGuard<'a, T>) -> &'a SgxThreadMutex {
    &guard.lock.inner
}
