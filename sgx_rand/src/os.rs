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

use crate::Rng;
use std::{fmt, io};

#[cfg(feature = "urand")]
use rand_core::RngCore;
#[cfg(feature = "urand")]
use rdrand::RdRand as RandRng;
#[cfg(feature = "trand")]
use sgx_trts::rand::Rng as RandRng;

/// A random number generator
pub struct RdRand(RandRng);

impl RdRand {
    /// Create a new `RdRand`.
    pub fn new() -> io::Result<RdRand> {
        #[cfg(feature = "trand")]
        {
            Ok(RdRand(RandRng::new()))
        }
        #[cfg(feature = "urand")]
        {
            use std::io::ErrorKind;
            let rng = RandRng::new().map_err(|_| ErrorKind::Unsupported)?;
            Ok(RdRand(rng))
        }
    }
}

impl Rng for RdRand {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
    fn fill_bytes(&mut self, v: &mut [u8]) {
        self.0.fill_bytes(v)
    }
}

impl fmt::Debug for RdRand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RdRand {{}}")
    }
}
