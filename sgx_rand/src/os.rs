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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SgxRng {{}}")
    }
}

fn next_u32(fill_buf: &mut dyn FnMut(&mut [u8])) -> u32 {
    let mut buf: [u8; 4] = [0; 4];
    fill_buf(&mut buf);
    unsafe { mem::transmute::<[u8; 4], u32>(buf) }
}

fn next_u64(fill_buf: &mut dyn FnMut(&mut [u8])) -> u64 {
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