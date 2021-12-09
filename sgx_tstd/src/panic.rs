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

//! Panic support in the standard library.

use crate::any::Any;
use crate::collections;
use crate::panicking;
use crate::sync::{SgxMutex, SgxRwLock};
use crate::thread::Result;

#[doc(hidden)]
#[allow_internal_unstable(libstd_sys_internals, const_format_args, core_panic)]
#[cfg_attr(not(test), rustc_diagnostic_item = "std_panic_2015_macro")]
#[rustc_macro_transparency = "semitransparent"]
pub macro panic_2015 {
    () => ({
        $crate::rt::begin_panic("explicit panic")
    }),
    ($msg:expr $(,)?) => ({
        $crate::rt::begin_panic($msg)
    }),
    // Special-case the single-argument case for const_panic.
    ("{}", $arg:expr $(,)?) => ({
        $crate::rt::panic_display(&$arg)
    }),
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::rt::panic_fmt($crate::const_format_args!($fmt, $($arg)+))
    }),
}

#[doc(hidden)]
pub use core::panic::panic_2021;

pub use crate::panicking::{set_hook, take_hook};

pub use core::panic::{Location, PanicInfo};

pub use core::panic::{AssertUnwindSafe, RefUnwindSafe, UnwindSafe};

/// Panic the current thread with the given message as the panic payload.
///
/// The message can be of any (`Any + Send`) type, not just strings.
///
/// The message is wrapped in a `Box<'static + Any + Send>`, which can be
/// accessed later using [`PanicInfo::payload`].
///
/// See the [`panic!`] macro for more information about panicking.
#[inline]
#[track_caller]
pub fn panic_any<M: 'static + Any + Send>(msg: M) -> ! {
    crate::panicking::begin_panic(msg);
}

impl<T: ?Sized> UnwindSafe for SgxMutex<T> {}
impl<T: ?Sized> UnwindSafe for SgxRwLock<T> {}

impl<T: ?Sized> RefUnwindSafe for SgxMutex<T> {}
impl<T: ?Sized> RefUnwindSafe for SgxRwLock<T> {}

// https://github.com/rust-lang/rust/issues/62301
impl<K, V, S> UnwindSafe for collections::HashMap<K, V, S>
where
    K: UnwindSafe,
    V: UnwindSafe,
    S: UnwindSafe,
{
}

/// Invokes a closure, capturing the cause of an unwinding panic if one occurs.
///
/// This function will return `Ok` with the closure's result if the closure
/// does not panic, and will return `Err(cause)` if the closure panics. The
/// `cause` returned is the object with which panic was originally invoked.
///
/// It is currently undefined behavior to unwind from Rust code into foreign
/// code, so this function is particularly useful when Rust is called from
/// another language (normally C). This can run arbitrary Rust code, capturing a
/// panic and allowing a graceful handling of the error.
///
/// It is **not** recommended to use this function for a general try/catch
/// mechanism. The [`Result`] type is more appropriate to use for functions that
/// can fail on a regular basis. Additionally, this function is not guaranteed
/// to catch all panics, see the "Notes" section below.
///
/// The closure provided is required to adhere to the [`UnwindSafe`] trait to ensure
/// that all captured variables are safe to cross this boundary. The purpose of
/// this bound is to encode the concept of [exception safety][rfc] in the type
/// system. Most usage of this function should not need to worry about this
/// bound as programs are naturally unwind safe without `unsafe` code. If it
/// becomes a problem the [`AssertUnwindSafe`] wrapper struct can be used to quickly
/// assert that the usage here is indeed unwind safe.
///
/// [rfc]: https://github.com/rust-lang/rfcs/blob/master/text/1236-stabilize-catch-panic.md
///
/// # Notes
///
/// Note that this function **might not catch all panics** in Rust. A panic in
/// Rust is not always implemented via unwinding, but can be implemented by
/// aborting the process as well. This function *only* catches unwinding panics,
/// not those that abort the process.
///
/// Also note that unwinding into Rust code with a foreign exception (e.g.
/// an exception thrown from C++ code) is undefined behavior.
///
/// # Examples
///
/// ```
/// use std::panic;
///
/// let result = panic::catch_unwind(|| {
///     println!("hello!");
/// });
/// assert!(result.is_ok());
///
/// let result = panic::catch_unwind(|| {
///     panic!("oh no!");
/// });
/// assert!(result.is_err());
/// ```
pub fn catch_unwind<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> Result<R> {
    unsafe { panicking::r#try(f) }
}

/// Triggers a panic without invoking the panic hook.
///
/// This is designed to be used in conjunction with [`catch_unwind`] to, for
/// example, carry a panic across a layer of C code.
///
/// # Notes
///
/// Note that panics in Rust are not always implemented via unwinding, but they
/// may be implemented by aborting the process. If this function is called when
/// panics are implemented this way then this function will abort the process,
/// not trigger an unwind.
///
/// # Examples
///
/// ```should_panic
/// use std::panic;
///
/// let result = panic::catch_unwind(|| {
///     panic!("oh no!");
/// });
///
/// if let Err(err) = result {
///     panic::resume_unwind(err);
/// }
/// ```
pub fn resume_unwind(payload: Box<dyn Any + Send>) -> ! {
    panicking::rust_panic_without_hook(payload)
}

/// Make all future panics abort directly without running the panic hook or unwinding.
///
/// There is no way to undo this; the effect lasts until the process exits or
/// execs (or the equivalent).
///
/// # Use after fork
///
/// This function is particularly useful for calling after `libc::fork`.  After `fork`, in a
/// multithreaded program it is (on many platforms) not safe to call the allocator.  It is also
/// generally highly undesirable for an unwind to unwind past the `fork`, because that results in
/// the unwind propagating to code that was only ever expecting to run in the parent.
///
/// `panic::always_abort()` helps avoid both of these.  It directly avoids any further unwinding,
/// and if there is a panic, the abort will occur without allocating provided that the arguments to
/// panic can be formatted without allocating.
///
/// Examples
///
/// ```no_run
/// #![feature(panic_always_abort)]
/// use std::panic;
///
/// panic::always_abort();
///
/// let _ = panic::catch_unwind(|| {
///     panic!("inside the catch");
/// });
///
/// // We will have aborted already, due to the panic.
/// unreachable!();
/// ```
pub fn always_abort() {
    crate::panicking::panic_count::set_always_abort();
}
