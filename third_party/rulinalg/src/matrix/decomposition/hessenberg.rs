use matrix::{Matrix, BaseMatrix, BaseMatrixMut, MatrixSlice, MatrixSliceMut};
use error::{Error, ErrorKind};

use std::any::Any;
use std::vec::*;

use libnum::{Float};

impl<T: Any + Float> Matrix<T> {
    /// Returns H, where H is the upper hessenberg form.
    ///
    /// If the transformation matrix is also required, you should
    /// use `upper_hess_decomp`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::Matrix;
    ///
    /// let a = matrix![2., 0., 1., 1.;
    ///                 2., 0., 1., 2.;
    ///                 1., 2., 0., 0.;
    ///                 2., 0., 1., 1.];
    /// let h = a.upper_hessenberg();
    ///
    /// println!("{:}", h.expect("Could not get upper Hessenberg form."));
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrix is not square.
    ///
    /// # Failures
    ///
    /// - The matrix cannot be reduced to upper hessenberg form.
    pub fn upper_hessenberg(mut self) -> Result<Matrix<T>, Error> {
        let n = self.rows;
        assert!(n == self.cols,
                "Matrix must be square to produce upper hessenberg.");

        for i in 0..n - 2 {
            let h_holder_vec: Matrix<T>;
            {
                let lower_slice = MatrixSlice::from_matrix(&self, [i + 1, i], n - i - 1, 1);
                // Try to get the house holder transform - else map error and pass up.
                h_holder_vec = try!(Matrix::make_householder_vec(&lower_slice.iter()
                        .cloned()
                        .collect::<Vec<_>>())
                    .map_err(|_| {
                        Error::new(ErrorKind::DecompFailure,
                                   "Cannot compute upper Hessenberg form.")
                    }));
            }

            {
                // Apply holder on the left
                let mut block =
                    MatrixSliceMut::from_matrix(&mut self, [i + 1, i], n - i - 1, n - i);
                block -= &h_holder_vec * (h_holder_vec.transpose() * &block) *
                         (T::one() + T::one());
            }

            {
                // Apply holder on the right
                let mut block = MatrixSliceMut::from_matrix(&mut self, [0, i + 1], n, n - i - 1);
                block -= (&block * &h_holder_vec) * h_holder_vec.transpose() *
                         (T::one() + T::one());
            }

        }

        // Enforce upper hessenberg
        for i in 0..self.cols - 2 {
            for j in i + 2..self.rows {
                unsafe {
                    *self.get_unchecked_mut([j, i]) = T::zero();
                }
            }
        }

        Ok(self)
    }

    /// Returns (U,H), where H is the upper hessenberg form
    /// and U is the unitary transform matrix.
    ///
    /// Note: The current transform matrix seems broken...
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::BaseMatrix;
    ///
    /// let a = matrix![1., 2., 3.;
    ///                 4., 5., 6.;
    ///                 7., 8., 9.];
    ///
    /// // u is the transform, h is the upper hessenberg form.
    /// let (u, h) = a.clone().upper_hess_decomp().expect("This matrix should decompose!");
    ///
    /// assert_matrix_eq!(h, u.transpose() * a * u, comp = abs, tol = 1e-12);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrix is not square.
    ///
    /// # Failures
    ///
    /// - The matrix cannot be reduced to upper hessenberg form.
    pub fn upper_hess_decomp(self) -> Result<(Matrix<T>, Matrix<T>), Error> {
        let n = self.rows;
        assert!(n == self.cols,
                "Matrix must be square to produce upper hessenberg.");

        // First we form the transformation.
        let mut transform = Matrix::identity(n);

        for i in (0..n - 2).rev() {
            let h_holder_vec: Matrix<T>;
            {
                let lower_slice = MatrixSlice::from_matrix(&self, [i + 1, i], n - i - 1, 1);
                h_holder_vec = try!(Matrix::make_householder_vec(&lower_slice.iter()
                        .cloned()
                        .collect::<Vec<_>>())
                    .map_err(|_| {
                        Error::new(ErrorKind::DecompFailure, "Could not compute eigenvalues.")
                    }));
            }

            let mut trans_block =
                MatrixSliceMut::from_matrix(&mut transform, [i + 1, i + 1], n - i - 1, n - i - 1);
            trans_block -= &h_holder_vec * (h_holder_vec.transpose() * &trans_block) *
                           (T::one() + T::one());
        }

        // Now we reduce to upper hessenberg
        Ok((transform, try!(self.upper_hessenberg())))
    }
}

#[cfg(test)]
mod tests {
    use matrix::Matrix;

    #[test]
    #[should_panic]
    fn test_non_square_upper_hessenberg() {
        let a: Matrix<f64> = Matrix::ones(2, 3);

        let _ = a.upper_hessenberg();
    }

    #[test]
    #[should_panic]
    fn test_non_square_upper_hess_decomp() {
        let a: Matrix<f64> = Matrix::ones(2, 3);

        let _ = a.upper_hess_decomp();
    }
}
