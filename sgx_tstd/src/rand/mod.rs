// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

use io;
use sys;
use core::cell::RefCell;
use core::fmt;
use core::mem;
use alloc::rc::Rc;

#[cfg(target_pointer_width = "32")]
use core_rand::IsaacRng as IsaacWordRng;
#[cfg(target_pointer_width = "64")]
use core_rand::Isaac64Rng as IsaacWordRng;

pub use core_rand::{Rand, Rng, SeedableRng};
pub use core_rand::{XorShiftRng, IsaacRng, Isaac64Rng};
pub use core_rand::reseeding;

pub mod reader;

/// The standard RNG. This is designed to be efficient on the current
/// platform.
#[derive(Copy, Clone)]
pub struct StdRng {
    rng: IsaacWordRng,
}

impl StdRng {
    /// Create a randomly seeded instance of `StdRng`.
    ///
    /// This is a very expensive operation as it has to read
    /// randomness from the operating system and use this in an
    /// expensive seeding operation. If one is only generating a small
    /// number of random numbers, or doesn't need the utmost speed for
    /// generating each number, `thread_rng` and/or `random` may be more
    /// appropriate.
    ///
    /// Reading the randomness from the OS may fail, and any error is
    /// propagated via the `io::Result` return value.
    pub fn new() -> io::Result<StdRng> {
        SgxRng::new().map(|mut r| StdRng { rng: r.gen() })
    }
}

impl Rng for StdRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }
}

impl<'a> SeedableRng<&'a [usize]> for StdRng {
    fn reseed(&mut self, seed: &'a [usize]) {
        // the internal RNG can just be seeded from the above
        // randomness.
        self.rng.reseed(unsafe {mem::transmute(seed)})
    }

    fn from_seed(seed: &'a [usize]) -> StdRng {
        StdRng { rng: SeedableRng::from_seed(unsafe {mem::transmute(seed)}) }
    }
}

/// Controls how the thread-local RNG is reseeded.
struct ThreadRngReseeder;

impl reseeding::Reseeder<StdRng> for ThreadRngReseeder {
    fn reseed(&mut self, rng: &mut StdRng) {
        *rng = match StdRng::new() {
            Ok(r) => r,
            Err(e) => panic!("could not reseed thread_rng: {}", e)
        }
    }
}
const THREAD_RNG_RESEED_THRESHOLD: usize = 32_768;
type ThreadRngInner = reseeding::ReseedingRng<StdRng, ThreadRngReseeder>;

/// The thread-local RNG.
#[derive(Clone)]
pub struct ThreadRng {
    rng: Rc<RefCell<ThreadRngInner>>,
}

impl fmt::Debug for ThreadRng {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("ThreadRng { .. }")
    }
}

/// Retrieve the lazily-initialized thread-local random number
/// generator, seeded by the system. Intended to be used in method
/// chaining style, e.g. `thread_rng().gen::<isize>()`.
///
/// The RNG provided will reseed itself from the operating system
/// after generating a certain amount of randomness.
///
/// The internal RNG used is platform and architecture dependent, even
/// if the operating system random number generator is rigged to give
/// the same sequence always. If absolute consistency is required,
/// explicitly select an RNG, e.g. `IsaacRng` or `Isaac64Rng`.
pub fn thread_rng() -> ThreadRng {
    // used to make space in TLS for a random number generator
    thread_local!(static THREAD_RNG_KEY: Rc<RefCell<ThreadRngInner>> = {
        let r = match StdRng::new() {
            Ok(r) => r,
            Err(e) => panic!("could not initialize thread_rng: {}", e)
        };
        let rng = reseeding::ReseedingRng::new(r,
                                               THREAD_RNG_RESEED_THRESHOLD,
                                               ThreadRngReseeder);
        Rc::new(RefCell::new(rng))
    });

    ThreadRng { rng: THREAD_RNG_KEY.with(|t| t.clone()) }
}

impl Rng for ThreadRng {
    fn next_u32(&mut self) -> u32 {
        self.rng.borrow_mut().next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.borrow_mut().next_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        self.rng.borrow_mut().fill_bytes(bytes)
    }
}

/// A random number generator that retrieves randomness straight from
/// the Sgx.
pub struct SgxRng(sys::rand::SgxRng);

impl SgxRng {
    /// Create a new `SgxRng`.
    pub fn new() -> io::Result<SgxRng> {
        sys::rand::SgxRng::new().map(SgxRng)
    }
}

impl Rng for SgxRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        self.0.fill_bytes(bytes)
    }
}