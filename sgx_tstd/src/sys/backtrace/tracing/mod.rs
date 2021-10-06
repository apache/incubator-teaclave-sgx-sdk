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

use crate::ffi::c_void;
use crate::fmt;

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
    inner: FrameImp,
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

    /// Returns the current stack pointer of this frame.
    ///
    /// In the case that a backend cannot recover the stack pointer for this
    /// frame, a null pointer is returned.
    pub fn sp(&self) -> *mut c_void {
        self.inner.sp()
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

    /// Returns the base address of the module to which the frame belongs.
    pub fn module_base_address(&self) -> Option<*mut c_void> {
        self.inner.module_base_address()
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("ip", &self.ip())
            .field("symbol_address", &self.symbol_address())
            .finish()
    }
}

mod gcc_s;
use self::gcc_s::trace as trace_imp;
use self::gcc_s::Frame as FrameImp;
