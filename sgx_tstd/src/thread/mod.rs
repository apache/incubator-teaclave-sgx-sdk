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

//! Native threads.

use sgx_types::{sgx_thread_t, sgx_thread_self};
use panicking;
use sys_common::thread_info;
use sync::{SgxMutex, SgxCondvar};
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::SeqCst;
use alloc::sync::Arc;
use time::Duration;

#[macro_use] mod local;
pub use self::local::{LocalKey, LocalKeyInner, AccessError};


///
/// The rsgx_thread_self function returns the unique thread identification.
///
/// # Description
///
/// The function is a simple wrap of get_thread_data() provided in the tRTS,
/// which provides a trusted thread unique identifier.
///
/// # Requirements
///
/// Library: libsgx_tstdc.a
///
/// # Return value
///
/// The return value cannot be NULL and is always valid as long as it is invoked by a thread inside the enclave.
///
pub fn rsgx_thread_self() -> sgx_thread_t {

    unsafe { sgx_thread_self() }
}

///
/// The rsgx_thread_equal function compares two thread identifiers.
///
/// # Description
///
/// The function compares two thread identifiers provided by sgx_thread_
/// self to determine if the IDs refer to the same trusted thread.
///
/// # Requirements
///
/// Library: libsgx_tstdc.a
///
/// # Return value
///
/// **true**
///
/// The two thread IDs are equal.
///
pub fn rsgx_thread_equal(a: sgx_thread_t, b: sgx_thread_t) -> bool {
    a == b
}

/// Gets a handle to the thread that invokes it.
pub fn current() -> SgxThread {
    thread_info::current_thread().expect("use of thread::current() need TCS policy is Bound")
}

/// Determines whether the current thread is unwinding because of panic.
///
/// A common use of this feature is to poison shared resources when writing
/// unsafe code, by checking `panicking` when the `drop` is called.
///
/// This is usually not needed when writing safe code, as [`SgxMutex`es][SgxMutex]
/// already poison themselves when a thread panics while holding the lock.
///
/// This can also be used in multithreaded applications, in order to send a
/// message to other threads warning that a thread has panicked (e.g. for
/// monitoring purposes).
pub fn panicking() -> bool {
    panicking::panicking()
}

// constants for park/unpark
const EMPTY: usize = 0;
const PARKED: usize = 1;
const NOTIFIED: usize = 2;

/// Blocks unless or until the current thread's token is made available.
///
/// A call to `park` does not guarantee that the thread will remain parked
/// forever, and callers should be prepared for this possibility.
///
/// # park and unpark
///
/// Every thread is equipped with some basic low-level blocking support, via the
/// [`thread::park`][`park`] function and [`thread::Thread::unpark`][`unpark`]
/// method. [`park`] blocks the current thread, which can then be resumed from
/// another thread by calling the [`unpark`] method on the blocked thread's
/// handle.
///
/// Conceptually, each [`Thread`] handle has an associated token, which is
/// initially not present:
///
/// * The [`thread::park`][`park`] function blocks the current thread unless or
///   until the token is available for its thread handle, at which point it
///   atomically consumes the token. It may also return *spuriously*, without
///   consuming the token. [`thread::park_timeout`] does the same, but allows
///   specifying a maximum time to block the thread for.
///
/// * The [`unpark`] method on a [`Thread`] atomically makes the token available
///   if it wasn't already.
///
/// In other words, each [`Thread`] acts a bit like a spinlock that can be
/// locked and unlocked using `park` and `unpark`.
///
/// The API is typically used by acquiring a handle to the current thread,
/// placing that handle in a shared data structure so that other threads can
/// find it, and then `park`ing. When some desired condition is met, another
/// thread calls [`unpark`] on the handle.
///
/// The motivation for this design is twofold:
///
/// * It avoids the need to allocate mutexes and condvars when building new
///   synchronization primitives; the threads already provide basic
///   blocking/signaling.
///
/// * It can be implemented very efficiently on many platforms.
///
pub fn park() {
    let thread = current();

    // If we were previously notified then we consume this notification and
    // return quickly.
    if thread.inner.state.compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst).is_ok() {
        return
    }

    // Otherwise we need to coordinate going to sleep
    let mut m = thread.inner.lock.lock().unwrap();
    match thread.inner.state.compare_exchange(EMPTY, PARKED, SeqCst, SeqCst) {
        Ok(_) => {}
        Err(NOTIFIED) => {
            // We must read here, even though we know it will be `NOTIFIED`.
            // This is because `unpark` may have been called again since we read
            // `NOTIFIED` in the `compare_exchange` above. We must perform an
            // acquire operation that synchronizes with that `unpark` to observe
            // any writes it made before the call to unpark. To do that we must
            // read from the write it made to `state`.
            let old = thread.inner.state.swap(EMPTY, SeqCst);
            assert_eq!(old, NOTIFIED, "park state changed unexpectedly");
            return;
        } // should consume this notification, so prohibit spurious wakeups in next park.
        Err(_) => panic!("inconsistent park state"),
    }
    loop {
        m = thread.inner.cvar.wait(m).unwrap();
        match thread.inner.state.compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst) {
            Ok(_) => return, // got a notification
            Err(_) => {} // spurious wakeup, go back to sleep
        }
    }
}

/// Use [`park_timeout`].
///
/// Blocks unless or until the current thread's token is made available or
/// the specified duration has been reached (may wake spuriously).
///
/// The semantics of this function are equivalent to [`park`] except
/// that the thread will be blocked for roughly no longer than `dur`. This
/// method should not be used for precise timing due to anomalies such as
/// preemption or platform differences that may not cause the maximum
/// amount of time waited to be precisely `ms` long.
///
/// See the [park documentation][`park`] for more detail.
///
/// [`park_timeout`]: fn.park_timeout.html
/// [`park`]: ../../std/thread/fn.park.html
pub fn park_timeout_ms(ms: u32) {
    park_timeout(Duration::from_millis(ms as u64))
}

/// Blocks unless or until the current thread's token is made available or
/// the specified duration has been reached (may wake spuriously).
///
/// The semantics of this function are equivalent to [`park`][park] except
/// that the thread will be blocked for roughly no longer than `dur`. This
/// method should not be used for precise timing due to anomalies such as
/// preemption or platform differences that may not cause the maximum
/// amount of time waited to be precisely `dur` long.
///
/// See the [park documentation][park] for more details.
///
/// # Platform-specific behavior
///
/// Platforms which do not support nanosecond precision for sleeping will have
/// `dur` rounded up to the nearest granularity of time they can sleep for.
///
/// # Examples
///
/// Waiting for the complete expiration of the timeout:
///
/// ```rust,no_run
/// use std::thread::park_timeout;
/// use std::time::{Instant, Duration};
///
/// let timeout = Duration::from_secs(2);
/// let beginning_park = Instant::now();
///
/// let mut timeout_remaining = timeout;
/// loop {
///     park_timeout(timeout_remaining);
///     let elapsed = beginning_park.elapsed();
///     if elapsed >= timeout {
///         break;
///     }
///     println!("restarting park_timeout after {:?}", elapsed);
///     timeout_remaining = timeout - elapsed;
/// }
/// ```
///
/// [park]: fn.park.html
pub fn park_timeout(dur: Duration) {
    let thread = current();

    // Like `park` above we have a fast path for an already-notified thread, and
    // afterwards we start coordinating for a sleep.
    // return quickly.
    if thread.inner.state.compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst).is_ok() {
        return
    }
    let m = thread.inner.lock.lock().unwrap();
    match thread.inner.state.compare_exchange(EMPTY, PARKED, SeqCst, SeqCst) {
        Ok(_) => {}
        Err(NOTIFIED) => {
            // We must read again here, see `park`.
            let old = thread.inner.state.swap(EMPTY, SeqCst);
            assert_eq!(old, NOTIFIED, "park state changed unexpectedly");
            return;
        } // should consume this notification, so prohibit spurious wakeups in next park.
        Err(_) => panic!("inconsistent park_timeout state"),
    }

    // Wait with a timeout, and if we spuriously wake up or otherwise wake up
    // from a notification we just want to unconditionally set the state back to
    // empty, either consuming a notification or un-flagging ourselves as
    // parked.
    let (_m, _result) = thread.inner.cvar.wait_timeout(m, dur).unwrap();
    match thread.inner.state.swap(EMPTY, SeqCst) {
        NOTIFIED => {} // got a notification, hurray!
        PARKED => {} // no notification, alas
        n => panic!("inconsistent park_timeout state: {}", n),
    }
}

/// A unique identifier for a running thread.
///
/// A `ThreadId` is an opaque object that has a unique value for each thread
/// that creates one. `ThreadId`s are not guaranteed to correspond to a thread's
/// system-designated identifier.
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct SgxThreadId {
    id: sgx_thread_t,
}

impl SgxThreadId {
    /// Generate a new unique thread ID.
    pub fn new() -> Self {
        SgxThreadId {
            id: rsgx_thread_self(),
        }
    }
}

/// The internal representation of a `Thread` handle
struct Inner {
    id: SgxThreadId,

    // state for thread park/unpark
    state: AtomicUsize,
    lock: SgxMutex<()>,
    cvar: SgxCondvar,
}

/// A handle to a thread.
///
#[derive(Clone)]
pub struct SgxThread {
    inner: Arc<Inner>,
}

impl SgxThread {

    /// Used only internally to construct a thread object without spawning
    pub(crate) fn new() -> Self {
        SgxThread {
            inner: Arc::new(Inner {
                id: SgxThreadId::new(),
                state: AtomicUsize::new(EMPTY),
                lock: SgxMutex::new(()),
                cvar: SgxCondvar::new(),
            })
        }
    }

    /// Gets the thread's unique identifier.
    pub fn id(&self) -> SgxThreadId {
        self.inner.id
    }

    /// Atomically makes the handle's token available if it is not already.
    ///
    /// Every thread is equipped with some basic low-level blocking support, via
    /// the [`park`][park] function and the `unpark()` method. These can be
    /// used as a more CPU-efficient implementation of a spinlock.
    ///
    pub fn unpark(&self) {
        loop {
            match self.inner.state.compare_exchange(EMPTY, NOTIFIED, SeqCst, SeqCst) {
                Ok(_) => return, // no one was waiting
                Err(NOTIFIED) => return, // already unparked
                Err(PARKED) => {} // gotta go wake someone up
                _ => panic!("inconsistent state in unpark"),
            }

            // Coordinate wakeup through the mutex and a condvar notification
            let _lock = self.inner.lock.lock().unwrap();
            match self.inner.state.compare_exchange(PARKED, NOTIFIED, SeqCst, SeqCst) {
                Ok(_) => return self.inner.cvar.signal(),
                Err(NOTIFIED) => return, // a different thread unparked
                Err(EMPTY) => {} // parked thread went away, try again
                _ => panic!("inconsistent state in unpark"),
            }
        }
    }
}
