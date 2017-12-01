use std::fmt;
use std::string::*;

use macros::ElementwiseComparator;
use macros::comparison::ComparisonFailure;


#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ScalarComparisonFailure<T, E> where E: ComparisonFailure {
    pub x: T,
    pub y: T,
    pub error: E
}

impl<T, E> fmt::Display for ScalarComparisonFailure<T, E>
    where T: fmt::Display, E: ComparisonFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "x = {x}, y = {y}.{reason}",
               x = self.x,
               y = self.y,
               reason = self.error.failure_reason()
                                  // Add a space before the reason
                                  .map(|s| format!(" {}", s))
                                  .unwrap_or(String::new()))
    }
}

#[doc(hidden)]
#[derive(Debug, PartialEq)]
pub enum ScalarComparisonResult<T, C, E>
    where T: Copy,
          C: ElementwiseComparator<T, E>,
          E: ComparisonFailure
{
    Match,
    Mismatch {
        comparator: C,
        mismatch: ScalarComparisonFailure<T, E>
    }
}

impl <T, C, E> ScalarComparisonResult<T, C, E>
    where T: Copy + fmt::Display, C: ElementwiseComparator<T, E>, E: ComparisonFailure {
    pub fn panic_message(&self) -> Option<String> {
        match self {
            &ScalarComparisonResult::Mismatch { ref comparator, ref mismatch } => {
                Some(format!("\n
Scalars x and y do not compare equal.

{mismatch}

Comparison criterion: {description}
\n",
                    description = comparator.description(),
                    mismatch = mismatch.to_string()
                ))
            },
            _ => None
        }
    }
}

#[doc(hidden)]
pub fn scalar_comparison<T, C, E>(x: T, y: T, comparator: C)
    -> ScalarComparisonResult<T, C, E>
    where T: Copy,
          C: ElementwiseComparator<T, E>,
          E: ComparisonFailure {

    match comparator.compare(x, y) {
        Err(error) => ScalarComparisonResult::Mismatch {
            comparator: comparator,
            mismatch: ScalarComparisonFailure {
                x: x,
                y: y,
                error: error
            }
        },
        _ => ScalarComparisonResult::Match
    }
}

/// Compare scalars for exact or approximate equality.
///
/// This macro works analogously to [assert_matrix_eq!](macro.assert_matrix_eq.html),
/// but is used for comparing scalars (e.g. integers, floating-point numbers)
/// rather than matrices. Please see the documentation for `assert_matrix_eq!`
/// for details about issues that arise when comparing floating-point numbers,
/// as well as an explanation for how these macros can be used to resolve
/// these issues.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rulinalg; fn main() {
/// let x = 3.00;
/// let y = 3.05;
/// // Assert that |x - y| <= 0.1
/// assert_scalar_eq!(x, y, comp = abs, tol = 0.1);
/// # }
/// ```
#[macro_export]
macro_rules! assert_scalar_eq {
    ($x:expr, $y:expr) => {
        {
            use $crate::macros::{scalar_comparison, ExactElementwiseComparator};
            let comp = ExactElementwiseComparator;
            let msg = scalar_comparison($x.clone(), $y.clone(), comp).panic_message();
            if let Some(msg) = msg {
                // Note: We need the panic to incur here inside of the macro in order
                // for the line number to be correct when using it for tests,
                // hence we build the panic message in code, but panic here.
                panic!("{msg}
Please see the documentation for ways to compare scalars approximately.\n\n",
                    msg = msg.trim_right());
            }
        }
    };
    ($x:expr, $y:expr, comp = exact) => {
        {
            use $crate::macros::{scalar_comparison, ExactElementwiseComparator};
            let comp = ExactElementwiseComparator;
            let msg = scalar_comparison($x.clone(), $y.clone(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = abs, tol = $tol:expr) => {
        {
            use $crate::macros::{scalar_comparison, AbsoluteElementwiseComparator};
            let comp = AbsoluteElementwiseComparator { tol: $tol };
            let msg = scalar_comparison($x.clone(), $y.clone(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = ulp, tol = $tol:expr) => {
        {
            use $crate::macros::{scalar_comparison, UlpElementwiseComparator};
            let comp = UlpElementwiseComparator { tol: $tol };
            let msg = scalar_comparison($x.clone(), $y.clone(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = float) => {
        {
            use $crate::macros::{scalar_comparison, FloatElementwiseComparator};
            let comp = FloatElementwiseComparator::default();
            let msg = scalar_comparison($x.clone(), $y.clone(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    // The following allows us to optionally tweak the epsilon and ulp tolerances
    // used in the default float comparator.
    ($x:expr, $y:expr, comp = float, $($key:ident = $val:expr),+) => {
        {
            use $crate::macros::{scalar_comparison, FloatElementwiseComparator};
            let comp = FloatElementwiseComparator::default()$(.$key($val))+;
            let msg = scalar_comparison($x.clone(), $y.clone(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::scalar_comparison;
    use macros::comparison::{
        ExactElementwiseComparator, ExactError
    };

    #[test]
    fn scalar_comparison_reports_correct_mismatch() {
        use super::ScalarComparisonResult::Mismatch;
        use super::ScalarComparisonFailure;

        let comp = ExactElementwiseComparator;

        {
            let x = 0.2;
            let y = 0.3;

            let expected = Mismatch {
                comparator: comp,
                mismatch: ScalarComparisonFailure {
                    x: 0.2, y: 0.3,
                    error: ExactError
                }
            };

            assert_eq!(scalar_comparison(x, y, comp), expected);
        }
    }

    #[test]
    pub fn scalar_eq_default_compare_self_for_integer() {
        let x = 2;
        assert_scalar_eq!(x, x);
    }

    #[test]
    pub fn scalar_eq_default_compare_self_for_floating_point() {
        let x = 2.0;
        assert_scalar_eq!(x, x);
    }

    #[test]
    #[should_panic]
    pub fn scalar_eq_default_mismatched_elements() {
        let x = 3;
        let y = 4;
        assert_scalar_eq!(x, y);
    }

    #[test]
    pub fn scalar_eq_exact_compare_self_for_integer() {
        let x = 2;
        assert_scalar_eq!(x, x, comp = exact);
    }

    #[test]
    pub fn scalar_eq_exact_compare_self_for_floating_point() {
        let x = 2.0;
        assert_scalar_eq!(x, x, comp = exact);;
    }

    #[test]
    #[should_panic]
    pub fn scalar_eq_exact_mismatched_elements() {
        let x = 3;
        let y = 4;
        assert_scalar_eq!(x, y, comp = exact);
    }

    #[test]
    pub fn scalar_eq_abs_compare_self_for_integer() {
        let x = 2;
        assert_scalar_eq!(x, x, comp = abs, tol = 1);
    }

    #[test]
    pub fn scalar_eq_abs_compare_self_for_floating_point() {
        let x = 2.0;
        assert_scalar_eq!(x, x, comp = abs, tol = 1e-8);
    }

    #[test]
    #[should_panic]
    pub fn scalar_eq_abs_mismatched_elements() {
        let x = 3.0;
        let y = 4.0;
        assert_scalar_eq!(x, y, comp = abs, tol = 1e-8);
    }

    #[test]
    pub fn scalar_eq_ulp_compare_self() {
        let x = 2.0;
        assert_scalar_eq!(x, x, comp = ulp, tol = 1);
    }

    #[test]
    #[should_panic]
    pub fn scalar_eq_ulp_mismatched_elements() {
        let x = 3.0;
        let y = 4.0;
        assert_scalar_eq!(x, y, comp = ulp, tol = 4);
    }

    #[test]
    pub fn scalar_eq_float_compare_self() {
        let x = 2.0;
        assert_scalar_eq!(x, x, comp = ulp, tol = 1);
    }

    #[test]
    #[should_panic]
    pub fn scalar_eq_float_mismatched_elements() {
        let x = 3.0;
        let y = 4.0;
        assert_scalar_eq!(x, y, comp = float);
    }

    #[test]
    pub fn scalar_eq_float_compare_self_with_eps() {
        let x = 2.0;
        assert_scalar_eq!(x, x, comp = float, eps = 1e-6);
    }

    #[test]
    pub fn scalar_eq_float_compare_self_with_ulp() {
        let x = 2.0;
        assert_scalar_eq!(x, x, comp = float, ulp = 12);
    }

    #[test]
    pub fn scalar_eq_float_compare_self_with_eps_and_ulp() {
        let x = 2.0;
        assert_scalar_eq!(x, x, comp = float, eps = 1e-6, ulp = 12);
        assert_scalar_eq!(x, x, comp = float, ulp = 12, eps = 1e-6);
    }

    #[test]
    pub fn scalar_eq_pass_by_ref()
    {
        let x = 0.0;

        // Exercise all the macro definitions and make sure that we are able to call it
        // when the arguments are references.
        assert_scalar_eq!(&x, &x);
        assert_scalar_eq!(&x, &x, comp = exact);
        assert_scalar_eq!(&x, &x, comp = abs, tol = 0.0);
        assert_scalar_eq!(&x, &x, comp = ulp, tol = 0);
        assert_scalar_eq!(&x, &x, comp = float);
        assert_scalar_eq!(&x, &x, comp = float, eps = 0.0, ulp = 0);
    }
}
