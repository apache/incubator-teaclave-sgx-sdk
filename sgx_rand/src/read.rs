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
            0 => return Err(io::Error::new(
                io::ErrorKind::Other,
                "end of file reached",
            )),
            n => buf = &mut mem::replace(&mut buf, &mut [])[n..],
        }
    }
    Ok(())
}
