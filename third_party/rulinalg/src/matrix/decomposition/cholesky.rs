use matrix::{Matrix, BaseMatrix};
use error::{Error, ErrorKind};
use matrix::decomposition::Decomposition;
use matrix::forward_substitution;
use vector::Vector;
use utils::dot;
use std::vec::*;

use std::any::Any;

use libnum::{Zero, Float};

/// Cholesky decomposition.
///
/// Given a square, symmetric positive definite matrix A,
/// there exists an invertible lower triangular matrix L
/// such that
///
/// A = L L<sup>T</sup>.
///
/// This is called the Cholesky decomposition of A.
/// For not too ill-conditioned A, the computation
/// of the decomposition is very robust, and it takes about
/// half the effort of an LU decomposition with partial pivoting.
///
/// # Applications
/// The Cholesky decomposition can be thought of as a specialized
/// LU decomposition for symmetric positive definite matrices,
/// and so its applications are similar to that of LU.
///
/// The following example shows how to compute the Cholesky
/// decomposition of a given matrix. In this example, we also
/// unpack the decomposition to retrieve the L matrix,
/// but in many practical applications we are not so concerned
/// with the factor itself. Instead, we may wish to
/// solve linear systems or compute the determinant or the
/// inverse of a symmetric positive definite matrix.
/// In this case, see the next subsections.
///
/// ```
/// # #[macro_use] extern crate rulinalg; fn main() {
/// use rulinalg::matrix::decomposition::Cholesky;
///
/// // Need to import Decomposition if we want to unpack
/// use rulinalg::matrix::decomposition::Decomposition;
///
/// let x = matrix![ 1.0,  3.0,  1.0;
///                  3.0, 13.0, 11.0;
///                  1.0, 11.0, 21.0 ];
/// let cholesky = Cholesky::decompose(x)
///                         .expect("Matrix is SPD.");
///
/// // Obtain the matrix factor L
/// let l = cholesky.unpack();
///
/// assert_matrix_eq!(l, matrix![1.0,  0.0,  0.0;
///                              3.0,  2.0,  0.0;
///                              1.0,  4.0,  2.0], comp = float);
/// # }
/// ```
///
/// ## Solving linear systems
/// After having decomposed the matrix, one may efficiently
/// solve linear systems for different right-hand sides.
///
/// ```
/// # #[macro_use] extern crate rulinalg; fn main() {
/// # use rulinalg::matrix::decomposition::Cholesky;
/// # let x = matrix![ 1.0,  3.0,  1.0;
/// #                  3.0, 13.0, 11.0;
/// #                  1.0, 11.0, 21.0 ];
/// # let cholesky = Cholesky::decompose(x).unwrap();
/// let b1 = vector![ 3.0,  2.0,  1.0];
/// let b2 = vector![-2.0,  1.0,  0.0];
/// let y1 = cholesky.solve(b1).expect("Matrix is invertible.");
/// let y2 = cholesky.solve(b2).expect("Matrix is invertible.");
/// assert_vector_eq!(y1, vector![ 23.25, -7.75,  3.0 ]);
/// assert_vector_eq!(y2, vector![-22.25,  7.75, -3.00 ]);
/// # }
/// ```
///
/// ## Computing the inverse of a matrix
///
/// While computing the inverse explicitly is rarely
/// the best solution to any given problem, it is sometimes
/// necessary. In this case, it is easily accessible
/// through the `inverse()` method on `Cholesky`.
///
/// # Computing the determinant of a matrix
///
/// As with LU decomposition, the `Cholesky` decomposition
/// exposes a method `det` for computing the determinant
/// of the decomposed matrix. This is a very cheap operation.
#[derive(Clone, Debug)]
pub struct Cholesky<T> {
    l: Matrix<T>
}

impl<T> Cholesky<T> where T: 'static + Float {
    /// Computes the Cholesky decomposition A = L L<sup>T</sup>
    /// for the given square, symmetric positive definite matrix.
    ///
    /// Note that the implementation cannot reliably and efficiently
    /// verify that the matrix truly is symmetric positive definite matrix,
    /// so it is the responsibility of the user to make sure that this is
    /// the case. In particular, if the input matrix is not SPD,
    /// the returned decomposition may not be a valid decomposition
    /// for the input matrix.
    ///
    /// # Errors
    /// - A diagonal entry is effectively zero to working precision.
    /// - A diagonal entry is negative.
    ///
    /// # Panics
    ///
    /// - The matrix must be square.
    pub fn decompose(matrix: Matrix<T>) -> Result<Self, Error> {
        assert!(matrix.rows() == matrix.cols(),
            "Matrix must be square for Cholesky decomposition.");
        let n = matrix.rows();

        // The implementation here is based on the
        // "Gaxpy-Rich Cholesky Factorization"
        // from Chapter 4.2.5 in
        // Matrix Computations, 4th Edition,
        // (Golub and Van Loan).

        // We consume the matrix we're given, and overwrite its
        // lower diagonal part with the L factor. However,
        // we ignore the strictly upper triangular part of the matrix,
        // because this saves us a few operations.
        // When the decomposition is unpacked, we will completely zero
        // the upper triangular part.
        let mut a = matrix;

        for j in 0 .. n {
            if j > 0 {
                // This is essentially a GAXPY operation y = y - Bx
                // where B is the [j .. n, 0 .. j] submatrix of A,
                // x is the [ j, 0 .. j ] submatrix of A,
                // and y is the [ j .. n, j ] submatrix of A
                for k in j .. n {
                    let kj_dot = {
                        let j_row = a.row(j).raw_slice();
                        let k_row = a.row(k).raw_slice();
                        dot(&k_row[0 .. j], &j_row[0 .. j])
                    };
                    a[[k, j]] = a[[k, j]] - kj_dot;
                }
            }

            let diagonal = a[[j, j]];
            if diagonal.abs() < T::epsilon() {
                return Err(Error::new(ErrorKind::DecompFailure,
                    "Matrix is singular to working precision."));
            } else if diagonal < T::zero() {
                return Err(Error::new(ErrorKind::DecompFailure,
                    "Diagonal entries of matrix are not all positive."));
            }

            let divisor = diagonal.sqrt();
            for k in j .. n {
                a[[k, j]] = a[[k, j]] / divisor;
            }
        }

        Ok(Cholesky {
            l: a
        })
    }

    /// Computes the determinant of the decomposed matrix.
    ///
    /// Note that the determinant of an empty matrix is considered
    /// to be equal to 1.
    pub fn det(&self) -> T {
        let l_det = self.l.diag()
                          .cloned()
                          .fold(T::one(), |a, b| a * b);
        l_det * l_det
    }

    /// Solves the linear system Ax = b.
    ///
    /// Here A is the decomposed matrix and b is the
    /// supplied vector.
    ///
    /// # Errors
    /// If the matrix is sufficiently ill-conditioned,
    /// it is possible that the solution cannot be obtained.
    ///
    /// # Panics
    /// - The supplied right-hand side vector must be
    ///   dimensionally compatible with the supplied matrix.
    pub fn solve(&self, b: Vector<T>) -> Result<Vector<T>, Error> {
        assert!(self.l.rows() == b.size(),
            "RHS vector and coefficient matrix must be
             dimensionally compatible.");
        // Solve Ly = b
        let y = forward_substitution(&self.l, b)?;
        // Solve L^T x = y
        transpose_back_substitution(&self.l, y)
    }

    /// Computes the inverse of the decomposed matrix.
    ///
    /// # Errors
    /// If the matrix is sufficiently ill-conditioned,
    /// it is possible that the inverse cannot be obtained.
    pub fn inverse(&self) -> Result<Matrix<T>, Error> {
        let n = self.l.rows();
        let mut inv = Matrix::zeros(n, n);
        let mut e = Vector::zeros(n);

        // Note: this is essentially the same as
        // PartialPivLu::inverse(), and consequently
        // the data access patterns here can also be
        // improved by way of using BLAS-3 calls.
        // Please see that function's implementation
        // for more details.

        // Solve for each column of the inverse matrix
        for i in 0 .. n {
            e[i] = T::one();
            let col = self.solve(e)?;

            for j in 0 .. n {
                inv[[j, i]] = col[j];
            }

            e = col.apply(&|_| T::zero());
        }

        Ok(inv)
    }
}

impl<T: Zero> Decomposition for Cholesky<T> {
    type Factors = Matrix<T>;

    fn unpack(self) -> Matrix<T> {
        use internal_utils::nullify_upper_triangular_part;
        let mut l = self.l;
        nullify_upper_triangular_part(&mut l);
        l
    }
}


impl<T> Matrix<T>
    where T: Any + Float
{
    /// Cholesky decomposition
    ///
    /// Returns the cholesky decomposition of a positive definite matrix.
    ///
    /// *NOTE*: This function is deprecated, and will be removed in a
    /// future release. Please see
    /// [Cholesky](decomposition/struct.Cholesky.html) for its
    /// replacement.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::Matrix;
    ///
    /// let m = matrix![1.0, 0.5, 0.5;
    ///                 0.5, 1.0, 0.5;
    ///                 0.5, 0.5, 1.0];
    ///
    /// let l = m.cholesky();
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrix is not square.
    ///
    /// # Failures
    ///
    /// - Matrix is not positive definite.
    //#[deprecated]
    pub fn cholesky(&self) -> Result<Matrix<T>, Error> {
        assert!(self.rows == self.cols,
                "Matrix must be square for Cholesky decomposition.");

        let mut new_data = Vec::<T>::with_capacity(self.rows() * self.cols());

        for i in 0..self.rows() {

            for j in 0..self.cols() {

                if j > i {
                    new_data.push(T::zero());
                    continue;
                }

                let mut sum = T::zero();
                for k in 0..j {
                    sum = sum + (new_data[i * self.cols() + k] * new_data[j * self.cols() + k]);
                }

                if j == i {
                    new_data.push((self[[i, i]] - sum).sqrt());
                } else {
                    let p = (self[[i, j]] - sum) / new_data[j * self.cols + j];

                    if !p.is_finite() {
                        return Err(Error::new(ErrorKind::DecompFailure,
                                              "Matrix is not positive definite."));
                    } else {

                    }
                    new_data.push(p);
                }
            }
        }

        Ok(Matrix {
            rows: self.rows(),
            cols: self.cols(),
            data: new_data,
        })
    }
}

/// Solves the square system L^T x = b,
/// where L is lower triangular
fn transpose_back_substitution<T>(l: &Matrix<T>, b: Vector<T>)
    -> Result<Vector<T>, Error> where T: Float {
    assert!(l.rows() == l.cols(), "Matrix L must be square.");
    assert!(l.rows() == b.size(), "L and b must be dimensionally compatible.");
    let n = l.rows();
    let mut x = b;

    for i in (0 .. n).rev() {
        let row = l.row(i).raw_slice();
        let diagonal = l[[i, i]];
        if diagonal.abs() < T::epsilon() {
            return Err(Error::new(ErrorKind::DivByZero,
                "Matrix L is singular to working precision."));
        }

        x[i] = x[i] / diagonal;

        // Apply the BLAS-1 operation
        // y <- y + α x
        // where α = - x[i],
        // y = x[0 .. i]
        // and x = l[i, 0 .. i]
        // TODO: Hopefully we'll have a more systematic way
        // of applying optimized BLAS-like operations in the future.
        // In this case, we should replace this loop with a call
        // to the appropriate function.
        for j in 0 .. i {
            x[j] = x[j] - x[i] * row[j];
        }
    }

    Ok(x)
}

#[cfg(test)]
mod tests {
    use matrix::Matrix;
    use matrix::decomposition::Decomposition;
    use vector::Vector;

    use super::Cholesky;
    use super::transpose_back_substitution;

    use quickcheck::TestResult;

    #[test]
    #[should_panic]
    #[allow(deprecated)]
    fn test_non_square_cholesky() {
        let a = Matrix::<f64>::ones(2, 3);

        let _ = a.cholesky();
    }

    #[test]
    fn cholesky_unpack_empty() {
        let x: Matrix<f64> = matrix![];
        let l = Cholesky::decompose(x.clone())
                            .unwrap()
                            .unpack();
        assert_matrix_eq!(l, x);
    }

    #[test]
    fn cholesky_unpack_1x1() {
        let x = matrix![ 4.0 ];
        let expected = matrix![ 2.0 ];
        let l = Cholesky::decompose(x)
                            .unwrap()
                            .unpack();
        assert_matrix_eq!(l, expected, comp = float);
    }

    #[test]
    fn cholesky_unpack_2x2() {
        {
            let x = matrix![ 9.0, -6.0;
                            -6.0, 20.0];
            let expected = matrix![ 3.0, 0.0;
                                   -2.0, 4.0];

            let l = Cholesky::decompose(x)
                        .unwrap()
                        .unpack();
            assert_matrix_eq!(l, expected, comp = float);
        }
    }

    #[test]
    fn cholesky_singular_fails() {
        {
            let x = matrix![0.0];
            assert!(Cholesky::decompose(x).is_err());
        }

        {
            let x = matrix![0.0, 0.0;
                            0.0, 1.0];
            assert!(Cholesky::decompose(x).is_err());
        }

        {
            let x = matrix![1.0, 0.0;
                            0.0, 0.0];
            assert!(Cholesky::decompose(x).is_err());
        }

        {
            let x = matrix![1.0,   3.0,   5.0;
                            3.0,   9.0,  15.0;
                            5.0,  15.0,  65.0];
            assert!(Cholesky::decompose(x).is_err());
        }
    }

    #[test]
    fn cholesky_det_empty() {
        let x: Matrix<f64> = matrix![];
        let cholesky = Cholesky::decompose(x).unwrap();
        assert_eq!(cholesky.det(), 1.0);
    }

    #[test]
    fn cholesky_det() {
        {
            let x = matrix![1.0];
            let cholesky = Cholesky::decompose(x).unwrap();
            assert_scalar_eq!(cholesky.det(), 1.0, comp = float);
        }

        {
            let x = matrix![1.0,   3.0,   5.0;
                            3.0,  18.0,  33.0;
                            5.0,  33.0,  65.0];
            let cholesky = Cholesky::decompose(x).unwrap();
            assert_scalar_eq!(cholesky.det(), 36.0, comp = float);
        }
    }

    #[test]
    fn cholesky_solve_examples() {
        {
            let a: Matrix<f64> = matrix![];
            let b: Vector<f64> = vector![];
            let expected: Vector<f64> = vector![];
            let cholesky = Cholesky::decompose(a).unwrap();
            let x = cholesky.solve(b).unwrap();
            assert_eq!(x, expected);
        }

        {
            let a = matrix![ 1.0 ];
            let b = vector![ 4.0 ];
            let expected = vector![ 4.0 ];
            let cholesky = Cholesky::decompose(a).unwrap();
            let x = cholesky.solve(b).unwrap();
            assert_vector_eq!(x, expected, comp = float);
        }

        {
            let a = matrix![ 4.0,  6.0;
                             6.0, 25.0];
            let b = vector![ 2.0,  4.0];
            let expected = vector![ 0.40625,  0.0625 ];
            let cholesky = Cholesky::decompose(a).unwrap();
            let x = cholesky.solve(b).unwrap();
            assert_vector_eq!(x, expected, comp = float);
        }
    }

    #[test]
    fn cholesky_inverse_examples() {
        {
            let a: Matrix<f64> = matrix![];
            let expected: Matrix<f64> = matrix![];
            let cholesky = Cholesky::decompose(a).unwrap();
            assert_eq!(cholesky.inverse().unwrap(), expected);
        }

        {
            let a = matrix![ 2.0 ];
            let expected = matrix![ 0.5 ];
            let cholesky = Cholesky::decompose(a).unwrap();
            assert_matrix_eq!(cholesky.inverse().unwrap(), expected,
                              comp = float);
        }

        {
            let a = matrix![ 4.0,  6.0;
                             6.0, 25.0];
            let expected = matrix![  0.390625, -0.09375;
                                    -0.093750 , 0.06250];
            let cholesky = Cholesky::decompose(a).unwrap();
            assert_matrix_eq!(cholesky.inverse().unwrap(), expected,
                              comp = float);
        }

        {
            let a = matrix![ 9.0,   6.0,   3.0;
                             6.0,  20.0,  10.0;
                             3.0,  10.0,  14.0];
            let expected = matrix![0.1388888888888889, -0.0416666666666667,  0.0               ;
                                  -0.0416666666666667,  0.0902777777777778, -0.0555555555555556;
                                                  0.0, -0.0555555555555556,  0.1111111111111111];
            let cholesky = Cholesky::decompose(a).unwrap();
            assert_matrix_eq!(cholesky.inverse().unwrap(), expected,
                              comp = float);
        }
    }

    quickcheck! {
        fn property_cholesky_of_identity_is_identity(n: usize) -> TestResult {
            if n > 30 {
                return TestResult::discard();
            }

            let x = Matrix::<f64>::identity(n);
            let l = Cholesky::decompose(x.clone()).map(|c| c.unpack());
            match l {
                Ok(l) => {
                    assert_matrix_eq!(l, x, comp = float);
                    TestResult::passed()
                },
                _ => TestResult::failed()
            }
        }
    }

    #[test]
    fn transpose_back_substitution_examples() {
        {
            let l: Matrix<f64> = matrix![];
            let b: Vector<f64> = vector![];
            let expected: Vector<f64> = vector![];
            let x = transpose_back_substitution(&l, b).unwrap();
            assert_vector_eq!(x, expected);
        }

        {
            let l = matrix![2.0];
            let b = vector![2.0];
            let expected = vector![1.0];
            let x = transpose_back_substitution(&l, b).unwrap();
            assert_vector_eq!(x, expected, comp = float);
        }

        {
            let l = matrix![2.0, 0.0;
                            3.0, 4.0];
            let b = vector![2.0, 1.0];
            let expected = vector![0.625, 0.25 ];
            let x = transpose_back_substitution(&l, b).unwrap();
            assert_vector_eq!(x, expected, comp = float);
        }

        {
            let l = matrix![ 2.0,  0.0,  0.0;
                             5.0, -1.0,  0.0;
                            -2.0,  0.0,  1.0];
            let b = vector![-1.0, 2.0, 3.0];
            let expected = vector![ 7.5, -2.0, 3.0 ];
            let x = transpose_back_substitution(&l, b).unwrap();
            assert_vector_eq!(x, expected, comp = float);
        }
    }
}
