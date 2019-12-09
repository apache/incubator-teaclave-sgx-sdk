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

//! A wrapper around another RNG that reseeds it after it
//! generates a certain number of random bytes.

use std::default::Default;

use crate::{Rng, SeedableRng};

/// How many bytes of entropy the underling RNG is allowed to generate
/// before it is reseeded
const DEFAULT_GENERATION_THRESHOLD: u64 = 32 * 1024;

/// A wrapper around any RNG which reseeds the underlying RNG after it
/// has generated a certain number of random bytes.
#[derive(Debug)]
pub struct ReseedingRng<R, Rsdr> {
    rng: R,
    generation_threshold: u64,
    bytes_generated: u64,
    /// Controls the behaviour when reseeding the RNG.
    pub reseeder: Rsdr,
}

impl<R: Rng, Rsdr: Reseeder<R>> ReseedingRng<R, Rsdr> {
    /// Create a new `ReseedingRng` with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `rng`: the random number generator to use.
    /// * `generation_threshold`: the number of bytes of entropy at which to reseed the RNG.
    /// * `reseeder`: the reseeding object to use.
    pub fn new(rng: R, generation_threshold: u64, reseeder: Rsdr) -> ReseedingRng<R,Rsdr> {
        ReseedingRng {
            rng: rng,
            generation_threshold: generation_threshold,
            bytes_generated: 0,
            reseeder: reseeder
        }
    }

    /// Reseed the internal RNG if the number of bytes that have been
    /// generated exceed the threshold.
    pub fn reseed_if_necessary(&mut self) {
        if self.bytes_generated >= self.generation_threshold {
            self.reseeder.reseed(&mut self.rng);
            self.bytes_generated = 0;
        }
    }
}


impl<R: Rng, Rsdr: Reseeder<R>> Rng for ReseedingRng<R, Rsdr> {
    fn next_u32(&mut self) -> u32 {
        self.reseed_if_necessary();
        self.bytes_generated += 4;
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.reseed_if_necessary();
        self.bytes_generated += 8;
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.reseed_if_necessary();
        self.bytes_generated += dest.len() as u64;
        self.rng.fill_bytes(dest)
    }
}

impl<S, R: SeedableRng<S>, Rsdr: Reseeder<R> + Default>
     SeedableRng<(Rsdr, S)> for ReseedingRng<R, Rsdr> {
    fn reseed(&mut self, (rsdr, seed): (Rsdr, S)) {
        self.rng.reseed(seed);
        self.reseeder = rsdr;
        self.bytes_generated = 0;
    }

    /// Create a new `ReseedingRng` from the given reseeder and
    /// seed. This uses a default value for `generation_threshold`.
    fn from_seed((rsdr, seed): (Rsdr, S)) -> ReseedingRng<R, Rsdr> {
        ReseedingRng {
            rng: SeedableRng::from_seed(seed),
            generation_threshold: DEFAULT_GENERATION_THRESHOLD,
            bytes_generated: 0,
            reseeder: rsdr
        }
    }
}

/// Something that can be used to reseed an RNG via `ReseedingRng`.
///
/// # Example
///
/// ```rust
/// use sgx_rand::{Rng, SeedableRng, StdRng};
/// use sgx_rand::reseeding::{Reseeder, ReseedingRng};
///
/// struct TickTockReseeder { tick: bool }
/// impl Reseeder<StdRng> for TickTockReseeder {
///     fn reseed(&mut self, rng: &mut StdRng) {
///         let val = if self.tick {0} else {1};
///         rng.reseed(&[val]);
///         self.tick = !self.tick;
///     }
/// }
/// fn main() {
///     let rsdr = TickTockReseeder { tick: true };
///
///     let inner = StdRng::new().unwrap();
///     let mut rng = ReseedingRng::new(inner, 10, rsdr);
///
///     // this will repeat, because it gets reseeded very regularly.
///     let s: String = rng.gen_ascii_chars().take(100).collect();
///     println!("{}", s);
/// }
///
/// ```
pub trait Reseeder<R> {
    /// Reseed the given RNG.
    fn reseed(&mut self, rng: &mut R);
}

/// Reseed an RNG using a `Default` instance. This reseeds by
/// replacing the RNG with the result of a `Default::default` call.
#[derive(Clone, Copy, Debug)]
pub struct ReseedWithDefault;

impl<R: Rng + Default> Reseeder<R> for ReseedWithDefault {
    fn reseed(&mut self, rng: &mut R) {
        *rng = Default::default();
    }
}
impl Default for ReseedWithDefault {
    fn default() -> ReseedWithDefault { ReseedWithDefault }
}
