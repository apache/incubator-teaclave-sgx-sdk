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
use sgx_trts::oom;
use sgx_trts::libc;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::fmt;
use core::alloc::AllocError;
use alloc_crate::boxed::Box;
use crate::sync::{mutex, SgxThreadMutex, SgxMutexGuard};
use crate::sys_common::poison::{self, LockResult, PoisonError};
use crate::time::Duration;
use crate::time::Instant;
use crate::sys::condvar as imp;
#[cfg(not(feature = "untrusted_time"))]
use crate::untrusted::time::InstantEx;

/// A type indicating whether a timed wait on a condition variable returned
/// due to a time out or not.
///
/// It is returned by the [`wait_timeout`] method.
///
/// [`wait_timeout`]: struct.Condvar.html#method.wait_timeout
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct WaitTimeoutResult(bool);

impl WaitTimeoutResult {
    /// Returns `true` if the wait was known to have timed out.
    ///
    pub fn timed_out(&self) -> bool {
        self.0
    }
}

pub struct SgxThreadCondvar(imp::SgxThreadCondvar);

unsafe impl Send for SgxThreadCondvar {}
unsafe impl Sync for SgxThreadCondvar {}

impl SgxThreadCondvar {
    pub const fn new() -> SgxThreadCondvar {
        SgxThreadCondvar(imp::SgxThreadCondvar::new())
    }

    #[inline]
    pub unsafe fn wait(&self, mutex: &SgxThreadMutex) -> SysError {
        self.0.wait(mutex::raw(mutex))
    }

    #[inline]
    pub unsafe fn wait_timeout(&self, mutex: &SgxThreadMutex, dur: Duration) -> SysError {
        self.0.wait_timeout(mutex::raw(mutex), dur)
    }

    #[inline]
    pub unsafe fn signal(&self) -> SysError {
        self.0.signal()
    }

    #[inline]
    pub unsafe fn broadcast(&self) -> SysError {
        self.0.broadcast()
    }

    #[inline]
    pub unsafe fn notify_one(&self) -> SysError {
        self.signal()
    }

    #[inline]
    pub unsafe fn notify_all(&self) -> SysError {
        self.broadcast()
    }

    #[inline]
    pub unsafe fn destroy(&self) -> SysError {
        self.0.destroy()
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
pub struct SgxCondvar {
    inner: Box<SgxThreadCondvar>,
    mutex: AtomicUsize,
}

impl SgxCondvar {
    ///
    /// Creates a new condition variable which is ready to be waited on and notified.
    ///
    pub fn new() -> SgxCondvar {
        SgxCondvar {
            inner: Box::new(SgxThreadCondvar::new()),
            mutex: AtomicUsize::new(0),
        }
    }

    /// Blocks the current thread until this condition variable receives a
    /// notification.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
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
            self.inner.wait(lock);
            mutex::guard_poison(&guard).get()
        };
        if poisoned { Err(PoisonError::new(guard)) } else { Ok(guard) }
    }

    /// Blocks the current thread until this condition variable receives a
    /// notification and the required condition is met. Spurious wakeups are
    /// ignored and this function will only return once the condition has been
    /// met.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    ///
    /// # Errors
    ///
    /// This function will return an error if the mutex being waited on is
    /// poisoned when this thread re-acquires the lock. For more information,
    /// see information about [poisoning] on the [`Mutex`] type.
    ///
    pub fn wait_until<'a, T, F>(
        &self,
        mut guard: SgxMutexGuard<'a, T>,
        mut condition: F
    ) -> LockResult<SgxMutexGuard<'a, T>>
    where
        F: FnMut(&mut T) -> bool,
    {
        while !condition(&mut *guard) {
            guard = self.wait(guard)?;
        }
        Ok(guard)
    }

    /// Blocks the current thread until this condition variable receives a
    /// notification and the provided condition is false.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    ///
    /// # Errors
    ///
    /// This function will return an error if the mutex being waited on is
    /// poisoned when this thread re-acquires the lock. For more information,
    /// see information about [poisoning] on the [`Mutex`] type.
    ///
    /// [`notify_one`]: #method.notify_one
    /// [`notify_all`]: #method.notify_all
    /// [poisoning]: ../sync/struct.Mutex.html#poisoning
    /// [`Mutex`]: ../sync/struct.Mutex.html
    ///
    pub fn wait_while<'a, T, F>(
        &self,
        mut guard: SgxMutexGuard<'a, T>,
        mut condition: F,
    ) -> LockResult<SgxMutexGuard<'a, T>>
    where
        F: FnMut(&mut T) -> bool,
    {
        while condition(&mut *guard) {
            guard = self.wait(guard)?;
        }
        Ok(guard)
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.
    ///
    /// The semantics of this function are equivalent to [`wait`]
    /// except that the thread will be blocked for roughly no longer
    /// than `ms` milliseconds. This method should not be used for
    /// precise timing due to anomalies such as preemption or platform
    /// differences that may not cause the maximum amount of time
    /// waited to be precisely `ms`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time.
    ///
    /// The returned boolean is `false` only if the timeout is known
    /// to have elapsed.
    ///
    /// Like [`wait`], the lock specified will be re-acquired when this function
    /// returns, regardless of whether the timeout elapsed or not.
    ///
    /// [`wait`]: #method.wait
    ///
    pub fn wait_timeout_ms<'a, T>(
        &self,
        guard: SgxMutexGuard<'a, T>,
        ms: u32,
    ) -> LockResult<(SgxMutexGuard<'a, T>, bool)> {
        let res = self.wait_timeout(guard, Duration::from_millis(ms as u64));
        poison::map_result(res, |(a, b)| (a, !b.timed_out()))
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.
    ///
    /// The semantics of this function are equivalent to [`wait`] except that
    /// the thread will be blocked for roughly no longer than `dur`. This
    /// method should not be used for precise timing due to anomalies such as
    /// preemption or platform differences that may not cause the maximum
    /// amount of time waited to be precisely `dur`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time. This function is susceptible to spurious wakeups.
    /// Condition variables normally have a boolean predicate associated with
    /// them, and the predicate must always be checked each time this function
    /// returns to protect against spurious wakeups. Additionally, it is
    /// typically desirable for the timeout to not exceed some duration in
    /// spite of spurious wakes, thus the sleep-duration is decremented by the
    /// amount slept. Alternatively, use the `wait_timeout_while` method
    /// to wait with a timeout while a predicate is true.
    ///
    /// The returned [`WaitTimeoutResult`] value indicates if the timeout is
    /// known to have elapsed.
    ///
    /// Like [`wait`], the lock specified will be re-acquired when this function
    /// returns, regardless of whether the timeout elapsed or not.
    ///
    /// [`wait`]: #method.wait
    /// [`wait_timeout_while`]: #method.wait_timeout_while
    /// [`WaitTimeoutResult`]: struct.WaitTimeoutResult.html
    ///
    pub fn wait_timeout<'a, T>(
        &self,
        guard: SgxMutexGuard<'a, T>,
        dur: Duration,
    ) -> LockResult<(SgxMutexGuard<'a, T>, WaitTimeoutResult)> {
        let (poisoned, result) = unsafe {
            let lock = mutex::guard_lock(&guard);
            self.verify(lock);
            let result = self.inner.wait_timeout(lock, dur);
            (mutex::guard_poison(&guard).get(), WaitTimeoutResult(result.err() == Some(libc::ETIMEDOUT)))
        };
        if poisoned { Err(PoisonError::new((guard, result))) } else { Ok((guard, result)) }
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.  Spurious wakes will not cause this function to
    /// return.
    ///
    /// The semantics of this function are equivalent to [`wait_until`] except
    /// that the thread will be blocked for roughly no longer than `dur`. This
    /// method should not be used for precise timing due to anomalies such as
    /// preemption or platform differences that may not cause the maximum
    /// amount of time waited to be precisely `dur`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time.
    ///
    /// The returned [`WaitTimeoutResult`] value indicates if the timeout is
    /// known to have elapsed without the condition being met.
    ///
    /// Like [`wait_until`], the lock specified will be re-acquired when this
    /// function returns, regardless of whether the timeout elapsed or not.
    ///
    /// [`wait_until`]: #method.wait_until
    /// [`wait_timeout`]: #method.wait_timeout
    /// [`WaitTimeoutResult`]: struct.WaitTimeoutResult.html
    ///
    pub fn wait_timeout_until<'a, T, F>(
    	&self,
    	mut guard: SgxMutexGuard<'a, T>,
        dur: Duration, mut condition: F,
	) -> LockResult<(SgxMutexGuard<'a, T>, WaitTimeoutResult)>
	where
	    F: FnMut(&mut T) -> bool,
	{
        let start = Instant::now();
        loop {
            if condition(&mut *guard) {
                return Ok((guard, WaitTimeoutResult(false)));
            }
            let timeout = match dur.checked_sub(start.elapsed()) {
                Some(timeout) => timeout,
                None => return Ok((guard, WaitTimeoutResult(true))),
            };
            guard = self.wait_timeout(guard, timeout)?.0;
        }
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.
    ///
    /// The semantics of this function are equivalent to [`wait_while`] except
    /// that the thread will be blocked for roughly no longer than `dur`. This
    /// method should not be used for precise timing due to anomalies such as
    /// preemption or platform differences that may not cause the maximum
    /// amount of time waited to be precisely `dur`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time.
    ///
    /// The returned [`WaitTimeoutResult`] value indicates if the timeout is
    /// known to have elapsed without the condition being met.
    ///
    /// Like [`wait_while`], the lock specified will be re-acquired when this
    /// function returns, regardless of whether the timeout elapsed or not.
    ///
    /// [`wait_while`]: #method.wait_while
    /// [`wait_timeout`]: #method.wait_timeout
    /// [`WaitTimeoutResult`]: struct.WaitTimeoutResult.html
    ///
    pub fn wait_timeout_while<'a, T, F>(
        &self,
        mut guard: SgxMutexGuard<'a, T>,
        dur: Duration,
        mut condition: F,
    ) -> LockResult<(SgxMutexGuard<'a, T>, WaitTimeoutResult)>
    where
        F: FnMut(&mut T) -> bool,
    {
        let start = Instant::now();
        loop {
            if !condition(&mut *guard) {
                return Ok((guard, WaitTimeoutResult(false)));
            }
            let timeout = match dur.checked_sub(start.elapsed()) {
                Some(timeout) => timeout,
                None => return Ok((guard, WaitTimeoutResult(true))),
            };
            guard = self.wait_timeout(guard, timeout)?.0;
        }
    }

    /// Wakes up one blocked thread on this condvar.
    ///
    /// If there is a blocked thread on this condition variable, then it will
    /// be woken up from its call to [`wait`] or [`wait_timeout`]. Calls to
    /// `notify_one` are not buffered in any way.
    ///
    /// To wake up all threads, see [`broadcast`].
    pub fn signal(&self) {
        unsafe { self.inner.signal(); }
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
                Err(r) if r == libc::ENOMEM => {
                    oom::rsgx_oom(AllocError)
                },
                _ => {},
            }
        }
    }

    #[inline]
    pub fn notify_one(&self) {
        self.signal()
    }

    #[inline]
    pub fn notify_all(&self) {
        self.broadcast()
    }

    fn verify(&self, mutex: &SgxThreadMutex) {
        let addr = mutex as *const _ as usize;
        match self.mutex.compare_and_swap(0, addr, Ordering::SeqCst) {
            // If we got out 0, then we have successfully bound the mutex to
            // this cvar.
            0 => {}

            // If we get out a value that's the same as `addr`, then someone
            // already beat us to the punch.
            n if n == addr => {}

            // Anything else and we're using more than one mutex on this cvar,
            // which is currently disallowed.
            _ => panic!(
                "attempted to use a condition variable with two \
                         mutexes"
            ),
        }
    }
}


impl fmt::Debug for SgxCondvar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Condvar { .. }")
    }
}

impl Default for SgxCondvar {
    /// Creates a `Condvar` which is ready to be waited on and notified.
    fn default() -> SgxCondvar {
        SgxCondvar::new()
    }
}

impl Drop for SgxCondvar {
    fn drop(&mut self) {
        let result = unsafe { self.inner.destroy() };
        debug_assert_eq!(result, Ok(()), "Error when destroy an SgxCondvar: {}", result.unwrap_err());
    }
}
