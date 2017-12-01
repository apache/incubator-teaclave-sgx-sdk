//! Tools for ULP-based comparison of floating point numbers.
use std::mem;
use std::borrow::*;

/// Represents the result of an ULP-based comparison between two floating point numbers.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UlpComparisonResult
{
    /// Signifies an exact match between two floating point numbers.
    ExactMatch,
    /// The difference in ULP between two floating point numbers.
    Difference(u64),
    /// The two floating point numbers have different signs,
    /// and cannot be compared in a meaningful way.
    IncompatibleSigns,
    /// One or both of the two floating point numbers is a NaN,
    /// in which case the ULP comparison is not meaningful.
    Nan
}

/// Floating point types for which two instances can be compared for Unit in the Last Place (ULP) difference.
///
/// Implementing this trait enables the usage of the `ulp` comparator in
/// [assert_matrix_eq!](../macro.assert_matrix_eq!.html) for the given type.
///
/// The definition here leverages the fact that for two adjacent floating point numbers,
/// their integer representations are also adjacent.
///
/// A somewhat accessible (but not exhaustive) guide on the topic is available in the popular article
/// [Comparing Floating Point Numbers, 2012 Edition]
/// (https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/).
///
/// Implementations for `f32` and `f64` are already available, and so users should not normally
/// need to implement this. In the case when a custom implementation is necessary,
/// please see the possible return values for [UlpComparisonResult](ulp/enum.UlpComparisonResult.html).
/// Otherwise, we can recommend to read the source code of the included `f32` and `f64` implementations.
pub trait Ulp {
    /// Returns the difference between two floating point numbers, measured in ULP.
    fn ulp_diff(a: &Self, b: &Self) -> UlpComparisonResult;
}

macro_rules! impl_float_ulp {
    ($ftype:ty, $itype:ty) => {
        impl Ulp for $ftype {
            fn ulp_diff(a: &Self, b: &Self) -> UlpComparisonResult {
                if a == b {
                    UlpComparisonResult::ExactMatch
                } else if a.is_nan() || b.is_nan() {
                    // ULP comparison does not make much sense for NaN
                    UlpComparisonResult::Nan
                } else if a.is_sign_positive() != b.is_sign_positive() {
                    // ULP is not meaningful when the signs of the two numbers differ
                    UlpComparisonResult::IncompatibleSigns
                } else {
                    // Otherwise, we compute the ULP diff as the difference of the signed integer representations
                    let a_int = unsafe { mem::transmute::<$ftype, $itype>(a.to_owned()) };
                    let b_int = unsafe { mem::transmute::<$ftype, $itype>(b.to_owned()) };
                    UlpComparisonResult::Difference((b_int - a_int).abs() as u64)
                }
            }
        }
    }
}

impl_float_ulp!(f32, i32);
impl_float_ulp!(f64, i64);

#[cfg(test)]
mod tests {
    use super::Ulp;
    use super::UlpComparisonResult;
    use std::mem;
    use std::{f32, f64};
    use quickcheck::TestResult;

    #[test]
    fn plus_minus_zero_is_exact_match_f32() {
        assert!(f32::ulp_diff(&0.0, &0.0) == UlpComparisonResult::ExactMatch);
        assert!(f32::ulp_diff(&-0.0, &-0.0) == UlpComparisonResult::ExactMatch);
        assert!(f32::ulp_diff(&0.0, &-0.0) == UlpComparisonResult::ExactMatch);
        assert!(f32::ulp_diff(&-0.0, &0.0) == UlpComparisonResult::ExactMatch);
    }

    #[test]
    fn plus_minus_zero_is_exact_match_f64() {
        assert!(f64::ulp_diff(&0.0, &0.0) == UlpComparisonResult::ExactMatch);
        assert!(f64::ulp_diff(&-0.0, &-0.0) == UlpComparisonResult::ExactMatch);
        assert!(f64::ulp_diff(&0.0, &-0.0) == UlpComparisonResult::ExactMatch);
        assert!(f64::ulp_diff(&-0.0, &0.0) == UlpComparisonResult::ExactMatch);
    }

    #[test]
    fn f32_double_nan() {
        assert!(f32::ulp_diff(&f32::NAN, &f32::NAN) == UlpComparisonResult::Nan);
    }

    #[test]
    fn f64_double_nan() {
        assert!(f64::ulp_diff(&f64::NAN, &f64::NAN) == UlpComparisonResult::Nan);
    }

    quickcheck! {
        fn property_exact_match_for_finite_f32_self_comparison(x: f32) -> TestResult {
            if x.is_finite() {
                TestResult::from_bool(f32::ulp_diff(&x, &x) == UlpComparisonResult::ExactMatch)
            } else {
                TestResult::discard()
            }
        }
    }

    quickcheck! {
        fn property_exact_match_for_finite_f64_self_comparison(x: f64) -> TestResult {
            if x.is_finite() {
                TestResult::from_bool(f64::ulp_diff(&x, &x) == UlpComparisonResult::ExactMatch)
            } else {
                TestResult::discard()
            }
        }
    }

    quickcheck! {
        fn property_recovers_ulp_diff_when_f32_constructed_from_i32(a: i32, b: i32) -> TestResult {
            if a == b {
                // Ignore self-comparisons, as it makes the below test have more complicated logic,
                // and moreover we test self-comparisons in another property.
                return TestResult::discard();
            }

            let x = unsafe { mem::transmute::<i32, f32>(a) };
            let y = unsafe { mem::transmute::<i32, f32>(b) };

            // Discard the input if it's non-finite or has different signs
            if x.is_finite() && y.is_finite() && x.signum() == y.signum() {
                TestResult::from_bool(f32::ulp_diff(&x, &y) == UlpComparisonResult::Difference((b - a).abs() as u64))
            } else {
                TestResult::discard()
            }
        }
    }

    quickcheck! {
        fn property_recovers_ulp_diff_when_f64_constructed_from_i64(a: i64, b: i64) -> TestResult {
            if a == b {
                // Ignore self-comparisons, as it makes the below test have more complicated logic,
                // and moreover we test self-comparisons in another property.
                return TestResult::discard();
            }

            let x = unsafe { mem::transmute::<i64, f64>(a) };
            let y = unsafe { mem::transmute::<i64, f64>(b) };

            // Discard the input if it's non-finite or has different signs
            if x.is_finite() && y.is_finite() && x.signum() == y.signum() {
                TestResult::from_bool(f64::ulp_diff(&x, &y) == UlpComparisonResult::Difference((b - a).abs() as u64))
            } else {
                TestResult::discard()
            }
        }
    }

    quickcheck! {
        fn property_f32_incompatible_signs_yield_corresponding_enum_value(x: f32, y: f32) -> TestResult {
            if x.signum() == y.signum() {
                TestResult::discard()
            } else if x.is_nan() || y.is_nan() {
                TestResult::discard()
            } else {
                TestResult::from_bool(f32::ulp_diff(&x, &y) == UlpComparisonResult::IncompatibleSigns)
            }
        }
    }

    quickcheck! {
        fn property_f64_incompatible_signs_yield_corresponding_enum_value(x: f64, y: f64) -> TestResult {
            if x.signum() == y.signum() {
                TestResult::discard()
            } else if x.is_nan() || y.is_nan() {
                TestResult::discard()
            } else {
                TestResult::from_bool(f64::ulp_diff(&x, &y) == UlpComparisonResult::IncompatibleSigns)
            }
        }
    }

    quickcheck! {
        fn property_f32_nan_gives_nan_enum_value(x: f32) -> bool {
            f32::ulp_diff(&f32::NAN, &x) == UlpComparisonResult::Nan
            && f32::ulp_diff(&x, &f32::NAN) == UlpComparisonResult::Nan
        }
    }

    quickcheck! {
        fn property_f64_nan_gives_nan_enum_value(x: f64) -> bool {
            f64::ulp_diff(&f64::NAN, &x) == UlpComparisonResult::Nan
            && f64::ulp_diff(&x, &f64::NAN) == UlpComparisonResult::Nan
        }
    }
}
