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
//! ## Accuracy
//!
//! Backtraces are attempted to be as accurate as possible, but no guarantees
//! are provided about the exact accuracy of a backtrace. Instruction pointers,
//! symbol names, filenames, line numbers, etc, may all be incorrect when
//! reported. Accuracy is attempted on a best-effort basis, however, any bug
//! reports are always welcome to indicate areas of improvement!
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
//! Sometimes libraries might not be available at runtime or something may go
//! wrong which would cause a backtrace to not be captured. Please feel free to
//! report issues with platforms where a backtrace cannot be captured though!
//!

// NB: A note on resolution of a backtrace:
//
// Backtraces primarily happen in two steps, one is where we actually capture
// the stack backtrace, giving us a list of instruction pointers corresponding
// to stack frames. Next we take these instruction pointers and, one-by-one,
// turn them into a human readable name (like `main`).
//
// The first phase can be somewhat expensive (walking the stack), especially
// on MSVC where debug information is consulted to return inline frames each as
// their own frame. The second phase, however, is almost always extremely
// expensive (on the order of milliseconds sometimes) when it's consulting debug
// information.
//
// We attempt to amortize this cost as much as possible by delaying resolution
// of an address to a human readable name for as long as possible. When
// `Backtrace::create` is called to capture a backtrace it doesn't actually
// perform any symbol resolution, but rather we lazily resolve symbols only just
// before they're needed for printing. This way we can make capturing a
// backtrace and throwing it away much cheaper, but actually printing a
// backtrace is still basically the same cost.
//
// This strategy comes at the cost of some synchronization required inside of a
// `Backtrace`, but that's a relatively small price to pay relative to capturing
// a backtrace or actually symbolizing it.

pub use crate::sys_common::backtrace::__rust_begin_short_backtrace;

use crate::cell::UnsafeCell;
use crate::ffi::c_void;
use crate::fmt;
use crate::fs;
use crate::enclave;
use crate::io;
use crate::panic::{BacktraceStyle, get_backtrace_style, set_backtrace_style};
use crate::path::Path;
use crate::sync::Once;
use crate::sys::backtrace::{self, BytesOrWideString};
use crate::sys_common::backtrace::{
    lock,
    output_filename,
    resolve_frame_unsynchronized,
    SymbolName,
};
use crate::vec::Vec;

/// A captured OS thread stack backtrace.
///
/// This type represents a stack backtrace for an OS thread captured at a
/// previous point in time. In some instances the `Backtrace` type may
/// internally be empty due to configuration. For more information see
/// `Backtrace::capture`.
#[must_use]
pub struct Backtrace {
    inner: Inner,
}

/// The current status of a backtrace, indicating whether it was captured or
/// whether it is empty for some other reason.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum BacktraceStatus {
    /// Capturing a backtrace is not supported, likely because it's not
    /// implemented for the current platform.
    Unsupported,
    /// Capturing a backtrace has been disabled through either the
    /// `RUST_LIB_BACKTRACE` or `RUST_BACKTRACE` environment variables.
    Disabled,
    /// A backtrace has been captured and the `Backtrace` should print
    /// reasonable information when rendered.
    Captured,
}

enum Inner {
    Unsupported,
    Disabled,
    Captured(LazilyResolvedCapture),
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

/// A single frame of a backtrace.
pub struct BacktraceFrame {
    frame: RawFrame,
    symbols: Vec<BacktraceSymbol>,
}

#[derive(Debug)]
enum RawFrame {
    Actual(backtrace::Frame),
}

struct BacktraceSymbol {
    name: Option<Vec<u8>>,
    filename: Option<BytesOrWide>,
    lineno: Option<u32>,
    colno: Option<u32>,
}

enum BytesOrWide {
    Bytes(Vec<u8>),
    Wide(Vec<u16>),
}

impl fmt::Debug for Backtrace {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let capture = match &self.inner {
            Inner::Unsupported => return fmt.write_str("<unsupported>"),
            Inner::Disabled => return fmt.write_str("<disabled>"),
            Inner::Captured(c) => c.force(),
        };

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

impl fmt::Debug for BacktraceFrame {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = fmt.debug_list();
        dbg.entries(&self.symbols);
        dbg.finish()
    }
}

impl fmt::Debug for BacktraceSymbol {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // FIXME: improve formatting: https://github.com/rust-lang/rust/issues/65280
        // FIXME: Also, include column numbers into the debug format as Display already has them.
        // Until there are stable per-frame accessors, the format shouldn't be changed:
        // https://github.com/rust-lang/rust/issues/65280#issuecomment-638966585
        write!(fmt, "{{ ")?;

        if let Some(fn_name) = self.name.as_ref().map(|b| SymbolName::new(b)) {
            write!(fmt, "fn: \"{:#}\"", fn_name)?;
        } else {
            write!(fmt, "fn: <unknown>")?;
        }

        if let Some(fname) = self.filename.as_ref() {
            write!(fmt, ", file: \"{:?}\"", fname)?;
        }

        if let Some(line) = self.lineno {
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
    /// Returns whether backtrace captures are enabled
    fn enabled() -> bool {
        match get_backtrace_style() {
            Some(style) => {
                match style {
                    BacktraceStyle::Full => true,
                    BacktraceStyle::Short => true,
                    BacktraceStyle::Off => false,
                }
            }
            None => false,
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

    /// Forcibly captures a full backtrace, regardless of environment variable
    /// configuration.
    ///
    /// This function behaves the same as `capture` except that it ignores the
    /// values of the `RUST_BACKTRACE` and `RUST_LIB_BACKTRACE` environment
    /// variables, always capturing a backtrace.
    ///
    /// Note that capturing a backtrace can be an expensive operation on some
    /// platforms, so this should be used with caution in performance-sensitive
    /// parts of code.
    #[inline(never)] // want to make sure there's a frame here to remove
    pub fn force_capture() -> Backtrace {
        if !Backtrace::enabled() {
            return Backtrace { inner: Inner::Disabled };
        }
        Backtrace::create(Backtrace::force_capture as usize)
    }

    /// Forcibly captures a disabled backtrace, regardless of environment
    /// variable configuration.
    pub const fn disabled() -> Backtrace {
        Backtrace { inner: Inner::Disabled }
    }

    // Capture a backtrace which start just before the function addressed by
    // `ip`
    fn create(ip: usize) -> Backtrace {
        // SAFETY: We don't attempt to lock this reentrantly.
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
            Inner::Captured(LazilyResolvedCapture::new(Capture {
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
    #[must_use]
    pub fn status(&self) -> BacktraceStatus {
        match self.inner {
            Inner::Unsupported => BacktraceStatus::Unsupported,
            Inner::Disabled => BacktraceStatus::Disabled,
            Inner::Captured(_) => BacktraceStatus::Captured,
        }
    }
}

impl<'a> Backtrace {
    /// Returns an iterator over the backtrace frames.
    #[must_use]
    pub fn frames(&'a self) -> &'a [BacktraceFrame] {
        if let Inner::Captured(c) = &self.inner { &c.force().frames } else { &[] }
    }
}

impl fmt::Display for Backtrace {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let capture = match &self.inner {
            Inner::Unsupported => return fmt.write_str("unsupported backtrace"),
            Inner::Disabled => return fmt.write_str("disabled backtrace"),
            Inner::Captured(c) => c.force(),
        };

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
            if frame.symbols.is_empty() {
                f.frame().print_raw(frame.frame.ip(), None, None, None)?;
            } else {
                for symbol in frame.symbols.iter() {
                    f.frame().print_raw_with_column(
                        frame.frame.ip(),
                        symbol.name.as_ref().map(|b| SymbolName::new(b)),
                        symbol.filename.as_ref().map(|b| match b {
                            BytesOrWide::Bytes(w) => BytesOrWideString::Bytes(w),
                            BytesOrWide::Wide(w) => BytesOrWideString::Wide(w),
                        }),
                        symbol.lineno,
                        symbol.colno,
                    )?;
                }
            }
        }
        f.finish()?;
        Ok(())
    }
}

struct LazilyResolvedCapture {
    sync: Once,
    capture: UnsafeCell<Capture>,
}

impl LazilyResolvedCapture {
    fn new(capture: Capture) -> Self {
        LazilyResolvedCapture { sync: Once::new(), capture: UnsafeCell::new(capture) }
    }

    fn force(&self) -> &Capture {
        self.sync.call_once(|| {
            // SAFETY: This exclusive reference can't overlap with any others
            // `Once` guarantees callers will block until this closure returns
            // `Once` also guarantees only a single caller will enter this closure
            unsafe { &mut *self.capture.get() }.resolve();
        });

        // SAFETY: This shared reference can't overlap with the exclusive reference above
        unsafe { &*self.capture.get() }
    }
}

// SAFETY: Access to the inner value is synchronized using a thread-safe `Once`
// So long as `Capture` is `Sync`, `LazilyResolvedCapture` is too
unsafe impl Sync for LazilyResolvedCapture where Capture: Sync {}

impl Capture {
    #[allow(clippy::infallible_destructuring_match)]
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
            let RawFrame::Actual(frame) = &frame.frame;
            unsafe {
                resolve_frame_unsynchronized(frame, |symbol| {
                    symbols.push(BacktraceSymbol {
                        name: symbol.name().map(|m| m.as_bytes().to_vec()),
                        filename: symbol.filename_raw().map(|b| match b {
                            BytesOrWideString::Bytes(b) => BytesOrWide::Bytes(b.to_owned()),
                            BytesOrWideString::Wide(b) => BytesOrWide::Wide(b.to_owned()),
                        }),
                        lineno: symbol.lineno(),
                        colno: symbol.colno(),
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
        }
    }
}

/// Controls how the backtrace should be formatted.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrintFormat {
    /// Show only relevant data from the backtrace.
    Short,
    /// Show all the frames with absolute path for files.
    Full,
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
    let style = match format {
        PrintFormat::Short => BacktraceStyle::Short,
        PrintFormat::Full => BacktraceStyle::Full,
    };
    set_backtrace_style(style);
    Ok(())
}
