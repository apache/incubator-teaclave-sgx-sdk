// Copyright (c) 2016 rust-threshold-secret-sharing developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Montgomery modular multiplication field.

use super::Field;

/// MontgomeryField32 Value (wraps an u32 for type-safety).
#[derive(Copy,Clone,Debug)]
pub struct Value(u32);

/// Implementation of Field with Montgomery modular multiplication.
///
/// See https://en.wikipedia.org/wiki/Montgomery_modular_multiplication
/// for general description of the scheme, or
/// http://www.hackersdelight.org/MontgomeryMultiplication.pdf for
/// implementation notes.
///
/// This implementation assumes R=2^32. In other terms, the modulus must be
/// in the u32 range. All values will be positive, in the 0..modulus range,
/// and represented by a u32.
pub struct MontgomeryField32 {
    pub n: u32, // the prime
    pub n_quote: u32,
    pub r_inv: u32, // r = 2^32
    pub r_cube: u32, // r^3 is used by inv()
}

impl MontgomeryField32 {
    pub fn new(prime: u32) -> MontgomeryField32 {
        let r = 1u64 << 32;
        let tmp = ::numtheory::mod_inverse(r as i64, prime as i64);
        let r_inv = if tmp < 0 {
            (tmp + prime as i64) as u32
        } else {
            tmp as u32
        };
        let tmp = ::numtheory::mod_inverse(prime as i64, r as i64);
        let n_quote = if tmp > 0 {
            (r as i64 - tmp) as u32
        } else {
            (r as i64 - tmp) as u32
        };
        let r_cube = ::numtheory::mod_pow(r as i64 % prime as i64, 3u32, prime as i64);
        MontgomeryField32 {
            n: prime,
            r_inv: r_inv,
            n_quote: n_quote,
            r_cube: r_cube as u32,
        }
    }

    fn redc(&self, a: u64) -> Value {
        let m: u64 = (a as u32).wrapping_mul(self.n_quote) as u64;
        let t: u32 = ((a + m * (self.n as u64)) >> 32) as u32;
        Value((if t >= (self.n) { t - (self.n) } else { t }))
    }
}

impl Field for MontgomeryField32 {
    type U = Value;

    fn modulus(&self) -> u64 {
        self.n as u64
    }

    fn add(&self, a: Self::U, b: Self::U) -> Self::U {
        let sum = a.0 as u64 + b.0 as u64;
        if sum > self.n as u64 {
            Value((sum - self.n as u64) as u32)
        } else {
            Value(sum as u32)
        }
    }

    fn sub(&self, a: Self::U, b: Self::U) -> Self::U {
        if a.0 > b.0 {
            Value(a.0 - b.0)
        } else {
            Value((a.0 as u64 + self.n as u64 - b.0 as u64) as u32)
        }
    }

    fn mul(&self, a: Self::U, b: Self::U) -> Self::U {
        self.redc((a.0 as u64).wrapping_mul(b.0 as u64))
    }

    fn inv(&self, a: Self::U) -> Self::U {
        let ar_modn_inv = ::numtheory::mod_inverse(a.0 as i64, self.n as i64);
        self.redc((ar_modn_inv as u64).wrapping_mul(self.r_cube as u64))
    }

    fn new(prime: u64) -> MontgomeryField32 {
        MontgomeryField32::new(prime as u32)
    }

    fn from_u64(&self, a: u64) -> Self::U {
        Value(((a << 32) % self.n as u64) as u32)
    }

    fn to_u64(&self, a: Self::U) -> u64 {
        a.0 as u64 * self.r_inv as u64 % self.n as u64
    }
}

#[cfg(test)]
all_fields_test!(MontgomeryField32);
