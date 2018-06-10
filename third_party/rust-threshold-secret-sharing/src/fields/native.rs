// Copyright (c) 2016 rust-threshold-secret-sharing developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Trivial native modular field.

use super::Field;


#[derive(Copy,Clone,Debug)]
pub struct Value(i64);

/// Trivial implementaion of Field using i64 values and performing a native
/// modulo reduction after each operation.
///
/// Actual values show not exceed the u32 or i32 ranges as multiplication
/// are performed "naively".
///
/// The mais purpose of this struct is to serve as a test reference to the 
/// more challenging implementations.
pub struct NativeField(i64);

impl Field for NativeField {
    type U = Value;

    fn new(prime: u64) -> NativeField {
        NativeField(prime as i64)
    }

    fn modulus(&self) -> u64 {
        self.0 as u64
    }

    fn from_u64(&self, a: u64) -> Self::U {
        Value(a as i64 % self.0)
    }

    fn to_u64(&self, a: Self::U) -> u64 {
        a.0 as u64
    }

    fn add(&self, a: Self::U, b: Self::U) -> Self::U {
        Value((a.0 + b.0) % self.0)
    }

    fn sub(&self, a: Self::U, b: Self::U) -> Self::U {
        let tmp = a.0 - b.0;
        if tmp > 0 {
            Value(tmp)
        } else {
            Value(tmp + self.0)
        }
    }

    fn mul(&self, a: Self::U, b: Self::U) -> Self::U {
        Value((a.0 * b.0) % self.0)
    }

    fn inv(&self, a: Self::U) -> Self::U {
        let tmp = ::numtheory::mod_inverse((a.0 % self.0) as i64, self.0 as i64);
        self.from_i64(tmp)
    }
}

#[cfg(test)]
all_fields_test!(NativeField);
