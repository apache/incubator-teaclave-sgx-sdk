// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

use core::ops::Range;
use core::iter::FusedIterator;
use core::str;
use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;

pub trait AsciiExt {
    /// Container type for copied ASCII characters.
    type Owned;

    /// Checks if the value is within the ASCII range.
    ///
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
    fn to_ascii_lowercase(&self) -> Self::Owned;

    /// Checks that two values are an ASCII case-insensitive match.
    ///
    /// Same as `to_ascii_lowercase(a) == to_ascii_lowercase(b)`,
    /// but without allocating and copying temporaries.
    ///
    fn eq_ignore_ascii_case(&self, other: &Self) -> bool;

    /// Converts this type to its ASCII upper case equivalent in-place.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To return a new uppercased value without modifying the existing one, use
    /// [`to_ascii_uppercase`].
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
    fn make_ascii_lowercase(&mut self);

    /// Checks if the value is an ASCII alphabetic character:
    /// U+0041 'A' ... U+005A 'Z' or U+0061 'a' ... U+007A 'z'.
    /// For strings, true if all characters in the string are
    /// ASCII alphabetic.
    ///
    fn is_ascii_alphabetic(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII uppercase character:
    /// U+0041 'A' ... U+005A 'Z'.
    /// For strings, true if all characters in the string are
    /// ASCII uppercase.
    ///
    fn is_ascii_uppercase(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII lowercase character:
    /// U+0061 'a' ... U+007A 'z'.
    /// For strings, true if all characters in the string are
    /// ASCII lowercase.
    ///
    fn is_ascii_lowercase(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII alphanumeric character:
    /// U+0041 'A' ... U+005A 'Z', U+0061 'a' ... U+007A 'z', or
    /// U+0030 '0' ... U+0039 '9'.
    /// For strings, true if all characters in the string are
    /// ASCII alphanumeric.
    ///
    fn is_ascii_alphanumeric(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII decimal digit:
    /// U+0030 '0' ... U+0039 '9'.
    /// For strings, true if all characters in the string are
    /// ASCII digits.
    ///
    fn is_ascii_digit(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII hexadecimal digit:
    /// U+0030 '0' ... U+0039 '9', U+0041 'A' ... U+0046 'F', or
    /// U+0061 'a' ... U+0066 'f'.
    /// For strings, true if all characters in the string are
    /// ASCII hex digits.
    ///
    fn is_ascii_hexdigit(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII punctuation character:
    /// U+0021 ... U+002F `! " # $ % & ' ( ) * + , - . /`
    /// U+003A ... U+0040 `: ; < = > ? @`
    /// U+005B ... U+0060 `[ \\ ] ^ _ \``
    /// U+007B ... U+007E `{ | } ~`
    /// For strings, true if all characters in the string are
    /// ASCII punctuation.
    ///
    fn is_ascii_punctuation(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII graphic character:
    /// U+0021 '@' ... U+007E '~'.
    /// For strings, true if all characters in the string are
    /// ASCII punctuation.
    ///
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
    fn is_ascii_whitespace(&self) -> bool { unimplemented!(); }

    /// Checks if the value is an ASCII control character:
    /// U+0000 NUL ... U+001F UNIT SEPARATOR, or U+007F DELETE.
    /// Note that most ASCII whitespace characters are control
    /// characters, but SPACE is not.
    ///
    fn is_ascii_control(&self) -> bool { unimplemented!(); }
}

impl AsciiExt for str {
    type Owned = String;

    #[inline]
    fn is_ascii(&self) -> bool {
        self.bytes().all(|b| b.is_ascii())
    }

    #[inline]
    fn to_ascii_uppercase(&self) -> String {
        let mut bytes = self.as_bytes().to_vec();
        bytes.make_ascii_uppercase();
        // make_ascii_uppercase() preserves the UTF-8 invariant.
        unsafe { String::from_utf8_unchecked(bytes) }
    }

    #[inline]
    fn to_ascii_lowercase(&self) -> String {
        let mut bytes = self.as_bytes().to_vec();
        bytes.make_ascii_lowercase();
        // make_ascii_uppercase() preserves the UTF-8 invariant.
        unsafe { String::from_utf8_unchecked(bytes) }
    }

    #[inline]
    fn eq_ignore_ascii_case(&self, other: &str) -> bool {
        self.as_bytes().eq_ignore_ascii_case(other.as_bytes())
    }

    fn make_ascii_uppercase(&mut self) {
        let me = unsafe { self.as_bytes_mut() };
        me.make_ascii_uppercase()
    }

    fn make_ascii_lowercase(&mut self) {
        let me = unsafe { self.as_bytes_mut() };
        me.make_ascii_lowercase()
    }

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

impl AsciiExt for [u8] {
    type Owned = Vec<u8>;
    #[inline]
    fn is_ascii(&self) -> bool {
        self.iter().all(|b| b.is_ascii())
    }

    #[inline]
    fn to_ascii_uppercase(&self) -> Vec<u8> {
        let mut me = self.to_vec();
        me.make_ascii_uppercase();
        return me
    }

    #[inline]
    fn to_ascii_lowercase(&self) -> Vec<u8> {
        let mut me = self.to_vec();
        me.make_ascii_lowercase();
        return me
    }

    #[inline]
    fn eq_ignore_ascii_case(&self, other: &[u8]) -> bool {
        self.len() == other.len() &&
        self.iter().zip(other).all(|(a, b)| {
            a.eq_ignore_ascii_case(b)
        })
    }

    fn make_ascii_uppercase(&mut self) {
        for byte in self {
            byte.make_ascii_uppercase();
        }
    }

    fn make_ascii_lowercase(&mut self) {
        for byte in self {
            byte.make_ascii_lowercase();
        }
    }

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

impl AsciiExt for u8 {
    type Owned = u8;
    #[inline]
    fn is_ascii(&self) -> bool { *self & 128 == 0 }
    #[inline]
    fn to_ascii_uppercase(&self) -> u8 { ASCII_UPPERCASE_MAP[*self as usize] }
    #[inline]
    fn to_ascii_lowercase(&self) -> u8 { ASCII_LOWERCASE_MAP[*self as usize] }
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &u8) -> bool {
        self.to_ascii_lowercase() == other.to_ascii_lowercase()
    }
    #[inline]
    fn make_ascii_uppercase(&mut self) { *self = self.to_ascii_uppercase(); }
    #[inline]
    fn make_ascii_lowercase(&mut self) { *self = self.to_ascii_lowercase(); }

    #[inline]
    fn is_ascii_alphabetic(&self) -> bool {
        if *self >= 0x80 { return false; }
        match ASCII_CHARACTER_CLASS[*self as usize] {
            L|Lx|U|Ux => true,
            _ => false
        }
    }

    #[inline]
    fn is_ascii_uppercase(&self) -> bool {
        if *self >= 0x80 { return false }
        match ASCII_CHARACTER_CLASS[*self as usize] {
            U|Ux => true,
            _ => false
        }
    }

    #[inline]
    fn is_ascii_lowercase(&self) -> bool {
        if *self >= 0x80 { return false }
        match ASCII_CHARACTER_CLASS[*self as usize] {
            L|Lx => true,
            _ => false
        }
    }

    #[inline]
    fn is_ascii_alphanumeric(&self) -> bool {
        if *self >= 0x80 { return false }
        match ASCII_CHARACTER_CLASS[*self as usize] {
            D|L|Lx|U|Ux => true,
            _ => false
        }
    }

    #[inline]
    fn is_ascii_digit(&self) -> bool {
        if *self >= 0x80 { return false }
        match ASCII_CHARACTER_CLASS[*self as usize] {
            D => true,
            _ => false
        }
    }

    #[inline]
    fn is_ascii_hexdigit(&self) -> bool {
        if *self >= 0x80 { return false }
        match ASCII_CHARACTER_CLASS[*self as usize] {
            D|Lx|Ux => true,
            _ => false
        }
    }

    #[inline]
    fn is_ascii_punctuation(&self) -> bool {
        if *self >= 0x80 { return false }
        match ASCII_CHARACTER_CLASS[*self as usize] {
            P => true,
            _ => false
        }
    }

    #[inline]
    fn is_ascii_graphic(&self) -> bool {
        if *self >= 0x80 { return false; }
        match ASCII_CHARACTER_CLASS[*self as usize] {
            Ux|U|Lx|L|D|P => true,
            _ => false
        }
    }

    #[inline]
    fn is_ascii_whitespace(&self) -> bool {
        if *self >= 0x80 { return false; }
        match ASCII_CHARACTER_CLASS[*self as usize] {
            Cw|W => true,
            _ => false
        }
    }

    #[inline]
    fn is_ascii_control(&self) -> bool {
        if *self >= 0x80 { return false; }
        match ASCII_CHARACTER_CLASS[*self as usize] {
            C|Cw => true,
            _ => false
        }
    }
}

impl AsciiExt for char {
    type Owned = char;
    #[inline]
    fn is_ascii(&self) -> bool {
        *self as u32 <= 0x7F
    }

    #[inline]
    fn to_ascii_uppercase(&self) -> char {
        if self.is_ascii() {
            (*self as u8).to_ascii_uppercase() as char
        } else {
            *self
        }
    }

    #[inline]
    fn to_ascii_lowercase(&self) -> char {
        if self.is_ascii() {
            (*self as u8).to_ascii_lowercase() as char
        } else {
            *self
        }
    }

    #[inline]
    fn eq_ignore_ascii_case(&self, other: &char) -> bool {
        self.to_ascii_lowercase() == other.to_ascii_lowercase()
    }

    #[inline]
    fn make_ascii_uppercase(&mut self) { *self = self.to_ascii_uppercase(); }
    #[inline]
    fn make_ascii_lowercase(&mut self) { *self = self.to_ascii_lowercase(); }

    #[inline]
    fn is_ascii_alphabetic(&self) -> bool {
        (*self as u32 <= 0x7f) && (*self as u8).is_ascii_alphabetic()
    }

    #[inline]
    fn is_ascii_uppercase(&self) -> bool {
        (*self as u32 <= 0x7f) && (*self as u8).is_ascii_uppercase()
    }

    #[inline]
    fn is_ascii_lowercase(&self) -> bool {
        (*self as u32 <= 0x7f) && (*self as u8).is_ascii_lowercase()
    }

    #[inline]
    fn is_ascii_alphanumeric(&self) -> bool {
        (*self as u32 <= 0x7f) && (*self as u8).is_ascii_alphanumeric()
    }

    #[inline]
    fn is_ascii_digit(&self) -> bool {
        (*self as u32 <= 0x7f) && (*self as u8).is_ascii_digit()
    }

    #[inline]
    fn is_ascii_hexdigit(&self) -> bool {
        (*self as u32 <= 0x7f) && (*self as u8).is_ascii_hexdigit()
    }

    #[inline]
    fn is_ascii_punctuation(&self) -> bool {
        (*self as u32 <= 0x7f) && (*self as u8).is_ascii_punctuation()
    }

    #[inline]
    fn is_ascii_graphic(&self) -> bool {
        (*self as u32 <= 0x7f) && (*self as u8).is_ascii_graphic()
    }

    #[inline]
    fn is_ascii_whitespace(&self) -> bool {
        (*self as u32 <= 0x7f) && (*self as u8).is_ascii_whitespace()
    }

    #[inline]
    fn is_ascii_control(&self) -> bool {
        (*self as u32 <= 0x7f) && (*self as u8).is_ascii_control()
    }
}

/// An iterator over the escaped version of a byte.
///
/// This `struct` is created by the [`escape_default`] function. See its
/// documentation for more.
///
pub struct EscapeDefault {
    range: Range<usize>,
    data: [u8; 4],
}

/// Returns an iterator that produces an escaped version of a `u8`.
///
/// The default is chosen with a bias toward producing literals that are
/// legal in a variety of languages, including C++11 and similar C-family
/// languages. The exact rules are:
///
/// - Tab, CR and LF are escaped as '\t', '\r' and '\n' respectively.
/// - Single-quote, double-quote and backslash chars are backslash-escaped.
/// - Any other chars in the range [0x20,0x7e] are not escaped.
/// - Any other chars are given hex escapes of the form '\xNN'.
/// - Unicode escapes are never generated by this function.
///
pub fn escape_default(c: u8) -> EscapeDefault {
    let (data, len) = match c {
        b'\t' => ([b'\\', b't', 0, 0], 2),
        b'\r' => ([b'\\', b'r', 0, 0], 2),
        b'\n' => ([b'\\', b'n', 0, 0], 2),
        b'\\' => ([b'\\', b'\\', 0, 0], 2),
        b'\'' => ([b'\\', b'\'', 0, 0], 2),
        b'"' => ([b'\\', b'"', 0, 0], 2),
        b'\x20' ... b'\x7e' => ([c, 0, 0, 0], 1),
        _ => ([b'\\', b'x', hexify(c >> 4), hexify(c & 0xf)], 4),
    };

    return EscapeDefault { range: (0.. len), data: data };

    fn hexify(b: u8) -> u8 {
        match b {
            0 ... 9 => b'0' + b,
            _ => b'a' + b - 10,
        }
    }
}

impl Iterator for EscapeDefault {
    type Item = u8;
    fn next(&mut self) -> Option<u8> { self.range.next().map(|i| self.data[i]) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.range.size_hint() }
}

impl DoubleEndedIterator for EscapeDefault {
    fn next_back(&mut self) -> Option<u8> {
        self.range.next_back().map(|i| self.data[i])
    }
}

impl ExactSizeIterator for EscapeDefault {}

impl FusedIterator for EscapeDefault {}

impl fmt::Debug for EscapeDefault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("EscapeDefault { .. }")
    }
}

static ASCII_LOWERCASE_MAP: [u8; 256] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    b' ', b'!', b'"', b'#', b'$', b'%', b'&', b'\'',
    b'(', b')', b'*', b'+', b',', b'-', b'.', b'/',
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
    b'8', b'9', b':', b';', b'<', b'=', b'>', b'?',
    b'@',

          b'a', b'b', b'c', b'd', b'e', b'f', b'g',
    b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o',
    b'p', b'q', b'r', b's', b't', b'u', b'v', b'w',
    b'x', b'y', b'z',

                      b'[', b'\\', b']', b'^', b'_',
    b'`', b'a', b'b', b'c', b'd', b'e', b'f', b'g',
    b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o',
    b'p', b'q', b'r', b's', b't', b'u', b'v', b'w',
    b'x', b'y', b'z', b'{', b'|', b'}', b'~', 0x7f,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
    0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
    0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
    0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
    0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7,
    0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
    0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
    0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
    0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
    0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
    0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7,
    0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
    0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];

static ASCII_UPPERCASE_MAP: [u8; 256] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    b' ', b'!', b'"', b'#', b'$', b'%', b'&', b'\'',
    b'(', b')', b'*', b'+', b',', b'-', b'.', b'/',
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
    b'8', b'9', b':', b';', b'<', b'=', b'>', b'?',
    b'@', b'A', b'B', b'C', b'D', b'E', b'F', b'G',
    b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O',
    b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W',
    b'X', b'Y', b'Z', b'[', b'\\', b']', b'^', b'_',
    b'`',

          b'A', b'B', b'C', b'D', b'E', b'F', b'G',
    b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O',
    b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W',
    b'X', b'Y', b'Z',

                      b'{', b'|', b'}', b'~', 0x7f,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
    0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
    0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
    0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
    0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7,
    0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
    0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
    0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
    0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
    0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
    0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7,
    0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
    0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];

enum AsciiCharacterClass {
    C,  // control
    Cw, // control whitespace
    W,  // whitespace
    D,  // digit
    L,  // lowercase
    Lx, // lowercase hex digit
    U,  // uppercase
    Ux, // uppercase hex digit
    P,  // punctuation
}
use self::AsciiCharacterClass::*;

static ASCII_CHARACTER_CLASS: [AsciiCharacterClass; 128] = [
//  _0 _1 _2 _3 _4 _5 _6 _7 _8 _9 _a _b _c _d _e _f
    C, C, C, C, C, C, C, C, C, Cw,Cw,C, Cw,Cw,C, C, // 0_
    C, C, C, C, C, C, C, C, C, C, C, C, C, C, C, C, // 1_
    W, P, P, P, P, P, P, P, P, P, P, P, P, P, P, P, // 2_
    D, D, D, D, D, D, D, D, D, D, P, P, P, P, P, P, // 3_
    P, Ux,Ux,Ux,Ux,Ux,Ux,U, U, U, U, U, U, U, U, U, // 4_
    U, U, U, U, U, U, U, U, U, U, U, P, P, P, P, P, // 5_
    P, Lx,Lx,Lx,Lx,Lx,Lx,L, L, L, L, L, L, L, L, L, // 6_
    L, L, L, L, L, L, L, L, L, L, L, P, P, P, P, C, // 7_
];

