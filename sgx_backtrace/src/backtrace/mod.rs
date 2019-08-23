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

use core::fmt;
use sgx_libc::c_void;

/// Inspects the current call-stack, passing all active frames into the closure
/// provided to calculate a stack trace.
///
/// This function is the workhorse of this library in calculating the stack
/// traces for a program. The given closure `cb` is yielded instances of a
/// `Frame` which represent information about that call frame on the stack. The
/// closure is yielded frames in a top-down fashion (most recently called
/// functions first).
///
/// The closure's return value is an indication of whether the backtrace should
/// continue. A return value of `false` will terminate the backtrace and return
/// immediately.
///
/// Once a `Frame` is acquired you will likely want to call `backtrace::resolve`
/// to convert the `ip` (instruction pointer) or symbol address to a `Symbol`
/// through which the name and/or filename/line number can be learned.
///
/// Note that this is a relatively low-level function and if you'd like to, for
/// example, capture a backtrace to be inspected later, then the `Backtrace`
/// type may be more appropriate.
///
/// # Required features
///
/// This function requires the `std` feature of the `backtrace` crate to be
/// enabled, and the `std` feature is enabled by default.
///
/// # Panics
///
/// This function strives to never panic, but if the `cb` provided panics then
/// some platforms will force a double panic to abort the process. Some
/// platforms use a C library which internally uses callbacks which cannot be
/// unwound through, so panicking from `cb` may trigger a process abort.
///
/// # Example
///
/// ```
/// extern crate backtrace;
///
/// fn main() {
///     backtrace::trace(|frame| {
///         // ...
///
///         true // continue the backtrace
///     });
/// }
/// ```
#[cfg(feature = "std")]
pub fn trace<F: FnMut(&Frame) -> bool>(cb: F) {
    let _guard = crate::lock::lock();
    unsafe { trace_unsynchronized(cb) }
}

/// Same as `trace`, only unsafe as it's unsynchronized.
///
/// This function does not have synchronization guarentees but is available
/// when the `std` feature of this crate isn't compiled in. See the `trace`
/// function for more documentation and examples.
///
/// # Panics
///
/// See information on `trace` for caveats on `cb` panicking.
pub unsafe fn trace_unsynchronized<F: FnMut(&Frame) -> bool>(mut cb: F) {
    trace_imp(&mut cb)
}

/// A trait representing one frame of a backtrace, yielded to the `trace`
/// function of this crate.
///
/// The tracing function's closure will be yielded frames, and the frame is
/// virtually dispatched as the underlying implementation is not always known
/// until runtime.
#[derive(Clone)]
pub struct Frame {
    pub(crate) inner: FrameImp,
}

impl Frame {
    /// Returns the current instruction pointer of this frame.
    ///
    /// This is normally the next instruction to execute in the frame, but not
    /// all implementations list this with 100% accuracy (but it's generally
    /// pretty close).
    ///
    /// It is recommended to pass this value to `backtrace::resolve` to turn it
    /// into a symbol name.
    pub fn ip(&self) -> *mut c_void {
        self.inner.ip()
    }

    /// Returns the starting symbol address of the frame of this function.
    ///
    /// This will attempt to rewind the instruction pointer returned by `ip` to
    /// the start of the function, returning that value. In some cases, however,
    /// backends will just return `ip` from this function.
    ///
    /// The returned value can sometimes be used if `backtrace::resolve` failed
    /// on the `ip` given above.
    pub fn symbol_address(&self) -> *mut c_void {
        self.inner.symbol_address()
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Frame")
            .field("ip", &self.ip())
            .field("symbol_address", &self.symbol_address())
            .finish()
    }
}

mod libunwind;
use self::libunwind::trace as trace_imp;
pub(crate) use self::libunwind::Frame as FrameImp;
