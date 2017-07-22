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
use sgx_trts::oom;
use super::mutex::{self, SgxThreadMutex, SgxMutexGuard};
use super::poison::{LockResult, PoisonError};
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(not(feature = "use_std"))]
use alloc::boxed::Box;

pub unsafe fn raw_cond(lock: &sgx_thread_cond_t) -> * mut sgx_thread_cond_t {
    lock as * const _ as * mut _
}

#[allow(dead_code)]
unsafe fn rsgx_thread_cond_init(cond: &sgx_thread_cond_t, unused: &sgx_thread_condattr_t ) -> SysError {

    let ret = sgx_thread_cond_init(raw_cond(cond), unused as * const sgx_thread_condattr_t);
    if ret == 0 { Ok(()) } else { Err(ret) }
}

unsafe fn rsgx_thread_cond_destroy(cond: &sgx_thread_cond_t) -> SysError {

    let ret = sgx_thread_cond_destroy(raw_cond(cond));
    if ret == 0 { Ok(()) } else { Err(ret) }
}

unsafe fn rsgx_thread_cond_wait(cond: &sgx_thread_cond_t, mutex: &sgx_thread_mutex_t) -> SysError {

    let ret = sgx_thread_cond_wait(raw_cond(cond), mutex::raw_mutex(mutex));
    if ret == 0 { Ok(()) } else { Err(ret) }
}

unsafe fn rsgx_thread_cond_signal(cond: &sgx_thread_cond_t) -> SysError {

    let ret = sgx_thread_cond_signal(raw_cond(cond));
    if ret == 0 { Ok(()) } else { Err(ret) }
}

unsafe fn rsgx_thread_cond_broadcast(cond: &sgx_thread_cond_t) -> SysError {

    let ret = sgx_thread_cond_broadcast(raw_cond(cond));
    if ret == 0 { Ok(()) } else { Err(ret) }
}

/// The structure of sgx condition.
pub struct SgxThreadCond {
    cond: sgx_thread_cond_t,
}

unsafe impl Send for SgxThreadCond {}
unsafe impl Sync for SgxThreadCond {}

impl SgxThreadCond {
    ///
    /// The function initializes a trusted condition variable within the enclave.
    ///
    /// # Description
    ///
    /// When a thread creates a condition variable within an enclave, it simply initializes the various
    /// fields of the object to indicate that the condition variable is available. The results of using
    /// a condition variable in a wait, signal or broadcast operation before it has been fully initialized
    /// are undefined. To avoid race conditions in the initialization of a condition variable, it is
    /// recommended statically initializing the condition variable with the macro SGX_THREAD_COND_INITIALIZER.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tstdc.a
    ///
    pub const fn new() -> Self {
        SgxThreadCond{cond: SGX_THREAD_COND_INITIALIZER}
    }

    ///
    /// The function waits on a condition variable within an enclave.
    ///
    /// # Description
    ///
    /// A condition variable is always used in conjunction with a mutex. To wait on a
    /// condition variable, a thread first needs to acquire the condition variable spin
    /// lock. After the spin lock is acquired, the thread updates the condition variable
    /// waiting queue. To avoid the lost wake-up signal problem, the condition variable
    /// spin lock is released after the mutex. This order ensures the function atomically
    /// releases the mutex and causes the calling thread to block on the condition variable,
    /// with respect to other threads accessing the mutex and the condition variable.
    /// After releasing the condition variable spin lock, the thread makes an OCALL to
    /// get suspended. When the thread is awakened, it acquires the condition variable
    /// spin lock. The thread then searches the condition variable queue. If the thread
    /// is in the queue, it means that the thread was already waiting on the condition
    /// variable outside the enclave, and it has been awakened unexpectedly. When this
    /// happens, the thread releases the condition variable spin lock, makes an OCALL
    /// and simply goes back to sleep. Otherwise, another thread has signaled or broadcasted
    /// the condition variable and this thread may proceed. Before returning, the thread
    /// releases the condition variable spin lock and acquires the mutex, ensuring that
    /// upon returning from the function call the thread still owns the mutex.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tstdc.a
    ///
    /// # Parameters
    ///
    /// **mutex**
    ///
    /// The trusted mutex object that will be unlocked when the thread is blocked inthe condition variable
    ///
    /// # Errors
    ///
    /// **EINVAL**
    ///
    /// The trusted condition variable or mutex object is invalid or the mutex is not locked.
    ///
    /// **EPERM**
    ///
    /// The trusted mutex is locked by another thread.
    ///
    #[inline]
    pub unsafe fn wait(&self, mutex: &SgxThreadMutex) -> SysError {
        rsgx_thread_cond_wait(&self.cond, mutex.get_raw())
    }

    ///
    /// The function wakes a pending thread waiting on the condition variable.
    ///
    /// # Description
    ///
    /// To signal a condition variable, a thread starts acquiring the condition variable
    /// spin-lock. Then it inspects the status of the condition variable queue. If the
    /// queue is empty it means that there are not any threads waiting on the condition
    /// variable. When that happens, the thread releases the condition variable and returns.
    /// However, if the queue is not empty, the thread removes the first thread waiting
    /// in the queue. The thread then makes an OCALL to wake up the thread that is suspended
    /// outside the enclave, but first the thread releases the condition variable spin-lock.
    /// Upon returning from the OCALL, the thread continues normal execution.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tstdc.a
    ///
    /// # Errors
    ///
    /// **EINVAL**
    ///
    /// The trusted condition variable is invalid.
    ///
    #[inline]
    pub unsafe fn signal(&self) -> SysError {
        rsgx_thread_cond_signal(&self.cond)
    }

    ///
    /// The function wakes all pending threads waiting on the condition variable.
    ///
    /// # Description
    ///
    /// Broadcast and signal operations on a condition variable are analogous. The
    /// only difference is that during a broadcast operation, the thread removes all
    /// the threads waiting on the condition variable queue and wakes up all the
    /// threads suspended outside the enclave in a single OCALL.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tstdc.a
    ///
    /// # Errors
    ///
    /// **EINVAL**
    ///
    /// The trusted condition variable is invalid.
    ///
    /// **ENOMEM**
    ///
    /// Internal memory allocation failed.
    ///
    #[inline]
    pub unsafe fn broadcast(&self) -> SysError {
        rsgx_thread_cond_broadcast(&self.cond)
    }

    ///
    /// The function destroys a trusted condition variable within an enclave.
    ///
    /// # Description
    ///
    /// The procedure first confirms that there are no threads waiting on the condition
    /// variable before it is destroyed. The destroy operation acquires the spin lock at
    /// the beginning of the operation to prevent other threads from signaling to or
    /// waiting on the condition variable.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tstdc.a
    ///
    /// # Errors
    ///
    /// **EINVAL**
    ///
    /// The trusted condition variable is invalid.
    ///
    /// **EBUSY**
    ///
    /// The condition variable has pending threads waiting on it.
    ///
    #[inline]
    pub unsafe fn destroy(&self) -> SysError {
        rsgx_thread_cond_destroy(&self.cond)
    }

    /// Get the pointer of sgx_thread_cond_t in SgxThreadCond.
    #[allow(dead_code)]
    #[inline]
    pub unsafe fn get_raw(&self) -> &sgx_thread_cond_t {
        &self.cond
    }
}

/// A Condition Variable
///
/// Condition variables represent the ability to block a thread such that it
/// consumes no CPU time while waiting for an event to occur. Condition
/// variables are typically associated with a boolean predicate (a condition)
/// and a mutex. The predicate is always verified inside of the mutex before
/// determining that a thread must block.
///
/// Functions in this module will block the current **thread** of execution and
/// are bindings to system-provided condition variables where possible. Note
/// that this module places one additional restriction over the system condition
/// variables: each condvar can be used with precisely one mutex at runtime. Any
/// attempt to use multiple mutexes on the same condition variable will result
/// in a runtime panic. If this is not desired, then the unsafe primitives in
/// `sys` do not have this restriction but may result in undefined behavior.
///
pub struct SgxCond {
    inner: Box<SgxThreadCond>,
    mutex: AtomicUsize,
}

impl SgxCond {
    ///
    /// Creates a new condition variable which is ready to be waited on and notified.
    ///
    pub fn new() -> Self {
        SgxCond {
            inner: Box::new(SgxThreadCond::new()),
            mutex: AtomicUsize::new(0),
        }
    }

    /// Blocks the current thread until this condition variable receives a
    /// notification.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`signal`] or [`broadcast`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    ///
    /// Note that this function is susceptible to spurious wakeups. Condition
    /// variables normally have a boolean predicate associated with them, and
    /// the predicate must always be checked each time this function returns to
    /// protect against spurious wakeups.
    ///
    /// # Errors
    ///
    /// This function will return an error if the mutex being waited on is
    /// poisoned when this thread re-acquires the lock. For more information,
    /// see information about [poisoning] on the [`SgxMutex`] type.
    ///
    /// # Panics
    ///
    /// This function will [`panic!`] if it is used with more than one mutex
    /// over time. Each condition variable is dynamically bound to exactly one
    /// mutex to ensure defined behavior across platforms. If this functionality
    /// is not desired, then unsafe primitives in `sys` are provided.
    pub fn wait<'a, T>(&self, guard: SgxMutexGuard<'a, T>) -> LockResult<SgxMutexGuard<'a, T>> {

        let poisoned = unsafe {
            let lock = mutex::guard_lock(&guard);
            self.verify(lock);
            let _ = self.inner.wait(lock);
            mutex::guard_poison(&guard).get()
        };
        if poisoned {
            Err(PoisonError::new(guard))
        } else {
            Ok(guard)
        }
    }

    /// Wakes up one blocked thread on this condvar.
    ///
    /// If there is a blocked thread on this condition variable, then it will
    /// be woken up from its call to [`wait`]. Calls to `signal` are not buffered
    /// in any way.
    ///
    /// To wake up all threads, see [`broadcast`].
    pub fn signal(&self) {
        unsafe { let _ = self.inner.signal(); }
    }

    /// Wakes up all blocked threads on this condvar.
    ///
    /// This method will ensure that any current waiters on the condition
    /// variable are awoken. Calls to `broadcast()` are not buffered in any
    /// way.
    ///
    /// To wake up only one thread, see [`signal`].
    pub fn broadcast(&self) {

        unsafe {
            let ret = self.inner.broadcast();
            match ret {
                Err(r) if r == ENOMEM => oom::rsgx_oom(),
                _ => {},
            }
        }
    }

    fn verify(&self, mutex: &SgxThreadMutex) {

        let addr = mutex as *const _ as usize;
        match self.mutex.compare_and_swap(0, addr, Ordering::SeqCst) {
            0 => {},
            n if n == addr => {},
            _ => panic!("attempted to use a condition variable with two mutexes."),
        }
    }
}

impl Drop for SgxCond {
    fn drop(&mut self) {
        unsafe { let _ = self.inner.destroy(); }
    }
}

impl Default for SgxCond {
    fn default() -> Self {
        SgxCond::new()
    }
}
