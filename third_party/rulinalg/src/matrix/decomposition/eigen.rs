use matrix::{Matrix, MatrixSliceMut, BaseMatrix, BaseMatrixMut};
use norm::Euclidean;
use error::{Error, ErrorKind};

use std::cmp;
use std::any::Any;
use std::vec::*;

use libnum::{Float, Signed};
use libnum::{cast, abs};

impl<T: Any + Float + Signed> Matrix<T> {
    fn balance_matrix(&mut self) {
        let n = self.rows();
        let radix = T::one() + T::one();

        debug_assert!(n == self.cols(),
                      "Matrix must be square to produce balance matrix.");

        let mut d = Matrix::<T>::identity(n);
        let mut converged = false;

        while !converged {
            converged = true;

            for i in 0..n {
                let mut c = self.select_cols(&[i]).norm(Euclidean);
                let mut r = self.select_rows(&[i]).norm(Euclidean);

                let s = c * c + r * r;
                let mut f = T::one();

                while c < r / radix {
                    c = c * radix;
                    r = r / radix;
                    f = f * radix;
                }

                while c >= r * radix {
                    c = c / radix;
                    r = r * radix;
                    f = f / radix;
                }

                if (c * c + r * r) < cast::<f64, T>(0.95).unwrap() * s {
                    converged = false;
                    d.data[i * (self.cols + 1)] = f * d.data[i * (self.cols + 1)];

                    for j in 0..n {
                        self.data[j * self.cols + i] = f * self.data[j * self.cols + i];
                        self.data[i * self.cols + j] = self.data[i * self.cols + j] / f;
                    }
                }
            }
        }
    }

    fn direct_2_by_2_eigenvalues(&self) -> Result<Vec<T>, Error> {
        // The characteristic polynomial of a 2x2 matrix A is
        // λ² − (a₁₁ + a₂₂)λ + (a₁₁a₂₂ − a₁₂a₂₁);
        // the quadratic formula suffices.
        let tr = self.data[0] + self.data[3];
        let det = self.data[0] * self.data[3] - self.data[1] * self.data[2];

        let two = T::one() + T::one();
        let four = two + two;

        let discr = tr * tr - four * det;

        if discr < T::zero() {
            Err(Error::new(ErrorKind::DecompFailure,
                           "Matrix has complex eigenvalues. Currently unsupported, sorry!"))
        } else {
            let discr_root = discr.sqrt();
            Ok(vec![(tr - discr_root) / two, (tr + discr_root) / two])
        }

    }

    fn francis_shift_eigenvalues(self) -> Result<Vec<T>, Error> {
        let n = self.rows();
        debug_assert!(n > 2,
                      "Francis shift only works on matrices greater than 2x2.");
        debug_assert!(n == self.cols, "Matrix must be square for Francis shift.");

        let mut h = try!(self
            .upper_hessenberg()
            .map_err(|_| Error::new(ErrorKind::DecompFailure, "Could not compute eigenvalues.")));
        h.balance_matrix();

        // The final index of the active matrix
        let mut p = n - 1;

        let eps = cast::<f64, T>(1e-20).expect("Failed to cast value for convergence check.");

        while p > 1 {
            let q = p - 1;
            let s = h[[q, q]] + h[[p, p]];
            let t = h[[q, q]] * h[[p, p]] - h[[q, p]] * h[[p, q]];

            let mut x = h[[0, 0]] * h[[0, 0]] + h[[0, 1]] * h[[1, 0]] - h[[0, 0]] * s + t;
            let mut y = h[[1, 0]] * (h[[0, 0]] + h[[1, 1]] - s);
            let mut z = h[[1, 0]] * h[[2, 1]];

            for k in 0..p - 1 {
                let r = cmp::max(1, k) - 1;

                let householder = try!(Matrix::make_householder(&[x, y, z]).map_err(|_| {
                    Error::new(ErrorKind::DecompFailure, "Could not compute eigenvalues.")
                }));

                {
                    // Apply householder transformation to block (on the left)
                    let h_block = MatrixSliceMut::from_matrix(&mut h, [k, r], 3, n - r);
                    let transformed = &householder * &h_block;
                    h_block.set_to(transformed.as_slice());
                }

                let r = cmp::min(k + 4, p + 1);

                {
                    // Apply householder transformation to the block (on the right)
                    let h_block = MatrixSliceMut::from_matrix(&mut h, [0, k], r, 3);
                    let transformed = &h_block * householder.transpose();
                    h_block.set_to(transformed.as_slice());
                }

                x = h[[k + 1, k]];
                y = h[[k + 2, k]];

                if k < p - 2 {
                    z = h[[k + 3, k]];
                }
            }

            let (c, s) = Matrix::givens_rot(x, y);
            let givens_mat = Matrix::new(2, 2, vec![c, -s, s, c]);

            {
                // Apply Givens rotation to the block (on the left)
                let h_block = MatrixSliceMut::from_matrix(&mut h, [q, p - 2], 2, n - p + 2);
                let transformed = &givens_mat * &h_block;
                h_block.set_to(transformed.as_slice());
            }

            {
                // Apply Givens rotation to block (on the right)
                let h_block = MatrixSliceMut::from_matrix(&mut h, [0, q], p + 1, 2);
                let transformed = &h_block * givens_mat.transpose();
                h_block.set_to(transformed.as_slice());
            }

            // Check for convergence
            if abs(h[[p, q]]) < eps * (abs(h[[q, q]]) + abs(h[[p, p]])) {
                h.data[p * h.cols + q] = T::zero();
                p -= 1;
            } else if abs(h[[p - 1, q - 1]]) < eps * (abs(h[[q - 1, q - 1]]) + abs(h[[q, q]])) {
                h.data[(p - 1) * h.cols + q - 1] = T::zero();
                p -= 2;
            }
        }

        Ok(h.diag().cloned().collect::<Vec<_>>())
    }

    /// Eigenvalues of a square matrix.
    ///
    /// Returns a Vec of eigenvalues.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::Matrix;
    ///
    /// let a = Matrix::new(4,4, (1..17).map(|v| v as f64).collect::<Vec<f64>>());
    /// let e = a.eigenvalues().expect("We should be able to compute these eigenvalues!");
    /// println!("{:?}", e);
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrix is not square.
    ///
    /// # Failures
    ///
    /// - Eigenvalues cannot be computed.
    pub fn eigenvalues(self) -> Result<Vec<T>, Error> {
        let n = self.rows();
        assert!(n == self.cols,
                "Matrix must be square for eigenvalue computation.");

        match n {
            1 => Ok(vec![self.data[0]]),
            2 => self.direct_2_by_2_eigenvalues(),
            _ => self.francis_shift_eigenvalues(),
        }
    }

    fn direct_2_by_2_eigendecomp(&self) -> Result<(Vec<T>, Matrix<T>), Error> {
        let eigenvalues = try!(self.direct_2_by_2_eigenvalues());
        // Thanks to
        // http://www.math.harvard.edu/archive/21b_fall_04/exhibits/2dmatrices/index.html
        // for this characterization—
        if self.data[2] != T::zero() {
            let decomp_data = vec![eigenvalues[0] - self.data[3],
                                   eigenvalues[1] - self.data[3],
                                   self.data[2],
                                   self.data[2]];
            Ok((eigenvalues, Matrix::new(2, 2, decomp_data)))
        } else if self.data[1] != T::zero() {
            let decomp_data = vec![self.data[1],
                                   self.data[1],
                                   eigenvalues[0] - self.data[0],
                                   eigenvalues[1] - self.data[0]];
            Ok((eigenvalues, Matrix::new(2, 2, decomp_data)))
        } else {
            Ok((eigenvalues, Matrix::new(2, 2, vec![T::one(), T::zero(), T::zero(), T::one()])))
        }
    }

    fn francis_shift_eigendecomp(self) -> Result<(Vec<T>, Matrix<T>), Error> {
        let n = self.rows();
        debug_assert!(n > 2,
                      "Francis shift only works on matrices greater than 2x2.");
        debug_assert!(n == self.cols, "Matrix must be square for Francis shift.");

        let (u, mut h) = try!(self.upper_hess_decomp().map_err(|_| {
            Error::new(ErrorKind::DecompFailure,
                       "Could not compute eigen decomposition.")
        }));
        h.balance_matrix();
        let mut transformation = Matrix::identity(n);

        // The final index of the active matrix
        let mut p = n - 1;

        let eps = cast::<f64, T>(1e-20).expect("Failed to cast value for convergence check.");

        while p > 1 {
            let q = p - 1;
            let s = h[[q, q]] + h[[p, p]];
            let t = h[[q, q]] * h[[p, p]] - h[[q, p]] * h[[p, q]];

            let mut x = h[[0, 0]] * h[[0, 0]] + h[[0, 1]] * h[[1, 0]] - h[[0, 0]] * s + t;
            let mut y = h[[1, 0]] * (h[[0, 0]] + h[[1, 1]] - s);
            let mut z = h[[1, 0]] * h[[2, 1]];

            for k in 0..p - 1 {
                let r = cmp::max(1, k) - 1;

                let householder = try!(Matrix::make_householder(&[x, y, z]).map_err(|_| {
                    Error::new(ErrorKind::DecompFailure,
                               "Could not compute eigen decomposition.")
                }));

                {
                    // Apply householder transformation to block (on the left)
                    let h_block = MatrixSliceMut::from_matrix(&mut h, [k, r], 3, n - r);
                    let transformed = &householder * &h_block;
                    h_block.set_to(transformed.as_slice());
                }

                let r = cmp::min(k + 4, p + 1);

                {
                    // Apply householder transformation to the block (on the right)
                    let h_block = MatrixSliceMut::from_matrix(&mut h, [0, k], r, 3);
                    let transformed = &h_block * householder.transpose();
                    h_block.set_to(transformed.as_slice());
                }

                {
                    // Update the transformation matrix
                    let trans_block =
                        MatrixSliceMut::from_matrix(&mut transformation, [0, k], n, 3);
                    let transformed = &trans_block * householder.transpose();
                    trans_block.set_to(transformed.as_slice());
                }

                x = h[[k + 1, k]];
                y = h[[k + 2, k]];

                if k < p - 2 {
                    z = h[[k + 3, k]];
                }
            }

            let (c, s) = Matrix::givens_rot(x, y);
            let givens_mat = Matrix::new(2, 2, vec![c, -s, s, c]);

            {
                // Apply Givens rotation to the block (on the left)
                let h_block = MatrixSliceMut::from_matrix(&mut h, [q, p - 2], 2, n - p + 2);
                let transformed = &givens_mat * &h_block;
                h_block.set_to(transformed.as_slice());
            }

            {
                // Apply Givens rotation to block (on the right)
                let h_block = MatrixSliceMut::from_matrix(&mut h, [0, q], p + 1, 2);
                let transformed = &h_block * givens_mat.transpose();
                h_block.set_to(transformed.as_slice());
            }

            {
                // Update the transformation matrix
                let trans_block = MatrixSliceMut::from_matrix(&mut transformation, [0, q], n, 2);
                let transformed = &trans_block * givens_mat.transpose();
                trans_block.set_to(transformed.as_slice());
            }

            // Check for convergence
            if abs(h[[p, q]]) < eps * (abs(h[[q, q]]) + abs(h[[p, p]])) {
                h.data[p * h.cols + q] = T::zero();
                p -= 1;
            } else if abs(h[[p - 1, q - 1]]) < eps * (abs(h[[q - 1, q - 1]]) + abs(h[[q, q]])) {
                h.data[(p - 1) * h.cols + q - 1] = T::zero();
                p -= 2;
            }
        }

        Ok((h.diag().cloned().collect::<Vec<_>>(), u * transformation))
    }

    /// Eigendecomposition of a square matrix.
    ///
    /// Returns a Vec of eigenvalues, and a matrix with eigenvectors as the columns.
    ///
    /// The eigenvectors are only gauranteed to be correct if the matrix is real-symmetric.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::Matrix;
    ///
    /// let a = matrix![3., 2., 4.;
    ///                 2., 0., 2.;
    ///                 4., 2., 3.];
    ///
    /// let (e, m) = a.eigendecomp().expect("We should be able to compute this eigendecomp!");
    /// println!("{:?}", e);
    /// println!("{:?}", m.data());
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrix is not square.
    ///
    /// # Failures
    ///
    /// - The eigen decomposition can not be computed.
    pub fn eigendecomp(self) -> Result<(Vec<T>, Matrix<T>), Error> {
        let n = self.rows();
        assert!(n == self.cols, "Matrix must be square for eigendecomp.");

        match n {
            1 => Ok((vec![self.data[0]], Matrix::ones(1, 1))),
            2 => self.direct_2_by_2_eigendecomp(),
            _ => self.francis_shift_eigendecomp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use matrix::Matrix;

    #[test]
    fn test_1_by_1_matrix_eigenvalues() {
        let a = Matrix::ones(1, 1) * 3.;
        assert_eq!(vec![3.], a.eigenvalues().unwrap());
    }

    #[test]
    fn test_2_by_2_matrix_eigenvalues() {
        let a = matrix![1., 2.; 3., 4.];
        // characteristic polynomial is λ² − 5λ − 2 = 0
        assert_eq!(vec![(5. - (33.0f32).sqrt()) / 2., (5. + (33.0f32).sqrt()) / 2.],
                   a.eigenvalues().unwrap());
    }

    #[test]
    fn test_2_by_2_matrix_zeros_eigenvalues() {
        let a = Matrix::zeros(2, 2);
        // characteristic polynomial is λ² = 0
        assert_eq!(vec![0.0, 0.0], a.eigenvalues().unwrap());
    }

    #[test]
    fn test_2_by_2_matrix_complex_eigenvalues() {
        // This test currently fails - complex eigenvalues would be nice though!
        let a = matrix![1., -3.; 1., 1.];
        // characteristic polynomial is λ² − λ + 4 = 0

        // Decomposition will fail
        assert!(a.eigenvalues().is_err());
    }

    #[test]
    fn test_2_by_2_matrix_eigendecomp() {
        let a = matrix![20., 4.; 20., 16.];
        let (eigenvals, eigenvecs) = a.clone().eigendecomp().unwrap();

        let lambda_1 = eigenvals[0];
        let lambda_2 = eigenvals[1];

        let v1 = vector![eigenvecs[[0, 0]], eigenvecs[[1, 0]]];
        let v2 = vector![eigenvecs[[0, 1]], eigenvecs[[1, 1]]];

        assert_vector_eq!(&a * &v1, &v1 * lambda_1, comp = float);
        assert_vector_eq!(&a * &v2, &v2 * lambda_2, comp = float);
    }

    #[test]
    fn test_3_by_3_eigenvals() {
        let a = matrix![17f64, 22., 27.;
                        22., 29., 36.;
                        27., 36., 45.];

        let eigs = a.eigenvalues().unwrap();

        let eig_1 = 90.4026;
        let eig_2 = 0.5973;
        let eig_3 = 0.0;

        assert!(eigs.iter().any(|x| (x - eig_1).abs() < 1e-4));
        assert!(eigs.iter().any(|x| (x - eig_2).abs() < 1e-4));
        assert!(eigs.iter().any(|x| (x - eig_3).abs() < 1e-4));
    }

    #[test]
    fn test_5_by_5_eigenvals() {
        let a = matrix![1f64, 2.0, 3.0, 4.0, 5.0;
                        2.0, 4.0, 1.0, 2.0, 1.0;
                        3.0, 1.0, 7.0, 1.0, 1.0;
                        4.0, 2.0, 1.0, -1.0, 3.0;
                        5.0, 1.0, 1.0, 3.0, 2.0];

        let eigs = a.eigenvalues().unwrap();

        let eig_1 = 12.174;
        let eig_2 = 5.2681;
        let eig_3 = -4.4942;
        let eig_4 = 2.9279;
        let eig_5 = -2.8758;

        assert!(eigs.iter().any(|x| (x - eig_1).abs() < 1e-4));
        assert!(eigs.iter().any(|x| (x - eig_2).abs() < 1e-4));
        assert!(eigs.iter().any(|x| (x - eig_3).abs() < 1e-4));
        assert!(eigs.iter().any(|x| (x - eig_4).abs() < 1e-4));
        assert!(eigs.iter().any(|x| (x - eig_5).abs() < 1e-4));
    }

    #[test]
    #[should_panic]
    fn test_non_square_eigenvalues() {
        let a: Matrix<f64> = Matrix::ones(2, 3);

        let _ = a.eigenvalues();
    }

    #[test]
    #[should_panic]
    fn test_non_square_eigendecomp() {
        let a: Matrix<f64> = Matrix::ones(2, 3);

        let _ = a.eigendecomp();
    }
}
