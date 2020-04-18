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

//! Operations on ASCII strings and characters.
//!
//! Most string operations in Rust act on UTF-8 strings. However, at times it
//! makes more sense to only consider the ASCII character set for a specific
//! operation.
//!
//! The [`AsciiExt`] trait provides methods that allow for character
//! operations that only act on the ASCII subset and leave non-ASCII characters
//! alone.
//!
//! The [`escape_default`] function provides an iterator over the bytes of an
//! escaped version of the character given.
//!
//! [`AsciiExt`]: trait.AsciiExt.html
//! [`escape_default`]: fn.escape_default.html

use alloc::vec::Vec;
use alloc::string::String;

pub use core::ascii::{escape_default, EscapeDefault};

/// Extension methods for ASCII-subset only operations.
///
/// Be aware that operations on seemingly non-ASCII characters can sometimes
/// have unexpected results. Consider this example:
///
pub trait AsciiExt {
    /// Container type for copied ASCII characters.
    type Owned;

    /// Checks if the value is within the ASCII range.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii(&self) -> bool;

    /// Makes a copy of the value in its ASCII upper case equivalent.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To uppercase the value in-place, use [`make_ascii_uppercase`].
    ///
    /// To uppercase ASCII characters in addition to non-ASCII characters, use
    /// [`str::to_uppercase`].
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    ///
    #[allow(deprecated)]
    fn to_ascii_uppercase(&self) -> Self::Owned;

    /// Makes a copy of the value in its ASCII lower case equivalent.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To lowercase the value in-place, use [`make_ascii_lowercase`].
    ///
    /// To lowercase ASCII characters in addition to non-ASCII characters, use
    /// [`str::to_lowercase`].
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    ///
    #[allow(deprecated)]
    fn to_ascii_lowercase(&self) -> Self::Owned;

    /// Checks that two values are an ASCII case-insensitive match.
    ///
    /// Same as `to_ascii_lowercase(a) == to_ascii_lowercase(b)`,
    /// but without allocating and copying temporaries.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn eq_ignore_ascii_case(&self, other: &Self) -> bool;

    /// Converts this type to its ASCII upper case equivalent in-place.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To return a new uppercased value without modifying the existing one, use
    /// [`to_ascii_uppercase`].
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    ///
    fn make_ascii_uppercase(&mut self);

    /// Converts this type to its ASCII lower case equivalent in-place.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To return a new lowercased value without modifying the existing one, use
    /// [`to_ascii_lowercase`].
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    ///
    fn make_ascii_lowercase(&mut self);
}

macro_rules! delegating_ascii_methods {
    () => {
        #[inline]
        fn is_ascii(&self) -> bool { self.is_ascii() }

        #[inline]
        fn to_ascii_uppercase(&self) -> Self::Owned { self.to_ascii_uppercase() }

        #[inline]
        fn to_ascii_lowercase(&self) -> Self::Owned { self.to_ascii_lowercase() }

        #[inline]
        fn eq_ignore_ascii_case(&self, o: &Self) -> bool { self.eq_ignore_ascii_case(o) }

        #[inline]
        fn make_ascii_uppercase(&mut self) { self.make_ascii_uppercase(); }

        #[inline]
        fn make_ascii_lowercase(&mut self) { self.make_ascii_lowercase(); }
    }
}

#[allow(deprecated)]
impl AsciiExt for u8 {
    type Owned = u8;

    delegating_ascii_methods!();
}

#[allow(deprecated)]
impl AsciiExt for char {
    type Owned = char;

    delegating_ascii_methods!();
}

#[allow(deprecated)]
impl AsciiExt for [u8] {
    type Owned = Vec<u8>;

    delegating_ascii_methods!();
}

#[allow(deprecated)]
impl AsciiExt for str {
    type Owned = String;

    delegating_ascii_methods!();
}