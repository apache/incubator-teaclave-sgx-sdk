// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! untrusted.rs: Safe, fast, zero-panic, zero-crashing, zero-allocation
//! parsing of untrusted inputs in Rust.
//!
//! <code>git clone https://github.com/briansmith/untrusted</code>
//!
//! untrusted.rs goes beyond Rust's normal safety guarantees by  also
//! guaranteeing that parsing will be panic-free, as long as
//! `untrusted::Input::as_slice_less_safe()` is not used. It avoids copying
//! data and heap allocation and strives to prevent common pitfalls such as
//! accidentally parsing input bytes multiple times. In order to meet these
//! goals, untrusted.rs is limited in functionality such that it works best for
//! input languages with a small fixed amount of lookahead such as ASN.1, TLS,
//! TCP/IP, and many other networking, IPC, and related protocols. Languages
//! that require more lookahead and/or backtracking require some significant
//! contortions to parse using this framework. It would not be realistic to use
//! it for parsing programming language code, for example.
//!
//! The overall pattern for using untrusted.rs is:
//!
//! 1. Write a recursive-descent-style parser for the input language, where the
//!    input data is given as a `&mut untrusted::Reader` parameter to each
//!    function. Each function should have a return type of `Result<V, E>` for
//!    some value type `V` and some error type `E`, either or both of which may
//!    be `()`. Functions for parsing the lowest-level language constructs
//!    should be defined. Those lowest-level functions will parse their inputs
//!    using `::read_byte()`, `Reader::peek()`, and similar functions.
//!    Higher-level language constructs are then parsed by calling the
//!    lower-level functions in sequence.
//!
//! 2. Wrap the top-most functions of your recursive-descent parser in
//!    functions that take their input data as an `untrusted::Input`. The
//!    wrapper functions should call the `Input`'s `read_all` (or a variant
//!    thereof) method. The wrapper functions are the only ones that should be
//!    exposed outside the parser's module.
//!
//! 3. After receiving the input data to parse, wrap it in an `untrusted::Input`
//!    using `untrusted::Input::from()` as early as possible. Pass the
//!    `untrusted::Input` to the wrapper functions when they need to be parsed.
//!
//! In general parsers built using `untrusted::Reader` do not need to explicitly
//! check for end-of-input unless they are parsing optional constructs, because
//! `Reader::read_byte()` will return `Err(EndOfInput)` on end-of-input.
//! Similarly, parsers using `untrusted::Reader` generally don't need to check
//! for extra junk at the end of the input as long as the parser's API uses the
//! pattern described above, as `read_all` and its variants automatically check
//! for trailing junk. `Reader::skip_to_end()` must be used when any remaining
//! unread input should be ignored without triggering an error.
//!
//! untrusted.rs works best when all processing of the input data is done
//! through the `untrusted::Input` and `untrusted::Reader` types. In
//! particular, avoid trying to parse input data using functions that take
//! byte slices. However, when you need to access a part of the input data as
//! a slice to use a function that isn't written using untrusted.rs,
//! `Input::as_slice_less_safe()` can be used.
//!
//! It is recommend to use `use untrusted;` and then `untrusted::Input`,
//! `untrusted::Reader`, etc., instead of using `use untrusted::*`. Qualifying
//! the names with `untrusted` helps remind the reader of the code that it is
//! dealing with *untrusted* input.
//!
//! # Examples
//!
//! [*ring*](https://github.com/briansmith/ring)'s parser for the subset of
//! ASN.1 DER it needs to understand,
//! [`ring::der`](https://github.com/briansmith/ring/blob/master/src/der.rs),
//! is built on top of untrusted.rs. *ring* also uses untrusted.rs to parse ECC
//! public keys, RSA PKCS#1 1.5 padding, and for all other parsing it does.
//!
//! All of [webpki](https://github.com/briansmith/webpki)'s parsing of X.509
//! certificates (also ASN.1 DER) is done using untrusted.rs.

#![doc(html_root_url="https://briansmith.org/rustdoc/")]

#![allow(
    missing_copy_implementations,
    missing_debug_implementations,
)]

// `#[derive(...)]` uses `#[allow(unused_qualifications)]` internally.
#![deny(
    unused_qualifications,
)]

#![forbid(
    anonymous_parameters,
    box_pointers,
    fat_ptr_transmutes,
    legacy_directory_ownership,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_results,
    variant_size_differences,
    warnings,
)]

#![no_std]

/// A wrapper around `&'a [u8]` that helps in writing panic-free code.
///
/// No methods of `Input` will ever panic.
#[derive(Clone, Copy, Debug, Eq)]
pub struct Input<'a> {
    value: no_panic::Slice<'a>
}

impl<'a> Input<'a> {
    /// Construct a new `Input` for the given input `bytes`.
    pub fn from(bytes: &'a [u8]) -> Input<'a> {
        // This limit is important for avoiding integer overflow. In particular,
        // `Reader` assumes that an `i + 1 > i` if `input.value.get(i)` does
        // not return `None`. According to the Rust language reference, the
        // maximum object size is `core::isize::MAX`, and in practice it is
        // impossible to create an object of size `core::usize::MAX` or larger.
        debug_assert!(bytes.len() < core::usize::MAX);
        Input { value: no_panic::Slice::new(bytes) }
    }

    /// Returns `true` if the input is empty and false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool { self.value.is_empty() }

    /// Returns an iterator over the input.
    #[inline]
    pub fn iter(&self) -> <&[u8] as IntoIterator>::IntoIter {
        self.value.into_iter()
    }

    /// Returns the length of the `Input`.
    #[inline]
    pub fn len(&self) -> usize { self.value.len() }

    /// Calls `read` with the given input as a `Reader`, ensuring that `read`
    /// consumed the entire input. If `read` does not consume the entire input,
    /// `incomplete_read` is returned.
    pub fn read_all<F, R, E>(&self, incomplete_read: E, read: F)
                             -> Result<R, E>
                             where F: FnOnce(&mut Reader<'a>) -> Result<R, E> {
        let mut input = Reader::new(*self);
        let result = try!(read(&mut input));
        if input.at_end() {
            Ok(result)
        } else {
            Err(incomplete_read)
        }
    }

    /// Like `read_all`, except taking an `FnMut`.
    pub fn read_all_mut<F, R, E>(&self, incomplete_read: E, mut read: F)
                                 -> Result<R, E>
                                 where F: FnMut(&mut Reader<'a>)
                                                -> Result<R, E> {
        let mut input = Reader::new(*self);
        let result = try!(read(&mut input));
        if input.at_end() {
            Ok(result)
        } else {
            Err(incomplete_read)
        }
    }

    /// Access the input as a slice so it can be processed by functions that
    /// are not written using the Input/Reader framework.
    #[inline]
    pub fn as_slice_less_safe(&self) -> &'a [u8] {
        self.value.as_slice_less_safe()
    }
}

// #[derive(PartialEq)] would result in lifetime bounds that are
// unnecessarily restrictive; see
// https://github.com/rust-lang/rust/issues/27950.
impl<'a, 'b> PartialEq<Input<'b>> for Input<'a> {
    #[inline]
    fn eq(&self, other: &Input<'b>) -> bool {
        self.as_slice_less_safe() == other.as_slice_less_safe()
    }
}

// https://github.com/rust-lang/rust/issues/27950
impl <'a, 'b> PartialEq<&'b [u8]> for Input<'a> {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_slice_less_safe() == *other
    }
}


/// Calls `read` with the given input as a `Reader`, ensuring that `read`
/// consumed the entire input. When `input` is `None`, `read` will be
/// called with `None`.
pub fn read_all_optional<'a, F, R, E>(input: Option<Input<'a>>,
                                      incomplete_read: E, read: F)
                                      -> Result<R, E>
                                      where F: FnOnce(Option<&mut Reader>)
                                                      -> Result<R, E> {
    match input {
        Some(input) => {
            let mut input = Reader::new(input);
            let result = try!(read(Some(&mut input)));
            if input.at_end() {
                Ok(result)
            } else {
                Err(incomplete_read)
            }
        },
        None => read(None)
    }
}


/// A read-only, forward-only* cursor into the data in an `Input`.
///
/// Using `Reader` to parse input helps to ensure that no byte of the input
/// will be accidentally processed more than once. Using `Reader` in
/// conjunction with `read_all`, `read_all_mut`, and `read_all_optional`
/// helps ensure that no byte of the input is accidentally left unprocessed.
/// The methods of `Reader` never panic, so `Reader` also assists the writing
/// of panic-free code.
///
/// \* `Reader` is not strictly forward-only because of the method
/// `get_input_between_marks`, which is provided mainly to support calculating
/// digests over parsed data.
#[derive(Debug)]
pub struct Reader<'a> {
    input: no_panic::Slice<'a>,
    i: usize
}

/// An index into the already-parsed input of a `Reader`.
pub struct Mark {
    i: usize
}

impl<'a> Reader<'a> {
    /// Construct a new Reader for the given input. Use `read_all`,
    /// `read_all_mut`, or `read_all_optional` instead of `Reader::new`
    /// whenever possible.
    #[inline]
    pub fn new(input: Input<'a>) -> Reader<'a> {
        Reader { input: input.value, i: 0 }
    }

    /// Returns `true` if the reader is at the end of the input, and `false`
    /// otherwise.
    #[inline]
    pub fn at_end(&self) -> bool { self.i == self.input.len() }

    /// Returns an `Input` for already-parsed input that has had its boundaries
    /// marked using `mark`.
    #[inline]
    pub fn get_input_between_marks(&self, mark1: Mark, mark2: Mark)
                                   -> Result<Input<'a>, EndOfInput> {
        self.input.get_slice(mark1.i..mark2.i)
                  .map(|subslice| Input { value: subslice })
                  .ok_or(EndOfInput)
    }

    /// Return the current position of the `Reader` for future use in a call
    /// to `get_input_between_marks`.
    #[inline]
    pub fn mark(&self) -> Mark { Mark { i: self.i } }

    /// Returns `true` if there is at least one more byte in the input and that
    /// byte is equal to `b`, and false otherwise.
    pub fn peek(&self, b: u8) -> bool {
        match self.input.get(self.i) {
            Some(actual_b) => return b == *actual_b,
            None => false
        }
    }

    /// Reads the next input byte.
    ///
    /// Returns `Ok(b)` where `b` is the next input byte, or `Err(EndOfInput)`
    /// if the `Reader` is at the end of the input.
    pub fn read_byte(&mut self) -> Result<u8, EndOfInput> {
        match self.input.get(self.i) {
            Some(b) => {
                self.i += 1; // safe from overflow; see Input::from().
                Ok(*b)
            }
            None => Err(EndOfInput)
        }
    }

    /// Skips `num_bytes` of the input.
    ///
    /// Returns `Ok(())` if there are at least `num_bytes` of input remaining,
    /// and `Err(EndOfInput)` otherwise.
    pub fn skip(&mut self, num_bytes: usize) -> Result<(), EndOfInput> {
        self.skip_and_get_input(num_bytes).map(|_| ())
    }

    /// Skips `num_bytes` of the input, returning the skipped input as an `Input`.
    ///
    /// Returns `Ok(i)` where `i` is an `Input` if there are at least
    /// `num_bytes` of input remaining, and `Err(EndOfInput)` otherwise.
    pub fn skip_and_get_input(&mut self, num_bytes: usize)
                              -> Result<Input<'a>, EndOfInput> {
        let new_i = try!(self.i.checked_add(num_bytes).ok_or(EndOfInput));
        let ret = self.input.get_slice(self.i..new_i)
                            .map(|subslice| Input { value: subslice })
                            .ok_or(EndOfInput);
        self.i = new_i;
        ret
    }

    /// Skips the reader to the end of the input, returning the skipped input
    /// as an `Input`.
    pub fn skip_to_end(&mut self) -> Input<'a> {
        let to_skip = self.input.len() - self.i;
        self.skip_and_get_input(to_skip).unwrap()
    }
}

/// The error type used to indicate the end of the input was reached before the
/// operation could be completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndOfInput;

mod no_panic {
    use core;

    /// A wrapper around a slice that exposes no functions that can panic.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Slice<'a> {
        bytes: &'a [u8]
    }

    impl<'a> Slice<'a> {
        #[inline]
        pub fn new(bytes: &'a [u8]) -> Slice<'a> {
            Slice { bytes }
        }

        #[inline]
        // TODO: https://github.com/rust-lang/rust/issues/35729#issuecomment-280872145
        //      pub fn get<I>(&self, i: I) -> Option<&I::Output>
        //          where I: core::slice::SliceIndex<u8>
        pub fn get(&self, i: usize) -> Option<&u8> { self.bytes.get(i) }

        // TODO: This will be replaced with `get()` once `get()` is made
        // generic over `SliceIndex`.
        #[inline]
        pub fn get_slice(&self, r: core::ops::Range<usize>)
                         -> Option<Slice<'a>> {
            self.bytes.get(r).map(|bytes| Slice { bytes })
        }

        #[inline]
        pub fn into_iter(&self) -> <&'a [u8] as IntoIterator>::IntoIter {
            self.bytes.into_iter()
        }

        #[inline]
        pub fn is_empty(&self) -> bool { self.bytes.is_empty() }

        #[inline]
        pub fn len(&self) -> usize { self.bytes.len() }

        #[inline]
        pub fn as_slice_less_safe(&self) -> &'a [u8] { self.bytes }
    }

} // mod no_panic

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_from() {
        let _ = Input::from(b"foo");
    }

    #[test]
    fn test_input_is_empty() {
        let input = Input::from(b"");
        assert!(input.is_empty());
        let input = Input::from(b"foo");
        assert!(!input.is_empty());
    }

    #[test]
    fn test_input_len() {
        let input = Input::from(b"foo");
        assert_eq!(input.len(), 3);
    }

    #[test]
    fn test_input_read_all() {
        let input = Input::from(b"foo");
        let result = input.read_all(EndOfInput, |input| {
            assert_eq!(b'f', try!(input.read_byte()));
            assert_eq!(b'o', try!(input.read_byte()));
            assert_eq!(b'o', try!(input.read_byte()));
            assert!(input.at_end());
            Ok(())
        });
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_input_read_all_unconsume() {
        let input = Input::from(b"foo");
        let result = input.read_all(EndOfInput, |input| {
            assert_eq!(b'f', try!(input.read_byte()));
            assert!(!input.at_end());
            Ok(())
        });
        assert_eq!(result, Err(EndOfInput));
    }

    #[test]
    fn test_input_as_slice_less_safe() {
        let slice = b"foo";
        let input = Input::from(slice);
        assert_eq!(input.as_slice_less_safe(), slice);
    }
}
