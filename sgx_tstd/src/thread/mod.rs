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

use sgx_types::{sgx_thread_t, sgx_thread_self, sgx_status_t};
use sgx_trts::enclave::*;
use core::any::Any;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::SeqCst;
use core::cell::UnsafeCell;
use core::mem;
use core::fmt;
use core::num::NonZeroU64;
use alloc_crate::sync::Arc;
use alloc_crate::str;
use crate::panic;
use crate::panicking;
use crate::sys_common::thread_info;
use crate::sync::{SgxMutex, SgxCondvar, Once, ONCE_INIT};
use crate::time::Duration;
use crate::sys::thread as imp;
use crate::io::{self, Error, ErrorKind};
use crate::ffi::{CStr, CString};
use crate::sys_common::{AsInner, IntoInner};

#[macro_use] mod local;
pub use self::local::{LocalKey, LocalKeyInner, AccessError};
#[derive(Debug)]
pub struct Builder {
    // A name for the thread-to-be, for identification in panic messages
    name: Option<String>,
}

impl Builder {
    pub fn new() -> Builder {
        if rsgx_get_thread_policy() != SgxThreadPolicy::Bound {
            panic!("The sgx thread policy must be Bound!");
        }
        Builder {
            name: None,
        }
    }

    pub fn name(mut self, name: String) -> Builder {
        self.name = Some(name);
        self
    }

    pub fn spawn<F, T>(self, f: F) -> io::Result<JoinHandle<T>> where
        F: FnOnce() -> T, F: Send + 'static, T: Send + 'static
    {
        unsafe { self.spawn_unchecked(f) }
    }

    pub unsafe fn spawn_unchecked<'a, F, T>(self, f: F) -> io::Result<JoinHandle<T>> where
        F: FnOnce() -> T, F: Send + 'a, T: Send + 'a
    {
        let Builder { name } = self;

        let my_thread = SgxThread::new(name);
        let their_thread = my_thread.clone();

        let my_packet : Arc<UnsafeCell<Option<Result<T>>>>
            = Arc::new(UnsafeCell::new(Some(Err(Box::new(Error::from(ErrorKind::SgxError))))));
        let their_packet = my_packet.clone();

        let main = move || {
            if let Some(name) = their_thread.cname() {
                imp::Thread::set_name(name);
            }
            thread_info::set(their_thread);
            #[cfg(feature = "backtrace")]
            let try_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                crate::sys_common::backtrace::__rust_begin_short_backtrace(f)
            }));
            #[cfg(not(feature = "backtrace"))]
            let try_result = panic::catch_unwind(panic::AssertUnwindSafe(f));
            *their_packet.get() = Some(try_result);
        };

        Ok(JoinHandle(JoinInner {
            // `imp::Thread::new` takes a closure with a `'static` lifetime, since it's passed
            // through FFI or otherwise used with low-level threading primitives that have no
            // notion of or way to enforce lifetimes.
            //
            // As mentioned in the `Safety` section of this function's documentation, the caller of
            // this function needs to guarantee that the passed-in lifetime is sufficiently long
            // for the lifetime of the thread.
            //
            // Similarly, the `sys` implementation must guarantee that no references to the closure
            // exist after the thread has terminated, which is signaled by `Thread::join`
            // returning.
            native: Some(imp::Thread::new(
                mem::transmute::<Box<dyn FnOnce() + 'a>, Box<dyn FnOnce() + 'static>>(Box::new(
                    main,
                )),
            )?),
            thread: my_thread,
            packet: Packet(my_packet),
        }))
    }
}

pub fn spawn<F, T>(f: F) -> JoinHandle<T> where
    F: FnOnce() -> T, F: Send + 'static, T: Send + 'static
{
    Builder::new().spawn(f).expect("failed to spawn thread")
}

pub fn yield_now() {
    imp::Thread::yield_now()
}

pub fn sleep_ms(ms: u32) {
    sleep(Duration::from_millis(ms as u64))
}

pub fn sleep(dur: Duration) {
    imp::Thread::sleep(dur)
}
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

pub fn current_td() -> SgxThreadData {
    unsafe {
        SgxThreadData::from_raw(rsgx_thread_self())
    }
}

/// Gets a handle to the thread that invokes it.
pub fn current() -> SgxThread {
    thread_info::current_thread().expect("use of thread::current() need TCS policy is Bound")
}

/// Determines whether the current thread is unwinding // state for thread park/unparkbecause of panic.
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
///   if it wasn't already. Because the token is initially absent, [`unpark`]
///   followed by [`park`] will result in the second call returning immediately.
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
/// The semantics of this function are equivalent to [`psgx_tstd::thread::JoinHandleark`][park] except
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

////////////////////////////////////////////////////////////////////////////////
// ThreadId
////////////////////////////////////////////////////////////////////////////////

/// A unique identifier for a running thread.
///
/// A `ThreadId` is an opaque object that has a unique value for each thread
/// that creates one. `ThreadId`s are not guaranteed to correspond to a thread's
/// system-designated identifier. A `ThreadId` can be retrieved from the [`id`]
/// method on a [`Thread`].
///
/// # Examples
///
/// ```
/// use std::thread;
///
/// let other_thread = thread::spawn(|| {
///     thread::current().id()
/// });
///
/// let other_thread_id = other_thread.join().unwrap();
/// assert!(thread::current().id() != other_thread_id);
/// ```
///
/// [`id`]: ../../std/thread/struct.Thread.html#method.id
/// [`Thread`]: ../../std/thread/struct.Thread.html
#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub struct ThreadId(NonZeroU64);

static mut GUARD: *mut SgxMutex<()> = 0 as *mut _;
static INIT: Once = ONCE_INIT;
impl ThreadId {
    // Generate a new unique thread ID.
    fn new() -> Self {
        // We never call `GUARD.init()`, so it is UB to attempt to
        // acquire this mutex reentrantly!
       unsafe {
            INIT.call_once(|| {
                GUARD = Box::into_raw(Box::new(SgxMutex::new(())));
            });
        }

        static mut COUNTER: u64 = 1;

        unsafe {
            let _guard = (*GUARD).lock();

            // If we somehow use up all our bits, panic so that we're not
            // covering up subtle bugs of IDs being reused.
            if COUNTER == crate::u64::MAX {
                panic!("failed to generate unique thread ID: bitspace exhausted");
            }

            let id = COUNTER;
            COUNTER += 1;

            ThreadId(NonZeroU64::new(id).unwrap())
        }
    }
}

/// The internal representation of a `Thread` handle
struct Inner {
    name: Option<CString>,      // Guaranteed to be UTF-8
    id: ThreadId,
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
    pub(crate) fn new(name: Option<String>) -> Self {
          let cname = name.map(|n| {
            CString::new(n).expect("thread name may not contain interior null bytes")
        });
        SgxThread {
            inner: Arc::new(Inner {
               name:cname,
                id: ThreadId::new(),
                state: AtomicUsize::new(EMPTY),
                lock: SgxMutex::new(()),
                cvar: SgxCondvar::new(),
            })
        }
    }

    /// Gets the thread's unique identifier.
    pub fn id(&self) -> ThreadId {
        self.inner.id
    }

    pub fn name(&self) -> Option<&str> {
        self.cname().map(|s| unsafe { str::from_utf8_unchecked(s.to_bytes()) } )
    }

    fn cname(&self) -> Option<&CStr> {
        self.inner.name.as_ref().map(|s| &**s)
    }

    /// Atomically makes the handle's token available if it is not already.
    ///
    /// Every thread is equipped with some basic low-level blocking support, via
    /// the [`park`][park] function and the `unpark()` method. These can be
    /// used as a more CPU-efficient implementation of a spinlock.
    ///
    pub fn unpark(&self) {
        // To ensure the unparked thread will observe any writes we made
        // before this call, we must perform a release operation that `park`
        // can synchronize with. To do that we must write `NOTIFIED` even if
        // `state` is already `NOTIFIED`. That is why this must be a swap
        // rather than a compare-and-swap that returns if it reads `NOTIFIED`
        // on failure.
        match self.inner.state.swap(NOTIFIED, SeqCst) {
            EMPTY => return, // no one was waiting
            NOTIFIED => return, // already unparked
            PARKED => {} // gotta go wake someone up
            _ => panic!("inconsistent state in unpark"),
        }

        // There is a period between when the parked thread sets `state` to
        // `PARKED` (or last checked `state` in the case of a spurious wake
        // up) and when it actually waits on `cvar`. If we were to notify
        // during this period it would be ignored and then when the parked
        // thread went to sleep it would never wake up. Fortunately, it has
        // `lock` locked at this stage so we can acquire `lock` to wait until
        // it is ready to receive the notification.
        //
        // Releasing `lock` before the call to `notify_one` means that when the
        // parked thread wakes it doesn't get woken only to have to wait for us
        // to release `lock`.
        drop(self.inner.lock.lock().unwrap());
        self.inner.cvar.signal()
    }
}

impl fmt::Debug for SgxThread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SgxThread")
            .field("id", &self.id())
            .field("name", &self.name())
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////
// JoinHandle
////////////////////////////////////////////////////////////////////////////////

/// A specialized [`Result`] type for threads.
///
/// Indicates the manner in which a thread exited.
///
/// A thread that completes without panicking is considered to exit successfully.
///
/// # Examples
///
/// ```no_run
/// use std::thread;
/// use std::fs;
///
/// fn copy_in_thread() -> thread::Result<()> {
///     thread::spawn(move || { fs::copy("foo.txt", "bar.txt").unwrap(); }).join()
/// }
///
/// fn main() {
///     match copy_in_thread() {
///         Ok(_) => println!("this is fine"),
///         Err(_) => println!("thread panicked"),
///     }
/// }
/// ```
///
/// [`Result`]: ../../std/result/enum.Result.html
pub type Result<T> = crate::result::Result<T, Box<dyn Any + Send + 'static>>;

// This packet is used to communicate the return value between the child thread
// and the parent thread. Memory is shared through the `Arc` within and there's
// no need for a mutex here because synchronization happens with `join()` (the
// parent thread never reads this packet until the child has exited).
//
// This packet itself is then stored into a `JoinInner` which in turns is placed
// in `JoinHandle` and `JoinGuard`. Due to the usage of `UnsafeCell` we need to
// manually worry about impls like Send and Sync. The type `T` should
// already always be Send (otherwise the thread could not have been created) and
// this type is inherently Sync because no methods take &self. Regardless,
// however, we add inheriting impls for Send/Sync to this type to ensure it's
// Send/Sync and that future modifications will still appropriately classify it.
struct Packet<T>(Arc<UnsafeCell<Option<Result<T>>>>);

unsafe impl<T: Send> Send for Packet<T> {}
unsafe impl<T: Sync> Sync for Packet<T> {}

/// Inner representation for JoinHandle
struct JoinInner<T> {
    native: Option<imp::Thread>,
    thread: SgxThread,
    packet: Packet<T>,
}

impl<T> JoinInner<T> {
    fn join(&mut self) -> Result<T> {
        let reval = self.native.take().unwrap().join();
        if reval != sgx_status_t::SGX_SUCCESS {
            Err(Box::new(Error::from(reval)))
        } else {
            unsafe {
                (*self.packet.0.get()).take().unwrap()
            }
        }
    }
}

/// An owned permission to join on a thread (block on its termination).
///
/// A `JoinHandle` *detaches* the associated thread when it is dropped, which
/// means that there is no longer any handle to thread and no way to `join`
/// on it.
///
/// Due to platform restrictions, it is not possible to [`Clone`] this
/// handle: the ability to join a thread is a uniquely-owned permission.
///
/// This `struct` is created by the [`thread::spawn`] function and the
/// [`thread::Builder::spawn`] method.
///
/// # Examples
///
/// Creation from [`thread::spawn`]:
///
/// ```
/// use std::thread;
///
/// let join_handle: thread::JoinHandle<_> = thread::spawn(|| {
///     // some work here
/// });
/// ```
///
/// Creation from [`thread::Builder::spawn`]:
///
/// ```
/// use std::thread;
///
/// let builder = thread::Builder::new();
///
/// let join_handle: thread::JoinHandle<_> = builder.spawn(|| {
///     // some work here
/// }).unwrap();
/// ```
///
/// Child being detached and outliving its parent:
///
/// ```no_run
/// use std::thread;
/// use std::time::Duration;
///
/// let original_thread = thread::spawn(|| {
///     let _detached_thread = thread::spawn(|| {
///         // Here we sleep to make sure that the first thread returns before.
///         thread::sleep(Duration::from_millis(10));
///         // This will be called, even though the JoinHandle is dropped.
///         println!("♫ Still alive ♫");
///     });
/// });
///
/// original_thread.join().expect("The thread being joined has panicked");
/// println!("Original thread is joined.");
///
/// // We make sure that the new thread has time to run, before the main
/// // thread returns.
///
/// thread::sleep(Duration::from_millis(1000));
/// ```
///
/// [`Clone`]: ../../std/clone/trait.Clone.html
/// [`thread::spawn`]: fn.spawn.html
/// [`thread::Builder::spawn`]: struct.Builder.html#method.spawn

pub struct JoinHandle<T>(JoinInner<T>);

unsafe impl<T> Send for JoinHandle<T> {}

unsafe impl<T> Sync for JoinHandle<T> {}

impl<T> JoinHandle<T> {
    /// Extracts a handle to the underlying thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::thread;
    ///
    /// let builder = thread::Builder::new();
    ///
    /// let join_handle: thread::JoinHandle<_> = builder.spawn(|| {
    ///     // some work here
    /// }).unwrap();
    ///
    /// let thread = join_handle.thread();
    /// println!("thread id: {:?}", thread.id());
    /// ```
    pub fn thread(&self) -> &SgxThread {
        &self.0.thread
    }

    /// Waits for the associated thread to finish.
    ///
    /// In terms of [atomic memory orderings],  the completion of the associated
    /// thread synchronizes with this function returning. In other words, all
    /// operations performed by that thread are ordered before all
    /// operations that happen after `join` returns.
    ///
    /// If the child thread panics, [`Err`] is returned with the parameter given
    /// to [`panic`].
    ///
    /// [`Err`]: ../../std/result/enum.Result.html#variant.Err
    /// [`panic`]: ../../std/macro.panic.html
    /// [atomic memory orderings]: ../../std/sync/atomic/index.html
    ///
    /// # Panics
    ///
    /// This function may panic on some platforms if a thread attempts to join
    /// itself or otherwise may create a deadlock with joining threads.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::thread;
    ///
    /// let builder = thread::Builder::new();
    ///
    /// let join_handle: thread::JoinHandle<_> = builder.spawn(|| {
    ///     // some work here
    /// }).unwrap();
    /// join_handle.join().expect("Couldn't join on the associated thread");
    /// ```
    pub fn join(mut self) -> Result<T> {
        self.0.join()
    }
}

impl<T> AsInner<imp::Thread> for JoinHandle<T> {
    fn as_inner(&self) -> &imp::Thread { self.0.native.as_ref().unwrap() }
}

impl<T> IntoInner<imp::Thread> for JoinHandle<T> {
    fn into_inner(self) -> imp::Thread { self.0.native.unwrap() }
}

impl<T> fmt::Debug for JoinHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("JoinHandle { .. }")
    }
}

fn _assert_sync_and_send() {
    fn _assert_both<T: Send + Sync>() {}
    _assert_both::<JoinHandle<()>>();
    _assert_both::<SgxThread>();
}