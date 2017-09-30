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

//! The exponential distribution.

use {Rng, Rand};
use distributions::{ziggurat, ziggurat_tables, Sample, IndependentSample};

/// A wrapper around an `f64` to generate Exp(1) random numbers.
///
/// See `Exp` for the general exponential distribution.
///
/// Implemented via the ZIGNOR variant[1] of the Ziggurat method. The
/// exact description in the paper was adjusted to use tables for the
/// exponential distribution rather than normal.
///
/// [1]: Jurgen A. Doornik (2005). [*An Improved Ziggurat Method to
/// Generate Normal Random
/// Samples*](http://www.doornik.com/research/ziggurat.pdf). Nuffield
/// College, Oxford
///
/// # Example
///
/// ```rust
/// use sgx_rand::distributions::exponential::Exp1;
///
/// let Exp1(x) = sgx_rand::random();
/// println!("{}", x);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Exp1(pub f64);

// This could be done via `-rng.gen::<f64>().ln()` but that is slower.
impl Rand for Exp1 {
    #[inline]
    fn rand<R:Rng>(rng: &mut R) -> Exp1 {
        #[inline]
        fn pdf(x: f64) -> f64 {
            (-x).exp()
        }
        #[inline]
        fn zero_case<R:Rng>(rng: &mut R, _u: f64) -> f64 {
            ziggurat_tables::ZIG_EXP_R - rng.gen::<f64>().ln()
        }

        Exp1(ziggurat(rng, false,
                      &ziggurat_tables::ZIG_EXP_X,
                      &ziggurat_tables::ZIG_EXP_F,
                      pdf, zero_case))
    }
}

/// The exponential distribution `Exp(lambda)`.
///
/// This distribution has density function: `f(x) = lambda *
/// exp(-lambda * x)` for `x > 0`.
///
/// # Example
///
/// ```rust
/// use sgx_rand::distributions::{Exp, IndependentSample};
///
/// let exp = Exp::new(2.0);
/// let v = exp.ind_sample(&mut sgx_rand::thread_rng());
/// println!("{} is from a Exp(2) distribution", v);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Exp {
    /// `lambda` stored as `1/lambda`, since this is what we scale by.
    lambda_inverse: f64
}

impl Exp {
    /// Construct a new `Exp` with the given shape parameter
    /// `lambda`. Panics if `lambda <= 0`.
    #[inline]
    pub fn new(lambda: f64) -> Exp {
        assert!(lambda > 0.0, "Exp::new called with `lambda` <= 0");
        Exp { lambda_inverse: 1.0 / lambda }
    }
}

impl Sample<f64> for Exp {
    fn sample<R: Rng>(&mut self, rng: &mut R) -> f64 { self.ind_sample(rng) }
}
impl IndependentSample<f64> for Exp {
    fn ind_sample<R: Rng>(&self, rng: &mut R) -> f64 {
        let Exp1(n) = rng.gen::<Exp1>();
        n * self.lambda_inverse
    }
}
