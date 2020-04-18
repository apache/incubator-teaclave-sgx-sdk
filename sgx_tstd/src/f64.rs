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

//! This module provides constants which are specific to the implementation
//! of the `f64` floating point data type.
//!
//! *[See also the `f64` primitive type](../../std/primitive.f64.html).*
//!
//! Mathematically significant numbers are provided in the `consts` sub-module.
//!
//! Although using these constants won’t cause compilation warnings,
//! new code should use the associated constants directly on the primitive type.

#![allow(missing_docs)]

use core::intrinsics;
use crate::sys::cmath;

pub use core::f64::consts;
pub use core::f64::{DIGITS, EPSILON, MANTISSA_DIGITS, RADIX};
pub use core::f64::{INFINITY, MAX_10_EXP, NAN, NEG_INFINITY};
pub use core::f64::{MAX, MIN, MIN_POSITIVE};
pub use core::f64::{MAX_EXP, MIN_10_EXP, MIN_EXP};

#[lang = "f64_runtime"]
impl f64 {
    /// Returns the largest integer less than or equal to a number.
    ///
    #[inline]
    pub fn floor(self) -> f64 {
        unsafe { intrinsics::floorf64(self) }
    }

    /// Returns the smallest integer greater than or equal to a number.
    ///
    #[inline]
    pub fn ceil(self) -> f64 {
        unsafe { intrinsics::ceilf64(self) }
    }

    /// Returns the nearest integer to a number. Round half-way cases away from
    /// `0.0`.
    ///
    #[inline]
    pub fn round(self) -> f64 {
        unsafe { intrinsics::roundf64(self) }
    }

    /// Returns the integer part of a number.
    ///
    #[inline]
    pub fn trunc(self) -> f64 {
        unsafe { intrinsics::truncf64(self) }
    }

    /// Returns the fractional part of a number.
    ///
    #[inline]
    pub fn fract(self) -> f64 {
        self - self.trunc()
    }

    /// Computes the absolute value of `self`. Returns `NAN` if the
    /// number is `NAN`.
    ///
    #[inline]
    pub fn abs(self) -> f64 {
        unsafe { intrinsics::fabsf64(self) }
    }

    /// Returns a number that represents the sign of `self`.
    ///
    /// - `1.0` if the number is positive, `+0.0` or `INFINITY`
    /// - `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
    /// - `NAN` if the number is `NAN`
    ///
    #[inline]
    pub fn signum(self) -> f64 {
        if self.is_nan() { NAN } else { 1.0_f64.copysign(self) }
    }

    /// Returns a number composed of the magnitude of `self` and the sign of
    /// `sign`.
    ///
    /// Equal to `self` if the sign of `self` and `sign` are the same, otherwise
    /// equal to `-self`. If `self` is a `NAN`, then a `NAN` with the sign of
    /// `sign` is returned.
    ///
    #[inline]
    pub fn copysign(self, sign: f64) -> f64 {
        unsafe { intrinsics::copysignf64(self, sign) }
    }

    /// Fused multiply-add. Computes `(self * a) + b` with only one rounding
    /// error, yielding a more accurate result than an unfused multiply-add.
    ///
    /// Using `mul_add` can be more performant than an unfused multiply-add if
    /// the target architecture has a dedicated `fma` CPU instruction.
    ///
    #[inline]
    pub fn mul_add(self, a: f64, b: f64) -> f64 {
        unsafe { intrinsics::fmaf64(self, a, b) }
    }

    /// Calculates Euclidean division, the matching method for `rem_euclid`.
    ///
    /// This computes the integer `n` such that
    /// `self = n * rhs + self.rem_euclid(rhs)`.
    /// In other words, the result is `self / rhs` rounded to the integer `n`
    /// such that `self >= n * rhs`.
    ///
    #[inline]
    pub fn div_euclid(self, rhs: f64) -> f64 {
        let q = (self / rhs).trunc();
        if self % rhs < 0.0 {
            return if rhs > 0.0 { q - 1.0 } else { q + 1.0 };
        }
        q
    }

    /// Calculates the least nonnegative remainder of `self (mod rhs)`.
    ///
    /// In particular, the return value `r` satisfies `0.0 <= r < rhs.abs()` in
    /// most cases. However, due to a floating point round-off error it can
    /// result in `r == rhs.abs()`, violating the mathematical definition, if
    /// `self` is much smaller than `rhs.abs()` in magnitude and `self < 0.0`.
    /// This result is not an element of the function's codomain, but it is the
    /// closest floating point number in the real numbers and thus fulfills the
    /// property `self == self.div_euclid(rhs) * rhs + self.rem_euclid(rhs)`
    /// approximatively.
    ///
    #[inline]
    pub fn rem_euclid(self, rhs: f64) -> f64 {
        let r = self % rhs;
        if r < 0.0 { r + rhs.abs() } else { r }
    }

    /// Raises a number to an integer power.
    ///
    /// Using this function is generally faster than using `powf`
    ///
    #[inline]
    pub fn powi(self, n: i32) -> f64 {
        unsafe { intrinsics::powif64(self, n) }
    }

    /// Raises a number to a floating point power.
    ///
    #[inline]
    pub fn powf(self, n: f64) -> f64 {
        unsafe { intrinsics::powf64(self, n) }
    }

    /// Returns the square root of a number.
    ///
    /// Returns NaN if `self` is a negative number.
    ///
    #[inline]
    pub fn sqrt(self) -> f64 {
        unsafe { intrinsics::sqrtf64(self) }
    }

    /// Returns `e^(self)`, (the exponential function).
    ///
    #[inline]
    pub fn exp(self) -> f64 {
        unsafe { intrinsics::expf64(self) }
    }

    /// Returns `2^(self)`.
    ///
    #[inline]
    pub fn exp2(self) -> f64 {
        unsafe { intrinsics::exp2f64(self) }
    }

    /// Returns the natural logarithm of the number.
    ///
    #[inline]
    pub fn ln(self) -> f64 {
        self.log_wrapper(|n| unsafe { intrinsics::logf64(n) })
    }

    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// The result may not be correctly rounded owing to implementation details;
    /// `self.log2()` can produce more accurate results for base 2, and
    /// `self.log10()` can produce more accurate results for base 10.
    ///
    #[inline]
    pub fn log(self, base: f64) -> f64 {
        self.ln() / base.ln()
    }

    /// Returns the base 2 logarithm of the number.
    ///
    #[inline]
    pub fn log2(self) -> f64 {
        self.log_wrapper(|n| {
            #[cfg(target_os = "android")]
            return crate::sys::android::log2f64(n);
            #[cfg(not(target_os = "android"))]
            return unsafe { intrinsics::log2f64(n) };
        })
    }

    /// Returns the base 10 logarithm of the number.
    ///
    #[inline]
    pub fn log10(self) -> f64 {
        self.log_wrapper(|n| unsafe { intrinsics::log10f64(n) })
    }

    /// The positive difference of two numbers.
    ///
    /// * If `self <= other`: `0:0`
    /// * Else: `self - other`
    ///
    #[inline]
    pub fn abs_sub(self, other: f64) -> f64 {
        unsafe { cmath::fdim(self, other) }
    }

    /// Returns the cubic root of a number.
    ///
    #[inline]
    pub fn cbrt(self) -> f64 {
        unsafe { cmath::cbrt(self) }
    }

    /// Calculates the length of the hypotenuse of a right-angle triangle given
    /// legs of length `x` and `y`.
    ///
    #[inline]
    pub fn hypot(self, other: f64) -> f64 {
        unsafe { cmath::hypot(self, other) }
    }

    /// Computes the sine of a number (in radians).
    ///
    #[inline]
    pub fn sin(self) -> f64 {
        unsafe { intrinsics::sinf64(self) }
    }

    /// Computes the cosine of a number (in radians).
    ///
    #[inline]
    pub fn cos(self) -> f64 {
        unsafe { intrinsics::cosf64(self) }
    }

    /// Computes the tangent of a number (in radians).
    ///
    #[inline]
    pub fn tan(self) -> f64 {
        unsafe { cmath::tan(self) }
    }

    /// Computes the arcsine of a number. Return value is in radians in
    /// the range [-pi/2, pi/2] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    #[inline]
    pub fn asin(self) -> f64 {
        unsafe { cmath::asin(self) }
    }

    /// Computes the arccosine of a number. Return value is in radians in
    /// the range [0, pi] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    #[inline]
    pub fn acos(self) -> f64 {
        unsafe { cmath::acos(self) }
    }

    /// Computes the arctangent of a number. Return value is in radians in the
    /// range [-pi/2, pi/2];
    ///
    #[inline]
    pub fn atan(self) -> f64 {
        unsafe { cmath::atan(self) }
    }

    /// Computes the four quadrant arctangent of `self` (`y`) and `other` (`x`) in radians.
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-pi/2, pi/2]`
    /// * `y >= 0`: `arctan(y/x) + pi` -> `(pi/2, pi]`
    /// * `y < 0`: `arctan(y/x) - pi` -> `(-pi, -pi/2)`
    ///
    #[inline]
    pub fn atan2(self, other: f64) -> f64 {
        unsafe { cmath::atan2(self, other) }
    }

    /// Simultaneously computes the sine and cosine of the number, `x`. Returns
    /// `(sin(x), cos(x))`.
    ///
    #[inline]
    pub fn sin_cos(self) -> (f64, f64) {
        (self.sin(), self.cos())
    }

    /// Returns `e^(self) - 1` in a way that is accurate even if the
    /// number is close to zero.
    ///
    #[inline]
    pub fn exp_m1(self) -> f64 {
        unsafe { cmath::expm1(self) }
    }

    /// Returns `ln(1+n)` (natural logarithm) more accurately than if
    /// the operations were performed separately.
    ///
    #[inline]
    pub fn ln_1p(self) -> f64 {
        unsafe { cmath::log1p(self) }
    }

    /// Hyperbolic sine function.
    ///
    #[inline]
    pub fn sinh(self) -> f64 {
        unsafe { cmath::sinh(self) }
    }

    /// Hyperbolic cosine function.
    ///
    #[inline]
    pub fn cosh(self) -> f64 {
        unsafe { cmath::cosh(self) }
    }

    /// Hyperbolic tangent function.
    ///
    #[inline]
    pub fn tanh(self) -> f64 {
        unsafe { cmath::tanh(self) }
    }

    /// Inverse hyperbolic sine function.
    ///
    #[inline]
    pub fn asinh(self) -> f64 {
        if self == NEG_INFINITY {
            NEG_INFINITY
        } else {
            (self + ((self * self) + 1.0).sqrt()).ln().copysign(self)
        }
    }

    /// Inverse hyperbolic cosine function.
    ///
    #[inline]
    pub fn acosh(self) -> f64 {
        if self < 1.0 { NAN } else { (self + ((self * self) - 1.0).sqrt()).ln() }
    }

    /// Inverse hyperbolic tangent function.
    ///
    #[inline]
    pub fn atanh(self) -> f64 {
        0.5 * ((2.0 * self) / (1.0 - self)).ln_1p()
    }

    /// Restrict a value to a certain interval unless it is NaN.
    ///
    /// Returns `max` if `self` is greater than `max`, and `min` if `self` is
    /// less than `min`. Otherwise this returns `self`.
    ///
    /// Not that this function returns NaN if the initial value was NaN as
    /// well.
    ///
    /// # Panics
    ///
    /// Panics if `min > max`, `min` is NaN, or `max` is NaN.
    ///
    #[inline]
    pub fn clamp(self, min: f64, max: f64) -> f64 {
        assert!(min <= max);
        let mut x = self;
        if x < min {
            x = min;
        }
        if x > max {
            x = max;
        }
        x
    }

    // Solaris/Illumos requires a wrapper around log, log2, and log10 functions
    // because of their non-standard behavior (e.g., log(-n) returns -Inf instead
    // of expected NaN).
    fn log_wrapper<F: Fn(f64) -> f64>(self, log_fn: F) -> f64 {
        if !cfg!(target_os = "solaris") {
            log_fn(self)
        } else {
            if self.is_finite() {
                if self > 0.0 {
                    log_fn(self)
                } else if self == 0.0 {
                    NEG_INFINITY // log(0) = -Inf
                } else {
                    NAN // log(-n) = NaN
                }
            } else if self.is_nan() {
                self // log(NaN) = NaN
            } else if self > 0.0 {
                self // log(Inf) = Inf
            } else {
                NAN // log(-Inf) = NaN
            }
        }
    }
}
