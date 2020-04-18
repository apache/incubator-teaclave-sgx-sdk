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

//! Utilities for random number generation
//!
//! This crate provides similar functionalities as `librand` in Rust

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(all(target_env = "sgx", target_vendor = "mesalock"), feature(rustc_private))]

extern crate sgx_types;
extern crate sgx_trts;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use std::boxed::Box;
use std::vec::Vec;
use std::cell::RefCell;
use std::marker;
use std::mem;
use std::io;
use std::rc::Rc;
use std::num::Wrapping as w;

pub use os::SgxRng;

pub use isaac::{IsaacRng, Isaac64Rng};
pub use chacha::ChaChaRng;

#[cfg(target_pointer_width = "32")]
use IsaacRng as IsaacWordRng;
#[cfg(target_pointer_width = "64")]
use Isaac64Rng as IsaacWordRng;

use distributions::{Range, IndependentSample};
use distributions::range::SampleRange;

pub mod distributions;
pub mod isaac;
pub mod chacha;
pub mod reseeding;
mod rand_impls;
pub mod os;
pub mod read;

#[allow(bad_style)]
type w64 = w<u64>;
#[allow(bad_style)]
type w32 = w<u32>;

/// A type that can be randomly generated using an `Rng`.
///
/// ## Built-in Implementations
///
/// This crate implements `Rand` for various primitive types.  Assuming the
/// provided `Rng` is well-behaved, these implementations generate values with
/// the following ranges and distributions:
///
/// * Integers (`i32`, `u32`, `isize`, `usize`, etc.): Uniformly distributed
///   over all values of the type.
/// * `char`: Uniformly distributed over all Unicode scalar values, i.e. all
///   code points in the range `0...0x10_FFFF`, except for the range
///   `0xD800...0xDFFF` (the surrogate code points).  This includes
///   unassigned/reserved code points.
/// * `bool`: Generates `false` or `true`, each with probability 0.5.
/// * Floating point types (`f32` and `f64`): Uniformly distributed in the
///   half-open range `[0, 1)`.  (The [`Open01`], [`Closed01`], [`Exp1`], and
///   [`StandardNormal`] wrapper types produce floating point numbers with
///   alternative ranges or distributions.)
///
/// [`Open01`]: struct.Open01.html
/// [`Closed01`]: struct.Closed01.html
/// [`Exp1`]: struct.Exp1.html
/// [`StandardNormal`]: struct.StandardNormal.html
///
/// The following aggregate types also implement `Rand` as long as their
/// component types implement it:
///
/// * Tuples and arrays: Each element of the tuple or array is generated
///   independently, using its own `Rand` implementation.
/// * `Option<T>`: Returns `None` with probability 0.5; otherwise generates a
///   random `T` and returns `Some(T)`.

pub trait Rand : Sized {
    /// Generates a random instance of this type using the specified source of
    /// randomness.
    fn rand<R: Rng>(rng: &mut R) -> Self;
}

/// A random number generator.
pub trait Rng {
    /// Return the next random u32.
    ///
    /// This rarely needs to be called directly, prefer `r.gen()` to
    /// `r.next_u32()`.
    // FIXME #rust-lang/rfcs#628: Should be implemented in terms of next_u64
    fn next_u32(&mut self) -> u32;

    /// Return the next random u64.
    ///
    /// By default this is implemented in terms of `next_u32`. An
    /// implementation of this trait must provide at least one of
    /// these two methods. Similarly to `next_u32`, this rarely needs
    /// to be called directly, prefer `r.gen()` to `r.next_u64()`.
    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | (self.next_u32() as u64)
    }

    /// Return the next random f32 selected from the half-open
    /// interval `[0, 1)`.
    ///
    /// This uses a technique described by Saito and Matsumoto at
    /// MCQMC'08. Given that the IEEE floating point numbers are
    /// uniformly distributed over [1,2), we generate a number in
    /// this range and then offset it onto the range [0,1). Our
    /// choice of bits (masking v. shifting) is arbitrary and
    /// should be immaterial for high quality generators. For low
    /// quality generators (ex. LCG), prefer bitshifting due to
    /// correlation between sequential low order bits.
    ///
    /// See:
    /// A PRNG specialized in double precision floating point numbers using
    /// an affine transition
    /// http://www.math.sci.hiroshima-u.ac.jp/~m-mat/MT/ARTICLES/dSFMT.pdf
    /// http://www.math.sci.hiroshima-u.ac.jp/~m-mat/MT/SFMT/dSFMT-slide-e.pdf
    ///
    /// By default this is implemented in terms of `next_u32`, but a
    /// random number generator which can generate numbers satisfying
    /// the requirements directly can overload this for performance.
    /// It is required that the return value lies in `[0, 1)`.
    ///
    /// See `Closed01` for the closed interval `[0,1]`, and
    /// `Open01` for the open interval `(0,1)`.
    fn next_f32(&mut self) -> f32 {
        const UPPER_MASK: u32 = 0x3F800000;
        const LOWER_MASK: u32 = 0x7FFFFF;
        let tmp = UPPER_MASK | (self.next_u32() & LOWER_MASK);
        let result: f32 = unsafe { mem::transmute(tmp) };
        result - 1.0
    }

    /// Return the next random f64 selected from the half-open
    /// interval `[0, 1)`.
    ///
    /// By default this is implemented in terms of `next_u64`, but a
    /// random number generator which can generate numbers satisfying
    /// the requirements directly can overload this for performance.
    /// It is required that the return value lies in `[0, 1)`.
    ///
    /// See `Closed01` for the closed interval `[0,1]`, and
    /// `Open01` for the open interval `(0,1)`.
    fn next_f64(&mut self) -> f64 {
        const UPPER_MASK: u64 = 0x3FF0000000000000;
        const LOWER_MASK: u64 = 0xFFFFFFFFFFFFF;
        let tmp = UPPER_MASK | (self.next_u64() & LOWER_MASK);
        let result: f64 = unsafe { mem::transmute(tmp) };
        result - 1.0
    }

    /// Fill `dest` with random data.
    ///
    /// This has a default implementation in terms of `next_u64` and
    /// `next_u32`, but should be overridden by implementations that
    /// offer a more efficient solution than just calling those
    /// methods repeatedly.
    ///
    /// This method does *not* have a requirement to bear any fixed
    /// relationship to the other methods, for example, it does *not*
    /// have to result in the same output as progressively filling
    /// `dest` with `self.gen::<u8>()`, and any such behaviour should
    /// not be relied upon.
    ///
    /// This method should guarantee that `dest` is entirely filled
    /// with new data, and may panic if this is impossible
    /// (e.g. reading past the end of a file that is being used as the
    /// source of randomness).
    ///
    /// # Example
    ///
    /// ```rust
    /// use sgx_rand::{thread_rng, Rng};
    ///
    /// let mut v = [0u8; 13579];
    /// thread_rng().fill_bytes(&mut v);
    /// println!("{:?}", &v[..]);
    /// ```
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // this could, in theory, be done by transmuting dest to a
        // [u64], but this is (1) likely to be undefined behaviour for
        // LLVM, (2) has to be very careful about alignment concerns,
        // (3) adds more `unsafe` that needs to be checked, (4)
        // probably doesn't give much performance gain if
        // optimisations are on.
        let mut count = 0;
        let mut num = 0;
        for byte in dest.iter_mut() {
            if count == 0 {
                // we could micro-optimise here by generating a u32 if
                // we only need a few more bytes to fill the vector
                // (i.e. at most 4).
                num = self.next_u64();
                count = 8;
            }

            *byte = (num & 0xff) as u8;
            num >>= 8;
            count -= 1;
        }
    }

    /// Return a random value of a `Rand` type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sgx_rand::{thread_rng, Rng};
    ///
    /// let mut rng = thread_rng();
    /// let x: u32 = rng.gen();
    /// println!("{}", x);
    /// println!("{:?}", rng.gen::<(f64, bool)>());
    /// ```
    #[inline(always)]
    fn gen<T: Rand>(&mut self) -> T where Self: Sized {
        Rand::rand(self)
    }

    /// Return an iterator that will yield an infinite number of randomly
    /// generated items.
    ///
    /// # Example
    ///
    /// ```
    /// use sgx_rand::{thread_rng, Rng};
    ///
    /// let mut rng = thread_rng();
    /// let x = rng.gen_iter::<u32>().take(10).collect::<Vec<u32>>();
    /// println!("{:?}", x);
    /// println!("{:?}", rng.gen_iter::<(f64, bool)>().take(5)
    ///                     .collect::<Vec<(f64, bool)>>());
    /// ```
    fn gen_iter<'a, T: Rand>(&'a mut self) -> Generator<'a, T, Self> where Self: Sized {
        Generator { rng: self, _marker: marker::PhantomData }
    }

    /// Generate a random value in the range [`low`, `high`).
    ///
    /// This is a convenience wrapper around
    /// `distributions::Range`. If this function will be called
    /// repeatedly with the same arguments, one should use `Range`, as
    /// that will amortize the computations that allow for perfect
    /// uniformity, as they only happen on initialization.
    ///
    /// # Panics
    ///
    /// Panics if `low >= high`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sgx_rand::{thread_rng, Rng};
    ///
    /// let mut rng = thread_rng();
    /// let n: u32 = rng.gen_range(0, 10);
    /// println!("{}", n);
    /// let m: f64 = rng.gen_range(-40.0f64, 1.3e5f64);
    /// println!("{}", m);
    /// ```
    fn gen_range<T: PartialOrd + SampleRange>(&mut self, low: T, high: T) -> T where Self: Sized {
        assert!(low < high, "Rng.gen_range called with low >= high");
        Range::new(low, high).ind_sample(self)
    }

    /// Return a bool with a 1 in n chance of true
    ///
    /// # Example
    ///
    /// ```rust
    /// use sgx_rand::{thread_rng, Rng};
    ///
    /// let mut rng = thread_rng();
    /// println!("{}", rng.gen_weighted_bool(3));
    /// ```
    fn gen_weighted_bool(&mut self, n: u32) -> bool where Self: Sized {
        n <= 1 || self.gen_range(0, n) == 0
    }

    /// Return an iterator of random characters from the set A-Z,a-z,0-9.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sgx_rand::{thread_rng, Rng};
    ///
    /// let s: String = thread_rng().gen_ascii_chars().take(10).collect();
    /// println!("{}", s);
    /// ```
    fn gen_ascii_chars<'a>(&'a mut self) -> AsciiGenerator<'a, Self> where Self: Sized {
        AsciiGenerator { rng: self }
    }

    /// Return a random element from `values`.
    ///
    /// Return `None` if `values` is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use sgx_rand::{thread_rng, Rng};
    ///
    /// let choices = [1, 2, 4, 8, 16, 32];
    /// let mut rng = thread_rng();
    /// println!("{:?}", rng.choose(&choices));
    /// assert_eq!(rng.choose(&choices[..0]), None);
    /// ```
    fn choose<'a, T>(&mut self, values: &'a [T]) -> Option<&'a T> where Self: Sized {
        if values.is_empty() {
            None
        } else {
            Some(&values[self.gen_range(0, values.len())])
        }
    }

    /// Return a mutable pointer to a random element from `values`.
    ///
    /// Return `None` if `values` is empty.
    fn choose_mut<'a, T>(&mut self, values: &'a mut [T]) -> Option<&'a mut T> where Self: Sized {
        if values.is_empty() {
            None
        } else {
            let len = values.len();
            Some(&mut values[self.gen_range(0, len)])
        }
    }

    /// Shuffle a mutable slice in place.
    ///
    /// This applies Durstenfeld's algorithm for the [Fisher�CYates shuffle](https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle#The_modern_algorithm)
    /// which produces an unbiased permutation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sgx_rand::{thread_rng, Rng};
    ///
    /// let mut rng = thread_rng();
    /// let mut y = [1, 2, 3];
    /// rng.shuffle(&mut y);
    /// println!("{:?}", y);
    /// rng.shuffle(&mut y);
    /// println!("{:?}", y);
    /// ```
    fn shuffle<T>(&mut self, values: &mut [T]) where Self: Sized {
        let mut i = values.len();
        while i >= 2 {
            // invariant: elements with index >= i have been locked in place.
            i -= 1;
            // lock element i in place.
            values.swap(i, self.gen_range(0, i + 1));
        }
    }
}

impl<'a, R: ?Sized> Rng for &'a mut R where R: Rng {
    fn next_u32(&mut self) -> u32 {
        (**self).next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        (**self).next_u64()
    }

    fn next_f32(&mut self) -> f32 {
        (**self).next_f32()
    }

    fn next_f64(&mut self) -> f64 {
        (**self).next_f64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        (**self).fill_bytes(dest)
    }
}

impl<R: ?Sized> Rng for Box<R> where R: Rng {
    fn next_u32(&mut self) -> u32 {
        (**self).next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        (**self).next_u64()
    }

    fn next_f32(&mut self) -> f32 {
        (**self).next_f32()
    }

    fn next_f64(&mut self) -> f64 {
        (**self).next_f64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        (**self).fill_bytes(dest)
    }
}

/// Iterator which will generate a stream of random items.
///
/// This iterator is created via the [`gen_iter`] method on [`Rng`].
///
/// [`gen_iter`]: trait.Rng.html#method.gen_iter
/// [`Rng`]: trait.Rng.html
#[derive(Debug)]
pub struct Generator<'a, T, R:'a> {
    rng: &'a mut R,
    _marker: marker::PhantomData<fn() -> T>,
}

impl<'a, T: Rand, R: Rng> Iterator for Generator<'a, T, R> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        Some(self.rng.gen())
    }
}

/// Iterator which will continuously generate random ascii characters.
///
/// This iterator is created via the [`gen_ascii_chars`] method on [`Rng`].
///
/// [`gen_ascii_chars`]: trait.Rng.html#method.gen_ascii_chars
/// [`Rng`]: trait.Rng.html
#[derive(Debug)]
pub struct AsciiGenerator<'a, R:'a> {
    rng: &'a mut R,
}

impl<'a, R: Rng> Iterator for AsciiGenerator<'a, R> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        const GEN_ASCII_STR_CHARSET: &'static [u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
              abcdefghijklmnopqrstuvwxyz\
              0123456789";
        Some(*self.rng.choose(GEN_ASCII_STR_CHARSET).unwrap() as char)
    }
}

/// A random number generator that can be explicitly seeded to produce
/// the same stream of randomness multiple times.
pub trait SeedableRng<Seed>: Rng {
    /// Reseed an RNG with the given seed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sgx_rand::{Rng, SeedableRng, StdRng};
    ///
    /// let seed: &[_] = &[1, 2, 3, 4];
    /// let mut rng: StdRng = SeedableRng::from_seed(seed);
    /// println!("{}", rng.gen::<f64>());
    /// rng.reseed(&[5, 6, 7, 8]);
    /// println!("{}", rng.gen::<f64>());
    /// ```
    fn reseed(&mut self, seed: Seed);

    /// Create a new RNG with the given seed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sgx_rand::{Rng, SeedableRng, StdRng};
    ///
    /// let seed: &[_] = &[1, 2, 3, 4];
    /// let mut rng: StdRng = SeedableRng::from_seed(seed);
    /// println!("{}", rng.gen::<f64>());
    /// ```
    fn from_seed(seed: Seed) -> Self;
}

/// An Xorshift[1] random number
/// generator.
///
/// The Xorshift algorithm is not suitable for cryptographic purposes
/// but is very fast. If you do not know for sure that it fits your
/// requirements, use a more secure one such as `IsaacRng` or `SgxRng`.
///
/// [1]: Marsaglia, George (July 2003). ["Xorshift
/// RNGs"](http://www.jstatsoft.org/v08/i14/paper). *Journal of
/// Statistical Software*. Vol. 8 (Issue 14).
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug)]
pub struct XorShiftRng {
    x: w32,
    y: w32,
    z: w32,
    w: w32,
}

impl XorShiftRng {
    /// Creates a new XorShiftRng instance which is not seeded.
    ///
    /// The initial values of this RNG are constants, so all generators created
    /// by this function will yield the same stream of random numbers. It is
    /// highly recommended that this is created through `SeedableRng` instead of
    /// this function
    pub fn new_unseeded() -> XorShiftRng {
        XorShiftRng {
            x: w(0x193a6754),
            y: w(0xa8a7d469),
            z: w(0x97830e05),
            w: w(0x113ba7bb),
        }
    }
}

impl Rng for XorShiftRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let x = self.x;
        let t = x ^ (x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        let w_ = self.w;
        self.w = w_ ^ (w_ >> 19) ^ (t ^ (t >> 8));
        self.w.0
    }
}

impl SeedableRng<[u32; 4]> for XorShiftRng {
    /// Reseed an XorShiftRng. This will panic if `seed` is entirely 0.
    fn reseed(&mut self, seed: [u32; 4]) {
        assert!(!seed.iter().all(|&x| x == 0),
                "XorShiftRng.reseed called with an all zero seed.");

        self.x = w(seed[0]);
        self.y = w(seed[1]);
        self.z = w(seed[2]);
        self.w = w(seed[3]);
    }

    /// Create a new XorShiftRng. This will panic if `seed` is entirely 0.
    fn from_seed(seed: [u32; 4]) -> XorShiftRng {
        assert!(!seed.iter().all(|&x| x == 0),
                "XorShiftRng::from_seed called with an all zero seed.");

        XorShiftRng {
            x: w(seed[0]),
            y: w(seed[1]),
            z: w(seed[2]),
            w: w(seed[3]),
        }
    }
}

impl Rand for XorShiftRng {
    fn rand<R: Rng>(rng: &mut R) -> XorShiftRng {
        let mut tuple: (u32, u32, u32, u32) = rng.gen();
        while tuple == (0, 0, 0, 0) {
            tuple = rng.gen();
        }
        let (x, y, z, w_) = tuple;
        XorShiftRng { x: w(x), y: w(y), z: w(z), w: w(w_) }
    }
}

/// A wrapper for generating floating point numbers uniformly in the
/// open interval `(0,1)` (not including either endpoint).
///
/// Use `Closed01` for the closed interval `[0,1]`, and the default
/// `Rand` implementation for `f32` and `f64` for the half-open
/// `[0,1)`.
///
/// # Example
/// ```rust
/// use sgx_rand::{random, Open01};
///
/// let Open01(val) = random::<Open01<f32>>();
/// println!("f32 from (0,1): {}", val);
/// ```
#[derive(Debug)]
pub struct Open01<F>(pub F);

/// A wrapper for generating floating point numbers uniformly in the
/// closed interval `[0,1]` (including both endpoints).
///
/// Use `Open01` for the closed interval `(0,1)`, and the default
/// `Rand` implementation of `f32` and `f64` for the half-open
/// `[0,1)`.
///
/// # Example
///
/// ```rust
/// use sgx_rand::{random, Closed01};
///
/// let Closed01(val) = random::<Closed01<f32>>();
/// println!("f32 from [0,1]: {}", val);
/// ```
#[derive(Debug)]
pub struct Closed01<F>(pub F);

/// The standard RNG. This is designed to be efficient on the current
/// platform.
#[derive(Copy, Clone, Debug)]
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

/// Create a weak random number generator with a default algorithm and seed.
///
/// It returns the fastest `Rng` algorithm currently available in Rust without
/// consideration for cryptography or security. If you require a specifically
/// seeded `Rng` for consistency over time you should pick one algorithm and
/// create the `Rng` yourself.
///
/// This will read randomness from the operating system to seed the
/// generator.
pub fn weak_rng() -> XorShiftRng {
    match SgxRng::new() {
        Ok(mut r) => r.gen(),
        Err(e) => panic!("weak_rng: failed to create seeded RNG: {:?}", e)
    }
}

/// Controls how the thread-local RNG is reseeded.
#[derive(Debug)]
struct ThreadRngReseeder;

impl reseeding::Reseeder<StdRng> for ThreadRngReseeder {
    fn reseed(&mut self, rng: &mut StdRng) {
        *rng = match StdRng::new() {
            Ok(r) => r,
            Err(e) => panic!("could not reseed thread_rng: {}", e)
        }
    }
}
const THREAD_RNG_RESEED_THRESHOLD: u64 = 32_768;
type ThreadRngInner = reseeding::ReseedingRng<StdRng, ThreadRngReseeder>;

/// The thread-local RNG.
#[derive(Clone, Debug)]
pub struct ThreadRng {
    rng: Rc<RefCell<ThreadRngInner>>,
}

/// Retrieve the lazily-initialized thread-local random number
/// generator, seeded by the system. Intended to be used in method
/// chaining style, e.g. `thread_rng().gen::<i32>()`.
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

/// Generates a random value using the thread-local random number generator.
///
/// `random()` can generate various types of random things, and so may require
/// type hinting to generate the specific type you want.
///
/// This function uses the thread local random number generator. This means
/// that if you're calling `random()` in a loop, caching the generator can
/// increase performance. An example is shown below.
///
/// # Examples
///
/// ```
/// let x = sgx_rand::random::<u8>();
/// println!("{}", x);
///
/// let y = sgx_rand::random::<f64>();
/// println!("{}", y);
///
/// if sgx_rand::random() { // generates a boolean
///     println!("Better lucky than good!");
/// }
/// ```
///
/// Caching the thread local random number generator:
///
/// ```
/// use sgx_rand::Rng;
///
/// let mut v = vec![1, 2, 3];
///
/// for x in v.iter_mut() {
///     *x = sgx_rand::random()
/// }
///
/// // would be faster as
///
/// let mut rng = sgx_rand::thread_rng();
///
/// for x in v.iter_mut() {
///     *x = rng.gen();
/// }
/// ```
#[inline]
pub fn random<T: Rand>() -> T {
    thread_rng().gen()
}

/// Randomly sample up to `amount` elements from a finite iterator.
/// The order of elements in the sample is not random.
///
/// # Example
///
/// ```rust
/// use sgx_rand::{thread_rng, sample};
///
/// let mut rng = thread_rng();
/// let sample = sample(&mut rng, 1..100, 5);
/// println!("{:?}", sample);
/// ```
pub fn sample<T, I, R>(rng: &mut R, iterable: I, amount: usize) -> Vec<T>
    where I: IntoIterator<Item=T>,
          R: Rng,
{
    let mut iter = iterable.into_iter();
    let mut reservoir: Vec<T> = iter.by_ref().take(amount).collect();
    // continue unless the iterator was exhausted
    if reservoir.len() == amount {
        for (i, elem) in iter.enumerate() {
            let k = rng.gen_range(0, i + 1 + amount);
            if let Some(spot) = reservoir.get_mut(k) {
                *spot = elem;
            }
        }
    }
    reservoir
}
