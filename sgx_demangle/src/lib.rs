//! Demangle Rust compiler symbol names.
//!
//! This crate provides a `demangle` function which will return a `Demangle`
//! sentinel value that can be used to learn about the demangled version of a
//! symbol name. The demangled representation will be the same as the original
//! if it doesn't look like a mangled symbol name.
//!
//! `Demangle` can be formatted with the `Display` trait. The alternate
//! modifier (`#`) can be used to format the symbol name without the
//! trailing hash value.
//!
//! # Examples
//!
//! ```
//! use rustc_demangle::demangle;
//!
//! assert_eq!(demangle("_ZN4testE").to_string(), "test");
//! assert_eq!(demangle("_ZN3foo3barE").to_string(), "foo::bar");
//! assert_eq!(demangle("foo").to_string(), "foo");
//! // With hash
//! assert_eq!(format!("{}", demangle("_ZN3foo17h05af221e174051e9E")), "foo::h05af221e174051e9");
//! // Without hash
//! assert_eq!(format!("{:#}", demangle("_ZN3foo17h05af221e174051e9E")), "foo");
//! ```

#![no_std]

#![cfg_attr(target_env = "sgx", feature(rustc_private))]

mod legacy;
mod v0;

use core::fmt;

/// Representation of a demangled symbol name.
pub struct Demangle<'a> {
    style: Option<DemangleStyle<'a>>,
    original: &'a str,
    suffix: &'a str,
}

enum DemangleStyle<'a> {
    Legacy(legacy::Demangle<'a>),
    V0(v0::Demangle<'a>),
}

/// De-mangles a Rust symbol into a more readable version
///
/// This function will take a **mangled** symbol and return a value. When printed,
/// the de-mangled version will be written. If the symbol does not look like
/// a mangled symbol, the original value will be written instead.
///
/// # Examples
///
/// ```
/// use rustc_demangle::demangle;
///
/// assert_eq!(demangle("_ZN4testE").to_string(), "test");
/// assert_eq!(demangle("_ZN3foo3barE").to_string(), "foo::bar");
/// assert_eq!(demangle("foo").to_string(), "foo");
/// ```
pub fn demangle(mut s: &str) -> Demangle {
    // During ThinLTO LLVM may import and rename internal symbols, so strip out
    // those endings first as they're one of the last manglings applied to symbol
    // names.
    let llvm = ".llvm.";
    if let Some(i) = s.find(llvm) {
        let candidate = &s[i + llvm.len()..];
        let all_hex = candidate.chars().all(|c| {
            match c {
                'A' ..= 'F' | '0' ..= '9' | '@' => true,
                _ => false,
            }
        });

        if all_hex {
            s = &s[..i];
        }
    }

    // Output like LLVM IR adds extra period-delimited words. See if
    // we are in that case and save the trailing words if so.
    let mut suffix = "";
    if let Some(i) = s.rfind("E.") {
        let (head, tail) = s.split_at(i + 1); // After the E, before the period

        if is_symbol_like(tail) {
            s = head;
            suffix = tail;
        }
    }

    let style = match legacy::demangle(s) {
        Ok(d) => Some(DemangleStyle::Legacy(d)),
        Err(()) => match v0::demangle(s) {
            Ok(d) => Some(DemangleStyle::V0(d)),
            Err(v0::Invalid) => None,
        },
    };
    Demangle {
        style: style,
        original: s,
        suffix: suffix,
    }
}

/// Error returned from the `try_demangle` function below when demangling fails.
#[derive(Debug, Clone)]
pub struct TryDemangleError {
    _priv: (),
}

/// The same as `demangle`, except return an `Err` if the string does not appear
/// to be a Rust symbol, rather than "demangling" the given string as a no-op.
///
/// ```
/// extern crate rustc_demangle;
///
/// let not_a_rust_symbol = "la la la";
///
/// // The `try_demangle` function will reject strings which are not Rust symbols.
/// assert!(rustc_demangle::try_demangle(not_a_rust_symbol).is_err());
///
/// // While `demangle` will just pass the non-symbol through as a no-op.
/// assert_eq!(rustc_demangle::demangle(not_a_rust_symbol).as_str(), not_a_rust_symbol);
/// ```
pub fn try_demangle(s: &str) -> Result<Demangle, TryDemangleError> {
    let sym = demangle(s);
    if sym.style.is_some() {
        Ok(sym)
    } else {
        Err(TryDemangleError { _priv: () })
    }
}

impl<'a> Demangle<'a> {
    /// Returns the underlying string that's being demangled.
    pub fn as_str(&self) -> &'a str {
        self.original
    }
}

fn is_symbol_like(s: &str) -> bool {
    s.chars().all(|c| {
        // Once `char::is_ascii_punctuation` and `char::is_ascii_alphanumeric`
        // have been stable for long enough, use those instead for clarity
        is_ascii_alphanumeric(c) || is_ascii_punctuation(c)
    })
}

// Copied from the documentation of `char::is_ascii_alphanumeric`
fn is_ascii_alphanumeric(c: char) -> bool {
    match c {
        '\u{0041}' ..= '\u{005A}' |
        '\u{0061}' ..= '\u{007A}' |
        '\u{0030}' ..= '\u{0039}' => true,
        _ => false,
    }
}

// Copied from the documentation of `char::is_ascii_punctuation`
fn is_ascii_punctuation(c: char) -> bool {
    match c {
        '\u{0021}' ..= '\u{002F}' |
        '\u{003A}' ..= '\u{0040}' |
        '\u{005B}' ..= '\u{0060}' |
        '\u{007B}' ..= '\u{007E}' => true,
        _ => false,
    }
}

impl<'a> fmt::Display for Demangle<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.style {
            None => f.write_str(self.original)?,
            Some(DemangleStyle::Legacy(ref d)) => {
                fmt::Display::fmt(d, f)?
            }
            Some(DemangleStyle::V0(ref d)) => {
                fmt::Display::fmt(d, f)?
            }
        }
        f.write_str(self.suffix)
    }
}

impl<'a> fmt::Debug for Demangle<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
