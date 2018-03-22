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

//! This module provides constants which are specific to the implementation
//! of the `f64` floating point data type.
//!
//! Mathematically significant numbers are provided in the `consts` sub-module.
//!

#![allow(missing_docs)]

use core::num;
use core::intrinsics;
use core::num::FpCategory;
use sys::cmath;

pub use core::f64::{RADIX, MANTISSA_DIGITS, DIGITS, EPSILON};
pub use core::f64::{MIN_EXP, MAX_EXP, MIN_10_EXP};
pub use core::f64::{MAX_10_EXP, NAN, INFINITY, NEG_INFINITY};
pub use core::f64::{MIN, MIN_POSITIVE, MAX};
pub use core::f64::consts;

#[lang = "f64"]
impl f64 {
    /// Returns `true` if this value is `NaN` and false otherwise.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let nan = f64::NAN;
    /// let f = 7.0_f64;
    ///
    /// assert!(nan.is_nan());
    /// assert!(!f.is_nan());
    /// ```
    #[inline]
    pub fn is_nan(self) -> bool { num::Float::is_nan(self) }

    /// Returns `true` if this value is positive infinity or negative infinity and
    /// false otherwise.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let f = 7.0f64;
    /// let inf = f64::INFINITY;
    /// let neg_inf = f64::NEG_INFINITY;
    /// let nan = f64::NAN;
    ///
    /// assert!(!f.is_infinite());
    /// assert!(!nan.is_infinite());
    ///
    /// assert!(inf.is_infinite());
    /// assert!(neg_inf.is_infinite());
    /// ```
    #[inline]
    pub fn is_infinite(self) -> bool { num::Float::is_infinite(self) }

    /// Returns `true` if this number is neither infinite nor `NaN`.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let f = 7.0f64;
    /// let inf: f64 = f64::INFINITY;
    /// let neg_inf: f64 = f64::NEG_INFINITY;
    /// let nan: f64 = f64::NAN;
    ///
    /// assert!(f.is_finite());
    ///
    /// assert!(!nan.is_finite());
    /// assert!(!inf.is_finite());
    /// assert!(!neg_inf.is_finite());
    /// ```
    #[inline]
    pub fn is_finite(self) -> bool { num::Float::is_finite(self) }

    /// Returns `true` if the number is neither zero, infinite,
    /// [subnormal][subnormal], or `NaN`.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let min = f64::MIN_POSITIVE; // 2.2250738585072014e-308f64
    /// let max = f64::MAX;
    /// let lower_than_min = 1.0e-308_f64;
    /// let zero = 0.0f64;
    ///
    /// assert!(min.is_normal());
    /// assert!(max.is_normal());
    ///
    /// assert!(!zero.is_normal());
    /// assert!(!f64::NAN.is_normal());
    /// assert!(!f64::INFINITY.is_normal());
    /// // Values between `0` and `min` are Subnormal.
    /// assert!(!lower_than_min.is_normal());
    /// ```
    /// [subnormal]: https://en.wikipedia.org/wiki/Denormal_number
    #[inline]
    pub fn is_normal(self) -> bool { num::Float::is_normal(self) }

    /// Returns the floating point category of the number. If only one property
    /// is going to be tested, it is generally faster to use the specific
    /// predicate instead.
    ///
    /// ```
    /// use std::num::FpCategory;
    /// use std::f64;
    ///
    /// let num = 12.4_f64;
    /// let inf = f64::INFINITY;
    ///
    /// assert_eq!(num.classify(), FpCategory::Normal);
    /// assert_eq!(inf.classify(), FpCategory::Infinite);
    /// ```
    #[inline]
    pub fn classify(self) -> FpCategory { num::Float::classify(self) }

    /// Returns the largest integer less than or equal to a number.
    ///
    /// ```
    /// let f = 3.99_f64;
    /// let g = 3.0_f64;
    ///
    /// assert_eq!(f.floor(), 3.0);
    /// assert_eq!(g.floor(), 3.0);
    /// ```
    #[inline]
    pub fn floor(self) -> f64 {
        unsafe { intrinsics::floorf64(self) }
    }

    /// Returns the smallest integer greater than or equal to a number.
    ///
    /// ```
    /// let f = 3.01_f64;
    /// let g = 4.0_f64;
    ///
    /// assert_eq!(f.ceil(), 4.0);
    /// assert_eq!(g.ceil(), 4.0);
    /// ```
    #[inline]
    pub fn ceil(self) -> f64 {
        unsafe { intrinsics::ceilf64(self) }
    }

    /// Returns the nearest integer to a number. Round half-way cases away from
    /// `0.0`.
    ///
    /// ```
    /// let f = 3.3_f64;
    /// let g = -3.3_f64;
    ///
    /// assert_eq!(f.round(), 3.0);
    /// assert_eq!(g.round(), -3.0);
    /// ```
    #[inline]
    pub fn round(self) -> f64 {
        unsafe { intrinsics::roundf64(self) }
    }

    /// Returns the integer part of a number.
    ///
    /// ```
    /// let f = 3.3_f64;
    /// let g = -3.7_f64;
    ///
    /// assert_eq!(f.trunc(), 3.0);
    /// assert_eq!(g.trunc(), -3.0);
    /// ```
    #[inline]
    pub fn trunc(self) -> f64 {
        unsafe { intrinsics::truncf64(self) }
    }

    /// Returns the fractional part of a number.
    ///
    /// ```
    /// let x = 3.5_f64;
    /// let y = -3.5_f64;
    /// let abs_difference_x = (x.fract() - 0.5).abs();
    /// let abs_difference_y = (y.fract() - (-0.5)).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    /// ```
    #[inline]
    pub fn fract(self) -> f64 { self - self.trunc() }

    /// Computes the absolute value of `self`. Returns `NAN` if the
    /// number is `NAN`.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = 3.5_f64;
    /// let y = -3.5_f64;
    ///
    /// let abs_difference_x = (x.abs() - x).abs();
    /// let abs_difference_y = (y.abs() - (-y)).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    ///
    /// assert!(f64::NAN.abs().is_nan());
    /// ```
    #[inline]
    pub fn abs(self) -> f64 { num::Float::abs(self) }

    /// Returns a number that represents the sign of `self`.
    ///
    /// - `1.0` if the number is positive, `+0.0` or `INFINITY`
    /// - `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
    /// - `NAN` if the number is `NAN`
    ///
    /// ```
    /// use std::f64;
    ///
    /// let f = 3.5_f64;
    ///
    /// assert_eq!(f.signum(), 1.0);
    /// assert_eq!(f64::NEG_INFINITY.signum(), -1.0);
    ///
    /// assert!(f64::NAN.signum().is_nan());
    /// ```
    #[inline]
    pub fn signum(self) -> f64 { num::Float::signum(self) }

    /// Returns `true` if and only if `self` has a positive sign, including `+0.0`, `NaN`s with
    /// positive sign bit and positive infinity.
    ///
    /// ```
    /// let f = 7.0_f64;
    /// let g = -7.0_f64;
    ///
    /// assert!(f.is_sign_positive());
    /// assert!(!g.is_sign_positive());
    /// ```
    #[inline]
    pub fn is_sign_positive(self) -> bool { num::Float::is_sign_positive(self) }

    #[inline]
    pub fn is_positive(self) -> bool { num::Float::is_sign_positive(self) }

    /// Returns `true` if and only if `self` has a negative sign, including `-0.0`, `NaN`s with
    /// negative sign bit and negative infinity.
    ///
    /// ```
    /// let f = 7.0_f64;
    /// let g = -7.0_f64;
    ///
    /// assert!(!f.is_sign_negative());
    /// assert!(g.is_sign_negative());
    /// ```
    #[inline]
    pub fn is_sign_negative(self) -> bool { num::Float::is_sign_negative(self) }

    #[inline]
    pub fn is_negative(self) -> bool { num::Float::is_sign_negative(self) }

    /// Fused multiply-add. Computes `(self * a) + b` with only one rounding
    /// error. This produces a more accurate result with better performance than
    /// a separate multiplication operation followed by an add.
    ///
    /// ```
    /// let m = 10.0_f64;
    /// let x = 4.0_f64;
    /// let b = 60.0_f64;
    ///
    /// // 100.0
    /// let abs_difference = (m.mul_add(x, b) - (m*x + b)).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn mul_add(self, a: f64, b: f64) -> f64 {
        unsafe { intrinsics::fmaf64(self, a, b) }
    }

    /// Takes the reciprocal (inverse) of a number, `1/x`.
    ///
    /// ```
    /// let x = 2.0_f64;
    /// let abs_difference = (x.recip() - (1.0/x)).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn recip(self) -> f64 { num::Float::recip(self) }

    /// Raises a number to an integer power.
    ///
    /// Using this function is generally faster than using `powf`
    ///
    /// ```
    /// let x = 2.0_f64;
    /// let abs_difference = (x.powi(2) - x*x).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn powi(self, n: i32) -> f64 { num::Float::powi(self, n) }

    /// Raises a number to a floating point power.
    ///
    /// ```
    /// let x = 2.0_f64;
    /// let abs_difference = (x.powf(2.0) - x*x).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn powf(self, n: f64) -> f64 {
        unsafe { intrinsics::powf64(self, n) }
    }

    /// Takes the square root of a number.
    ///
    /// Returns NaN if `self` is a negative number.
    ///
    /// ```
    /// let positive = 4.0_f64;
    /// let negative = -4.0_f64;
    ///
    /// let abs_difference = (positive.sqrt() - 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// assert!(negative.sqrt().is_nan());
    /// ```
    #[inline]
    pub fn sqrt(self) -> f64 {
        if self < 0.0 {
            NAN
        } else {
            unsafe { intrinsics::sqrtf64(self) }
        }
    }

    /// Returns `e^(self)`, (the exponential function).
    ///
    /// ```
    /// let one = 1.0_f64;
    /// // e^1
    /// let e = one.exp();
    ///
    /// // ln(e) - 1 == 0
    /// let abs_difference = (e.ln() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn exp(self) -> f64 {
        unsafe { intrinsics::expf64(self) }
    }

    /// Returns `2^(self)`.
    ///
    /// ```
    /// let f = 2.0_f64;
    ///
    /// // 2^2 - 4 == 0
    /// let abs_difference = (f.exp2() - 4.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn exp2(self) -> f64 {
        unsafe { intrinsics::exp2f64(self) }
    }

    /// Returns the natural logarithm of the number.
    ///
    /// ```
    /// let one = 1.0_f64;
    /// // e^1
    /// let e = one.exp();
    ///
    /// // ln(e) - 1 == 0
    /// let abs_difference = (e.ln() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn ln(self) -> f64 {
        self.log_wrapper(|n| { unsafe { intrinsics::logf64(n) } })
    }

    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// The result may not be correctly rounded owing to implementation details;
    /// `self.log2()` can produce more accurate results for base 2, and
    /// `self.log10()` can produce more accurate results for base 10.
    ///
    /// ```
    /// let five = 5.0_f64;
    ///
    /// // log5(5) - 1 == 0
    /// let abs_difference = (five.log(5.0) - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn log(self, base: f64) -> f64 { self.ln() / base.ln() }

    /// Returns the base 2 logarithm of the number.
    ///
    /// ```
    /// let two = 2.0_f64;
    ///
    /// // log2(2) - 1 == 0
    /// let abs_difference = (two.log2() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn log2(self) -> f64 {
        self.log_wrapper(|n| {
            return unsafe { intrinsics::log2f64(n) };
        })
    }

    /// Returns the base 10 logarithm of the number.
    ///
    /// ```
    /// let ten = 10.0_f64;
    ///
    /// // log10(10) - 1 == 0
    /// let abs_difference = (ten.log10() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn log10(self) -> f64 {
        self.log_wrapper(|n| { unsafe { intrinsics::log10f64(n) } })
    }

    /// Converts radians to degrees.
    ///
    /// ```
    /// use std::f64::consts;
    ///
    /// let angle = consts::PI;
    ///
    /// let abs_difference = (angle.to_degrees() - 180.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn to_degrees(self) -> f64 { num::Float::to_degrees(self) }

    /// Converts degrees to radians.
    ///
    /// ```
    /// use std::f64::consts;
    ///
    /// let angle = 180.0_f64;
    ///
    /// let abs_difference = (angle.to_radians() - consts::PI).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn to_radians(self) -> f64 { num::Float::to_radians(self) }

    /// Returns the maximum of the two numbers.
    ///
    /// ```
    /// let x = 1.0_f64;
    /// let y = 2.0_f64;
    ///
    /// assert_eq!(x.max(y), y);
    /// ```
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    #[inline]
    pub fn max(self, other: f64) -> f64 {
        num::Float::max(self, other)
    }

    /// Returns the minimum of the two numbers.
    ///
    /// ```
    /// let x = 1.0_f64;
    /// let y = 2.0_f64;
    ///
    /// assert_eq!(x.min(y), x);
    /// ```
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    #[inline]
    pub fn min(self, other: f64) -> f64 {
        num::Float::min(self, other)
    }

    /// The positive difference of two numbers.
    ///
    /// * If `self <= other`: `0:0`
    /// * Else: `self - other`
    ///
    /// ```
    /// let x = 3.0_f64;
    /// let y = -3.0_f64;
    ///
    /// let abs_difference_x = (x.abs_sub(1.0) - 2.0).abs();
    /// let abs_difference_y = (y.abs_sub(1.0) - 0.0).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    /// ```
    #[inline]
     pub fn abs_sub(self, other: f64) -> f64 {
         unsafe { cmath::fdim(self, other) }
     }

    /// Takes the cubic root of a number.
    ///
    /// ```
    /// let x = 8.0_f64;
    ///
    /// // x^(1/3) - 2 == 0
    /// let abs_difference = (x.cbrt() - 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn cbrt(self) -> f64 {
        unsafe { cmath::cbrt(self) }
    }

    /// Calculates the length of the hypotenuse of a right-angle triangle given
    /// legs of length `x` and `y`.
    ///
    /// ```
    /// let x = 2.0_f64;
    /// let y = 3.0_f64;
    ///
    /// // sqrt(x^2 + y^2)
    /// let abs_difference = (x.hypot(y) - (x.powi(2) + y.powi(2)).sqrt()).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn hypot(self, other: f64) -> f64 {
        unsafe { cmath::hypot(self, other) }
    }

    /// Computes the sine of a number (in radians).
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = f64::consts::PI/2.0;
    ///
    /// let abs_difference = (x.sin() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn sin(self) -> f64 {
        unsafe { intrinsics::sinf64(self) }
    }

    /// Computes the cosine of a number (in radians).
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = 2.0*f64::consts::PI;
    ///
    /// let abs_difference = (x.cos() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn cos(self) -> f64 {
        unsafe { intrinsics::cosf64(self) }
    }

    /// Computes the tangent of a number (in radians).
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = f64::consts::PI/4.0;
    /// let abs_difference = (x.tan() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-14);
    /// ```
    #[inline]
    pub fn tan(self) -> f64 {
        unsafe { cmath::tan(self) }
    }

    /// Computes the arcsine of a number. Return value is in radians in
    /// the range [-pi/2, pi/2] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// ```
    /// use std::f64;
    ///
    /// let f = f64::consts::PI / 2.0;
    ///
    /// // asin(sin(pi/2))
    /// let abs_difference = (f.sin().asin() - f64::consts::PI / 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn asin(self) -> f64 {
        unsafe { cmath::asin(self) }
    }

    /// Computes the arccosine of a number. Return value is in radians in
    /// the range [0, pi] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// ```
    /// use std::f64;
    ///
    /// let f = f64::consts::PI / 4.0;
    ///
    /// // acos(cos(pi/4))
    /// let abs_difference = (f.cos().acos() - f64::consts::PI / 4.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn acos(self) -> f64 {
        unsafe { cmath::acos(self) }
    }

    /// Computes the arctangent of a number. Return value is in radians in the
    /// range [-pi/2, pi/2];
    ///
    /// ```
    /// let f = 1.0_f64;
    ///
    /// // atan(tan(1))
    /// let abs_difference = (f.tan().atan() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn atan(self) -> f64 {
        unsafe { cmath::atan(self) }
    }

    /// Computes the four quadrant arctangent of `self` (`y`) and `other` (`x`).
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-pi/2, pi/2]`
    /// * `y >= 0`: `arctan(y/x) + pi` -> `(pi/2, pi]`
    /// * `y < 0`: `arctan(y/x) - pi` -> `(-pi, -pi/2)`
    ///
    /// ```
    /// use std::f64;
    ///
    /// let pi = f64::consts::PI;
    /// // All angles from horizontal right (+x)
    /// // 45 deg counter-clockwise
    /// let x1 = 3.0_f64;
    /// let y1 = -3.0_f64;
    ///
    /// // 135 deg clockwise
    /// let x2 = -3.0_f64;
    /// let y2 = 3.0_f64;
    ///
    /// let abs_difference_1 = (y1.atan2(x1) - (-pi/4.0)).abs();
    /// let abs_difference_2 = (y2.atan2(x2) - 3.0*pi/4.0).abs();
    ///
    /// assert!(abs_difference_1 < 1e-10);
    /// assert!(abs_difference_2 < 1e-10);
    /// ```
    #[inline]
    pub fn atan2(self, other: f64) -> f64 {
        unsafe { cmath::atan2(self, other) }
    }

    /// Simultaneously computes the sine and cosine of the number, `x`. Returns
    /// `(sin(x), cos(x))`.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = f64::consts::PI/4.0;
    /// let f = x.sin_cos();
    ///
    /// let abs_difference_0 = (f.0 - x.sin()).abs();
    /// let abs_difference_1 = (f.1 - x.cos()).abs();
    ///
    /// assert!(abs_difference_0 < 1e-10);
    /// assert!(abs_difference_1 < 1e-10);
    /// ```
    #[inline]
    pub fn sin_cos(self) -> (f64, f64) {
        (self.sin(), self.cos())
    }

    /// Returns `e^(self) - 1` in a way that is accurate even if the
    /// number is close to zero.
    ///
    /// ```
    /// let x = 7.0_f64;
    ///
    /// // e^(ln(7)) - 1
    /// let abs_difference = (x.ln().exp_m1() - 6.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn exp_m1(self) -> f64 {
        unsafe { cmath::expm1(self) }
    }

    /// Returns `ln(1+n)` (natural logarithm) more accurately than if
    /// the operations were performed separately.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let x = f64::consts::E - 1.0;
    ///
    /// // ln(1 + (e - 1)) == ln(e) == 1
    /// let abs_difference = (x.ln_1p() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn ln_1p(self) -> f64 {
        unsafe { cmath::log1p(self) }
    }

    /// Hyperbolic sine function.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0_f64;
    ///
    /// let f = x.sinh();
    /// // Solving sinh() at 1 gives `(e^2-1)/(2e)`
    /// let g = (e*e - 1.0)/(2.0*e);
    /// let abs_difference = (f - g).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn sinh(self) -> f64 {
        unsafe { cmath::sinh(self) }
    }

    /// Hyperbolic cosine function.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0_f64;
    /// let f = x.cosh();
    /// // Solving cosh() at 1 gives this result
    /// let g = (e*e + 1.0)/(2.0*e);
    /// let abs_difference = (f - g).abs();
    ///
    /// // Same result
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    #[inline]
    pub fn cosh(self) -> f64 {
        unsafe { cmath::cosh(self) }
    }

    /// Hyperbolic tangent function.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0_f64;
    ///
    /// let f = x.tanh();
    /// // Solving tanh() at 1 gives `(1 - e^(-2))/(1 + e^(-2))`
    /// let g = (1.0 - e.powi(-2))/(1.0 + e.powi(-2));
    /// let abs_difference = (f - g).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    #[inline]
    pub fn tanh(self) -> f64 {
        unsafe { cmath::tanh(self) }
    }

    /// Inverse hyperbolic sine function.
    ///
    /// ```
    /// let x = 1.0_f64;
    /// let f = x.sinh().asinh();
    ///
    /// let abs_difference = (f - x).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    #[inline]
    pub fn asinh(self) -> f64 {
        if self == NEG_INFINITY {
            NEG_INFINITY
        } else {
            (self + ((self * self) + 1.0).sqrt()).ln()
        }
    }

    /// Inverse hyperbolic cosine function.
    ///
    /// ```
    /// let x = 1.0_f64;
    /// let f = x.cosh().acosh();
    ///
    /// let abs_difference = (f - x).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    #[inline]
    pub fn acosh(self) -> f64 {
        match self {
            x if x < 1.0 => NAN,
            x => (x + ((x * x) - 1.0).sqrt()).ln(),
        }
    }

    /// Inverse hyperbolic tangent function.
    ///
    /// ```
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let f = e.tanh().atanh();
    ///
    /// let abs_difference = (f - e).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    #[inline]
    pub fn atanh(self) -> f64 {
        0.5 * ((2.0 * self) / (1.0 - self)).ln_1p()
    }

    // Solaris/Illumos requires a wrapper around log, log2, and log10 functions
    // because of their non-standard behavior (e.g. log(-n) returns -Inf instead
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

    /// Raw transmutation to `u64`.
    ///
    /// This is currently identical to `transmute::<f64, u64>(self)` on all platforms.
    ///
    /// See `from_bits` for some discussion of the portability of this operation
    /// (there are almost no issues).
    ///
    /// Note that this function is distinct from `as` casting, which attempts to
    /// preserve the *numeric* value, and not the bitwise value.
    ///
    #[inline]
    pub fn to_bits(self) -> u64 {
        unsafe { ::mem::transmute(self) }
    }

    /// Raw transmutation from `u64`.
    ///
    /// This is currently identical to `transmute::<u64, f64>(v)` on all platforms.
    /// It turns out this is incredibly portable, for two reasons:
    ///
    /// * Floats and Ints have the same endianness on all supported platforms.
    /// * IEEE-754 very precisely specifies the bit layout of floats.
    ///
    /// However there is one caveat: prior to the 2008 version of IEEE-754, how
    /// to interpret the NaN signaling bit wasn't actually specified. Most platforms
    /// (notably x86 and ARM) picked the interpretation that was ultimately
    /// standardized in 2008, but some didn't (notably MIPS). As a result, all
    /// signaling NaNs on MIPS are quiet NaNs on x86, and vice-versa.
    ///
    /// Rather than trying to preserve signaling-ness cross-platform, this
    /// implementation favours preserving the exact bits. This means that
    /// any payloads encoded in NaNs will be preserved even if the result of
    /// this method is sent over the network from an x86 machine to a MIPS one.
    ///
    /// If the results of this method are only manipulated by the same
    /// architecture that produced them, then there is no portability concern.
    ///
    /// If the input isn't NaN, then there is no portability concern.
    ///
    /// If you don't care about signalingness (very likely), then there is no
    /// portability concern.
    ///
    /// Note that this function is distinct from `as` casting, which attempts to
    /// preserve the *numeric* value, and not the bitwise value.
    ///
    #[inline]
    pub fn from_bits(v: u64) -> Self {
        unsafe { ::mem::transmute(v) }
    }
}
