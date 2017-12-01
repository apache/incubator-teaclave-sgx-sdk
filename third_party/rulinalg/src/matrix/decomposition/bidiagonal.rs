use matrix::{Matrix, BaseMatrix, BaseMatrixMut, MatrixSlice, MatrixSliceMut};
use error::{Error, ErrorKind};
use std::vec::*;

use std;
use std::any::Any;

use libnum::Float;

impl<T> Matrix<T>
    where T: Any + Float
{
    /// Converts matrix to bidiagonal form
    ///
    /// Returns (B, U, V), where B is bidiagonal and `self = U B V_T`.
    ///
    /// Note that if `self` has `self.rows() > self.cols()` the matrix will
    /// be transposed and then reduced - this will lead to a sub-diagonal instead
    /// of super-diagonal.
    ///
    /// # Failures
    ///
    /// - The matrix cannot be reduced to bidiagonal form.
    pub fn bidiagonal_decomp(mut self) -> Result<(Matrix<T>, Matrix<T>, Matrix<T>), Error> {
        let mut flipped = false;

        if self.rows < self.cols {
            flipped = true;
            self = self.transpose()
        }

        let m = self.rows;
        let n = self.cols;

        let mut u = Matrix::identity(m);
        let mut v = Matrix::identity(n);

        for k in 0..n {
            let h_holder: Matrix<T>;
            {
                let lower_slice = MatrixSlice::from_matrix(&self, [k, k], m - k, 1);
                h_holder = try!(Matrix::make_householder(&lower_slice.iter()
                        .cloned()
                        .collect::<Vec<_>>())
                    .map_err(|_| {
                        Error::new(ErrorKind::DecompFailure, "Cannot compute bidiagonal form.")
                    }));
            }

            {
                // Apply householder on the left to kill under diag.
                let lower_self_block = MatrixSliceMut::from_matrix(&mut self, [k, k], m - k, n - k);
                let transformed_self = &h_holder * &lower_self_block;
                lower_self_block.set_to(transformed_self.as_slice());
                let lower_u_block = MatrixSliceMut::from_matrix(&mut u, [0, k], m, m - k);
                let transformed_u = &lower_u_block * h_holder;
                lower_u_block.set_to(transformed_u.as_slice());
            }

            if k < n - 2 {
                let row: &[T];
                unsafe {
                    // Get the kth row from column k+1 to end.
                    row = std::slice::from_raw_parts(self.data
                                                         .as_ptr()
                                                         .offset((k * self.cols + k + 1) as isize),
                                                     n - k - 1);
                }

                let row_h_holder = try!(Matrix::make_householder(row).map_err(|_| {
                    Error::new(ErrorKind::DecompFailure, "Cannot compute bidiagonal form.")
                }));

                {
                    // Apply householder on the right to kill right of super diag.
                    let lower_self_block =
                        MatrixSliceMut::from_matrix(&mut self, [k, k + 1], m - k, n - k - 1);

                    let transformed_self = &lower_self_block * &row_h_holder;
                    lower_self_block.set_to(transformed_self.as_slice());
                    let lower_v_block =
                        MatrixSliceMut::from_matrix(&mut v, [0, k + 1], n, n - k - 1);
                    let transformed_v = &lower_v_block * row_h_holder;
                    lower_v_block.set_to(transformed_v.as_slice());

                }
            }
        }

        // Trim off the zerod blocks.
        self.data.truncate(n * n);
        self.rows = n;
        u = MatrixSlice::from_matrix(&u, [0, 0], m, n).into_matrix();

        if flipped {
            Ok((self.transpose(), v, u))
        } else {
            Ok((self, u, v))
        }

    }
}

#[cfg(test)]
mod tests {
    use matrix::{BaseMatrix, Matrix};

    fn validate_bidiag(mat: &Matrix<f64>,
                       b: &Matrix<f64>,
                       u: &Matrix<f64>,
                       v: &Matrix<f64>,
                       upper: bool) {
        for (idx, row) in b.row_iter().enumerate() {
            let pair_start = if upper { idx } else { idx.saturating_sub(1) };
            assert!(!row.iter().take(pair_start).any(|&x| x > 1e-10));
            assert!(!row.iter().skip(pair_start + 2).any(|&x| x > 1e-10));
        }

        let recovered = u * b * v.transpose();

        assert_eq!(recovered.rows(), mat.rows());
        assert_eq!(recovered.cols(), mat.cols());

        assert!(!mat.data()
            .iter()
            .zip(recovered.data().iter())
            .any(|(&x, &y)| (x - y).abs() > 1e-10));
    }

    #[test]
    fn test_bidiagonal_square() {
        let mat = matrix![1f64, 2.0, 3.0, 4.0, 5.0;
                          2.0, 4.0, 1.0, 2.0, 1.0;
                          3.0, 1.0, 7.0, 1.0, 1.0;
                          4.0, 2.0, 1.0, -1.0, 3.0;
                          5.0, 1.0, 1.0, 3.0, 2.0];
        let (b, u, v) = mat.clone().bidiagonal_decomp().unwrap();
        validate_bidiag(&mat, &b, &u, &v, true);
    }

    #[test]
    fn test_bidiagonal_non_square() {
        let mat = matrix![1f64, 2.0, 3.0;
                          4.0, 5.0, 2.0;
                          4.0, 1.0, 2.0;
                          1.0, 3.0, 1.0;
                          7.0, 1.0, 1.0];
        let (b, u, v) = mat.clone().bidiagonal_decomp().unwrap();
        validate_bidiag(&mat, &b, &u, &v, true);

        let mat = matrix![1f64, 2.0, 3.0, 4.0, 5.0;
                          2.0, 4.0, 1.0, 2.0, 1.0;
                          3.0, 1.0, 7.0, 1.0, 1.0];
        let (b, u, v) = mat.clone().bidiagonal_decomp().unwrap();
        validate_bidiag(&mat, &b, &u, &v, false);
    }
}
