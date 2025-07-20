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
// under the License.

use crate::ffi::c_void;
use crate::fmt;
use crate::str;
use crate::sys::backtrace::BytesOrWideString;
use crate::sys::backtrace::Frame;

use sgx_demangle::{try_demangle, Demangle};

pub enum ResolveWhat<'a> {
    Address(*mut c_void),
    Frame(&'a Frame),
}

impl<'a> ResolveWhat<'a> {
    #[allow(dead_code)]
    fn address_or_ip(&self) -> *mut c_void {
        match self {
            ResolveWhat::Address(a) => adjust_ip(*a),
            ResolveWhat::Frame(f) => adjust_ip(f.ip()),
        }
    }
}

// IP values from stack frames are typically (always?) the instruction
// *after* the call that's the actual stack trace. Symbolizing this on
// causes the filename/line number to be one ahead and perhaps into
// the void if it's near the end of the function.
//
// This appears to basically always be the case on all platforms, so we always
// subtract one from a resolved ip to resolve it to the previous call
// instruction instead of the instruction being returned to.
//
// Ideally we would not do this. Ideally we would require callers of the
// `resolve` APIs here to manually do the -1 and account that they want location
// information for the *previous* instruction, not the current. Ideally we'd
// also expose on `Frame` if we are indeed the address of the next instruction
// or the current.
//
// For now though this is a pretty niche concern so we just internally always
// subtract one. Consumers should keep working and getting pretty good results,
// so we should be good enough.
fn adjust_ip(a: *mut c_void) -> *mut c_void {
    if a.is_null() {
        a
    } else {
        (a as usize - 1) as *mut c_void
    }
}

/// Same as `resolve`, only unsafe as it's unsynchronized.
///
/// This function does not have synchronization guarentees but is available when
/// the `std` feature of this crate isn't compiled in. See the `resolve`
/// function for more documentation and examples.
///
/// # Panics
///
/// See information on `resolve` for caveats on `cb` panicking.
pub unsafe fn resolve_unsynchronized<F>(addr: *mut c_void, mut cb: F)
where
    F: FnMut(&Symbol),
{
    resolve_imp(ResolveWhat::Address(addr), &mut cb)
}

/// Same as `resolve_frame`, only unsafe as it's unsynchronized.
///
/// This function does not have synchronization guarentees but is available
/// when the `std` feature of this crate isn't compiled in. See the
/// `resolve_frame` function for more documentation and examples.
///
/// # Panics
///
/// See information on `resolve_frame` for caveats on `cb` panicking.
pub unsafe fn resolve_frame_unsynchronized<F>(frame: &Frame, mut cb: F)
where
    F: FnMut(&Symbol),
{
    resolve_imp(ResolveWhat::Frame(frame), &mut cb)
}

/// A trait representing the resolution of a symbol in a file.
///
/// This trait is yielded as a trait object to the closure given to the
/// `backtrace::resolve` function, and it is virtually dispatched as it's
/// unknown which implementation is behind it.
///
/// A symbol can give contextual information about a function, for example the
/// name, filename, line number, precise address, etc. Not all information is
/// always available in a symbol, however, so all methods return an `Option`.
pub struct Symbol {
    // TODO: this lifetime bound needs to be persisted eventually to `Symbol`,
    // but that's currently a breaking change. For now this is safe since
    // `Symbol` is only ever handed out by reference and can't be cloned.
    name: Option<Vec<u8>>,
    addr: Option<*mut c_void>,
    filename: Option<Vec<u8>>,
    lineno: Option<u32>,
    colno: Option<u32>,
}

impl Symbol {
    /// Returns the name of this function.
    ///
    /// The returned structure can be used to query various properties about the
    /// symbol name:
    ///
    /// * The `Display` implementation will print out the demangled symbol.
    /// * The raw `str` value of the symbol can be accessed (if it's valid
    ///   utf-8).
    /// * The raw bytes for the symbol name can be accessed.
    pub fn name(&self) -> Option<SymbolName<'_>> {
        self.name.as_ref().map(|m| SymbolName::new(m.as_slice()))
    }

    /// Returns the starting address of this function.
    pub fn addr(&self) -> Option<*mut c_void> {
        self.addr
    }

    /// Returns the raw filename as a slice. This is mainly useful for `no_std`
    /// environments.
    pub fn filename_raw(&self) -> Option<BytesOrWideString<'_>> {
        self.filename
            .as_ref()
            .map(|f| BytesOrWideString::Bytes(f.as_slice()))
    }

    /// Returns the column number for where this symbol is currently executing.
    ///
    /// Only gimli currently provides a value here and even then only if `filename`
    /// returns `Some`, and so it is then consequently subject to similar caveats.
    pub fn colno(&self) -> Option<u32> {
        self.colno
    }

    /// Returns the line number for where this symbol is currently executing.
    ///
    /// This return value is typically `Some` if `filename` returns `Some`, and
    /// is consequently subject to similar caveats.
    pub fn lineno(&self) -> Option<u32> {
        self.lineno
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Symbol");
        if let Some(name) = self.name() {
            d.field("name", &name);
        }
        if let Some(addr) = self.addr() {
            d.field("addr", &addr);
        }

        #[cfg(feature = "std")]
        {
            if let Some(filename) = self.filename() {
                d.field("filename", &filename);
            }
        }

        if let Some(lineno) = self.lineno() {
            d.field("lineno", &lineno);
        }
        if let Some(colno) = self.colno() {
            d.field("colno", &colno);
        }
        d.finish()
    }
}

/// A wrapper around a symbol name to provide ergonomic accessors to the
/// demangled name, the raw bytes, the raw string, etc.
// Allow dead code for when the `cpp_demangle` feature is not enabled.
#[allow(dead_code)]
pub struct SymbolName<'a> {
    bytes: &'a [u8],
    demangled: Option<Demangle<'a>>,
}

impl<'a> SymbolName<'a> {
    /// Creates a new symbol name from the raw underlying bytes.
    pub fn new(bytes: &'a [u8]) -> SymbolName<'a> {
        let str_bytes = str::from_utf8(bytes).ok();
        let demangled = str_bytes.and_then(|s| try_demangle(s).ok());

        SymbolName {
            bytes,
            demangled,
        }
    }

    /// Returns the raw (mangled) symbol name as a `str` if the symbol is valid utf-8.
    ///
    /// Use the `Display` implementation if you want the demangled version.
    pub fn as_str(&self) -> Option<&'a str> {
        self.demangled
            .as_ref()
            .map(|s| s.as_str())
            .or_else(|| str::from_utf8(self.bytes).ok())
    }

    /// Returns the raw symbol name as a list of bytes
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }
}

fn format_symbol_name(
    fmt: fn(&str, &mut fmt::Formatter<'_>) -> fmt::Result,
    mut bytes: &[u8],
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    while !bytes.is_empty() {
        match str::from_utf8(bytes) {
            Ok(name) => {
                fmt(name, f)?;
                break;
            }
            Err(err) => {
                fmt("\u{FFFD}", f)?;

                match err.error_len() {
                    Some(len) => bytes = &bytes[err.valid_up_to() + len..],
                    None => break,
                }
            }
        }
    }
    Ok(())
}

impl<'a> fmt::Display for SymbolName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref s) = self.demangled {
            s.fmt(f)
        } else {
            format_symbol_name(fmt::Display::fmt, self.bytes, f)
        }
    }
}

impl<'a> fmt::Debug for SymbolName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref s) = self.demangled {
            s.fmt(f)
        } else {
            format_symbol_name(fmt::Debug::fmt, self.bytes, f)
        }
    }
}

mod libbacktrace;
use self::libbacktrace::resolve as resolve_imp;
