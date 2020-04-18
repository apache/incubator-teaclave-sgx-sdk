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

//! Support for capturing a stack backtrace of an OS thread
//!
//! This module contains the support necessary to capture a stack backtrace of a
//! running OS thread from the OS thread itself. The `Backtrace` type supports
//! capturing a stack trace via the `Backtrace::capture` and
//! `Backtrace::force_capture` functions.
//!
//! A backtrace is typically quite handy to attach to errors (e.g. types
//! implementing `std::error::Error`) to get a causal chain of where an error
//! was generated.
//!
//! > **Note**: this module is unstable and is designed in [RFC 2504], and you
//! > can learn more about its status in the [tracking issue].
//!
//! [RFC 2504]: https://github.com/rust-lang/rfcs/blob/master/text/2504-fix-error.md
//! [tracking issue]: https://github.com/rust-lang/rust/issues/53487
//!
//! ## Accuracy
//!
//! Backtraces are attempted to be as accurate as possible, but no guarantees
//! are provided about the exact accuracy of a backtrace. Instruction pointers,
//! symbol names, filenames, line numbers, etc, may all be incorrect when
//! reported. Accuracy is attempted on a best-effort basis, however, and bugs
//! are always welcome to indicate areas of improvement!
//!
//! For most platforms a backtrace with a filename/line number requires that
//! programs be compiled with debug information. Without debug information
//! filenames/line numbers will not be reported.
//!
//! ## Platform support
//!
//! Not all platforms that libstd compiles for support capturing backtraces.
//! Some platforms simply do nothing when capturing a backtrace. To check
//! whether the platform supports capturing backtraces you can consult the
//! `BacktraceStatus` enum as a result of `Backtrace::status`.
//!
//! Like above with accuracy platform support is done on a best effort basis.
//! Sometimes libraries may not be available at runtime or something may go
//! wrong which would cause a backtrace to not be captured. Please feel free to
//! report issues with platforms where a backtrace cannot be captured though!
//!

pub use crate::sys_common::backtrace::__rust_begin_short_backtrace;
pub use crate::sys_common::backtrace::PrintFormat;

use crate::enclave;
use crate::sys::backtrace::{self, BytesOrWideString};
use crate::path::Path;
use crate::io;
use crate::sync::SgxMutex;
use crate::sys_common::backtrace::{
    lock,
    output_filename,
    rust_backtrace_env,
    RustBacktrace,
    set_enabled,
    SymbolName,
    resolve_frame_unsynchronized,
};
use crate::untrusted::fs;
use core::ffi::c_void;
use core::fmt;
use alloc_crate::vec::Vec;

/// A captured OS thread stack backtrace.
///
/// This type represents a stack backtrace for an OS thread captured at a
/// previous point in time. In some instances the `Backtrace` type may
/// internally be empty due to configuration. For more information see
/// `Backtrace::capture`.
pub struct Backtrace {
    inner: Inner,
}

/// The current status of a backtrace, indicating whether it was captured or
/// whether it is empty for some other reason.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum BacktraceStatus {
    /// Capturing a backtrace is not supported.
    Unsupported,
    /// Capturing a backtrace has been disabled.
    Disabled,
    /// A backtrace has been captured and the `Backtrace` should print
    /// reasonable information when rendered.
    Captured,
}

enum Inner {
    Unsupported,
    Disabled,
    Captured(SgxMutex<Capture>),
}

struct Capture {
    actual_start: usize,
    resolved: bool,
    frames: Vec<BacktraceFrame>,
}

fn _assert_send_sync() {
    fn _assert<T: Send + Sync>() {}
    _assert::<Backtrace>();
}

struct BacktraceFrame {
    frame: RawFrame,
    symbols: Vec<BacktraceSymbol>,
}

enum RawFrame {
    Actual(backtrace::Frame),
    #[cfg(test)]
    Fake,
}

struct BacktraceSymbol {
    name: Option<Vec<u8>>,
    filename: Option<BytesOrWide>,
    lineno: Option<u32>,
}

enum BytesOrWide {
    Bytes(Vec<u8>),
    Wide(Vec<u16>),
}

impl fmt::Debug for Backtrace {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut capture = match &self.inner {
            Inner::Unsupported => return fmt.write_str("<unsupported>"),
            Inner::Disabled => return fmt.write_str("<disabled>"),
            Inner::Captured(c) => c.lock().unwrap(),
        };
        capture.resolve();

        let frames = &capture.frames[capture.actual_start..];

        write!(fmt, "Backtrace ")?;

        let mut dbg = fmt.debug_list();

        for frame in frames {
            if frame.frame.ip().is_null() {
                continue;
            }

            dbg.entries(&frame.symbols);
        }

        dbg.finish()
    }
}

impl fmt::Debug for BacktraceSymbol {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{{ ")?;

        if let Some(fn_name) = self.name.as_ref().map(|b| SymbolName::new(b)) {
            write!(fmt, "fn: \"{:#}\"", fn_name)?;
        } else {
            write!(fmt, "fn: <unknown>")?;
        }

        if let Some(fname) = self.filename.as_ref() {
            write!(fmt, ", file: \"{:?}\"", fname)?;
        }

        if let Some(line) = self.lineno.as_ref() {
            write!(fmt, ", line: {:?}", line)?;
        }

        write!(fmt, " }}")
    }
}

impl fmt::Debug for BytesOrWide {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        output_filename(
            fmt,
            match self {
                BytesOrWide::Bytes(w) => BytesOrWideString::Bytes(w),
                BytesOrWide::Wide(w) => BytesOrWideString::Wide(w),
            },
            backtrace::PrintFmt::Short,
            None,
        )
    }
}

impl Backtrace {
    /// Returns whether backtrace captures are enabled.
    fn enabled() -> bool {
        match rust_backtrace_env() {
            RustBacktrace::Print(_) => true,
            _ => false,
        }
    }

    /// Capture a stack backtrace of the current thread.
    ///
    /// This function will capture a stack backtrace of the current OS thread of
    /// execution, returning a `Backtrace` type which can be later used to print
    /// the entire stack trace or render it to a string.
    ///
    #[inline(never)] // want to make sure there's a frame here to remove
    pub fn capture() -> Backtrace {
        if !Backtrace::enabled() {
            return Backtrace { inner: Inner::Disabled };
        }
        Backtrace::create(Backtrace::capture as usize)
    }

    /// Forcibly captures a full backtrace.
    ///
    #[inline(never)] // want to make sure there's a frame here to remove
    pub fn force_capture() -> Backtrace {
        if !Backtrace::enabled() {
            return Backtrace { inner: Inner::Disabled };
        }
        Backtrace::create(Backtrace::force_capture as usize)
    }

    // Capture a backtrace which start just before the function addressed by
    // `ip`
    fn create(ip: usize) -> Backtrace {
        let _lock = lock();
        let mut frames = Vec::new();
        let mut actual_start = None;
        unsafe {
            backtrace::trace_unsynchronized(|frame| {
                frames.push(BacktraceFrame {
                    frame: RawFrame::Actual(frame.clone()),
                    symbols: Vec::new(),
                });
                if frame.symbol_address() as usize == ip && actual_start.is_none() {
                    actual_start = Some(frames.len());
                }
                true
            });
        }

        // If no frames came out assume that this is an unsupported platform
        // since `backtrace` doesn't provide a way of learning this right now,
        // and this should be a good enough approximation.
        let inner = if frames.is_empty() {
            Inner::Unsupported
        } else {
            Inner::Captured(SgxMutex::new(Capture {
                actual_start: actual_start.unwrap_or(0),
                frames,
                resolved: false,
            }))
        };

        Backtrace { inner }
    }

    /// Returns the status of this backtrace, indicating whether this backtrace
    /// request was unsupported, disabled, or a stack trace was actually
    /// captured.
    pub fn status(&self) -> BacktraceStatus {
        match self.inner {
            Inner::Unsupported => BacktraceStatus::Unsupported,
            Inner::Disabled => BacktraceStatus::Disabled,
            Inner::Captured(_) => BacktraceStatus::Captured,
        }
    }
}

impl fmt::Display for Backtrace {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut capture = match &self.inner {
            Inner::Unsupported => return fmt.write_str("unsupported backtrace"),
            Inner::Disabled => return fmt.write_str("disabled backtrace"),
            Inner::Captured(c) => c.lock().unwrap(),
        };
        capture.resolve();

        let full = fmt.alternate();
        let (frames, style) = if full {
            (&capture.frames[..], backtrace::PrintFmt::Full)
        } else {
            (&capture.frames[capture.actual_start..], backtrace::PrintFmt::Short)
        };

        // When printing paths we try to strip the cwd if it exists, otherwise
        // we just print the path as-is. Note that we also only do this for the
        // short format, because if it's full we presumably want to print
        // everything.
        // let cwd = crate::env::current_dir();
        let mut print_path = move |fmt: &mut fmt::Formatter<'_>, path: BytesOrWideString<'_>| {
            output_filename(fmt, path, style, None)
        };

        let mut f = backtrace::BacktraceFmt::new(fmt, style, &mut print_path);
        f.add_context()?;
        for frame in frames {
            let mut f = f.frame();
            if frame.symbols.is_empty() {
                f.print_raw(frame.frame.ip(), None, None, None)?;
            } else {
                for symbol in frame.symbols.iter() {
                    f.print_raw(
                        frame.frame.ip(),
                        symbol.name.as_ref().map(|b| SymbolName::new(b)),
                        symbol.filename.as_ref().map(|b| match b {
                            BytesOrWide::Bytes(w) => BytesOrWideString::Bytes(w),
                            BytesOrWide::Wide(w) => BytesOrWideString::Wide(w),
                        }),
                        symbol.lineno,
                    )?;
                }
            }
        }
        f.finish()?;
        Ok(())
    }
}

impl Capture {
    fn resolve(&mut self) {
        // If we're already resolved, nothing to do!
        if self.resolved {
            return;
        }
        self.resolved = true;

        // Use the global backtrace lock to synchronize this as it's a
        // requirement of the `backtrace` crate, and then actually resolve
        // everything.
        let _lock = lock();
        for frame in self.frames.iter_mut() {
            let symbols = &mut frame.symbols;
            let frame = match &frame.frame {
                RawFrame::Actual(frame) => frame,
                #[cfg(test)]
                RawFrame::Fake => unimplemented!(),
            };
            unsafe {
                resolve_frame_unsynchronized(frame, |symbol| {
                    symbols.push(BacktraceSymbol {
                        name: symbol.name().map(|m| m.as_bytes().to_vec()),
                        filename: symbol.filename_raw().map(|b| match b {
                            BytesOrWideString::Bytes(b) => BytesOrWide::Bytes(b.to_owned()),
                            BytesOrWideString::Wide(b) => BytesOrWide::Wide(b.to_owned()),
                        }),
                        lineno: symbol.lineno(),
                    });
                });
            }
        }
    }
}

impl RawFrame {
    fn ip(&self) -> *mut c_void {
        match self {
            RawFrame::Actual(frame) => frame.ip(),
            #[cfg(test)]
            RawFrame::Fake => 1 as *mut c_void,
        }
    }
}

/// Enable backtrace for dumping call stack on crash.
///
/// Enabling backtrace here makes the panic information along with call stack
/// information. Very similar to normal `RUST_BACKTRACE=1` flag in Rust.
///
/// * `path` - The path of enclave's file image. This is essential for dumping
/// symbol information on crash point.
/// * Always return `Ok(())`
pub fn enable_backtrace<P: AsRef<Path>>(path: P, format: PrintFormat) -> io::Result<()> {
    let _ = fs::metadata(&path)?;
    enclave::set_enclave_path(path)?;
    set_enabled(format);
    Ok(())
}
