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

//! A wrapper around any Read to treat it as an RNG.

#![allow(dead_code)]

use io::prelude::*;
use rand::Rng;

/// An RNG that reads random bytes straight from a `Read`. This will
/// work best with an infinite reader, but this is not required.
///
/// # Panics
///
/// It will panic if it there is insufficient data to fulfill a request.
pub struct ReaderRng<R> {
    reader: R
}

impl<R: Read> ReaderRng<R> {
    /// Create a new `ReaderRng` from a `Read`.
    pub fn new(r: R) -> ReaderRng<R> {
        ReaderRng {
            reader: r
        }
    }
}

impl<R: Read> Rng for ReaderRng<R> {
    fn next_u32(&mut self) -> u32 {
        // This is designed for speed: reading a LE integer on a LE
        // platform just involves blitting the bytes into the memory
        // of the u32, similarly for BE on BE; avoiding byteswapping.
        let mut bytes = [0; 4];
        self.fill_bytes(&mut bytes);
        unsafe { *(bytes.as_ptr() as *const u32) }
    }
    fn next_u64(&mut self) -> u64 {
        // see above for explanation.
        let mut bytes = [0; 8];
        self.fill_bytes(&mut bytes);
        unsafe { *(bytes.as_ptr() as *const u64) }
    }
    fn fill_bytes(&mut self, mut v: &mut [u8]) {
        while !v.is_empty() {
            let t = v;
            match self.reader.read(t) {
                Ok(0) => panic!("ReaderRng.fill_bytes: EOF reached"),
                Ok(n) => v = t.split_at_mut(n).1,
                Err(e) => panic!("ReaderRng.fill_bytes: {}", e),
            }
        }
    }
}