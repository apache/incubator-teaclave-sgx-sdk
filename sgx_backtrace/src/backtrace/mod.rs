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
