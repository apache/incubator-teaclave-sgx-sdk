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

//! A wrapper around any Read to treat it as an RNG.

#![allow(dead_code)]

use crate::io::prelude::*;
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