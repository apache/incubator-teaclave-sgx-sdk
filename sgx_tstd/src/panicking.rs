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

//! Implementation of various bits and pieces of the `panic!` macro and
//! associated runtime pieces.
//!
//! Specifically, this module contains the implementation of:
//!
//! * Panic hooks
//! * Executing a panic up to doing the actual implementation
//! * Shims around "try"

use sgx_trts::trts::rsgx_abort;
use core::any::Any;
use core::fmt;
use core::intrinsics;
use core::mem::{self, ManuallyDrop};
use core::panic::{BoxMeUp, Location, PanicInfo};
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use alloc_crate::boxed::Box;
use alloc_crate::string::String;
use crate::sys::stdio::panic_output;
use crate::sys_common::{thread_info, util};
use crate::thread;

// Binary interface to the panic runtime that the standard library depends on.
//
// The standard library is tagged with `#![needs_panic_runtime]` (introduced in
// RFC 1513) to indicate that it requires some other crate tagged with
// `#![panic_runtime]` to exist somewhere. Each panic runtime is intended to
// implement these symbols (with the same signatures) so we can get matched up
// to them.
//
// One day this may look a little less ad-hoc with the compiler helping out to
// hook up these functions, but it is not this day!
#[allow(improper_ctypes)]
extern "C" {
    fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static);

    /// `payload` is actually a `*mut &mut dyn BoxMeUp` but that would cause FFI warnings.
    /// It cannot be `Box<dyn BoxMeUp>` because the other end of this call does not depend
    /// on liballoc, and thus cannot use `Box`.
    #[unwind(allowed)]
    fn __rust_start_panic(payload: usize) -> u32;
}

/// This function is called by the panic runtime if FFI code catches a Rust
/// panic but doesn't rethrow it. We don't support this case since it messes
/// with our panic count.
#[rustc_std_internal_symbol]
extern "C" fn __rust_drop_panic() -> ! {
    rtabort!("Rust panics must be rethrown");
}

#[rustc_std_internal_symbol]
extern "C" fn __rust_foreign_exception() -> ! {
    rtabort!("Rust cannot catch foreign exceptions");
}

static PANIC_HANDLER: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

#[cfg(not(feature = "stdio"))]
#[allow(unused_variables)]
fn default_panic_handler(info: &PanicInfo<'_>) {

}

#[cfg(feature = "stdio")]
fn default_panic_handler(info: &PanicInfo<'_>) {
    #[cfg(feature = "backtrace")]
    use crate::sys_common::backtrace::{self, RustBacktrace};

    #[allow(unused_variables)]
    let panics = update_panic_count(0);

    #[cfg(feature = "backtrace")]
    let backtrace_env = {
        use crate::sys::backtrace::PrintFmt;
        if panics >= 2 {
            RustBacktrace::Print(PrintFmt::Full)
        } else {
            backtrace::rust_backtrace_env()
        }
    };

    // The current implementation always returns `Some`.
    let location = info.location().unwrap();

    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };
    let thread = thread_info::current_thread();
    let name = thread.as_ref().and_then(|t| t.name()).unwrap_or("<unnamed>");

    let write = |err: &mut dyn crate::io::Write| {
        let _ = writeln!(err, "thread '{}' panicked at '{}', {}", name, msg, location);

        #[cfg(feature = "backtrace")]
        {
            use core::sync::atomic::AtomicBool;
            static FIRST_PANIC: AtomicBool = AtomicBool::new(true);

            match backtrace_env {
                RustBacktrace::Print(format) => drop(backtrace::print(err, format)),
                RustBacktrace::Disabled => {}
                RustBacktrace::RuntimeDisabled => {
                    if FIRST_PANIC.swap(false, Ordering::SeqCst) {
                        let _ = writeln!(
                            err,
                            "note: Call backtrace::enable_backtrace with 'PrintFormat::Short/Full' for a backtrace."
                        );
                    }
                }
            }
        }
    };

    if let Some(mut out) = panic_output() {
        write(&mut out);
    }
}

/// Registers a custom panic handler, replacing any that was previously registered.
pub fn set_panic_handler(handler: fn(&PanicInfo<'_>)) {
    if thread::panicking() {
        panic!("cannot modify the panic hook from a panicking thread");
    }
    PANIC_HANDLER.store(handler as *mut (), Ordering::SeqCst);
}

pub fn take_panic_handler() -> fn(&PanicInfo<'_>) {
    let hook = PANIC_HANDLER.swap(ptr::null_mut(), Ordering::SeqCst);
    if hook.is_null() { default_panic_handler } else { unsafe { mem::transmute(hook) } }
}

fn panic_handler(info: &PanicInfo<'_>) {
    let hook = PANIC_HANDLER.load(Ordering::SeqCst);
    let handler: fn(&PanicInfo<'_>) =
        if hook.is_null() { default_panic_handler } else { unsafe { mem::transmute(hook) } };
    handler(info);
}

pub fn update_panic_count(amt: isize) -> usize {
    use core::cell::Cell;
    thread_local! { static PANIC_COUNT: Cell<usize> = Cell::new(0) }

    PANIC_COUNT.with(|c| {
        let next = (c.get() as isize + amt) as usize;
        c.set(next);
        next
    })
}

/// Invoke a closure, capturing the cause of an unwinding panic if one occurs.
pub unsafe fn r#try<R, F: FnOnce() -> R>(f: F) -> Result<R, Box<dyn Any + Send>> {
    union Data<F, R> {
        f: ManuallyDrop<F>,
        r: ManuallyDrop<R>,
        p: ManuallyDrop<Box<dyn Any + Send>>,
    }

    // We do some sketchy operations with ownership here for the sake of
    // performance. We can only pass pointers down to `do_call` (can't pass
    // objects by value), so we do all the ownership tracking here manually
    // using a union.
    //
    // We go through a transition where:
    //
    // * First, we set the data field `f` to be the argumentless closure that we're going to call.
    // * When we make the function call, the `do_call` function below, we take
    //   ownership of the function pointer. At this point the `data` union is
    //   entirely uninitialized.
    // * If the closure successfully returns, we write the return value into the
    //   data's return slot (field `r`).
    // * If the closure panics (`do_catch` below), we write the panic payload into field `p`.
    // * Finally, when we come back out of the `try` intrinsic we're
    //   in one of two states:
    //
    //      1. The closure didn't panic, in which case the return value was
    //         filled in. We move it out of `data.r` and return it.
    //      2. The closure panicked, in which case the panic payload was
    //         filled in. We move it out of `data.p` and return it.
    //
    // Once we stack all that together we should have the "most efficient'
    // method of calling a catch panic whilst juggling ownership.
    let mut data = Data { f: ManuallyDrop::new(f) };

    let data_ptr = &mut data as *mut _ as *mut u8;
    return if do_try(do_call::<F, R>, data_ptr, do_catch::<F, R>) == 0 {
        Ok(ManuallyDrop::into_inner(data.r))
    } else {
        Err(ManuallyDrop::into_inner(data.p))
    };

    // Compatibility wrapper around the try intrinsic for bootstrap.
    //
    // We also need to mark it #[inline(never)] to work around a bug on MinGW
    // targets: the unwinding implementation was relying on UB, but this only
    // becomes a problem in practice if inlining is involved.
    #[cfg(not(bootstrap))]
    use intrinsics::r#try as do_try;
    #[cfg(bootstrap)]
    #[inline(never)]
    unsafe fn do_try(try_fn: fn(*mut u8), data: *mut u8, catch_fn: fn(*mut u8, *mut u8)) -> i32 {
        use core::mem::MaybeUninit;
        type TryPayload = *mut u8;

        let mut payload: MaybeUninit<TryPayload> = MaybeUninit::uninit();
        let payload_ptr = payload.as_mut_ptr() as *mut u8;
        let r = intrinsics::r#try(try_fn, data, payload_ptr);
        if r != 0 {
            catch_fn(data, payload.assume_init())
        }
        r
    }

    // We consider unwinding to be rare, so mark this function as cold. However,
    // do not mark it no-inline -- that decision is best to leave to the
    // optimizer (in most cases this function is not inlined even as a normal,
    // non-cold function, though, as of the writing of this comment).
    #[cold]
    unsafe fn cleanup(payload: *mut u8) -> Box<dyn Any + Send + 'static> {
        let obj = Box::from_raw(__rust_panic_cleanup(payload));
        update_panic_count(-1);
        obj
    }

    // See comment on do_try above for why #[inline(never)] is needed on bootstrap.
    #[cfg_attr(bootstrap, inline(never))]
    #[cfg_attr(not(bootstrap), inline)]
    fn do_call<F: FnOnce() -> R, R>(data: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let data = &mut (*data);
            let f = ManuallyDrop::take(&mut data.f);
            data.r = ManuallyDrop::new(f());
        }
    }

    // We *do* want this part of the catch to be inlined: this allows the
    // compiler to properly track accesses to the Data union and optimize it
    // away most of the time.
    #[inline]
    fn do_catch<F: FnOnce() -> R, R>(data: *mut u8, payload: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let data = &mut (*data);
            let obj = cleanup(payload);
            data.p = ManuallyDrop::new(obj);
        }
    }
}

/// Determines whether the current thread is unwinding because of panic.
pub fn panicking() -> bool {
    update_panic_count(0) != 0
}

/// The entry point for panicking with a formatted message.
///
/// This is designed to reduce the amount of code required at the call
/// site as much as possible (so that `panic!()` has as low an impact
/// on (e.g.) the inlining of other functions as possible), by moving
/// the actual formatting into this shared place.
#[cold]
// If panic_immediate_abort, inline the abort call,
// otherwise avoid inlining because of it is cold path.
#[track_caller]
#[inline(never)]
pub fn begin_panic_fmt(msg: &fmt::Arguments<'_>) -> ! {
    let info = PanicInfo::internal_constructor(Some(msg), Location::caller());
    begin_panic_handler(&info)
}

/// Entry point of panics from the libcore crate (`panic_impl` lang item).
#[panic_handler]
#[unwind(allowed)]
pub fn begin_panic_handler(info: &PanicInfo<'_>) -> ! {
    struct PanicPayload<'a> {
        inner: &'a fmt::Arguments<'a>,
        string: Option<String>,
    }

    impl<'a> PanicPayload<'a> {
        fn new(inner: &'a fmt::Arguments<'a>) -> PanicPayload<'a> {
            PanicPayload { inner, string: None }
        }

        fn fill(&mut self) -> &mut String {
            use core::fmt::Write;

            let inner = self.inner;
            // Lazily, the first time this gets called, run the actual string formatting.
            self.string.get_or_insert_with(|| {
                let mut s = String::new();
                drop(s.write_fmt(*inner));
                s
            })
        }
    }

    unsafe impl<'a> BoxMeUp for PanicPayload<'a> {
        fn take_box(&mut self) -> *mut (dyn Any + Send) {
            // We do two allocations here, unfortunately. But (a) they're required with the current
            // scheme, and (b) we don't handle panic + OOM properly anyway (see comment in
            // begin_panic below).
            let contents = mem::take(self.fill());
            Box::into_raw(Box::new(contents))
        }

        fn get(&mut self) -> &(dyn Any + Send) {
            self.fill()
        }
    }

    let loc = info.location().unwrap(); // The current implementation always returns Some
    let msg = info.message().unwrap(); // The current implementation always returns Some
    rust_panic_with_hook(&mut PanicPayload::new(msg), info.message(), loc);
}

/// This is the entry point of panicking for the non-format-string variants of
/// panic!() and assert!(). In particular, this is the only entry point that supports
/// arbitrary payloads, not just format strings.
#[lang = "begin_panic"]
// lang item for CTFE panic support
// never inline unless panic_immediate_abort to avoid code
// bloat at the call sites as much as possible
#[cold]
#[track_caller]
pub fn begin_panic<M: Any + Send>(msg: M) -> ! {
    rust_panic_with_hook(&mut PanicPayload::new(msg), None, Location::caller());

    struct PanicPayload<A> {
        inner: Option<A>,
    }

    impl<A: Send + 'static> PanicPayload<A> {
        fn new(inner: A) -> PanicPayload<A> {
            PanicPayload { inner: Some(inner) }
        }
    }

    unsafe impl<A: Send + 'static> BoxMeUp for PanicPayload<A> {
        fn take_box(&mut self) -> *mut (dyn Any + Send) {
            // Note that this should be the only allocation performed in this code path. Currently
            // this means that panic!() on OOM will invoke this code path, but then again we're not
            // really ready for panic on OOM anyway. If we do start doing this, then we should
            // propagate this allocation to be performed in the parent of this thread instead of the
            // thread that's panicking.
            let data = match self.inner.take() {
                Some(a) => Box::new(a) as Box<dyn Any + Send>,
                None => Box::new(()),
            };
            Box::into_raw(data)
        }

        fn get(&mut self) -> &(dyn Any + Send) {
            match self.inner {
                Some(ref a) => a,
                None => &(),
            }
        }
    }
}

/// Central point for dispatching panics.
///
/// Executes the primary logic for a panic, including checking for recursive
/// panics, panic hooks, and finally dispatching to the panic runtime to either
/// abort or unwind.
fn rust_panic_with_hook(
    payload: &mut dyn BoxMeUp,
    message: Option<&fmt::Arguments<'_>>,
    location: &Location<'_>,
) -> ! {
    let panics = update_panic_count(1);

    // If this is the third nested call (e.g., panics == 2, this is 0-indexed),
    // the panic hook probably triggered the last panic, otherwise the
    // double-panic check would have aborted the process. In this case abort the
    // process real quickly as we don't want to try calling it again as it'll
    // probably just panic again.
    if panics > 2 {
        util::dumb_print(format_args!(
            "thread panicked while processing \
                                       panic. aborting.\n"
        ));
        rsgx_abort()
    }

    let mut info = PanicInfo::internal_constructor(message, location);
    info.set_payload(payload.get());
    panic_handler(&info);

    if panics > 1 {
        // If a thread panics while it's already unwinding then we
        // have limited options. Currently our preference is to
        // just abort. In the future we may consider resuming
        // unwinding or otherwise exiting the thread cleanly.
        util::dumb_print(format_args!(
            "thread panicked while panicking. \
                                       aborting.\n"
        ));
        rsgx_abort()
    }

    rust_panic(payload)
}

/// This is the entry point for `resume_unwind`.
/// It just forwards the payload to the panic runtime.
pub fn rust_panic_without_hook(payload: Box<dyn Any + Send>) -> ! {
    update_panic_count(1);

    struct RewrapBox(Box<dyn Any + Send>);

    unsafe impl BoxMeUp for RewrapBox {
        fn take_box(&mut self) -> *mut (dyn Any + Send) {
            Box::into_raw(mem::replace(&mut self.0, Box::new(())))
        }

        fn get(&mut self) -> &(dyn Any + Send) {
            &*self.0
        }
    }

    rust_panic(&mut RewrapBox(payload))
}

/// An unmangled function (through `rustc_std_internal_symbol`) on which to slap
/// yer breakpoints.
#[inline(never)]
#[rustc_std_internal_symbol]
fn rust_panic(mut msg: &mut dyn BoxMeUp) -> ! {
    let code = unsafe {
        let obj = &mut msg as *mut &mut dyn BoxMeUp;
        __rust_start_panic(obj as usize)
    };
    rtabort!("failed to initiate panic, error {}", code)
}
