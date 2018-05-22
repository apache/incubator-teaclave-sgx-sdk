// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

//! Implementation of various bits and pieces of the `panic!` macro and
//! associated runtime pieces.

use sgx_trts::trts::rsgx_abort;
use thread;
use core::mem;
use core::fmt;
use core::panic::{PanicInfo, Location};
use core::any::Any;
use core::ptr;
use core::raw;
use core::sync::atomic::{AtomicPtr, Ordering};
use alloc::boxed::Box;
use alloc::string::String;

static PANIC_HANDLER: AtomicPtr<()> = AtomicPtr::new(default_panic_handler as * mut ());

#[cfg(not(feature = "stdio"))]
#[allow(unused_variables)]
fn default_panic_handler(msg: &str, file: &str, line: u32, col: u32) {

}

#[cfg(feature = "stdio")]
fn default_panic_handler(msg: &str, file: &str, line: u32, col: u32) {

    use sys::stdio::Stderr;

    #[cfg(feature = "backtrace")]
    use sys_common::backtrace;

    #[cfg(feature = "backtrace")]
    let log_backtrace = {
        let panics = update_panic_count(0);

        if panics >= 2 {
            Some(backtrace::PrintFormat::Full)
        } else {
            backtrace::log_enabled()
        }
    };

    let mut err = Stderr::new().ok();
    let write = |err: &mut ::io::Write| {
        let _ = writeln!(err, "thread panicked at '{}', {}:{}:{}",
                        msg, file, line, col);

        #[cfg(feature = "backtrace")]
        {
            use core::sync::atomic::{AtomicBool, Ordering};

            static FIRST_PANIC: AtomicBool = AtomicBool::new(true);

            if let Some(format) = log_backtrace {
                let _ = backtrace::print(err, format);
            } else if FIRST_PANIC.compare_and_swap(true, false, Ordering::SeqCst) {
                //let _ = writeln!(err, "note: Compile with `RUST_BACKTRACE=1` for a backtrace.");
                let _ = writeln!(err, "note: Call backtrace::enable_backtrace with 'PrintFormat::Short' for a backtrace.");
            }
        }
    };

    match err.as_mut() {
        Some(ref mut err) => { write(err) },
        None => {},
    }
}

/// Registers a custom panic handler, replacing any that was previously registered.
pub fn set_panic_handler(handler: fn(&str, &str, u32, u32)) {
    if thread::panicking() {
        panic!("cannot modify the panic hook from a panicking thread");
    }
    PANIC_HANDLER.store(handler as * mut (), Ordering::SeqCst);
}

fn panic_handler<'a>(info: &'a PanicInfo) {

    let location = info.location().unwrap();  // The current implementation always returns Some
    let file = location.file();
    let line = location.line();
    let col = location.column();

    let msg: &str = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        }
    };

    let value = PANIC_HANDLER.load(Ordering::SeqCst);
    let handler: fn(&str, &str, u32, u32) = unsafe{ mem::transmute(value) };
    handler(msg, file, line, col);
}


#[allow(improper_ctypes)]
extern {
    fn __rust_maybe_catch_panic(f: fn(*mut u8),
                                data: *mut u8,
                                data_ptr: *mut usize,
                                vtable_ptr: *mut usize) -> u32;
    #[unwind(allowed)]
    fn __rust_start_panic(data: usize, vtable: usize) -> u32;
}

pub fn update_panic_count(amt: isize) -> usize {

    use core::cell::Cell;
    thread_local! { static PANIC_COUNT: Cell<usize> = Cell::new(0) }

    PANIC_COUNT.with(|c| {
        let next = (c.get() as isize + amt) as usize;
        c.set(next);
        return next
    })
}

/// Invoke a closure, capturing the cause of an unwinding panic if one occurs.
pub unsafe fn try<R, F: FnOnce() -> R>(f: F) -> Result<R, Box<Any + Send>> {
    #[allow(unions_with_drop_fields)]
    union Data<F, R> {
        f: F,
        r: R,
    }

    // We do some sketchy operations with ownership here for the sake of
    // performance. We can only  pass pointers down to
    // `__rust_maybe_catch_panic` (can't pass objects by value), so we do all
    // the ownership tracking here manually using a union.
    //
    // We go through a transition where:
    //
    // * First, we set the data to be the closure that we're going to call.
    // * When we make the function call, the `do_call` function below, we take
    //   ownership of the function pointer. At this point the `Data` union is
    //   entirely uninitialized.
    // * If the closure successfully returns, we write the return value into the
    //   data's return slot. Note that `ptr::write` is used as it's overwriting
    //   uninitialized data.
    // * Finally, when we come back out of the `__rust_maybe_catch_panic` we're
    //   in one of two states:
    //
    //      1. The closure didn't panic, in which case the return value was
    //         filled in. We move it out of `data` and return it.
    //      2. The closure panicked, in which case the return value wasn't
    //         filled in. In this case the entire `data` union is invalid, so
    //         there is no need to drop anything.
    //
    // Once we stack all that together we should have the "most efficient'
    // method of calling a catch panic whilst juggling ownership.
    let mut any_data = 0_usize;
    let mut any_vtable = 0_usize;
    let mut data = Data {
        f: f,
    };

    let r = __rust_maybe_catch_panic(do_call::<F, R>,
                                     &mut data as *mut _ as *mut u8,
                                     &mut any_data,
                                     &mut any_vtable);

    return if r == 0 {
        debug_assert!(update_panic_count(0) == 0);
        Ok(data.r)
    } else {
        update_panic_count(-1);
        debug_assert!(update_panic_count(0) == 0);
        Err(mem::transmute(raw::TraitObject {
            data: any_data as *mut _,
            vtable: any_vtable as *mut _,
        }))
    };

    fn do_call<F: FnOnce() -> R, R>(data: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let f = ptr::read(&mut (*data).f);
            ptr::write(&mut (*data).r, f());
        }
    }
}

/// Determines whether the current thread is unwinding because of panic.
pub fn panicking() -> bool {
    update_panic_count(0) != 0
}

/// Entry point of panic from the libcore crate.
#[lang = "panic_fmt"]
#[unwind(allowed)]
pub extern fn rust_begin_panic(msg: fmt::Arguments,
                               file: &'static str,
                               line: u32,
                               col: u32) -> ! {
    begin_panic_fmt(&msg, &(file, line, col))
}

/// The entry point for panicking with a formatted message.
///
/// This is designed to reduce the amount of code required at the call
/// site as much as possible (so that `panic!()` has as low an impact
/// on (e.g.) the inlining of other functions as possible), by moving
/// the actual formatting into this shared place.
#[inline(never)] #[cold]
pub fn begin_panic_fmt(msg: &fmt::Arguments,
                       file_line_col: &(&'static str, u32, u32)) -> ! {

    use core::fmt::Write;
    use alloc::string::String;

    // We do two allocations here, unfortunately. But (a) they're
    // required with the current scheme, and (b) we don't handle
    // panic + OOM properly anyway (see comment in begin_panic
    // below).

    let mut s = String::new();
    let _ = s.write_fmt(*msg);
    rust_panic_with_hook(Box::new(s), Some(msg), file_line_col)
}

/// This is the entry point of panicking for panic!() and assert!().
#[inline(never)] #[cold] // avoid code bloat at the call sites as much as possible
pub fn begin_panic<M: Any + Send>(msg: M, file_line_col: &(&'static str, u32, u32)) -> ! {
    // Note that this should be the only allocation performed in this code path.
    // Currently this means that panic!() on OOM will invoke this code path,
    // but then again we're not really ready for panic on OOM anyway. If
    // we do start doing this, then we should propagate this allocation to
    // be performed in the parent of this thread instead of the thread that's
    // panicking.

    rust_panic_with_hook(Box::new(msg), None, file_line_col)
}

/// Executes the primary logic for a panic, including checking for recursive
/// panics and panic hooks.
///
/// This is the entry point or panics from libcore, formatted panics, and
/// `Box<Any>` panics. Here we'll verify that we're not panicking recursively,
/// run panic hooks, and then delegate to the actual implementation of panics.
#[inline(never)]
#[cold]
fn rust_panic_with_hook(payload: Box<Any + Send>,
                        message: Option<&fmt::Arguments>,
                        file_line_col: &(&'static str, u32, u32)) -> ! {
    let (file, line, col) = *file_line_col;

    let panics = update_panic_count(1);

    // If this is the third nested call (e.g. panics == 2, this is 0-indexed),
    // the panic hook probably triggered the last panic, otherwise the
    // double-panic check would have aborted the process. In this case abort the
    // process real quickly as we don't want to try calling it again as it'll
    // probably just panic again.
    if panics > 2 {
        //util::dumb_print(format_args!("thread panicked while processing \
        //                               panic. aborting.\n"));
        rsgx_abort()
    }

    {
        let info = PanicInfo::internal_constructor(
            &*payload,
            message,
            Location::internal_constructor(file, line, col),
        );
        panic_handler(&info);
    }


    if panics > 1 {
        // If a thread panics while it's already unwinding then we
        // have limited options. Currently our preference is to
        // just abort. In the future we may consider resuming
        // unwinding or otherwise exiting the thread cleanly.
        //util::dumb_print(format_args!("thread panicked while panicking. \
        //                               aborting.\n"));
        rsgx_abort()
    }

    rust_panic(payload)
}

/// Shim around rust_panic. Called by resume_unwind.
pub fn update_count_then_panic(msg: Box<Any + Send>) -> ! {
    update_panic_count(1);
    rust_panic(msg)
}

/// A private no-mangle function on which to slap yer breakpoints.
#[no_mangle]
#[allow(unused_variables)]
#[allow(private_no_mangle_fns)] // yes we get it, but we like breakpoints
pub fn rust_panic(msg: Box<Any + Send>) -> ! {
    let code = unsafe {
        let obj = mem::transmute::<_, raw::TraitObject>(msg);
        __rust_start_panic(obj.data as usize, obj.vtable as usize)
    };
    //rtabort!("failed to initiate panic, error {}", code)
    rsgx_abort()
}
