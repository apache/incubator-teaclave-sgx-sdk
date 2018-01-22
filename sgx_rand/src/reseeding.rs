// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

//! A wrapper around another RNG that reseeds it after it
//! generates a certain number of random bytes.

use std::default::Default;

use {Rng, SeedableRng};

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
