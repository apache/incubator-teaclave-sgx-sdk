//! The norm module
//!
//! This module contains implementations of various linear algebra norms.
//! The implementations are contained within the `VectorNorm` and 
//! `MatrixNorm` traits. This module also contains `VectorMetric` and
//! `MatrixMetric` traits which are used to compute the metric distance.
//!
//! These traits can be used directly by importing implementors from
//! this module. In most cases it will be easier to use the `norm` and
//! `metric` functions which exist for both vectors and matrices. These
//! functions take generic arguments for the norm to be used.
//!
//! In general you should use the least generic norm that fits your purpose.
//! For example you would choose to use a `Euclidean` norm instead of an
//! `Lp(2.0)` norm - despite them being mathematically equivalent. 
//!
//! # Defining your own norm
//!
//! Note that these traits enforce no requirements on the norm. It is up
//! to the user to ensure that they define a norm correctly.
//!
//! To define your own norm you need to implement the `MatrixNorm`
//! and/or the `VectorNorm` on your own struct. When you have defined
//! a norm you get the _induced metric_ for free. This means that any
//! object which implements the `VectorNorm` or `MatrixNorm` will
//! automatically implement the `VectorMetric` and `MatrixMetric` traits
//! respectively. This induced metric will compute the norm of the
//! difference between the vectors or matrices.

use matrix::BaseMatrix;
use vector::Vector;
use utils;

use std::ops::Sub;
use libnum::Float;

/// Trait for vector norms
pub trait VectorNorm<T> {
    /// Computes the vector norm.
    fn norm(&self, v: &Vector<T>) -> T;
}

/// Trait for vector metrics.
pub trait VectorMetric<T> {
    /// Computes the metric distance between two vectors.
    fn metric(&self, v1: &Vector<T>, v2: &Vector<T>) -> T;
}

/// Trait for matrix norms.
pub trait MatrixNorm<T, M: BaseMatrix<T>> {
    /// Computes the matrix norm.
    fn norm(&self, m: &M) -> T;
}

/// Trait for matrix metrics.
pub trait MatrixMetric<'a, 'b, T, M1: 'a + BaseMatrix<T>, M2: 'b + BaseMatrix<T>> {
    /// Computes the metric distance between two matrices.
    fn metric(&self, m1: &'a M1, m2: &'b M2) -> T;
}

/// The induced vector metric
///
/// Given a norm `N`, the induced vector metric `M` computes
/// the metric distance, `d`, between two vectors `v1` and `v2`
/// as follows:
///
/// `d = M(v1, v2) = N(v1 - v2)`
impl<U, T> VectorMetric<T> for U
    where U: VectorNorm<T>, T: Copy + Sub<T, Output=T> {
    fn metric(&self, v1: &Vector<T>, v2: &Vector<T>) -> T {
        self.norm(&(v1 - v2))
    }
}

/// The induced matrix metric
///
/// Given a norm `N`, the induced matrix metric `M` computes
/// the metric distance, `d`, between two matrices `m1` and `m2`
/// as follows:
///
/// `d = M(m1, m2) = N(m1 - m2)`
impl<'a, 'b, U, T, M1, M2> MatrixMetric<'a, 'b, T, M1, M2> for U
    where U: MatrixNorm<T, ::matrix::Matrix<T>>,
    M1: 'a + BaseMatrix<T>,
    M2: 'b + BaseMatrix<T>,
    &'a M1: Sub<&'b M2, Output=::matrix::Matrix<T>> {

    fn metric(&self, m1: &'a M1, m2: &'b M2) -> T {
        self.norm(&(m1 - m2))
    }
}

/// The Euclidean norm
///
/// The Euclidean norm computes the square-root
/// of the sum of squares.
///
/// `||v|| = SQRT(SUM(v_i * v_i))`
#[derive(Debug)]
pub struct Euclidean;

impl<T: Float> VectorNorm<T> for Euclidean {
    fn norm(&self, v: &Vector<T>) -> T {
        utils::dot(v.data(), v.data()).sqrt()
    }
}

impl<T: Float, M: BaseMatrix<T>> MatrixNorm<T, M> for Euclidean {
    fn norm(&self, m: &M) -> T {
        let mut s = T::zero();

        for row in m.row_iter() {
            s = s + utils::dot(row.raw_slice(), row.raw_slice());
        }

        s.sqrt()
    }
}

/// The Lp norm
///
/// The
/// [Lp norm](https://en.wikipedia.org/wiki/Norm_(mathematics)#p-norm)
/// computes the `p`th root of the sum of elements
/// to the `p`th power.
///
/// The Lp norm requires `p` to be greater than
/// or equal `1`.
///
/// We use an enum for this norm to allow us to explicitly handle
/// special cases at compile time. For example, we have an `Infinity`
/// variant which handles the special case when the `Lp` norm is a
/// supremum over absolute values. The `Integer` variant gives us a
/// performance boost when `p` is an integer.
///
/// You should avoid matching directly against this enum as it is likely
/// to grow.
#[derive(Debug)]
pub enum Lp<T: Float> {
    /// The L-infinity norm (supremum)
    Infinity,
    /// The Lp norm where p is an integer
    Integer(i32),
    /// The Lp norm where p is a float
    Float(T)
}

impl<T: Float> VectorNorm<T> for Lp<T> {
    fn norm(&self, v: &Vector<T>) -> T {
        match *self {
            Lp::Infinity => {
                // Compute supremum
                let mut abs_sup = T::zero();
                for d in v.iter().map(|d| d.abs()) {
                    if d > abs_sup {
                        abs_sup = d;
                    }
                }
                abs_sup
            },
            Lp::Integer(p) => {
                assert!(p >= 1, "p value in Lp norm must be >= 1");
                // Compute standard lp norm
                let mut s = T::zero();
                for x in v {
                    s = s + x.abs().powi(p);
                }
                s.powf(T::from(p).expect("Could not cast i32 to float").recip())
            },
            Lp::Float(p) => {
                assert!(p >= T::one(), "p value in Lp norm must be >= 1");
                // Compute standard lp norm
                let mut s = T::zero();
                for x in v {
                    s = s + x.abs().powf(p);
                }
                s.powf(p.recip())
            }
        }
    }
}

impl<T: Float, M: BaseMatrix<T>> MatrixNorm<T, M> for Lp<T> {
    fn norm(&self, m: &M) -> T {
        match *self {
            Lp::Infinity => {
                // Compute supremum
                let mut abs_sup = T::zero();
                for d in m.iter().map(|d| d.abs()) {
                    if d > abs_sup {
                        abs_sup = d;
                    }
                }
                abs_sup
            },
            Lp::Integer(p) => {
                assert!(p >= 1, "p value in Lp norm must be >= 1");
                // Compute standard lp norm
                let mut s = T::zero();
                for x in m.iter() {
                    s = s + x.abs().powi(p);
                }
                s.powf(T::from(p).expect("Could not cast i32 to float").recip())
            },
            Lp::Float(p) => {
                assert!(p >= T::one(), "p value in Lp norm must be >= 1");
                // Compute standard lp norm
                let mut s = T::zero();
                for x in m.iter() {
                    s = s + x.abs().powf(p);
                }
                s.powf(p.recip())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use libnum::Float;
    use std::f64;

    use super::*;
    use vector::Vector;
    use matrix::{Matrix, MatrixSlice};

    #[test]
    fn test_euclidean_vector_norm() {
        let v = vector![3.0, 4.0];
        assert_scalar_eq!(VectorNorm::norm(&Euclidean, &v), 5.0, comp = float);
    }

    #[test]
    fn test_euclidean_matrix_norm() {
        let m = matrix![3.0, 4.0;
                        1.0, 3.0];
        assert_scalar_eq!(MatrixNorm::norm(&Euclidean, &m), 35.0.sqrt(), comp = float);
    }

    #[test]
    fn test_euclidean_matrix_slice_norm() {
        let m = matrix![3.0, 4.0;
                        1.0, 3.0];

        let slice = MatrixSlice::from_matrix(&m, [0,0], 1, 2);
        assert_scalar_eq!(MatrixNorm::norm(&Euclidean, &slice), 5.0, comp = float);
    }

    #[test]
    fn test_euclidean_vector_metric() {
        let v = vector![3.0, 4.0];
        assert_scalar_eq!(VectorMetric::metric(&Euclidean, &v, &v), 0.0, comp = float);

        let v1 = vector![0.0, 0.0];
        assert_scalar_eq!(VectorMetric::metric(&Euclidean, &v, &v1), 5.0, comp = float);

        let v2 = vector![4.0, 3.0];
        assert_scalar_eq!(VectorMetric::metric(&Euclidean, &v, &v2), 2.0.sqrt(), comp = float);
    }

    #[test]
    #[should_panic]
    fn test_euclidean_vector_metric_bad_dim() {
        let v = vector![3.0, 4.0];
        let v2 = vector![1.0, 2.0, 3.0];

        VectorMetric::metric(&Euclidean, &v, &v2);
    }

    #[test]
    fn test_euclidean_matrix_metric() {
        let m = matrix![3.0, 4.0;
                        1.0, 3.0];
        assert_scalar_eq!(MatrixMetric::metric(&Euclidean, &m, &m), 0.0, comp = float);

        let m1 = Matrix::zeros(2, 2);
        assert_scalar_eq!(MatrixMetric::metric(&Euclidean, &m, &m1), 35.0.sqrt(), comp = float);

        let m2 = matrix![2.0, 3.0;
                         2.0, 4.0];
        assert_scalar_eq!(MatrixMetric::metric(&Euclidean, &m, &m2), 2.0, comp = float);
    }

    #[test]
    #[should_panic]
    fn test_euclidean_matrix_metric_bad_dim() {
        let m = matrix![3.0, 4.0];
        let m2 = matrix![1.0, 2.0, 3.0];

        MatrixMetric::metric(&Euclidean, &m, &m2);
    }

    #[test]
    fn test_euclidean_matrix_slice_metric() {
        let m = matrix![
            1.0, 1.0, 1.0;
            1.0, 1.0, 1.0;
            1.0, 1.0, 1.0
        ];

        let m2 = matrix![
            0.0, 0.0, 0.0;
            0.0, 0.0, 0.0;
            0.0, 0.0, 0.0
        ];

        let m_slice = MatrixSlice::from_matrix(
            &m, [0; 2], 1, 2
        );

        let m2_slice = MatrixSlice::from_matrix(
            &m2, [0; 2], 1, 2
        );

        assert_scalar_eq!(MatrixMetric::metric(&Euclidean, &m_slice, &m2_slice), 2.0.sqrt(), comp = exact);
    }

    #[test]
    #[should_panic]
    fn test_euclidean_matrix_slice_metric_bad_dim() {
        let m = matrix![3.0, 4.0];
        let m2 = matrix![1.0, 2.0, 3.0];

        let m_slice = MatrixSlice::from_matrix(
            &m, [0; 2], 1, 1
        );

        let m2_slice = MatrixSlice::from_matrix(
            &m2, [0; 2], 1, 2
        );

        MatrixMetric::metric(&Euclidean, &m_slice, &m2_slice);
    }

    #[test]
    fn test_lp_vector_supremum() {
        let v = vector![-5.0, 3.0];

        let sup = VectorNorm::norm(&Lp::Infinity, &v);
        assert_eq!(sup, 5.0);
    }

    #[test]
    fn test_lp_matrix_supremum() {
        let m = matrix![0.0, -2.0;
                        3.5, 1.0];

        let sup = MatrixNorm::norm(&Lp::Infinity, &m);
        assert_eq!(sup, 3.5);
    }

    #[test]
    fn test_lp_vector_one() {
        let v = vector![1.0, 2.0, -2.0];
        assert_eq!(VectorNorm::norm(&Lp::Integer(1), &v), 5.0);
    }

    #[test]
    fn test_lp_matrix_one() {
        let m = matrix![1.0, -2.0;
                        0.5, 1.0];
        assert_eq!(MatrixNorm::norm(&Lp::Integer(1), &m), 4.5);
    }

    #[test]
    fn test_lp_vector_float() {
        let v = vector![1.0, 2.0, -2.0];
        assert_eq!(VectorNorm::norm(&Lp::Float(1.0), &v), 5.0);
    }

    #[test]
    fn test_lp_matrix_float() {
        let m = matrix![1.0, -2.0;
                        0.5, 1.0];
        assert_eq!(MatrixNorm::norm(&Lp::Float(1.0), &m), 4.5);
    }

    #[test]
    #[should_panic]
    fn test_lp_vector_bad_p() {
        let v = Vector::new(vec![]);
        VectorNorm::norm(&Lp::Float(0.5), &v);
    }

    #[test]
    #[should_panic]
    fn test_lp_matrix_bad_p() {
        let m = matrix![];
        MatrixNorm::norm(&Lp::Float(0.5), &m);
    }

    #[test]
    #[should_panic]
    fn test_lp_vector_bad_int_p() {
        let v: Vector<f64> = Vector::new(vec![]);
        VectorNorm::norm(&Lp::Integer(0), &v);
    }

    #[test]
    #[should_panic]
    fn test_lp_matrix_bad_int_p() {
        let m: Matrix<f64> = matrix![];
        MatrixNorm::norm(&Lp::Integer(0), &m);
    }
}
