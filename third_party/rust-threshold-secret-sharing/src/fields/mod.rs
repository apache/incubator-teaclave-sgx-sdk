// Copyright (c) 2016 rust-threshold-secret-sharing developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! This module implements in-place 2-radix and 3-radix numeric theory
//! transformations (FFT on modular fields).

pub mod fft;

/// Abstract Field definition.
///
/// This trait is not meant to represent a general field in the strict
/// mathematical sense but it has everything we need to make the FFT to work.
pub trait Field {
    type U: Copy;

    /// Create a modular field for the given prime.
    ///
    /// In the current state of implementation, only values in the u32 range
    /// should be used.
    fn new(prime: u64) -> Self;

    /// Get the modulus.
    fn modulus(&self) -> u64;

    /// Convert a u64 to a modular integer.
    fn from_u64(&self, a: u64) -> Self::U;

    /// Convert a modular integer to u64 in the 0..modulus range.
    fn to_u64(&self, a: Self::U) -> u64;

    /// Convert a i64 to a modular integer.
    fn from_i64(&self, a: i64) -> Self::U {
        let a = a % self.modulus() as i64;
        if a >= 0 {
            self.from_u64(a as u64)
        } else {
            self.from_u64((a + self.modulus() as i64) as u64)
        }
    }

    /// Convert a modular integer to i64 in the -modulus/2..+modulus/2 range.
    fn to_i64(&self, a: Self::U) -> i64 {
        let a = self.to_u64(a);
        if a > self.modulus() / 2 {
            a as i64 - self.modulus() as i64
        } else {
            a as i64
        }
    }

    /// Get the Zero value.
    fn zero(&self) -> Self::U {
        self.from_u64(0)
    }

    /// Get the One value.
    fn one(&self) -> Self::U {
        self.from_u64(1)
    }

    /// Perfoms a modular addition.
    fn add(&self, a: Self::U, b: Self::U) -> Self::U;

    /// Perfoms a modular substraction.
    fn sub(&self, a: Self::U, b: Self::U) -> Self::U;

    /// Perfoms a modular multiplication.
    fn mul(&self, a: Self::U, b: Self::U) -> Self::U;

    /// Perfoms a modular inverse.
    fn inv(&self, a: Self::U) -> Self::U;

    /// Perfoms a modular exponentiation (x^e % modulus).
    ///
    /// Implements exponentiation by squaring.
    fn qpow(&self, mut x: Self::U, mut e: u32) -> Self::U {
        let mut acc = self.one();
        while e > 0 {
            if e % 2 == 0 {
                // even
                // no-op
            } else {
                // odd
                acc = self.mul(acc, x);
            }
            x = self.mul(x, x);  // waste one of these by having it here but code is simpler (tiny bit)
            e = e >> 1;
        }
        acc
    }
}

macro_rules! all_fields_test {
    ($field:ty) => {
        #[test] fn test_convert() { ::fields::test::test_convert::<$field>(); }
        #[test] fn test_add() { ::fields::test::test_add::<$field>(); }
        #[test] fn test_sub() { ::fields::test::test_sub::<$field>(); }
        #[test] fn test_mul() { ::fields::test::test_mul::<$field>(); }
        #[test] fn test_qpow() { ::fields::test::test_qpow::<$field>(); }
        #[test] fn test_fft2() { ::fields::fft::test::test_fft2::<$field>(); }
        #[test] fn test_fft2_inverse() { ::fields::fft::test::test_fft2_inverse::<$field>(); }
        #[test] fn test_fft2_big() { ::fields::fft::test::test_fft2_big::<$field>(); }
        #[test] fn test_fft3() { ::fields::fft::test::test_fft3::<$field>(); }
        #[test] fn test_fft3_inverse() { ::fields::fft::test::test_fft3_inverse::<$field>(); }
        #[test] fn test_fft3_big() { ::fields::fft::test::test_fft3_big::<$field>(); }
    }
}

pub mod native;
pub mod montgomery;

#[cfg(test)]
pub mod test {
    use super::Field;

    pub fn test_convert<F: Field>() {
        let zp = F::new(17);
        for i in 0u64..20 {
            assert_eq!(zp.to_u64(zp.from_u64(i)), i % 17);
        }
    }

    pub fn test_add<F: Field>() {
        let zp = F::new(17);
        assert_eq!(zp.to_u64(zp.add(zp.from_u64(8), zp.from_u64(2))), 10);
        assert_eq!(zp.to_u64(zp.add(zp.from_u64(8), zp.from_u64(13))), 4);
    }

    pub fn test_sub<F: Field>() {
        let zp = F::new(17);
        assert_eq!(zp.to_u64(zp.sub(zp.from_u64(8), zp.from_u64(2))), 6);
        assert_eq!(zp.to_u64(zp.sub(zp.from_u64(8), zp.from_u64(13))),
                   (17 + 8 - 13) % 17);
    }

    pub fn test_mul<F: Field>() {
        let zp = F::new(17);
        assert_eq!(zp.to_u64(zp.mul(zp.from_u64(8), zp.from_u64(2))),
                   (8 * 2) % 17);
        assert_eq!(zp.to_u64(zp.mul(zp.from_u64(8), zp.from_u64(5))),
                   (8 * 5) % 17);
    }

    pub fn test_qpow<F: Field>() {
        let zp = F::new(17);
        assert_eq!(zp.to_u64(zp.qpow(zp.from_u64(2), 0)), 1);
        assert_eq!(zp.to_u64(zp.qpow(zp.from_u64(2), 3)), 8);
        assert_eq!(zp.to_u64(zp.qpow(zp.from_u64(2), 6)), 13);
    }
}
