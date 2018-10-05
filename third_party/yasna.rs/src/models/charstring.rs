// Copyright 2016 Masaki Hara
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;
use std::prelude::v1::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PrintableString {
    string: String,
}

impl PrintableString {
    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        for &b in bytes.iter() {
            let ok =
                (b'0' <= b && b <= b'9') ||
                (b'A' <= b && b <= b'Z') ||
                (b'a' <= b && b <= b'z') ||
                b == b' ' || b == b'\'' || b == b'(' || b == b')' ||
                b == b'+' || b == b',' || b == b'-' || b == b'.' ||
                b == b'/' || b == b':' || b == b'=' || b == b'?';
            if !ok {
                return None;
            }
        }
        return Some(PrintableString {
            string: String::from_utf8(bytes).unwrap(),
        });
    }
}

impl Deref for PrintableString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        return &self.string;
    }
}
