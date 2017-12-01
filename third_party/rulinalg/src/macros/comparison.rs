use ulp;
use ulp::Ulp;
use std::string::*;

use libnum::{Num, Float};

use std::fmt;

/// Trait that describes elementwise comparators for [assert_matrix_eq!](../macro.assert_matrix_eq!.html).
///
/// Usually you should not need to interface with this trait directly. It is a part of the documentation
/// only so that the trait bounds for the comparators are made public.
pub trait ElementwiseComparator<T, E> where T: Copy, E: ComparisonFailure {
    /// Compares two elements.
    ///
    /// Returns the error associated with the comparison if it failed.
    fn compare(&self, x: T, y: T) -> Result<(), E>;

    /// A description of the comparator.
    fn description(&self) -> String;
}

#[doc(hidden)]
pub trait ComparisonFailure {
    fn failure_reason(&self) -> Option<String>;
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AbsoluteError<T>(pub T);

/// The `abs` comparator used with [assert_matrix_eq!](../macro.assert_matrix_eq!.html).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AbsoluteElementwiseComparator<T> {
    /// The maximum absolute difference tolerated (inclusive).
    pub tol: T
}

impl<T> ComparisonFailure for AbsoluteError<T> where T: fmt::Display {
    fn failure_reason(&self) -> Option<String> {
        Some(
            format!("Absolute error: {error}.", error = self.0)
        )
    }
}

impl<T> ElementwiseComparator<T, AbsoluteError<T>> for AbsoluteElementwiseComparator<T>
    where T: Copy + fmt::Display + Num + PartialOrd<T> {

    fn compare(&self, a: T, b: T) -> Result<(), AbsoluteError<T>> {
        assert!(self.tol >= T::zero());

        // Note: Cannot use num::abs because we do not want to restrict
        // ourselves to Signed types (i.e. we still want to be able to
        // handle unsigned types).

        if a == b {
            Ok(())
        } else {
            let distance = if a > b { a - b } else { b - a };
            if distance <= self.tol {
                Ok(())
            } else {
                Err(AbsoluteError(distance))
            }
        }
    }

    fn description(&self) -> String {
        format!("absolute difference, |x - y| <= {tol}.", tol = self.tol)
    }
}

/// The `exact` comparator used with [assert_matrix_eq!](../macro.assert_matrix_eq!.html).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ExactElementwiseComparator;

#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ExactError;

impl ComparisonFailure for ExactError {
    fn failure_reason(&self) -> Option<String> { None }
}

impl<T> ElementwiseComparator<T, ExactError> for ExactElementwiseComparator
    where T: Copy + fmt::Display + PartialEq<T> {

    fn compare(&self, a: T, b: T) -> Result<(), ExactError> {
        if a == b {
            Ok(())
        } else {
            Err(ExactError)
        }
    }

    fn description(&self) -> String {
        format!("exact equality x == y.")
    }
}

/// The `ulp` comparator used with [assert_matrix_eq!](../macro.assert_matrix_eq!.html).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UlpElementwiseComparator {
    /// The maximum difference in ULP units tolerated (inclusive).
    pub tol: u64
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UlpError(pub ulp::UlpComparisonResult);

impl ComparisonFailure for UlpError {
    fn failure_reason(&self) -> Option<String> {
        use ulp::UlpComparisonResult;
        match self.0 {
            UlpComparisonResult::Difference(diff) =>
                Some(format!("Difference: {diff} ULP.", diff=diff)),
            UlpComparisonResult::IncompatibleSigns =>
                Some(format!("Numbers have incompatible signs.")),
            _ => None
        }
    }
}

impl<T> ElementwiseComparator<T, UlpError> for UlpElementwiseComparator
    where T: Copy + Ulp {

    fn compare(&self, a: T, b: T) -> Result<(), UlpError> {
        let diff = Ulp::ulp_diff(&a, &b);
        match diff {
            ulp::UlpComparisonResult::ExactMatch => Ok(()),
            ulp::UlpComparisonResult::Difference(diff) if diff <= self.tol => Ok(()),
            _ => Err(UlpError(diff))
        }
    }

    fn description(&self) -> String {
        format!("ULP difference less than or equal to {tol}. See documentation for details.",
                tol = self.tol)
    }
}

/// The `float` comparator used with [assert_matrix_eq!](../macro.assert_matrix_eq!.html).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FloatElementwiseComparator<T> {
    abs: AbsoluteElementwiseComparator<T>,
    ulp: UlpElementwiseComparator
}

#[doc(hidden)]
#[allow(dead_code)]
impl<T> FloatElementwiseComparator<T> where T: Float + Ulp {
    pub fn default() -> Self {
        FloatElementwiseComparator {
            abs: AbsoluteElementwiseComparator { tol: T::epsilon() },
            ulp: UlpElementwiseComparator { tol: 4 }
        }
    }

    pub fn eps(self, eps: T) -> Self {
        FloatElementwiseComparator {
            abs: AbsoluteElementwiseComparator { tol: eps },
            ulp: self.ulp
        }
    }

    pub fn ulp(self, max_ulp: u64) -> Self {
        FloatElementwiseComparator {
            abs: self.abs,
            ulp: UlpElementwiseComparator { tol: max_ulp }
        }
    }
}

impl<T> ElementwiseComparator<T, UlpError> for FloatElementwiseComparator<T>
    where T: Copy + Ulp + Float + fmt::Display {
    fn compare(&self, a: T, b: T) -> Result<(), UlpError> {
        // First perform an absolute comparison with a presumably very small epsilon tolerance
        if let Err(_) = self.abs.compare(a, b) {
            // Then fall back to an ULP-based comparison
            self.ulp.compare(a, b)
        } else {
            // If the epsilon comparison succeeds, we have a match
             Ok(())
        }
    }

    fn description(&self) -> String {
        format!("
Epsilon-sized absolute comparison, followed by an ULP-based comparison.
Please see the documentation for details.
Epsilon:       {eps}
ULP tolerance: {ulp}",
            eps = self.abs.tol,
            ulp = self.ulp.tol)
    }
}

#[cfg(test)]
mod tests {
    use macros::comparison::{
        AbsoluteElementwiseComparator, AbsoluteError, ElementwiseComparator,
        ExactElementwiseComparator, ExactError,
        UlpElementwiseComparator, UlpError,
        FloatElementwiseComparator,
    };
    use ulp::{Ulp, UlpComparisonResult};
    use quickcheck::TestResult;
    use std::f64;

    /// Returns the next adjacent floating point number (in the direction of positive infinity)
    fn next_f64(x: f64) -> f64 {
        use std::mem;
        let as_int = unsafe { mem::transmute::<f64, i64>(x) };
        unsafe { mem::transmute::<i64, f64>(as_int + 1) }
    }

    #[test]
    pub fn absolute_comparator_integer() {
        let comp = AbsoluteElementwiseComparator { tol: 1 };

        assert_eq!(comp.compare(0, 0), Ok(()));
        assert_eq!(comp.compare(1, 0), Ok(()));
        assert_eq!(comp.compare(-1, 0), Ok(()));
        assert_eq!(comp.compare(2, 0), Err(AbsoluteError(2)));
        assert_eq!(comp.compare(-2, 0), Err(AbsoluteError(2)));
    }

    #[test]
    pub fn absolute_comparator_floating_point() {
        let comp = AbsoluteElementwiseComparator { tol: 1.0 };

        // Note: floating point math is not generally exact, but
        // here we only compare with 0.0, so we can expect exact results.
        assert_eq!(comp.compare(0.0, 0.0), Ok(()));
        assert_eq!(comp.compare(1.0, 0.0), Ok(()));
        assert_eq!(comp.compare(-1.0, 0.0), Ok(()));
        assert_eq!(comp.compare(2.0, 0.0), Err(AbsoluteError(2.0)));
        assert_eq!(comp.compare(-2.0, 0.0), Err(AbsoluteError(2.0)));
    }

    quickcheck! {
        fn property_absolute_comparator_is_symmetric_i64(a: i64, b: i64, tol: i64) -> TestResult {
            if tol <= 0 {
                return TestResult::discard()
            }

            let comp = AbsoluteElementwiseComparator { tol: tol };
            TestResult::from_bool(comp.compare(a, b) == comp.compare(b, a))
        }
    }

    quickcheck! {
        fn property_absolute_comparator_is_symmetric_f64(a: f64, b: f64, tol: f64) -> TestResult {
            if tol <= 0.0 {
                return TestResult::discard()
            }

            // Floating point math is not exact, but the AbsoluteElementwiseComparator is designed
            // so that it gives exactly the same result when the argument positions are reversed
            let comp = AbsoluteElementwiseComparator { tol: tol };
            TestResult::from_bool(comp.compare(a, b) == comp.compare(b, a))
        }
    }

    quickcheck! {
        fn property_absolute_comparator_tolerance_is_not_strict_f64(tol: f64) -> TestResult {
            if tol <= 0.0 || !tol.is_finite() {
                return TestResult::discard()
            }

            // The comparator is defined by <=, not <
            let comp = AbsoluteElementwiseComparator { tol: tol };
            let includes_tol = comp.compare(tol, 0.0).is_ok();
            let excludes_next_after_tol = comp.compare(next_f64(tol), 0.0).is_err();
            TestResult::from_bool(includes_tol && excludes_next_after_tol)
        }
    }

    #[test]
    pub fn exact_comparator_integer() {
        let comp = ExactElementwiseComparator;

        assert_eq!(comp.compare(0, 0), Ok(()));
        assert_eq!(comp.compare(1, 0), Err(ExactError));
        assert_eq!(comp.compare(-1, 0), Err(ExactError));
        assert_eq!(comp.compare(1, -1), Err(ExactError));
    }

    #[test]
    pub fn exact_comparator_floating_point() {
        let comp = ExactElementwiseComparator;

        assert_eq!(comp.compare(0.0, 0.0), Ok(()));
        assert_eq!(comp.compare(-0.0, -0.0), Ok(()));
        assert_eq!(comp.compare(-0.0, 0.0), Ok(()));
        assert_eq!(comp.compare(1.0, 0.0), Err(ExactError));
        assert_eq!(comp.compare(-1.0, 0.0), Err(ExactError));
        assert_eq!(comp.compare(f64::NAN, 5.0), Err(ExactError));
    }

    quickcheck! {
        fn property_exact_comparator_is_symmetric_i64(a: i64, b: i64) -> bool {
            let comp = ExactElementwiseComparator;
            comp.compare(a, b) == comp.compare(b, a)
        }
    }

    quickcheck! {
        fn property_exact_comparator_is_symmetric_f64(a: f64, b: f64) -> bool {
            let comp = ExactElementwiseComparator;
            comp.compare(a, b) == comp.compare(b, a)
        }
    }

    quickcheck! {
        fn property_exact_comparator_matches_equality_operator_i64(a: i64, b: i64) -> bool {
            let comp = ExactElementwiseComparator;
            let result = comp.compare(a, b);

            match a == b {
                true =>  result == Ok(()),
                false => result == Err(ExactError)
            }
        }
    }

    quickcheck! {
        fn property_exact_comparator_matches_equality_operator_f64(a: f64, b: f64) -> bool {
            let comp = ExactElementwiseComparator;
            let result = comp.compare(a, b);

            match a == b {
                true =>  result == Ok(()),
                false => result == Err(ExactError)
            }
        }
    }

    #[test]
    pub fn ulp_comparator_f64() {
        // The Ulp implementation has its own set of tests, so we just want
        // to make a sample here
        let comp = UlpElementwiseComparator { tol: 1 };

        assert_eq!(comp.compare(0.0, 0.0), Ok(()));
        assert_eq!(comp.compare(0.0, -0.0), Ok(()));
        assert_eq!(comp.compare(-1.0, 1.0), Err(UlpError(UlpComparisonResult::IncompatibleSigns)));
        assert_eq!(comp.compare(1.0, 0.0), Err(UlpError(f64::ulp_diff(&1.0, &0.0))));
        assert_eq!(comp.compare(f64::NAN, 0.0), Err(UlpError(UlpComparisonResult::Nan)));;
    }

    quickcheck! {
        fn property_ulp_comparator_is_symmetric(a: f64, b: f64, tol: u64) -> TestResult {
            if tol == 0 {
                return TestResult::discard()
            }

            let comp = UlpElementwiseComparator { tol: tol };
            TestResult::from_bool(comp.compare(a, b) == comp.compare(b, a))
        }
    }

    quickcheck! {
        fn property_ulp_comparator_matches_ulp_trait(a: f64, b: f64, tol: u64) -> bool {
            let comp = UlpElementwiseComparator { tol: tol };
            let result = comp.compare(a, b);

            use ulp::UlpComparisonResult::{ExactMatch, Difference};

            match f64::ulp_diff(&a, &b) {
                ExactMatch =>                      result.is_ok(),
                Difference(diff) if diff <= tol => result.is_ok(),
                otherwise =>                       result == Err(UlpError(otherwise))
            }
        }
    }

    quickcheck! {
        fn property_ulp_comparator_next_f64_is_ok_when_inside_tolerance(x: f64) -> TestResult {
            let y = next_f64(x);

            if !(x.is_finite() && y.is_finite() && x.signum() == y.signum()) {
                return TestResult::discard()
            }

            let comp0 = UlpElementwiseComparator { tol: 0 };
            let comp1 = UlpElementwiseComparator { tol: 1 };

            let tol_0_fails = comp0.compare(x, y) == Err(UlpError(UlpComparisonResult::Difference(1)));
            let tol_1_succeeds = comp1.compare(x, y) == Ok(());

            TestResult::from_bool(tol_0_fails && tol_1_succeeds)
        }
    }

    quickcheck! {
        fn property_float_comparator_matches_abs_with_zero_ulp_tol(a: f64, b: f64, abstol: f64) -> TestResult {
            if abstol <= 0.0 {
                return TestResult::discard()
            }

            let abstol = abstol.abs();
            let comp = FloatElementwiseComparator::default().eps(abstol).ulp(0);
            let abscomp = AbsoluteElementwiseComparator { tol: abstol };
            let result = comp.compare(a, b);

            // Recall that the float comparator returns UlpError, so we cannot compare the results
            // of abscomp directly
            TestResult::from_bool(match abscomp.compare(a, b) {
                Err(AbsoluteError(_)) =>   result.is_err(),
                Ok(_) =>                   result.is_ok()
            })
        }
    }

    quickcheck! {
        fn property_float_comparator_matches_ulp_with_zero_eps_tol(a: f64, b: f64, max_ulp: u64) -> bool {
            let comp = FloatElementwiseComparator::default().eps(0.0).ulp(max_ulp);
            let ulpcomp = UlpElementwiseComparator { tol: max_ulp };

            comp.compare(a, b) == ulpcomp.compare(a, b)
        }
    }
}
