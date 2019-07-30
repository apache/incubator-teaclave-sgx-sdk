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

//! Interfaces to the operating system provided random number
//! generators.

use std::{io, mem, fmt};
use crate::Rng;

/// A random number generator
pub struct SgxRng(imp::SgxRng);

impl SgxRng {
    /// Create a new `SgxRng`.
    pub fn new() -> io::Result<SgxRng> {
        imp::SgxRng::new().map(SgxRng)
    }
}

impl Rng for SgxRng {
    fn next_u32(&mut self) -> u32 { self.0.next_u32() }
    fn next_u64(&mut self) -> u64 { self.0.next_u64() }
    fn fill_bytes(&mut self, v: &mut [u8]) { self.0.fill_bytes(v) }
}

impl fmt::Debug for SgxRng {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SgxRng {{}}")
    }
}

fn next_u32(fill_buf: &mut FnMut(&mut [u8])) -> u32 {
    let mut buf: [u8; 4] = [0; 4];
    fill_buf(&mut buf);
    unsafe { mem::transmute::<[u8; 4], u32>(buf) }
}

fn next_u64(fill_buf: &mut FnMut(&mut [u8])) -> u64 {
    let mut buf: [u8; 8] = [0; 8];
    fill_buf(&mut buf);
    unsafe { mem::transmute::<[u8; 8], u64>(buf) }
}

mod imp {

    use sgx_types::*;
    use sgx_trts::trts::rsgx_read_rand;
    use std::io;

    use super::{next_u32, next_u64};
    use crate::Rng;

    fn getrandom(buf: &mut [u8]) -> SgxError {
        rsgx_read_rand(buf)
    }

    fn getrandom_fill_bytes(v: &mut [u8]) {
        getrandom(v).expect("unexpected getrandom error");
    }

    #[allow(dead_code)]
    fn is_getrandom_available() -> bool { true }

    pub struct SgxRng;

    impl SgxRng {
        /// Create a new `SgxRng`.
        pub fn new() -> io::Result<SgxRng> {
            Ok(SgxRng)
        }
    }

    impl Rng for SgxRng {
        fn next_u32(&mut self) -> u32 {
            next_u32(&mut getrandom_fill_bytes)
        }
        fn next_u64(&mut self) -> u64 {
            next_u64(&mut getrandom_fill_bytes)
        }
        fn fill_bytes(&mut self, v: &mut [u8]) {
            getrandom_fill_bytes(v)
        }
    }
}
