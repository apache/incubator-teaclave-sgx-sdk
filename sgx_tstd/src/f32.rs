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

#![allow(missing_docs)]

use core::num;
use core::intrinsics;
use core::num::FpCategory;

pub use core::f32::{RADIX, MANTISSA_DIGITS, DIGITS, EPSILON};
pub use core::f32::{MIN_EXP, MAX_EXP, MIN_10_EXP};
pub use core::f32::{MAX_10_EXP, NAN, INFINITY, NEG_INFINITY};
pub use core::f32::{MIN, MIN_POSITIVE, MAX};
pub use core::f32::consts;

#[allow(dead_code)]
mod cmath {
    use sgx_types::{c_float, c_int};
    #[link(name = "sgx_tstdc")]
    extern {
        pub fn cbrtf(n: c_float) -> c_float;
        pub fn erff(n: c_float) -> c_float;
        pub fn erfcf(n: c_float) -> c_float;
        pub fn expm1f(n: c_float) -> c_float;
        pub fn fdimf(a: c_float, b: c_float) -> c_float;
        pub fn fmodf(a: c_float, b: c_float) -> c_float;
        pub fn ilogbf(n: c_float) -> c_int;
        pub fn logbf(n: c_float) -> c_float;
        pub fn log1pf(n: c_float) -> c_float;
        pub fn modff(n: c_float, iptr: &mut c_float) -> c_float;
        pub fn nextafterf(x: c_float, y: c_float) -> c_float;
        pub fn tgammaf(n: c_float) -> c_float;
        pub fn lgammaf_r(n: c_float, sign: &mut c_int) -> c_float;
        pub fn hypotf(x: c_float, y: c_float) -> c_float;
    }

    // See the comments in the `floor` function for why MSVC is special
    // here.
    #[link(name = "sgx_tstdc")]
    extern {
        pub fn acosf(n: c_float) -> c_float;
        pub fn asinf(n: c_float) -> c_float;
        pub fn atan2f(a: c_float, b: c_float) -> c_float;
        pub fn atanf(n: c_float) -> c_float;
        pub fn coshf(n: c_float) -> c_float;
        pub fn sinhf(n: c_float) -> c_float;
        pub fn tanf(n: c_float) -> c_float;
        pub fn tanhf(n: c_float) -> c_float;
    }
}

#[lang = "f32"]
impl f32 {
    /// Returns `true` if this value is `NaN` and false otherwise.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let nan = f32::NAN;
    /// let f = 7.0_f32;
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
    /// use std::f32;
    ///
    /// let f = 7.0f32;
    /// let inf = f32::INFINITY;
    /// let neg_inf = f32::NEG_INFINITY;
    /// let nan = f32::NAN;
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
    /// use std::f32;
    ///
    /// let f = 7.0f32;
    /// let inf = f32::INFINITY;
    /// let neg_inf = f32::NEG_INFINITY;
    /// let nan = f32::NAN;
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
    /// use std::f32;
    ///
    /// let min = f32::MIN_POSITIVE; // 1.17549435e-38f32
    /// let max = f32::MAX;
    /// let lower_than_min = 1.0e-40_f32;
    /// let zero = 0.0_f32;
    ///
    /// assert!(min.is_normal());
    /// assert!(max.is_normal());
    ///
    /// assert!(!zero.is_normal());
    /// assert!(!f32::NAN.is_normal());
    /// assert!(!f32::INFINITY.is_normal());
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
    /// use std::f32;
    ///
    /// let num = 12.4_f32;
    /// let inf = f32::INFINITY;
    ///
    /// assert_eq!(num.classify(), FpCategory::Normal);
    /// assert_eq!(inf.classify(), FpCategory::Infinite);
    /// ```
    #[inline]
    pub fn classify(self) -> FpCategory { num::Float::classify(self) }

    /// Returns the largest integer less than or equal to a number.
    ///
    /// ```
    /// let f = 3.99_f32;
    /// let g = 3.0_f32;
    ///
    /// assert_eq!(f.floor(), 3.0);
    /// assert_eq!(g.floor(), 3.0);
    /// ```
    #[inline]
    pub fn floor(self) -> f32 {
        // On MSVC LLVM will lower many math intrinsics to a call to the
        // corresponding function. On MSVC, however, many of these functions
        // aren't actually available as symbols to call, but rather they are all
        // `static inline` functions in header files. This means that from a C
        // perspective it's "compatible", but not so much from an ABI
        // perspective (which we're worried about).
        //
        // The inline header functions always just cast to a f64 and do their
        // operation, so we do that here as well, but only for MSVC targets.
        //
        // Note that there are many MSVC-specific float operations which
        // redirect to this comment, so `floorf` is just one case of a missing
        // function on MSVC, but there are many others elsewhere.
        #[cfg(target_env = "msvc")]
        return (self as f64).floor() as f32;
        #[cfg(not(target_env = "msvc"))]
        return unsafe { intrinsics::floorf32(self) };
    }

    /// Returns the smallest integer greater than or equal to a number.
    ///
    /// ```
    /// let f = 3.01_f32;
    /// let g = 4.0_f32;
    ///
    /// assert_eq!(f.ceil(), 4.0);
    /// assert_eq!(g.ceil(), 4.0);
    /// ```
    #[inline]
    pub fn ceil(self) -> f32 {
        // see notes above in `floor`
        #[cfg(target_env = "msvc")]
        return (self as f64).ceil() as f32;
        #[cfg(not(target_env = "msvc"))]
        return unsafe { intrinsics::ceilf32(self) };
    }

    /// Returns the nearest integer to a number. Round half-way cases away from
    /// `0.0`.
    ///
    /// ```
    /// let f = 3.3_f32;
    /// let g = -3.3_f32;
    ///
    /// assert_eq!(f.round(), 3.0);
    /// assert_eq!(g.round(), -3.0);
    /// ```
    #[inline]
    pub fn round(self) -> f32 {
        unsafe { intrinsics::roundf32(self) }
    }

    /// Returns the integer part of a number.
    ///
    /// ```
    /// let f = 3.3_f32;
    /// let g = -3.7_f32;
    ///
    /// assert_eq!(f.trunc(), 3.0);
    /// assert_eq!(g.trunc(), -3.0);
    /// ```
    #[inline]
    pub fn trunc(self) -> f32 {
        unsafe { intrinsics::truncf32(self) }
    }

    /// Returns the fractional part of a number.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 3.5_f32;
    /// let y = -3.5_f32;
    /// let abs_difference_x = (x.fract() - 0.5).abs();
    /// let abs_difference_y = (y.fract() - (-0.5)).abs();
    ///
    /// assert!(abs_difference_x <= f32::EPSILON);
    /// assert!(abs_difference_y <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn fract(self) -> f32 { self - self.trunc() }

    /// Computes the absolute value of `self`. Returns `NAN` if the
    /// number is `NAN`.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 3.5_f32;
    /// let y = -3.5_f32;
    ///
    /// let abs_difference_x = (x.abs() - x).abs();
    /// let abs_difference_y = (y.abs() - (-y)).abs();
    ///
    /// assert!(abs_difference_x <= f32::EPSILON);
    /// assert!(abs_difference_y <= f32::EPSILON);
    ///
    /// assert!(f32::NAN.abs().is_nan());
    /// ```
    #[inline]
    pub fn abs(self) -> f32 { num::Float::abs(self) }

    /// Returns a number that represents the sign of `self`.
    ///
    /// - `1.0` if the number is positive, `+0.0` or `INFINITY`
    /// - `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
    /// - `NAN` if the number is `NAN`
    ///
    /// ```
    /// use std::f32;
    ///
    /// let f = 3.5_f32;
    ///
    /// assert_eq!(f.signum(), 1.0);
    /// assert_eq!(f32::NEG_INFINITY.signum(), -1.0);
    ///
    /// assert!(f32::NAN.signum().is_nan());
    /// ```
    #[inline]
    pub fn signum(self) -> f32 { num::Float::signum(self) }

    /// Returns `true` if and only if `self` has a positive sign, including `+0.0`, `NaN`s with
    /// positive sign bit and positive infinity.
    ///
    /// ```
    /// let f = 7.0_f32;
    /// let g = -7.0_f32;
    ///
    /// assert!(f.is_sign_positive());
    /// assert!(!g.is_sign_positive());
    /// ```
    #[inline]
    pub fn is_sign_positive(self) -> bool { num::Float::is_sign_positive(self) }

    /// Returns `true` if and only if `self` has a negative sign, including `-0.0`, `NaN`s with
    /// negative sign bit and negative infinity.
    ///
    /// ```
    /// let f = 7.0f32;
    /// let g = -7.0f32;
    ///
    /// assert!(!f.is_sign_negative());
    /// assert!(g.is_sign_negative());
    /// ```
    #[inline]
    pub fn is_sign_negative(self) -> bool { num::Float::is_sign_negative(self) }

    /// Fused multiply-add. Computes `(self * a) + b` with only one rounding
    /// error. This produces a more accurate result with better performance than
    /// a separate multiplication operation followed by an add.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let m = 10.0_f32;
    /// let x = 4.0_f32;
    /// let b = 60.0_f32;
    ///
    /// // 100.0
    /// let abs_difference = (m.mul_add(x, b) - (m*x + b)).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn mul_add(self, a: f32, b: f32) -> f32 {
        unsafe { intrinsics::fmaf32(self, a, b) }
    }

    /// Takes the reciprocal (inverse) of a number, `1/x`.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 2.0_f32;
    /// let abs_difference = (x.recip() - (1.0/x)).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn recip(self) -> f32 { num::Float::recip(self) }

    /// Raises a number to an integer power.
    ///
    /// Using this function is generally faster than using `powf`
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 2.0_f32;
    /// let abs_difference = (x.powi(2) - x*x).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn powi(self, n: i32) -> f32 { num::Float::powi(self, n) }

    /// Raises a number to a floating point power.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 2.0_f32;
    /// let abs_difference = (x.powf(2.0) - x*x).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn powf(self, n: f32) -> f32 {
        // see notes above in `floor`
        #[cfg(target_env = "msvc")]
        return (self as f64).powf(n as f64) as f32;
        #[cfg(not(target_env = "msvc"))]
        return unsafe { intrinsics::powf32(self, n) };
    }

    /// Takes the square root of a number.
    ///
    /// Returns NaN if `self` is a negative number.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let positive = 4.0_f32;
    /// let negative = -4.0_f32;
    ///
    /// let abs_difference = (positive.sqrt() - 2.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// assert!(negative.sqrt().is_nan());
    /// ```
    #[inline]
    pub fn sqrt(self) -> f32 {
        if self < 0.0 {
            NAN
        } else {
            unsafe { intrinsics::sqrtf32(self) }
        }
    }

    /// Returns `e^(self)`, (the exponential function).
    ///
    /// ```
    /// use std::f32;
    ///
    /// let one = 1.0f32;
    /// // e^1
    /// let e = one.exp();
    ///
    /// // ln(e) - 1 == 0
    /// let abs_difference = (e.ln() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn exp(self) -> f32 {
        // see notes above in `floor`
        #[cfg(target_env = "msvc")]
        return (self as f64).exp() as f32;
        #[cfg(not(target_env = "msvc"))]
        return unsafe { intrinsics::expf32(self) };
    }

    /// Returns `2^(self)`.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let f = 2.0f32;
    ///
    /// // 2^2 - 4 == 0
    /// let abs_difference = (f.exp2() - 4.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn exp2(self) -> f32 {
        unsafe { intrinsics::exp2f32(self) }
    }

    /// Returns the natural logarithm of the number.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let one = 1.0f32;
    /// // e^1
    /// let e = one.exp();
    ///
    /// // ln(e) - 1 == 0
    /// let abs_difference = (e.ln() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn ln(self) -> f32 {
        // see notes above in `floor`
        #[cfg(target_env = "msvc")]
        return (self as f64).ln() as f32;
        #[cfg(not(target_env = "msvc"))]
        return unsafe { intrinsics::logf32(self) };
    }

    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let ten = 10.0f32;
    /// let two = 2.0f32;
    ///
    /// // log10(10) - 1 == 0
    /// let abs_difference_10 = (ten.log(10.0) - 1.0).abs();
    ///
    /// // log2(2) - 1 == 0
    /// let abs_difference_2 = (two.log(2.0) - 1.0).abs();
    ///
    /// assert!(abs_difference_10 <= f32::EPSILON);
    /// assert!(abs_difference_2 <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn log(self, base: f32) -> f32 { self.ln() / base.ln() }

    /// Returns the base 2 logarithm of the number.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let two = 2.0f32;
    ///
    /// // log2(2) - 1 == 0
    /// let abs_difference = (two.log2() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn log2(self) -> f32 {
        #[cfg(target_os = "android")]
        return ::sys::android::log2f32(self);
        #[cfg(not(target_os = "android"))]
        return unsafe { intrinsics::log2f32(self) };
    }

    /// Returns the base 10 logarithm of the number.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let ten = 10.0f32;
    ///
    /// // log10(10) - 1 == 0
    /// let abs_difference = (ten.log10() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn log10(self) -> f32 {
        // see notes above in `floor`
        #[cfg(target_env = "msvc")]
        return (self as f64).log10() as f32;
        #[cfg(not(target_env = "msvc"))]
        return unsafe { intrinsics::log10f32(self) };
    }

    /// Converts radians to degrees.
    ///
    /// ```
    /// use std::f32::{self, consts};
    ///
    /// let angle = consts::PI;
    ///
    /// let abs_difference = (angle.to_degrees() - 180.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn to_degrees(self) -> f32 { num::Float::to_degrees(self) }

    /// Converts degrees to radians.
    ///
    /// ```
    /// use std::f32::{self, consts};
    ///
    /// let angle = 180.0f32;
    ///
    /// let abs_difference = (angle.to_radians() - consts::PI).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn to_radians(self) -> f32 { num::Float::to_radians(self) }

    /// Returns the maximum of the two numbers.
    ///
    /// ```
    /// let x = 1.0f32;
    /// let y = 2.0f32;
    ///
    /// assert_eq!(x.max(y), y);
    /// ```
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    
    #[inline]
    pub fn max(self, other: f32) -> f32 {
        num::Float::max(self, other)
    }

    /// Returns the minimum of the two numbers.
    ///
    /// ```
    /// let x = 1.0f32;
    /// let y = 2.0f32;
    ///
    /// assert_eq!(x.min(y), x);
    /// ```
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    #[inline]
    pub fn min(self, other: f32) -> f32 {
        num::Float::min(self, other)
    }

    /// The positive difference of two numbers.
    ///
    /// * If `self <= other`: `0:0`
    /// * Else: `self - other`
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 3.0f32;
    /// let y = -3.0f32;
    ///
    /// let abs_difference_x = (x.abs_sub(1.0) - 2.0).abs();
    /// let abs_difference_y = (y.abs_sub(1.0) - 0.0).abs();
    ///
    /// assert!(abs_difference_x <= f32::EPSILON);
    /// assert!(abs_difference_y <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn abs_sub(self, other: f32) -> f32 {
        unsafe { cmath::fdimf(self, other) }
    }

    /// Takes the cubic root of a number.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 8.0f32;
    ///
    /// // x^(1/3) - 2 == 0
    /// let abs_difference = (x.cbrt() - 2.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn cbrt(self) -> f32 {
        unsafe { cmath::cbrtf(self) }
    }

    /// Calculates the length of the hypotenuse of a right-angle triangle given
    /// legs of length `x` and `y`.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 2.0f32;
    /// let y = 3.0f32;
    ///
    /// // sqrt(x^2 + y^2)
    /// let abs_difference = (x.hypot(y) - (x.powi(2) + y.powi(2)).sqrt()).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn hypot(self, other: f32) -> f32 {
        unsafe { cmath::hypotf(self, other) }
    }

    /// Computes the sine of a number (in radians).
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = f32::consts::PI/2.0;
    ///
    /// let abs_difference = (x.sin() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn sin(self) -> f32 {
        // see notes in `core::f32::Float::floor`
        #[cfg(target_env = "msvc")]
        return (self as f64).sin() as f32;
        #[cfg(not(target_env = "msvc"))]
        return unsafe { intrinsics::sinf32(self) };
    }

    /// Computes the cosine of a number (in radians).
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 2.0*f32::consts::PI;
    ///
    /// let abs_difference = (x.cos() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn cos(self) -> f32 {
        // see notes in `core::f32::Float::floor`
        #[cfg(target_env = "msvc")]
        return (self as f64).cos() as f32;
        #[cfg(not(target_env = "msvc"))]
        return unsafe { intrinsics::cosf32(self) };
    }

    /// Computes the tangent of a number (in radians).
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = f32::consts::PI / 4.0;
    /// let abs_difference = (x.tan() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn tan(self) -> f32 {
        unsafe { cmath::tanf(self) }
    }

    /// Computes the arcsine of a number. Return value is in radians in
    /// the range [-pi/2, pi/2] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// ```
    /// use std::f32;
    ///
    /// let f = f32::consts::PI / 2.0;
    ///
    /// // asin(sin(pi/2))
    /// let abs_difference = (f.sin().asin() - f32::consts::PI / 2.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn asin(self) -> f32 {
        unsafe { cmath::asinf(self) }
    }

    /// Computes the arccosine of a number. Return value is in radians in
    /// the range [0, pi] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// ```
    /// use std::f32;
    ///
    /// let f = f32::consts::PI / 4.0;
    ///
    /// // acos(cos(pi/4))
    /// let abs_difference = (f.cos().acos() - f32::consts::PI / 4.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn acos(self) -> f32 {
        unsafe { cmath::acosf(self) }
    }

    /// Computes the arctangent of a number. Return value is in radians in the
    /// range [-pi/2, pi/2];
    ///
    /// ```
    /// use std::f32;
    ///
    /// let f = 1.0f32;
    ///
    /// // atan(tan(1))
    /// let abs_difference = (f.tan().atan() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn atan(self) -> f32 {
        unsafe { cmath::atanf(self) }
    }

    /// Computes the four quadrant arctangent of `self` (`y`) and `other` (`x`).
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-pi/2, pi/2]`
    /// * `y >= 0`: `arctan(y/x) + pi` -> `(pi/2, pi]`
    /// * `y < 0`: `arctan(y/x) - pi` -> `(-pi, -pi/2)`
    ///
    /// ```
    /// use std::f32;
    ///
    /// let pi = f32::consts::PI;
    /// // All angles from horizontal right (+x)
    /// // 45 deg counter-clockwise
    /// let x1 = 3.0f32;
    /// let y1 = -3.0f32;
    ///
    /// // 135 deg clockwise
    /// let x2 = -3.0f32;
    /// let y2 = 3.0f32;
    ///
    /// let abs_difference_1 = (y1.atan2(x1) - (-pi/4.0)).abs();
    /// let abs_difference_2 = (y2.atan2(x2) - 3.0*pi/4.0).abs();
    ///
    /// assert!(abs_difference_1 <= f32::EPSILON);
    /// assert!(abs_difference_2 <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn atan2(self, other: f32) -> f32 {
        unsafe { cmath::atan2f(self, other) }
    }

    /// Simultaneously computes the sine and cosine of the number, `x`. Returns
    /// `(sin(x), cos(x))`.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = f32::consts::PI/4.0;
    /// let f = x.sin_cos();
    ///
    /// let abs_difference_0 = (f.0 - x.sin()).abs();
    /// let abs_difference_1 = (f.1 - x.cos()).abs();
    ///
    /// assert!(abs_difference_0 <= f32::EPSILON);
    /// assert!(abs_difference_1 <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn sin_cos(self) -> (f32, f32) {
        (self.sin(), self.cos())
    }

    /// Returns `e^(self) - 1` in a way that is accurate even if the
    /// number is close to zero.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 6.0f32;
    ///
    /// // e^(ln(6)) - 1
    /// let abs_difference = (x.ln().exp_m1() - 5.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn exp_m1(self) -> f32 {
        unsafe { cmath::expm1f(self) }
    }

    /// Returns `ln(1+n)` (natural logarithm) more accurately than if
    /// the operations were performed separately.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = f32::consts::E - 1.0;
    ///
    /// // ln(1 + (e - 1)) == ln(e) == 1
    /// let abs_difference = (x.ln_1p() - 1.0).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn ln_1p(self) -> f32 {
        unsafe { cmath::log1pf(self) }
    }

    /// Hyperbolic sine function.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let e = f32::consts::E;
    /// let x = 1.0f32;
    ///
    /// let f = x.sinh();
    /// // Solving sinh() at 1 gives `(e^2-1)/(2e)`
    /// let g = (e*e - 1.0)/(2.0*e);
    /// let abs_difference = (f - g).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn sinh(self) -> f32 {
        unsafe { cmath::sinhf(self) }
    }

    /// Hyperbolic cosine function.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let e = f32::consts::E;
    /// let x = 1.0f32;
    /// let f = x.cosh();
    /// // Solving cosh() at 1 gives this result
    /// let g = (e*e + 1.0)/(2.0*e);
    /// let abs_difference = (f - g).abs();
    ///
    /// // Same result
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn cosh(self) -> f32 {
        unsafe { cmath::coshf(self) }
    }

    /// Hyperbolic tangent function.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let e = f32::consts::E;
    /// let x = 1.0f32;
    ///
    /// let f = x.tanh();
    /// // Solving tanh() at 1 gives `(1 - e^(-2))/(1 + e^(-2))`
    /// let g = (1.0 - e.powi(-2))/(1.0 + e.powi(-2));
    /// let abs_difference = (f - g).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn tanh(self) -> f32 {
        unsafe { cmath::tanhf(self) }
    }

    /// Inverse hyperbolic sine function.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 1.0f32;
    /// let f = x.sinh().asinh();
    ///
    /// let abs_difference = (f - x).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn asinh(self) -> f32 {
        if self == NEG_INFINITY {
            NEG_INFINITY
        } else {
            (self + ((self * self) + 1.0).sqrt()).ln()
        }
    }

    /// Inverse hyperbolic cosine function.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let x = 1.0f32;
    /// let f = x.cosh().acosh();
    ///
    /// let abs_difference = (f - x).abs();
    ///
    /// assert!(abs_difference <= f32::EPSILON);
    /// ```
    #[inline]
    pub fn acosh(self) -> f32 {
        match self {
            x if x < 1.0 => ::f32::NAN,
            x => (x + ((x * x) - 1.0).sqrt()).ln(),
        }
    }

    /// Inverse hyperbolic tangent function.
    ///
    /// ```
    /// use std::f32;
    ///
    /// let e = f32::consts::E;
    /// let f = e.tanh().atanh();
    ///
    /// let abs_difference = (f - e).abs();
    ///
    /// assert!(abs_difference <= 1e-5);
    /// ```
    #[inline]
    pub fn atanh(self) -> f32 {
        0.5 * ((2.0 * self) / (1.0 - self)).ln_1p()
    }

    /// Raw transmutation to `u32`.
    ///
    /// Converts the `f32` into its raw memory representation,
    /// similar to the `transmute` function.
    ///
    /// Note that this function is distinct from casting.
    ///
    #[inline]
    pub fn to_bits(self) -> u32 {
        unsafe { ::mem::transmute(self) }
    }

    /// Raw transmutation from `u32`.
    ///
    /// Converts the given `u32` containing the float's raw memory
    /// representation into the `f32` type, similar to the
    /// `transmute` function.
    ///
    /// There is only one difference to a bare `transmute`:
    /// Due to the implications onto Rust's safety promises being
    /// uncertain, if the representation of a signaling NaN "sNaN" float
    /// is passed to the function, the implementation is allowed to
    /// return a quiet NaN instead.
    ///
    /// Note that this function is distinct from casting.
    ///
    #[inline]
    pub fn from_bits(mut v: u32) -> Self {
        const EXP_MASK: u32   = 0x7F800000;
        const FRACT_MASK: u32 = 0x007FFFFF;
        if v & EXP_MASK == EXP_MASK && v & FRACT_MASK != 0 {
            // While IEEE 754-2008 specifies encodings for quiet NaNs
            // and signaling ones, certain MIPS and PA-RISC
            // CPUs treat signaling NaNs differently.
            // Therefore to be safe, we pass a known quiet NaN
            // if v is any kind of NaN.
            // The check above only assumes IEEE 754-1985 to be
            // valid.
            v = unsafe { ::mem::transmute(NAN) };
        }
        unsafe { ::mem::transmute(v) }
    }
}