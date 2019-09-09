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

use std::io::{self, Read};
use std::mem;
use crate::Rng;

/// An RNG that reads random bytes straight from a `Read`. This will
/// work best with an infinite reader, but this is not required.
///
/// # Panics
///
/// It will panic if it there is insufficient data to fulfill a request.
///
/// # Example
///
/// ```rust
/// use sgx_rand::{read, Rng};
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
/// let mut rng = read::ReadRng::new(&data[..]);
/// println!("{:x}", rng.gen::<u32>());
/// ```
#[derive(Debug)]
pub struct ReadRng<R> {
    reader: R
}

impl<R: Read> ReadRng<R> {
    /// Create a new `ReadRng` from a `Read`.
    pub fn new(r: R) -> ReadRng<R> {
        ReadRng {
            reader: r
        }
    }
}

impl<R: Read> Rng for ReadRng<R> {
    fn next_u32(&mut self) -> u32 {
        // This is designed for speed: reading a LE integer on a LE
        // platform just involves blitting the bytes into the memory
        // of the u32, similarly for BE on BE; avoiding byteswapping.
        let mut buf = [0; 4];
        fill(&mut self.reader, &mut buf).unwrap();
        unsafe { *(buf.as_ptr() as *const u32) }
    }
    fn next_u64(&mut self) -> u64 {
        // see above for explanation.
        let mut buf = [0; 8];
        fill(&mut self.reader, &mut buf).unwrap();
        unsafe { *(buf.as_ptr() as *const u64) }
    }
    fn fill_bytes(&mut self, v: &mut [u8]) {
        if v.len() == 0 { return }
        fill(&mut self.reader, v).unwrap();
    }
}

fn fill(r: &mut dyn Read, mut buf: &mut [u8]) -> io::Result<()> {
    while buf.len() > 0 {
        match (r.read(buf))? {
            0 => return Err(io::Error::new(io::ErrorKind::Other,
                                           "end of file reached")),
            n => buf = &mut mem::replace(&mut buf, &mut [])[n..],
        }
    }
    Ok(())
}
