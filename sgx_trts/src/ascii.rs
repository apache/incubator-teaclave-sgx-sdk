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

pub use core::ascii::{EscapeDefault, escape_default};

/// Extension methods for ASCII-subset only operations.
///
/// Be aware that operations on seemingly non-ASCII characters can sometimes
/// have unexpected results.
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

    /// Checks if the value is an ASCII alphabetic character:
    /// U+0041 'A' ... U+005A 'Z' or U+0061 'a' ... U+007A 'z'.
    /// For strings, true if all characters in the string are
    /// ASCII alphabetic.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii_alphabetic(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII uppercase character:
    /// U+0041 'A' ... U+005A 'Z'.
    /// For strings, true if all characters in the string are
    /// ASCII uppercase.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii_uppercase(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII lowercase character:
    /// U+0061 'a' ... U+007A 'z'.
    /// For strings, true if all characters in the string are
    /// ASCII lowercase.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii_lowercase(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII alphanumeric character:
    /// U+0041 'A' ... U+005A 'Z', U+0061 'a' ... U+007A 'z', or
    /// U+0030 '0' ... U+0039 '9'.
    /// For strings, true if all characters in the string are
    /// ASCII alphanumeric.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii_alphanumeric(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII decimal digit:
    /// U+0030 '0' ... U+0039 '9'.
    /// For strings, true if all characters in the string are
    /// ASCII digits.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii_digit(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII hexadecimal digit:
    /// U+0030 '0' ... U+0039 '9', U+0041 'A' ... U+0046 'F', or
    /// U+0061 'a' ... U+0066 'f'.
    /// For strings, true if all characters in the string are
    /// ASCII hex digits.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii_hexdigit(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII punctuation character:
    ///
    /// U+0021 ... U+002F `! " # $ % & ' ( ) * + , - . /`
    /// U+003A ... U+0040 `: ; < = > ? @`
    /// U+005B ... U+0060 ``[ \\ ] ^ _ ` ``
    /// U+007B ... U+007E `{ | } ~`
    ///
    /// For strings, true if all characters in the string are
    /// ASCII punctuation.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii_punctuation(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII graphic character:
    /// U+0021 '!' ... U+007E '~'.
    /// For strings, true if all characters in the string are
    /// ASCII graphic characters.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii_graphic(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII whitespace character:
    /// U+0020 SPACE, U+0009 HORIZONTAL TAB, U+000A LINE FEED,
    /// U+000C FORM FEED, or U+000D CARRIAGE RETURN.
    /// For strings, true if all characters in the string are
    /// ASCII whitespace.
    ///
    /// Rust uses the WhatWG Infra Standard's [definition of ASCII
    /// whitespace][infra-aw].  There are several other definitions in
    /// wide use.  For instance, [the POSIX locale][pct] includes
    /// U+000B VERTICAL TAB as well as all the above characters,
    /// but—from the very same specification—[the default rule for
    /// "field splitting" in the Bourne shell][bfs] considers *only*
    /// SPACE, HORIZONTAL TAB, and LINE FEED as whitespace.
    ///
    /// If you are writing a program that will process an existing
    /// file format, check what that format's definition of whitespace is
    /// before using this function.
    ///
    /// [infra-aw]: https://infra.spec.whatwg.org/#ascii-whitespace
    /// [pct]: http://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap07.html#tag_07_03_01
    /// [bfs]: http://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_05
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii_whitespace(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII control character:
    /// U+0000 NUL ... U+001F UNIT SEPARATOR, or U+007F DELETE.
    /// Note that most ASCII whitespace characters are control
    /// characters, but SPACE is not.
    ///
    /// # Note
    ///
    /// This method will be deprecated in favor of the identically-named
    /// inherent methods on `u8`, `char`, `[u8]` and `str`.
    fn is_ascii_control(&self) -> bool { unimplemented!(); }
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

macro_rules! delegating_ascii_ctype_methods {
    () => {
        #[inline]
        fn is_ascii_alphabetic(&self) -> bool { self.is_ascii_alphabetic() }

        #[inline]
        fn is_ascii_uppercase(&self) -> bool { self.is_ascii_uppercase() }

        #[inline]
        fn is_ascii_lowercase(&self) -> bool { self.is_ascii_lowercase() }

        #[inline]
        fn is_ascii_alphanumeric(&self) -> bool { self.is_ascii_alphanumeric() }

        #[inline]
        fn is_ascii_digit(&self) -> bool { self.is_ascii_digit() }

        #[inline]
        fn is_ascii_hexdigit(&self) -> bool { self.is_ascii_hexdigit() }

        #[inline]
        fn is_ascii_punctuation(&self) -> bool { self.is_ascii_punctuation() }

        #[inline]
        fn is_ascii_graphic(&self) -> bool { self.is_ascii_graphic() }

        #[inline]
        fn is_ascii_whitespace(&self) -> bool { self.is_ascii_whitespace() }

        #[inline]
        fn is_ascii_control(&self) -> bool { self.is_ascii_control() }
    }
}

#[allow(deprecated)]
impl AsciiExt for u8 {
    type Owned = u8;

    delegating_ascii_methods!();
    delegating_ascii_ctype_methods!();
}

#[allow(deprecated)]
impl AsciiExt for char {
    type Owned = char;

    delegating_ascii_methods!();
    delegating_ascii_ctype_methods!();
}

#[allow(deprecated)]
impl AsciiExt for [u8] {
    type Owned = Vec<u8>;

    delegating_ascii_methods!();

    #[inline]
    fn is_ascii_alphabetic(&self) -> bool {
        self.iter().all(|b| b.is_ascii_alphabetic())
    }

    #[inline]
    fn is_ascii_uppercase(&self) -> bool {
        self.iter().all(|b| b.is_ascii_uppercase())
    }

    #[inline]
    fn is_ascii_lowercase(&self) -> bool {
        self.iter().all(|b| b.is_ascii_lowercase())
    }

    #[inline]
    fn is_ascii_alphanumeric(&self) -> bool {
        self.iter().all(|b| b.is_ascii_alphanumeric())
    }

    #[inline]
    fn is_ascii_digit(&self) -> bool {
        self.iter().all(|b| b.is_ascii_digit())
    }

    #[inline]
    fn is_ascii_hexdigit(&self) -> bool {
        self.iter().all(|b| b.is_ascii_hexdigit())
    }

    #[inline]
    fn is_ascii_punctuation(&self) -> bool {
        self.iter().all(|b| b.is_ascii_punctuation())
    }

    #[inline]
    fn is_ascii_graphic(&self) -> bool {
        self.iter().all(|b| b.is_ascii_graphic())
    }

    #[inline]
    fn is_ascii_whitespace(&self) -> bool {
        self.iter().all(|b| b.is_ascii_whitespace())
    }

    #[inline]
    fn is_ascii_control(&self) -> bool {
        self.iter().all(|b| b.is_ascii_control())
    }
}

#[allow(deprecated)]
impl AsciiExt for str {
    type Owned = String;

    delegating_ascii_methods!();

    #[inline]
    fn is_ascii_alphabetic(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_alphabetic())
    }

    #[inline]
    fn is_ascii_uppercase(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_uppercase())
    }

    #[inline]
    fn is_ascii_lowercase(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_lowercase())
    }

    #[inline]
    fn is_ascii_alphanumeric(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_alphanumeric())
    }

    #[inline]
    fn is_ascii_digit(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_digit())
    }

    #[inline]
    fn is_ascii_hexdigit(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_hexdigit())
    }

    #[inline]
    fn is_ascii_punctuation(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_punctuation())
    }

    #[inline]
    fn is_ascii_graphic(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_graphic())
    }

    #[inline]
    fn is_ascii_whitespace(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_whitespace())
    }

    #[inline]
    fn is_ascii_control(&self) -> bool {
        self.bytes().all(|b| b.is_ascii_control())
    }
}