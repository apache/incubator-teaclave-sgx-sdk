// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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
//! *[See also the `f64` primitive type](../../std/primitive.f64.html).*
//!
//! Mathematically significant numbers are provided in the `consts` sub-module.

#![allow(missing_docs)]

use intrinsics;
use sys::cmath;

pub use core::f64::{RADIX, MANTISSA_DIGITS, DIGITS, EPSILON};
pub use core::f64::{MIN_EXP, MAX_EXP, MIN_10_EXP};
pub use core::f64::{MAX_10_EXP, NAN, INFINITY, NEG_INFINITY};
pub use core::f64::{MIN, MIN_POSITIVE, MAX};
pub use core::f64::consts;

#[lang = "f64_runtime"]
impl f64 {
    /// Returns the largest integer less than or equal to a number.
    ///
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    pub fn abs(self) -> f64 {
        unsafe { intrinsics::fabsf64(self) }
    }

    /// Returns a number that represents the sign of `self`.
    ///
    /// - `1.0` if the number is positive, `+0.0` or `INFINITY`
    /// - `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
    /// - `NAN` if the number is `NAN`
    ///
    /// # Examples
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
    pub fn signum(self) -> f64 {
        if self.is_nan() {
            NAN
        } else {
            unsafe { intrinsics::copysignf64(1.0, self) }
        }
    }

    /// Fused multiply-add. Computes `(self * a) + b` with only one rounding
    /// error, yielding a more accurate result than an unfused multiply-add.
    ///
    /// Using `mul_add` can be more performant than an unfused multiply-add if
    /// the target architecture has a dedicated `fma` CPU instruction.
    ///
    /// # Examples
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

    /// Calculates Euclidean division, the matching method for `mod_euc`.
    ///
    /// This computes the integer `n` such that
    /// `self = n * rhs + self.mod_euc(rhs)`.
    /// In other words, the result is `self / rhs` rounded to the integer `n`
    /// such that `self >= n * rhs`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(euclidean_division)]
    /// let a: f64 = 7.0;
    /// let b = 4.0;
    /// assert_eq!(a.div_euc(b), 1.0); // 7.0 > 4.0 * 1.0
    /// assert_eq!((-a).div_euc(b), -2.0); // -7.0 >= 4.0 * -2.0
    /// assert_eq!(a.div_euc(-b), -1.0); // 7.0 >= -4.0 * -1.0
    /// assert_eq!((-a).div_euc(-b), 2.0); // -7.0 >= -4.0 * 2.0
    /// ```
    #[inline]
    pub fn div_euc(self, rhs: f64) -> f64 {
        let q = (self / rhs).trunc();
        if self % rhs < 0.0 {
            return if rhs > 0.0 { q - 1.0 } else { q + 1.0 }
        }
        q
    }

    /// Calculates the Euclidean modulo (self mod rhs), which is never negative.
    ///
    /// In particular, the result `n` satisfies `0 <= n < rhs.abs()`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(euclidean_division)]
    /// let a: f64 = 7.0;
    /// let b = 4.0;
    /// assert_eq!(a.mod_euc(b), 3.0);
    /// assert_eq!((-a).mod_euc(b), 1.0);
    /// assert_eq!(a.mod_euc(-b), 3.0);
    /// assert_eq!((-a).mod_euc(-b), 1.0);
    /// ```
    #[inline]
    pub fn mod_euc(self, rhs: f64) -> f64 {
        let r = self % rhs;
        if r < 0.0 {
            r + rhs.abs()
        } else {
            r
        }
    }

    /// Raises a number to an integer power.
    ///
    /// Using this function is generally faster than using `powf`
    ///
    /// # Examples
    ///
    /// ```
    /// let x = 2.0_f64;
    /// let abs_difference = (x.powi(2) - x*x).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    pub fn powi(self, n: i32) -> f64 {
        unsafe { intrinsics::powif64(self, n) }
    }

    /// Raises a number to a floating point power.
    ///
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
            #[cfg(target_os = "android")]
            return ::sys::android::log2f64(n);
            #[cfg(not(target_os = "android"))]
            return unsafe { intrinsics::log2f64(n) };
        })
    }

    /// Returns the base 10 logarithm of the number.
    ///
    /// # Examples
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

    /// Takes the cubic root of a number.
    ///
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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

    /// Computes the four quadrant arctangent of `self` (`y`) and `other` (`x`) in radians.
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-pi/2, pi/2]`
    /// * `y >= 0`: `arctan(y/x) + pi` -> `(pi/2, pi]`
    /// * `y < 0`: `arctan(y/x) - pi` -> `(-pi, -pi/2)`
    ///
    /// # Examples
    ///
    /// ```
    /// use std::f64;
    ///
    /// let pi = f64::consts::PI;
    /// // Positive angles measured counter-clockwise
    /// // from positive x axis
    /// // -pi/4 radians (45 deg clockwise)
    /// let x1 = 3.0_f64;
    /// let y1 = -3.0_f64;
    ///
    /// // 3pi/4 radians (135 deg counter-clockwise)
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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
}
