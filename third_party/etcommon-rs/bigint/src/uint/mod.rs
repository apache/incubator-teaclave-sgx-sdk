// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Code derived from original work by Andrew Poelstra <apoelstra@wpsoftware.net>

// Rust Bitcoin Library
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! Big unsigned integer types.
//!
//! Implementation of a various large-but-fixed sized unsigned integer types.
//! The functions here are designed to be fast. There are optional `x86_64`
//! implementations for even more speed, hidden behind the `x64_arithmetic`
//! feature flag.

#[cfg(feature = "std")] use std::fmt;
#[cfg(feature = "std")] use std::str::FromStr;
#[cfg(feature = "std")] use std::ops::{Shr, Shl, BitAnd, BitOr, BitXor, Not, Div, Rem, Mul, Add, Sub};
#[cfg(feature = "std")] use std::cmp::Ordering;

#[cfg(not(feature = "std"))] use core::fmt;
#[cfg(not(feature = "std"))] use core::str::FromStr;
#[cfg(not(feature = "std"))] use core::ops::{Shr, Shl, BitAnd, BitOr, BitXor, Not, Div, Rem, Mul, Add, Sub};
#[cfg(not(feature = "std"))] use core::cmp::Ordering;

#[cfg(all(not(feature = "std"), feature = "string"))]
use alloc::String;

use byteorder::{ByteOrder, BigEndian, LittleEndian};
#[cfg(feature = "string")]
use hexutil::{ParseHexError, read_hex};

#[cfg(feature = "rlp")]
mod rlp;

/// Conversion from decimal string error
#[derive(Debug, PartialEq)]
pub enum FromDecStrErr {
    /// Char not from range 0-9
    InvalidCharacter,
    /// Value does not fit into type
    InvalidLength,
}

macro_rules! impl_map_from {
	($thing:ident, $from:ty, $to:ty) => {
		impl From<$from> for $thing {
			fn from(value: $from) -> $thing {
				From::from(value as $to)
			}
		}
	}
}

#[cfg(not(all(asm_available, target_arch="x86_64")))]
macro_rules! uint_overflowing_add {
	($name:ident, $n_words:expr, $self_expr: expr, $other: expr) => ({
		uint_overflowing_add_reg!($name, $n_words, $self_expr, $other)
	})
}

macro_rules! uint_overflowing_add_reg {
	($name:ident, $n_words:expr, $self_expr: expr, $other: expr) => ({
		let $name(ref me) = $self_expr;
		let $name(ref you) = $other;

		let mut ret = [0u64; $n_words];
		let mut carry = 0u64;

		for i in 0..$n_words {
			let (res1, overflow1) = me[i].overflowing_add(you[i]);
			let (res2, overflow2) = res1.overflowing_add(carry);

			ret[i] = res2;
			carry = overflow1 as u64 + overflow2 as u64;
		}

		($name(ret), carry > 0)
	})
}

#[cfg(all(asm_available, target_arch="x86_64"))]
macro_rules! uint_overflowing_add {
	(U256, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; $n_words] = unsafe { ::std::mem::uninitialized() };
		let self_t: &[u64; $n_words] = &$self_expr.0;
		let other_t: &[u64; $n_words] = &$other.0;

		let overflow: u8;
		unsafe {
			asm!("
				add $9, $0
				adc $10, $1
				adc $11, $2
				adc $12, $3
				setc %al
				"
			: "=r"(result[0]), "=r"(result[1]), "=r"(result[2]), "=r"(result[3]), "={al}"(overflow)
			: "0"(self_t[0]), "1"(self_t[1]), "2"(self_t[2]), "3"(self_t[3]),
			  "mr"(other_t[0]), "mr"(other_t[1]), "mr"(other_t[2]), "mr"(other_t[3])
			:
			:
			);
		}
		(U256(result), overflow != 0)
	});
	(U512, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; $n_words] = unsafe { ::std::mem::uninitialized() };
		let self_t: &[u64; $n_words] = &$self_expr.0;
		let other_t: &[u64; $n_words] = &$other.0;

		let overflow: u8;

		unsafe {
			asm!("
				add $15, $0
				adc $16, $1
				adc $17, $2
				adc $18, $3
				lodsq
				adc $11, %rax
				stosq
				lodsq
				adc $12, %rax
				stosq
				lodsq
				adc $13, %rax
				stosq
				lodsq
				adc $14, %rax
				stosq
				setc %al

				": "=r"(result[0]), "=r"(result[1]), "=r"(result[2]), "=r"(result[3]),

			  "={al}"(overflow) /* $0 - $4 */

            : "{rdi}"(&result[4] as *const u64) /* $5 */
			  "{rsi}"(&other_t[4] as *const u64) /* $6 */
			  "0"(self_t[0]), "1"(self_t[1]), "2"(self_t[2]), "3"(self_t[3]),
		  	  "m"(self_t[4]), "m"(self_t[5]), "m"(self_t[6]), "m"(self_t[7]),
			  /* $7 - $14 */

			  "mr"(other_t[0]), "mr"(other_t[1]), "mr"(other_t[2]), "mr"(other_t[3]),
              "m"(other_t[4]), "m"(other_t[5]), "m"(other_t[6]), "m"(other_t[7]) /* $15 - $22 */
			: "rdi", "rsi"
			:
			);
		}
		(U512(result), overflow != 0)
	});

	($name:ident, $n_words:expr, $self_expr: expr, $other: expr) => (
		uint_overflowing_add_reg!($name, $n_words, $self_expr, $other)
	)
}

#[cfg(not(all(asm_available, target_arch="x86_64")))]
macro_rules! uint_overflowing_sub {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		uint_overflowing_sub_reg!($name, $n_words, $self_expr, $other)
	})
}

macro_rules! uint_overflowing_sub_reg {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let $name(ref me) = $self_expr;
		let $name(ref you) = $other;

		let mut ret = [0u64; $n_words];
		let mut carry = 0u64;

		for i in 0..$n_words {
			let (res1, overflow1) = me[i].overflowing_sub(you[i]);
			let (res2, overflow2) = res1.overflowing_sub(carry);

			ret[i] = res2;
			carry = overflow1 as u64 + overflow2 as u64;
		}

		($name(ret), carry > 0)

	})
}

#[cfg(all(asm_available, target_arch="x86_64"))]
macro_rules! uint_overflowing_sub {
	(U256, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; $n_words] = unsafe { ::std::mem::uninitialized() };
		let self_t: &[u64; $n_words] = &$self_expr.0;
		let other_t: &[u64; $n_words] = &$other.0;

		let overflow: u8;
		unsafe {
			asm!("
				sub $9, $0
				sbb $10, $1
				sbb $11, $2
				sbb $12, $3
				setb %al
				"
				: "=r"(result[0]), "=r"(result[1]), "=r"(result[2]), "=r"(result[3]), "={al}"(overflow)
				: "0"(self_t[0]), "1"(self_t[1]), "2"(self_t[2]), "3"(self_t[3]), "mr"(other_t[0]), "mr"(other_t[1]), "mr"(other_t[2]), "mr"(other_t[3])
				:
				:
			);
		}
		(U256(result), overflow != 0)
	});
	(U512, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; $n_words] = unsafe { ::std::mem::uninitialized() };
		let self_t: &[u64; $n_words] = &$self_expr.0;
		let other_t: &[u64; $n_words] = &$other.0;

		let overflow: u8;

		unsafe {
			asm!("
				sub $15, $0
				sbb $16, $1
				sbb $17, $2
				sbb $18, $3
				lodsq
				sbb $19, %rax
				stosq
				lodsq
				sbb $20, %rax
				stosq
				lodsq
				sbb $21, %rax
				stosq
				lodsq
				sbb $22, %rax
				stosq
				setb %al
				"
			: "=r"(result[0]), "=r"(result[1]), "=r"(result[2]), "=r"(result[3]),

			  "={al}"(overflow) /* $0 - $4 */

			: "{rdi}"(&result[4] as *const u64) /* $5 */
		 	 "{rsi}"(&self_t[4] as *const u64) /* $6 */
			  "0"(self_t[0]), "1"(self_t[1]), "2"(self_t[2]), "3"(self_t[3]),
			  "m"(self_t[4]), "m"(self_t[5]), "m"(self_t[6]), "m"(self_t[7]),
			  /* $7 - $14 */

			  "m"(other_t[0]), "m"(other_t[1]), "m"(other_t[2]), "m"(other_t[3]),
			  "m"(other_t[4]), "m"(other_t[5]), "m"(other_t[6]), "m"(other_t[7]) /* $15 - $22 */
			: "rdi", "rsi"
			:
			);
		}
		(U512(result), overflow != 0)
	});
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		uint_overflowing_sub_reg!($name, $n_words, $self_expr, $other)
	})
}

#[cfg(all(asm_available, target_arch="x86_64"))]
macro_rules! uint_overflowing_mul {
	(U256, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; $n_words] = unsafe { ::std::mem::uninitialized() };
		let self_t: &[u64; $n_words] = &$self_expr.0;
		let other_t: &[u64; $n_words] = &$other.0;

		let overflow: u64;
		unsafe {
			asm!("
				mov $5, %rax
				mulq $9
				mov %rax, $0
				mov %rdx, $1

				mov $5, %rax
				mulq $10
				add %rax, $1
				adc $$0, %rdx
				mov %rdx, $2

				mov $5, %rax
				mulq $11
				add %rax, $2
				adc $$0, %rdx
				mov %rdx, $3

				mov $5, %rax
				mulq $12
				add %rax, $3
				adc $$0, %rdx
				mov %rdx, %rcx

				mov $6, %rax
				mulq $9
				add %rax, $1
				adc %rdx, $2
				adc $$0, $3
				adc $$0, %rcx

				mov $6, %rax
				mulq $10
				add %rax, $2
				adc %rdx, $3
				adc $$0, %rcx
				adc $$0, $3
				adc $$0, %rcx

				mov $6, %rax
				mulq $11
				add %rax, $3
				adc $$0, %rdx
				or %rdx, %rcx

				mov $7, %rax
				mulq $9
				add %rax, $2
				adc %rdx, $3
				adc $$0, %rcx

				mov $7, %rax
				mulq $10
				add %rax, $3
				adc $$0, %rdx
				or %rdx, %rcx

				mov $8, %rax
				mulq $9
				add %rax, $3
				or %rdx, %rcx

				cmpq $$0, %rcx
				jne 2f

				mov $8, %rcx
				jrcxz 12f

				mov $12, %rcx
				mov $11, %rax
				or %rax, %rcx
				mov $10, %rax
				or %rax, %rcx
				jmp 2f

				12:
				mov $12, %rcx
				jrcxz 11f

				mov $7, %rcx
				mov $6, %rax
				or %rax, %rcx

				cmpq $$0, %rcx
				jne 2f

				11:
				mov $11, %rcx
				jrcxz 2f
				mov $7, %rcx

				2:
				"
				: /* $0 */ "={r8}"(result[0]), /* $1 */ "={r9}"(result[1]), /* $2 */ "={r10}"(result[2]),
				  /* $3 */ "={r11}"(result[3]), /* $4 */  "={rcx}"(overflow)

				: /* $5 */ "m"(self_t[0]), /* $6 */ "m"(self_t[1]), /* $7 */  "m"(self_t[2]),
				  /* $8 */ "m"(self_t[3]), /* $9 */ "m"(other_t[0]), /* $10 */ "m"(other_t[1]),
				  /* $11 */ "m"(other_t[2]), /* $12 */ "m"(other_t[3])
           		: "rax", "rdx"
				:

			);
		}
		(U256(result), overflow > 0)
	});
	($name:ident, $n_words:expr, $self_expr: expr, $other: expr) => (
		uint_overflowing_mul_reg!($name, $n_words, $self_expr, $other)
	)
}

#[cfg(not(all(asm_available, target_arch="x86_64")))]
macro_rules! uint_overflowing_mul {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		uint_overflowing_mul_reg!($name, $n_words, $self_expr, $other)
	})
}

macro_rules! uint_overflowing_mul_reg {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let $name(ref me) = $self_expr;
		let $name(ref you) = $other;
		let mut ret = [0u64; 2*$n_words];

		for i in 0..$n_words {
			if you[i] == 0 {
				continue;
			}

			let mut carry2 = 0u64;
			let (b_u, b_l) = split(you[i]);

			for j in 0..$n_words {
				if me[j] == 0 && carry2 == 0 {
					continue;
				}

				let a = split(me[j]);

				// multiply parts
				let (c_l, overflow_l) = mul_u32(a, b_l, ret[i + j]);
				let (c_u, overflow_u) = mul_u32(a, b_u, c_l >> 32);
				ret[i + j] = (c_l & 0xFFFFFFFF) + (c_u << 32);

				// No overflow here
				let res = (c_u >> 32) + (overflow_u << 32);
				// possible overflows
				let (res, o1) = res.overflowing_add(overflow_l);
				let (res, o2) = res.overflowing_add(carry2);
				let (res, o3) = res.overflowing_add(ret[i + j + 1]);
				ret[i + j + 1] = res;

				// Only single overflow possible there
				carry2 = (o1 | o2 | o3) as u64;
			}
		}

		let mut res = [0u64; $n_words];
		let mut overflow = false;
		for i in 0..$n_words {
			res[i] = ret[i];
		}

		for i in $n_words..2*$n_words {
			overflow |= ret[i] != 0;
		}

		($name(res), overflow)
	})
}

macro_rules! overflowing {
	($op: expr, $overflow: expr) => (
		{
			let (overflow_x, overflow_overflow) = $op;
			$overflow |= overflow_overflow;
			overflow_x
		}
	);
	($op: expr) => (
		{
			let (overflow_x, _overflow_overflow) = $op;
			overflow_x
		}
	);
}

macro_rules! panic_on_overflow {
	($name: expr) => {
		if $name {
			panic!("arithmetic operation overflow")
		}
	}
}

#[inline(always)]
fn mul_u32(a: (u64, u64), b: u64, carry: u64) -> (u64, u64) {
    let upper = b * a.0;
    let lower = b * a.1;

    let (res1, overflow1) = lower.overflowing_add(upper << 32);
    let (res2, overflow2) = res1.overflowing_add(carry);

    let carry = (upper >> 32) + overflow1 as u64 + overflow2 as u64;
    (res2, carry)
}

#[inline(always)]
fn split(a: u64) -> (u64, u64) {
    (a >> 32, a & 0xFFFFFFFF)
}

macro_rules! construct_uint {
	($name:ident, $n_words:expr) => (
		/// Little-endian large integer type
		#[repr(C)]
		#[derive(Copy, Clone, Eq, PartialEq, Hash)]
		pub struct $name(pub [u64; $n_words]);

		impl $name {
			/// Convert from a decimal string.
			pub fn from_dec_str(value: &str) -> Result<Self, FromDecStrErr> {
				if !value.bytes().all(|b| b >= 48 && b <= 57) {
					return Err(FromDecStrErr::InvalidCharacter)
				}

				let mut res = Self::default();
				for b in value.bytes().map(|b| b - 48) {
					let (r, overflow) = res.overflowing_mul_u32(10);
					if overflow {
						return Err(FromDecStrErr::InvalidLength);
					}
					let (r, overflow) = r.overflowing_add(b.into());
					if overflow {
						return Err(FromDecStrErr::InvalidLength);
					}
					res = r;
				}
				Ok(res)
			}

			/// Conversion to u32
			#[inline]
			pub fn low_u32(&self) -> u32 {
				let &$name(ref arr) = self;
				arr[0] as u32
			}

			/// Conversion to u64
			#[inline]
			pub fn low_u64(&self) -> u64 {
				let &$name(ref arr) = self;
				arr[0]
			}

			/// Conversion to u32 with overflow checking
			///
			/// # Panics
			///
			/// Panics if the number is larger than 2^32.
			#[inline]
			pub fn as_u32(&self) -> u32 {
				let &$name(ref arr) = self;
				if (arr[0] & (0xffffffffu64 << 32)) != 0 {
					panic!("Integer overflow when casting U256")
				}
				self.as_u64() as u32
			}

			/// Conversion to u64 with overflow checking
			///
			/// # Panics
			///
			/// Panics if the number is larger than 2^64.
			#[inline]
			pub fn as_u64(&self) -> u64 {
				let &$name(ref arr) = self;
				for i in 1..$n_words {
					if arr[i] != 0 {
						panic!("Integer overflow when casting U256")
					}
				}
				arr[0]
			}

		    /// Conversion to usize with overflow checking
			///
			/// # Panics
			///
			/// Panics if the number is larger than usize::max_value().
            #[inline]
            pub fn as_usize(&self) -> usize {
                #[cfg(feature = "std")]
                use std::mem::size_of;

                #[cfg(not(feature = "std"))]
                use core::mem::size_of;

                if size_of::<usize>() > size_of::<u64>() && size_of::<usize>() < size_of::<u32>() {
                    panic!("Unsupported platform")
                }
                if size_of::<usize>() == size_of::<u64>() {
                    self.as_u64() as usize
                } else {
                    self.as_u32() as usize
                }
            }

			/// Whether this is zero.
			#[inline]
			pub fn is_zero(&self) -> bool {
				let &$name(ref arr) = self;
				for i in 0..$n_words { if arr[i] != 0 { return false; } }
				return true;
			}

			/// Return the least number of bits needed to represent the number
			#[inline]
			pub fn bits(&self) -> usize {
				let &$name(ref arr) = self;
				for i in 1..$n_words {
					if arr[$n_words - i] > 0 { return (0x40 * ($n_words - i + 1)) - arr[$n_words - i].leading_zeros() as usize; }
				}
				0x40 - arr[0].leading_zeros() as usize
			}

			/// Return if specific bit is set.
			///
			/// # Panics
			///
			/// Panics if `index` exceeds the bit width of the number.
			#[inline]
			pub fn bit(&self, index: usize) -> bool {
				let &$name(ref arr) = self;
				arr[index / 64] & (1 << (index % 64)) != 0
			}

			/// Return specific byte.
			///
			/// # Panics
			///
			/// Panics if `index` exceeds the byte width of the number.
			#[inline]
			pub fn byte(&self, index: usize) -> u8 {
				let &$name(ref arr) = self;
				(arr[index / 8] >> (((index % 8)) * 8)) as u8
			}

            /// Return specific byte in big-endian format.
            ///
			/// # Panics
			///
			/// Panics if `index` exceeds the byte width of the number.
            #[inline]
            pub fn index(&self, index: usize) -> u8 {
                let index = $n_words * 8 - 1 - index;
                self.byte(index)
            }

			/// Write to the slice in big-endian format.
			#[inline]
			pub fn to_big_endian(&self, bytes: &mut [u8]) {
				debug_assert!($n_words * 8 == bytes.len());
				for i in 0..$n_words {
					BigEndian::write_u64(&mut bytes[8 * i..], self.0[$n_words - i - 1]);
				}
			}

			/// Write to the slice in little-endian format.
			#[inline]
			pub fn to_little_endian(&self, bytes: &mut [u8]) {
				debug_assert!($n_words * 8 == bytes.len());
				for i in 0..$n_words {
					LittleEndian::write_u64(&mut bytes[8 * i..], self.0[i]);
				}
			}

			/// Create `10**n` as this type.
			///
			/// # Panics
			///
			/// Panics if the result overflows the type.
			#[inline]
			pub fn exp10(n: usize) -> Self {
				match n {
					0 => Self::from(1u64),
					_ => Self::exp10(n - 1).mul_u32(10)
				}
			}

			/// Zero (additive identity) of this type.
			#[inline]
			pub fn zero() -> Self {
				From::from(0u64)
			}

			/// One (multiplicative identity) of this type.
			#[inline]
			pub fn one() -> Self {
				From::from(1u64)
			}

			/// The maximum value which can be inhabited by this type.
			#[inline]
			pub fn max_value() -> Self {
				let mut result = [0; $n_words];
				for i in 0..$n_words {
					result[i] = u64::max_value();
				}
				$name(result)
			}

			/// The minimum value which can be inhabited by this type.
			#[inline]
			pub fn min_value() -> Self {
			    $name([0; $n_words])
			}

			/// Fast exponentation by squaring
			/// https://en.wikipedia.org/wiki/Exponentiation_by_squaring
			///
			/// # Panics
			///
			/// Panics if the result overflows the type.
			pub fn pow(self, expon: Self) -> Self {
				if expon.is_zero() {
					return Self::one()
				}
				let is_even = |x : &Self| x.low_u64() & 1 == 0;

				let u_one = Self::one();
				let mut y = u_one;
				let mut n = expon;
				let mut x = self;
				while n > u_one {
					if is_even(&n) {
						x = x * x;
						n = n >> 1;
					} else {
						y = x * y;
						x = x * x;
						// to reduce odd number by 1 we should just clear the last bit
						n.0[$n_words-1] = n.0[$n_words-1] & ((!0u64)>>1);
						n = n >> 1;
					}
				}
				x * y
			}

			/// Fast exponentation by squaring
			/// https://en.wikipedia.org/wiki/Exponentiation_by_squaring
			pub fn overflowing_pow(self, expon: Self) -> (Self, bool) {
				if expon.is_zero() { return (Self::one(), false) }

				let is_even = |x : &Self| x.low_u64() & 1 == 0;

				let u_one = Self::one();
				let mut y = u_one;
				let mut n = expon;
				let mut x = self;
				let mut overflow = false;

				while n > u_one {
					if is_even(&n) {
						x = overflowing!(x.overflowing_mul(x), overflow);
						n = n >> 1;
					} else {
						y = overflowing!(x.overflowing_mul(y), overflow);
						x = overflowing!(x.overflowing_mul(x), overflow);
						n = (n - u_one) >> 1;
					}
				}
				let res = overflowing!(x.overflowing_mul(y), overflow);
				(res, overflow)
			}

			/// Optimized instructions
			#[inline(always)]
			pub fn overflowing_add(self, other: $name) -> ($name, bool) {
				uint_overflowing_add!($name, $n_words, self, other)
			}

			/// Addition which saturates at the maximum value.
			pub fn saturating_add(self, other: $name) -> $name {
				match self.overflowing_add(other) {
					(_, true) => $name::max_value(),
					(val, false) => val,
				}
			}

			/// Subtraction which underflows and returns a flag if it does.
			#[inline(always)]
			pub fn overflowing_sub(self, other: $name) -> ($name, bool) {
				uint_overflowing_sub!($name, $n_words, self, other)
			}

			/// Subtraction which saturates at zero.
			pub fn saturating_sub(self, other: $name) -> $name {
				match self.overflowing_sub(other) {
					(_, true) => $name::zero(),
					(val, false) => val,
				}
			}

			/// Multiply with overflow, returning a flag if it does.
			#[inline(always)]
			pub fn overflowing_mul(self, other: $name) -> ($name, bool) {
				uint_overflowing_mul!($name, $n_words, self, other)
			}

			/// Multiplication which saturates at the maximum value..
			pub fn saturating_mul(self, other: $name) -> $name {
				match self.overflowing_mul(other) {
					(_, true) => $name::max_value(),
					(val, false) => val,
				}
			}

			/// Division with overflow
			pub fn overflowing_div(self, other: $name) -> ($name, bool) {
				(self / other, false)
			}

			/// Modulus with overflow.
			pub fn overflowing_rem(self, other: $name) -> ($name, bool) {
				(self % other, false)
			}

			/// Negation with overflow.
			pub fn overflowing_neg(self) -> ($name, bool) {
				(!self, true)
			}
		}

		impl $name {
			/// Multiplication by u32
			#[allow(dead_code)] // not used when multiplied with inline assembly
			fn mul_u32(self, other: u32) -> Self {
				let (ret, overflow) = self.overflowing_mul_u32(other);
				panic_on_overflow!(overflow);
				ret
			}

			/// Overflowing multiplication by u32
			#[allow(dead_code)] // not used when multiplied with inline assembly
			fn overflowing_mul_u32(self, other: u32) -> (Self, bool) {
				let $name(ref arr) = self;
				let mut ret = [0u64; $n_words];
				let mut carry = 0;
				let o = other as u64;

				for i in 0..$n_words {
					let (res, carry2) = mul_u32(split(arr[i]), o, carry);
					ret[i] = res;
					carry = carry2;
				}

				($name(ret), carry > 0)
			}
		}

		impl Default for $name {
			fn default() -> Self {
				$name::zero()
			}
		}

		impl From<u64> for $name {
			fn from(value: u64) -> $name {
				let mut ret = [0; $n_words];
				ret[0] = value;
				$name(ret)
			}
		}


		impl_map_from!($name, u8, u64);
		impl_map_from!($name, u16, u64);
		impl_map_from!($name, u32, u64);
		impl_map_from!($name, usize, u64);

		impl From<i64> for $name {
			fn from(value: i64) -> $name {
				match value >= 0 {
					true => From::from(value as u64),
					false => { panic!("Unsigned integer can't be created from negative value"); }
				}
			}
		}

		impl_map_from!($name, i8, i64);
		impl_map_from!($name, i16, i64);
		impl_map_from!($name, i32, i64);
		impl_map_from!($name, isize, i64);

		impl<'a> From<&'a [u8]> for $name {
			fn from(bytes: &[u8]) -> $name {
				assert!($n_words * 8 >= bytes.len());

				let mut ret = [0; $n_words];
				for i in 0..bytes.len() {
					let rev = bytes.len() - 1 - i;
					let pos = rev / 8;
					ret[pos] += (bytes[i] as u64) << ((rev % 8) * 8);
				}
				$name(ret)
			}
		}

        #[cfg(feature = "string")]
		impl FromStr for $name {
			type Err = ParseHexError;

			fn from_str(value: &str) -> Result<$name, Self::Err> {
			    read_hex(value).map(|s| {
			        let z: &[u8] = s.as_ref();
			        $name::from(z)
			    })
			}
		}

		impl Add<$name> for $name {
			type Output = $name;

			fn add(self, other: $name) -> $name {
				let (result, overflow) = self.overflowing_add(other);
				panic_on_overflow!(overflow);
				result
			}
		}

		impl Sub<$name> for $name {
			type Output = $name;

			#[inline]
			fn sub(self, other: $name) -> $name {
				let (result, overflow) = self.overflowing_sub(other);
				panic_on_overflow!(overflow);
				result
			}
		}

		impl Mul<$name> for $name {
			type Output = $name;

			fn mul(self, other: $name) -> $name {
				let (result, overflow) = self.overflowing_mul(other);
				panic_on_overflow!(overflow);
				result
			}
		}

		impl Div<$name> for $name {
			type Output = $name;

			fn div(self, other: $name) -> $name {
				let mut sub_copy = self;
				let mut shift_copy = other;
				let mut ret = [0u64; $n_words];

				let my_bits = self.bits();
				let your_bits = other.bits();

				// Check for division by 0
				assert!(your_bits != 0);

				// Early return in case we are dividing by a larger number than us
				if my_bits < your_bits {
					return $name(ret);
				}

				// Bitwise long division
				let mut shift = my_bits - your_bits;
				shift_copy = shift_copy << shift;
				loop {
					if sub_copy >= shift_copy {
						ret[shift / 64] |= 1 << (shift % 64);
						sub_copy = overflowing!(sub_copy.overflowing_sub(shift_copy));
					}
					shift_copy = shift_copy >> 1;
					if shift == 0 { break; }
					shift -= 1;
				}

				$name(ret)
			}
		}

		impl Rem<$name> for $name {
			type Output = $name;

			fn rem(self, other: $name) -> $name {
				let times = self / other;
				self - (times * other)
			}
		}

		impl BitAnd<$name> for $name {
			type Output = $name;

			#[inline]
			fn bitand(self, other: $name) -> $name {
				let $name(ref arr1) = self;
				let $name(ref arr2) = other;
				let mut ret = [0u64; $n_words];
				for i in 0..$n_words {
					ret[i] = arr1[i] & arr2[i];
				}
				$name(ret)
			}
		}

		impl BitXor<$name> for $name {
			type Output = $name;

			#[inline]
			fn bitxor(self, other: $name) -> $name {
				let $name(ref arr1) = self;
				let $name(ref arr2) = other;
				let mut ret = [0u64; $n_words];
				for i in 0..$n_words {
					ret[i] = arr1[i] ^ arr2[i];
				}
				$name(ret)
			}
		}

		impl BitOr<$name> for $name {
			type Output = $name;

			#[inline]
			fn bitor(self, other: $name) -> $name {
				let $name(ref arr1) = self;
				let $name(ref arr2) = other;
				let mut ret = [0u64; $n_words];
				for i in 0..$n_words {
					ret[i] = arr1[i] | arr2[i];
				}
				$name(ret)
			}
		}

		impl Not for $name {
			type Output = $name;

			#[inline]
			fn not(self) -> $name {
				let $name(ref arr) = self;
				let mut ret = [0u64; $n_words];
				for i in 0..$n_words {
					ret[i] = !arr[i];
				}
				$name(ret)
			}
		}

		impl Shl<usize> for $name {
			type Output = $name;

			fn shl(self, shift: usize) -> $name {
				let $name(ref original) = self;
				let mut ret = [0u64; $n_words];
				let word_shift = shift / 64;
				let bit_shift = shift % 64;

				// shift
				for i in word_shift..$n_words {
					ret[i] = original[i - word_shift] << bit_shift;
				}
				// carry
				if bit_shift > 0 {
					for i in word_shift+1..$n_words {
						ret[i] += original[i - 1 - word_shift] >> (64 - bit_shift);
					}
				}
				$name(ret)
			}
		}

		impl Shr<usize> for $name {
			type Output = $name;

			fn shr(self, shift: usize) -> $name {
				let $name(ref original) = self;
				let mut ret = [0u64; $n_words];
				let word_shift = shift / 64;
				let bit_shift = shift % 64;

				// shift
				for i in word_shift..$n_words {
					ret[i - word_shift] = original[i] >> bit_shift;
				}

				// Carry
				if bit_shift > 0 {
					for i in word_shift+1..$n_words {
						ret[i - word_shift - 1] += original[i] << (64 - bit_shift);
					}
				}

				$name(ret)
			}
		}

		impl Ord for $name {
			fn cmp(&self, other: &$name) -> Ordering {
				let &$name(ref me) = self;
				let &$name(ref you) = other;
				let mut i = $n_words;
				while i > 0 {
					i -= 1;
					if me[i] < you[i] { return Ordering::Less; }
					if me[i] > you[i] { return Ordering::Greater; }
				}
				Ordering::Equal
			}
		}

		impl PartialOrd for $name {
			fn partial_cmp(&self, other: &$name) -> Option<Ordering> {
				Some(self.cmp(other))
			}
		}

		impl fmt::Debug for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				fmt::Display::fmt(self, f)
			}
		}

		impl fmt::Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				if self.is_zero() {
					return write!(f, "0");
				}

                let mut s = [0u8; $n_words * 20];
                let mut i = $n_words * 20;
				let mut current = *self;
				let ten = $name::from(10);

				while !current.is_zero() {
                    i = i - 1;
                    s[i] = (current % ten).low_u32() as u8;
					current = current / ten;
				}

                for i in i..($n_words * 20) {
                    write!(f, "{}", s[i])?;
                }

                Ok(())
			}
		}

		impl fmt::LowerHex for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				let &$name(ref data) = self;
				let mut latch = false;
				for ch in data.iter().rev() {
					for x in 0..16 {
						let nibble = (ch & (15u64 << ((15 - x) * 4) as u64)) >> (((15 - x) * 4) as u64);
						if !latch { latch = nibble != 0 }
						if latch {
							try!(write!(f, "{:x}", nibble));
						}
					}
				}
				Ok(())
			}
		}

		impl fmt::UpperHex for $name {
		    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		        let &$name(ref data) = self;
				let mut latch = false;
				for ch in data.iter().rev() {
					for x in 0..16 {
						let nibble = (ch & (15u64 << ((15 - x) * 4) as u64)) >> (((15 - x) * 4) as u64);
						if !latch { latch = nibble != 0 }
						if latch {
							try!(write!(f, "{:X}", nibble));
						}
					}
				}
				Ok(())
		    }
		}

        #[cfg(feature = "string")]
		impl From<&'static str> for $name {
			fn from(s: &'static str) -> Self {
				s.parse().unwrap()
			}
		}
	);
}

construct_uint!(U512, 8);
construct_uint!(U256, 4);
construct_uint!(U128, 2);

impl U256 {
    /// Equals `floor(log2(*))`. This is always an integer.
    pub fn log2floor(&self) -> usize {
        assert!(*self != U256::zero());
        let mut l: usize = 256;
        for i in 0..4 {
            let i = 3 - i;
            if self.0[i] == 0u64 {
                l -= 64;
            } else {
                l -= self.0[i].leading_zeros() as usize;
                if l == 0 {
                    return l
                } else {
                    return l - 1;
                }
            }
        }
        return l;
    }

    /// Multiplies two 256-bit integers to produce full 512-bit integer
    /// No overflow possible
    #[cfg(all(asm_available, target_arch="x86_64"))]
    pub fn full_mul(self, other: U256) -> U512 {
        let self_t: &[u64; 4] = &self.0;
        let other_t: &[u64; 4] = &other.0;
        let mut result: [u64; 8] = unsafe { ::std::mem::uninitialized() };
        unsafe {
            asm!("
				mov $8, %rax
				mulq $12
				mov %rax, $0
				mov %rdx, $1

				mov $8, %rax
				mulq $13
				add %rax, $1
				adc $$0, %rdx
				mov %rdx, $2

				mov $8, %rax
				mulq $14
				add %rax, $2
				adc $$0, %rdx
				mov %rdx, $3

				mov $8, %rax
				mulq $15
				add %rax, $3
				adc $$0, %rdx
				mov %rdx, $4

				mov $9, %rax
				mulq $12
				add %rax, $1
				adc %rdx, $2
				adc $$0, $3
				adc $$0, $4
				xor $5, $5
				adc $$0, $5
				xor $6, $6
				adc $$0, $6
				xor $7, $7
				adc $$0, $7

				mov $9, %rax
				mulq $13
				add %rax, $2
				adc %rdx, $3
				adc $$0, $4
				adc $$0, $5
				adc $$0, $6
				adc $$0, $7

				mov $9, %rax
				mulq $14
				add %rax, $3
				adc %rdx, $4
				adc $$0, $5
				adc $$0, $6
				adc $$0, $7

				mov $9, %rax
				mulq $15
				add %rax, $4
				adc %rdx, $5
				adc $$0, $6
				adc $$0, $7

				mov $10, %rax
				mulq $12
				add %rax, $2
				adc %rdx, $3
				adc $$0, $4
				adc $$0, $5
				adc $$0, $6
				adc $$0, $7

				mov $10, %rax
				mulq $13
				add %rax, $3
				adc %rdx, $4
				adc $$0, $5
				adc $$0, $6
				adc $$0, $7

				mov $10, %rax
				mulq $14
				add %rax, $4
				adc %rdx, $5
				adc $$0, $6
				adc $$0, $7

				mov $10, %rax
				mulq $15
				add %rax, $5
				adc %rdx, $6
				adc $$0, $7

				mov $11, %rax
				mulq $12
				add %rax, $3
				adc %rdx, $4
				adc $$0, $5
				adc $$0, $6
				adc $$0, $7

				mov $11, %rax
				mulq $13
				add %rax, $4
				adc %rdx, $5
				adc $$0, $6
				adc $$0, $7

				mov $11, %rax
				mulq $14
				add %rax, $5
				adc %rdx, $6
				adc $$0, $7

				mov $11, %rax
				mulq $15
				add %rax, $6
				adc %rdx, $7
				"
            : /* $0 */ "={r8}"(result[0]), /* $1 */ "={r9}"(result[1]), /* $2 */ "={r10}"(result[2]),
			  /* $3 */ "={r11}"(result[3]), /* $4 */ "={r12}"(result[4]), /* $5 */ "={r13}"(result[5]),
			  /* $6 */ "={r14}"(result[6]), /* $7 */ "={r15}"(result[7])

            : /* $8 */ "m"(self_t[0]), /* $9 */ "m"(self_t[1]), /* $10 */  "m"(self_t[2]),
			  /* $11 */ "m"(self_t[3]), /* $12 */ "m"(other_t[0]), /* $13 */ "m"(other_t[1]),
			  /* $14 */ "m"(other_t[2]), /* $15 */ "m"(other_t[3])
			: "rax", "rdx"
			:
			);
        }
        U512(result)
    }

    /// Multiplies two 256-bit integers to produce full 512-bit integer
    /// No overflow possible
    #[cfg(not(all(asm_available, target_arch="x86_64")))]
    pub fn full_mul(self, other: U256) -> U512 {
        let U256(ref me) = self;
        let U256(ref you) = other;
        let mut ret = [0u64; 8];

        for i in 0..4 {
            if you[i] == 0 {
                continue;
            }

            let mut carry2 = 0u64;
            let (b_u, b_l) = split(you[i]);

            for j in 0..4 {
                if me[j] == 0 && carry2 == 0 {
                    continue;
                }

                let a = split(me[j]);

                // multiply parts
                let (c_l, overflow_l) = mul_u32(a, b_l, ret[i + j]);
                let (c_u, overflow_u) = mul_u32(a, b_u, c_l >> 32);
                ret[i + j] = (c_l & 0xFFFFFFFF) + (c_u << 32);

                // No overflow here
                let res = (c_u >> 32) + (overflow_u << 32);
                // possible overflows
                let (res, o1) = res.overflowing_add(overflow_l);
                let (res, o2) = res.overflowing_add(carry2);
                let (res, o3) = res.overflowing_add(ret[i + j + 1]);
                ret[i + j + 1] = res;

                // Only single overflow possible there
                carry2 = (o1 | o2 | o3) as u64;
            }
        }

        U512(ret)
    }
}

impl From<U256> for U512 {
    fn from(value: U256) -> U512 {
        let U256(ref arr) = value;
        let mut ret = [0; 8];
        ret[0] = arr[0];
        ret[1] = arr[1];
        ret[2] = arr[2];
        ret[3] = arr[3];
        U512(ret)
    }
}

impl From<U512> for U256 {
    fn from(value: U512) -> U256 {
        let U512(ref arr) = value;
        if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
            panic!("Overflow");
        }
        let mut ret = [0; 4];
        ret[0] = arr[0];
        ret[1] = arr[1];
        ret[2] = arr[2];
        ret[3] = arr[3];
        U256(ret)
    }
}

impl<'a> From<&'a U256> for U512 {
    fn from(value: &'a U256) -> U512 {
        let U256(ref arr) = *value;
        let mut ret = [0; 8];
        ret[0] = arr[0];
        ret[1] = arr[1];
        ret[2] = arr[2];
        ret[3] = arr[3];
        U512(ret)
    }
}

impl<'a> From<&'a U512> for U256 {
    fn from(value: &'a U512) -> U256 {
        let U512(ref arr) = *value;
        if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
            panic!("Overflow");
        }
        let mut ret = [0; 4];
        ret[0] = arr[0];
        ret[1] = arr[1];
        ret[2] = arr[2];
        ret[3] = arr[3];
        U256(ret)
    }
}

impl From<U256> for U128 {
    fn from(value: U256) -> U128 {
        let U256(ref arr) = value;
        if arr[2] | arr[3] != 0 {
            panic!("Overflow");
        }
        let mut ret = [0; 2];
        ret[0] = arr[0];
        ret[1] = arr[1];
        U128(ret)
    }
}

impl From<U512> for U128 {
    fn from(value: U512) -> U128 {
        let U512(ref arr) = value;
        if arr[2] | arr[3] | arr[4] | arr[5] | arr[6] | arr[7] != 0 {
            panic!("Overflow");
        }
        let mut ret = [0; 2];
        ret[0] = arr[0];
        ret[1] = arr[1];
        U128(ret)
    }
}

impl From<U128> for U512 {
    fn from(value: U128) -> U512 {
        let U128(ref arr) = value;
        let mut ret = [0; 8];
        ret[0] = arr[0];
        ret[1] = arr[1];
        U512(ret)
    }
}

impl From<U128> for U256 {
    fn from(value: U128) -> U256 {
        let U128(ref arr) = value;
        let mut ret = [0; 4];
        ret[0] = arr[0];
        ret[1] = arr[1];
        U256(ret)
    }
}

impl From<U256> for u64 {
    fn from(value: U256) -> u64 {
        value.as_u64()
    }
}

impl From<U256> for u32 {
    fn from(value: U256) -> u32 {
        value.as_u32()
    }
}

#[cfg(feature="heapsizeof")]
known_heap_size!(0, U128, U256);

#[cfg(test)] mod tests;
