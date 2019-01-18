use std::fmt;
use std::vec::*;
use std::string::*;
use std::borrow::*;
use macros::ElementwiseComparator;

use macros::comparison::ComparisonFailure;

const MAX_MISMATCH_REPORTS: usize = 12;

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VectorElementComparisonFailure<T, E> where E: ComparisonFailure {
    pub x: T,
    pub y: T,
    pub error: E,
    pub index: usize
}

impl<T, E> fmt::Display for VectorElementComparisonFailure<T, E>
    where T: fmt::Display, E: ComparisonFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "#{index}: x = {x}, y = {y}.{reason}",
               index = self.index,
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
pub enum VectorComparisonResult<T, C, E>
    where T: Copy,
          C: ElementwiseComparator<T, E>,
          E: ComparisonFailure {
    Match,
    MismatchedDimensions {
        dim_x: usize,
        dim_y: usize
    },
    MismatchedElements {
        comparator: C,
        mismatches: Vec<VectorElementComparisonFailure<T, E>>
    }
}

impl <T, C, E> VectorComparisonResult<T, C, E>
    where T: Copy + fmt::Display, C: ElementwiseComparator<T, E>, E: ComparisonFailure {
    pub fn panic_message(&self) -> Option<String> {
        match self {
            &VectorComparisonResult::MismatchedElements { ref comparator, ref mismatches } => {
                let mut formatted_mismatches = String::new();

                let mismatches_overflow = mismatches.len() > MAX_MISMATCH_REPORTS;
                let overflow_msg = if mismatches_overflow {
                    let num_hidden_entries = mismatches.len() - MAX_MISMATCH_REPORTS;
                    format!(" ... ({} mismatching elements not shown)\n", num_hidden_entries)
                } else {
                    String::new()
                };

                for mismatch in mismatches.iter().take(MAX_MISMATCH_REPORTS) {
                    formatted_mismatches.push_str(" ");
                    formatted_mismatches.push_str(&mismatch.to_string());
                    formatted_mismatches.push_str("\n");
                }

                // Strip off the last newline from the above
                formatted_mismatches = formatted_mismatches.trim_start().to_string();

                Some(format!("\n
Vectors X and Y have {num} mismatched element pairs.
The mismatched elements are listed below, in the format
#index: x = X[index], y = Y[index].

{mismatches}
{overflow_msg}
Comparison criterion: {description}
\n",
                    num = mismatches.len(),
                    description = comparator.description(),
                    mismatches = formatted_mismatches,
                    overflow_msg = overflow_msg))
            },
            &VectorComparisonResult::MismatchedDimensions { dim_x, dim_y } => {
                Some(format!("\n
Dimensions of vectors X and Y do not match.
 dim(X) = {dim_x}
 dim(Y) = {dim_y}
\n",
                    dim_x = dim_x,
                    dim_y = dim_y))
            },
            _ => None
        }
    }
}

#[doc(hidden)]
pub fn elementwise_vector_comparison<T, C, E>(x: &[T], y: &[T], comparator: C)
    -> VectorComparisonResult<T, C, E>
    where T: Copy,
          C: ElementwiseComparator<T, E>,
          E: ComparisonFailure {
    // The reason this function takes a slice and not a Vector ref,
    // is that we the assert_vector_eq! macro to work with both
    // references and owned values
    if x.len() == y.len() {
        let n = x.len();
        let mismatches = {
            let mut mismatches = Vec::new();
            for i in 0 .. n {
                let a = x[i].to_owned();
                let b = y[i].to_owned();
                if let Err(error) = comparator.compare(a, b) {
                    mismatches.push(VectorElementComparisonFailure {
                        x: a,
                        y: b,
                        error: error,
                        index: i
                    });
                }
            }
            mismatches
        };

        if mismatches.is_empty() {
            VectorComparisonResult::Match
        } else {
            VectorComparisonResult::MismatchedElements {
                comparator: comparator,
                mismatches: mismatches
            }
        }
    } else {
        VectorComparisonResult::MismatchedDimensions { dim_x: x.len(), dim_y: y.len() }
    }
}

/// Compare vectors for exact or approximate equality.
///
/// This macro works analogously to [assert_matrix_eq!](macro.assert_matrix_eq.html),
/// but is used for comparing instances of [Vector](vector/struct.Vector.html) rather than
/// matrices. Please see the documentation for `assert_matrix_eq!`
/// for details about issues that arise when comparing floating-point numbers,
/// as well as an explanation for how these macros can be used to resolve
/// these issues.
#[macro_export]
macro_rules! assert_vector_eq {
    ($x:expr, $y:expr) => {
        {
            // Note: The reason we take slices of both x and y is that if x or y are passed as references,
            // we don't attempt to call elementwise_matrix_comparison with a &&BaseMatrix type (double reference),
            // which does not work due to generics.
            use $crate::macros::{elementwise_vector_comparison, ExactElementwiseComparator};
            let comp = ExactElementwiseComparator;
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                // Note: We need the panic to incur here inside of the macro in order
                // for the line number to be correct when using it for tests,
                // hence we build the panic message in code, but panic here.
                panic!("{msg}
Please see the documentation for ways to compare vectors approximately.\n\n",
                    msg = msg.trim_right());
            }
        }
    };
    ($x:expr, $y:expr, comp = exact) => {
        {
            use $crate::macros::{elementwise_vector_comparison, ExactElementwiseComparator};
            let comp = ExactElementwiseComparator;
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = abs, tol = $tol:expr) => {
        {
            use $crate::macros::{elementwise_vector_comparison, AbsoluteElementwiseComparator};
            let comp = AbsoluteElementwiseComparator { tol: $tol };
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = ulp, tol = $tol:expr) => {
        {
            use $crate::macros::{elementwise_vector_comparison, UlpElementwiseComparator};
            let comp = UlpElementwiseComparator { tol: $tol };
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = float) => {
        {
            use $crate::macros::{elementwise_vector_comparison, FloatElementwiseComparator};
            let comp = FloatElementwiseComparator::default();
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    // This following allows us to optionally tweak the epsilon and ulp tolerances
    // used in the default float comparator.
    ($x:expr, $y:expr, comp = float, $($key:ident = $val:expr),+) => {
        {
            use $crate::macros::{elementwise_vector_comparison, FloatElementwiseComparator};
            let comp = FloatElementwiseComparator::default()$(.$key($val))+;
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{
        elementwise_vector_comparison,
        VectorComparisonResult};
    use macros::comparison::{
        ExactElementwiseComparator, ExactError
    };
    use quickcheck::TestResult;

    quickcheck! {
        fn property_elementwise_vector_comparison_incompatible_vectors_yields_dimension_mismatch(
            m: usize,
            n: usize) -> TestResult {
            if m == n {
                return TestResult::discard()
            }

            // It does not actually matter which comparator we use here, but we need to pick one
            let comp = ExactElementwiseComparator;
            let ref x = vector![0; m];
            let ref y = vector![0; n];

            let expected = VectorComparisonResult::MismatchedDimensions { dim_x: m, dim_y: n };

            TestResult::from_bool(elementwise_vector_comparison(x.data(), y.data(), comp) == expected)
        }
    }

    quickcheck! {
        fn property_elementwise_vector_comparison_vector_matches_self(m: usize) -> bool {
            let comp = ExactElementwiseComparator;
            let ref x = vector![0; m];

            elementwise_vector_comparison(x.data(), x.data(), comp) == VectorComparisonResult::Match
        }
    }

    #[test]
    fn elementwise_vector_comparison_reports_correct_mismatches() {
        use super::VectorComparisonResult::MismatchedElements;
        use super::VectorElementComparisonFailure;

        let comp = ExactElementwiseComparator;

        {
            // Single element vectors
            let x = vector![1];
            let y = vector![2];

            let expected = MismatchedElements {
                comparator: comp,
                mismatches: vec![VectorElementComparisonFailure {
                    x: 1, y: 2,
                    error: ExactError,
                    index: 0
                }]
            };

            assert_eq!(elementwise_vector_comparison(x.data(), y.data(), comp), expected);
        }

        {
            // Mismatch for first and last elements of a vector
            let x = vector![0, 1, 2];
            let y = vector![1, 1, 3];
            let mismatches = vec![
                VectorElementComparisonFailure {
                    x: 0, y: 1,
                    error: ExactError,
                    index: 0
                },
                VectorElementComparisonFailure {
                    x: 2, y: 3,
                    error: ExactError,
                    index: 2
                }
            ];

            let expected = MismatchedElements {
                comparator: comp,
                mismatches: mismatches
            };

            assert_eq!(elementwise_vector_comparison(x.data(), y.data(), comp), expected);
        }

        {
            // Check some arbitrary elements
            let x = vector![0, 1, 2, 3, 4, 5];
            let y = vector![0, 2, 2, 3, 5, 5];

            let mismatches = vec![
                VectorElementComparisonFailure {
                    x: 1, y: 2,
                    error: ExactError,
                    index: 1
                },
                VectorElementComparisonFailure {
                    x: 4, y: 5,
                    error: ExactError,
                    index: 4
                }
            ];

            let expected = MismatchedElements {
                comparator: comp,
                mismatches: mismatches
            };

            assert_eq!(elementwise_vector_comparison(x.data(), y.data(), comp), expected);
        }
    }

    #[test]
    pub fn vector_eq_default_compare_self_for_integer() {
        let x = vector![1, 2, 3 , 4];
        assert_vector_eq!(x, x);
    }

    #[test]
    pub fn vector_eq_default_compare_self_for_floating_point() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        assert_vector_eq!(x, x);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_default_mismatched_elements() {
        let x = vector![1, 2, 3, 4];
        let y = vector![1, 2, 4, 4];
        assert_vector_eq!(x, y);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_default_mismatched_dimensions() {
        let x = vector![1, 2, 3, 4];
        let y = vector![1, 2, 3];
        assert_vector_eq!(x, y);
    }

    #[test]
    pub fn vector_eq_exact_compare_self_for_integer() {
        let x = vector![1, 2, 3, 4];
        assert_vector_eq!(x, x, comp = exact);
    }

    #[test]
    pub fn vector_eq_exact_compare_self_for_floating_point() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        assert_vector_eq!(x, x, comp = exact);;
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_exact_mismatched_elements() {
        let x = vector![1, 2, 3, 4];
        let y = vector![1, 2, 4, 4];
        assert_vector_eq!(x, y, comp = exact);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_exact_mismatched_dimensions() {
        let x = vector![1, 2, 3, 4];
        let y = vector![1, 2, 3];
        assert_vector_eq!(x, y, comp = exact);
    }

    #[test]
    pub fn vector_eq_abs_compare_self_for_integer() {
        let x = vector![1, 2, 3, 4];
        assert_vector_eq!(x, x, comp = abs, tol = 1);
    }

    #[test]
    pub fn vector_eq_abs_compare_self_for_floating_point() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        assert_vector_eq!(x, x, comp = abs, tol = 1e-8);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_abs_mismatched_elements() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        let y = vector![1.0, 2.0, 4.0, 4.0];
        assert_vector_eq!(x, y, comp = abs, tol = 1e-8);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_abs_mismatched_dimensions() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        let y = vector![1.0, 2.0, 4.0];
        assert_vector_eq!(x, y, comp = abs, tol = 1e-8);
    }

    #[test]
    pub fn vector_eq_ulp_compare_self() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        assert_vector_eq!(x, x, comp = ulp, tol = 1);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_ulp_mismatched_elements() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        let y = vector![1.0, 2.0, 4.0, 4.0];
        assert_vector_eq!(x, y, comp = ulp, tol = 4);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_ulp_mismatched_dimensions() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        let y = vector![1.0, 2.0, 4.0];
        assert_vector_eq!(x, y, comp = ulp, tol = 4);
    }

    #[test]
    pub fn vector_eq_float_compare_self() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        assert_vector_eq!(x, x, comp = ulp, tol = 1);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_float_mismatched_elements() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        let y = vector![1.0, 2.0, 4.0, 4.0];
        assert_vector_eq!(x, y, comp = float);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_float_mismatched_dimensions() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        let y = vector![1.0, 2.0, 4.0];
        assert_vector_eq!(x, y, comp = float);
    }

    #[test]
    pub fn vector_eq_float_compare_self_with_eps() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        assert_vector_eq!(x, x, comp = float, eps = 1e-6);
    }

    #[test]
    pub fn vector_eq_float_compare_self_with_ulp() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        assert_vector_eq!(x, x, comp = float, ulp = 12);
    }

    #[test]
    pub fn vector_eq_float_compare_self_with_eps_and_ulp() {
        let x = vector![1.0, 2.0, 3.0, 4.0];
        assert_vector_eq!(x, x, comp = float, eps = 1e-6, ulp = 12);
        assert_vector_eq!(x, x, comp = float, ulp = 12, eps = 1e-6);
    }

    #[test]
    pub fn vector_eq_pass_by_ref()
    {
        let x = vector![0.0];

        // Exercise all the macro definitions and make sure that we are able to call it
        // when the arguments are references.
        assert_vector_eq!(&x, &x);
        assert_vector_eq!(&x, &x, comp = exact);
        assert_vector_eq!(&x, &x, comp = abs, tol = 0.0);
        assert_vector_eq!(&x, &x, comp = ulp, tol = 0);
        assert_vector_eq!(&x, &x, comp = float);
        assert_vector_eq!(&x, &x, comp = float, eps = 0.0, ulp = 0);
    }
}
