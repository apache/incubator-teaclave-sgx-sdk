use matrix::{Matrix, BaseMatrix, BaseMatrixMut, MatrixSlice, MatrixSliceMut};
use error::{Error, ErrorKind};

use std::any::Any;
use std::cmp;
use std::vec::*;

use libnum::{Float, Signed};

/// Ensures that all singular values in the given singular value decomposition
/// are non-negative, making necessary corrections to the singular vectors.
///
/// The SVD is represented by matrices `(b, u, v)`, where `b` is the diagonal matrix
/// containing the singular values, `u` is the matrix of left singular vectors
/// and v is the matrix of right singular vectors.
fn correct_svd_signs<T>(mut b: Matrix<T>,
                        mut u: Matrix<T>,
                        mut v: Matrix<T>)
                        -> (Matrix<T>, Matrix<T>, Matrix<T>)
    where T: Any + Float + Signed
{

    // When correcting the signs of the singular vectors, we can choose
    // to correct EITHER u or v. We make the choice depending on which matrix has the
    // least number of rows. Later we will need to multiply all elements in columns by
    // -1, which might be significantly faster in corner cases if we pick the matrix
    // with the least amount of rows.
    {
        let ref mut shortest_matrix = if u.rows() <= v.rows() { &mut u } else { &mut v };
        let column_length = shortest_matrix.rows();
        let num_singular_values = cmp::min(b.rows(), b.cols());

        for i in 0..num_singular_values {
            if b[[i, i]] < T::zero() {
                // Swap sign of singular value and column in u
                b[[i, i]] = b[[i, i]].abs();

                // Access the column as a slice and flip sign
                let mut column = shortest_matrix.sub_slice_mut([0, i], column_length, 1);
                column *= -T::one();
            }
        }
    }
    (b, u, v)
}

fn sort_svd<T>(mut b: Matrix<T>,
               mut u: Matrix<T>,
               mut v: Matrix<T>)
               -> (Matrix<T>, Matrix<T>, Matrix<T>)
    where T: Any + Float + Signed
{

    assert!(u.cols() == b.cols() && b.cols() == v.cols());

    // This unfortunately incurs two allocations since we have no (simple)
    // way to iterate over a matrix diagonal, only to copy it into a new Vector
    let mut indexed_sorted_values: Vec<_> = b.diag().cloned().enumerate().collect();

    // Sorting a vector of indices simultaneously with the singular values
    // gives us a mapping between old and new (final) column indices.
    indexed_sorted_values.sort_by(|&(_, ref x), &(_, ref y)| {
        x.partial_cmp(y)
            .expect("All singular values should be finite, and thus sortable.")
            .reverse()
    });

    // Set the diagonal elements of the singular value matrix
    for (i, &(_, value)) in indexed_sorted_values.iter().enumerate() {
        b[[i, i]] = value;
    }

    // Assuming N columns, the simultaneous sorting of indices and singular values yields
    // a set of N (i, j) pairs which correspond to columns which must be swapped. However,
    // for any (i, j) in this set, there is also (j, i). Keeping both of these would make us
    // swap the columns back and forth, so we must remove the duplicates. We can avoid
    // any further sorting or hashsets or similar by noting that we can simply
    // remove any (i, j) for which j >= i. This also removes (i, i) pairs,
    // i.e. columns that don't need to be swapped.
    let swappable_pairs = indexed_sorted_values.into_iter()
        .enumerate()
        .map(|(new_index, (old_index, _))| (old_index, new_index))
        .filter(|&(old_index, new_index)| old_index < new_index);

    for (old_index, new_index) in swappable_pairs {
        u.swap_cols(old_index, new_index);
        v.swap_cols(old_index, new_index);
    }

    (b, u, v)
}

impl<T: Any + Float + Signed> Matrix<T> {
    /// Singular Value Decomposition
    ///
    /// Computes the SVD using the Golub-Reinsch algorithm.
    ///
    /// Returns Σ, U, V, such that `self` = U Σ V<sup>T</sup>. Σ is a diagonal matrix whose elements
    /// correspond to the non-negative singular values of the matrix. The singular values are ordered in
    /// non-increasing order. U and V have orthonormal columns, and each column represents the
    /// left and right singular vectors for the corresponding singular value in Σ, respectively.
    ///
    /// If `self` has M rows and N columns, the dimensions of the returned matrices
    /// are as follows.
    ///
    /// If M >= N:
    ///
    /// - `Σ`: N x N
    /// - `U`: M x N
    /// - `V`: N x N
    ///
    /// If M < N:
    ///
    /// - `Σ`: M x M
    /// - `U`: M x M
    /// - `V`: N x M
    ///
    /// Note: This version of the SVD is sometimes referred to as the 'economy SVD'.
    ///
    /// # Failures
    ///
    /// This function may fail in some cases. The current decomposition whilst being
    /// efficient is fairly basic. Hopefully the algorithm can be made not to fail in the near future.
    pub fn svd(self) -> Result<(Matrix<T>, Matrix<T>, Matrix<T>), Error> {
        let (b, u, v) = try!(self.svd_unordered());
        Ok(sort_svd(b, u, v))
    }

    fn svd_unordered(self) -> Result<(Matrix<T>, Matrix<T>, Matrix<T>), Error> {
        let (b, u, v) = try!(self.svd_golub_reinsch());

        // The Golub-Reinsch implementation sometimes spits out negative singular values,
        // so we need to correct these.
        Ok(correct_svd_signs(b, u, v))
    }

    fn svd_golub_reinsch(mut self) -> Result<(Matrix<T>, Matrix<T>, Matrix<T>), Error> {
        let mut flipped = false;

        // The algorithm assumes rows > cols. If this is not the case we transpose and fix later.
        if self.cols > self.rows {
            self = self.transpose();
            flipped = true;
        }

        let eps = T::from(3.0).unwrap() * T::epsilon();
        let n = self.cols;

        // Get the bidiagonal decomposition
        let (mut b, mut u, mut v) = try!(self.bidiagonal_decomp()
            .map_err(|_| Error::new(ErrorKind::DecompFailure, "Could not compute SVD.")));

        loop {
            // Values to count the size of lower diagonal block
            let mut q = 0;
            let mut on_lower = true;

            // Values to count top block
            let mut p = 0;
            let mut on_middle = false;

            // Iterate through and hard set the super diag if converged
            for i in (0..n - 1).rev() {
                let (b_ii, b_sup_diag, diag_abs_sum): (T, T, T);
                unsafe {
                    b_ii = *b.get_unchecked([i, i]);
                    b_sup_diag = b.get_unchecked([i, i + 1]).abs();
                    diag_abs_sum = eps * (b_ii.abs() + b.get_unchecked([i + 1, i + 1]).abs());
                }
                if b_sup_diag <= diag_abs_sum {
                    // Adjust q or p to define boundaries of sup-diagonal box
                    if on_lower {
                        q += 1;
                    } else if on_middle {
                        on_middle = false;
                        p = i + 1;
                    }
                    unsafe {
                        *b.get_unchecked_mut([i, i + 1]) = T::zero();
                    }
                } else {
                    if on_lower {
                        // No longer on the lower diagonal
                        on_middle = true;
                        on_lower = false;
                    }
                }
            }

            // We have converged!
            if q == n - 1 {
                break;
            }

            // Zero off diagonals if needed.
            for i in p..n - q - 1 {
                let (b_ii, b_sup_diag): (T, T);
                unsafe {
                    b_ii = *b.get_unchecked([i, i]);
                    b_sup_diag = *b.get_unchecked([i, i + 1]);
                }

                if b_ii.abs() < eps {
                    let (c, s) = Matrix::<T>::givens_rot(b_ii, b_sup_diag);
                    let givens = Matrix::new(2, 2, vec![c, s, -s, c]);
                    let b_i = MatrixSliceMut::from_matrix(&mut b, [i, i], 1, 2);
                    let zerod_line = &b_i * givens;

                    b_i.set_to(zerod_line.as_slice());
                }
            }

            // Apply Golub-Kahan svd step
            unsafe {
                try!(Matrix::<T>::golub_kahan_svd_step(&mut b, &mut u, &mut v, p, q)
                    .map_err(|_| Error::new(ErrorKind::DecompFailure, "Could not compute SVD.")));
            }
        }

        if flipped {
            Ok((b.transpose(), v, u))
        } else {
            Ok((b, u, v))
        }

    }

    /// This function is unsafe as it makes assumptions about the dimensions
    /// of the inputs matrices and does not check them. As a result if misused
    /// this function can call `get_unchecked` on invalid indices.
    unsafe fn golub_kahan_svd_step(b: &mut Matrix<T>,
                                   u: &mut Matrix<T>,
                                   v: &mut Matrix<T>,
                                   p: usize,
                                   q: usize)
                                   -> Result<(), Error> {
        let n = b.rows();

        // C is the lower, right 2x2 square of aTa, where a is the
        // middle block of b (between p and n-q).
        //
        // Computed as xTx + yTy, where y is the bottom 2x2 block of a
        // and x are the two columns above it within a.
        let c: Matrix<T>;
        {
            let y = MatrixSlice::from_matrix(&b, [n - q - 2, n - q - 2], 2, 2).into_matrix();
            if n - q - p - 2 > 0 {
                let x = MatrixSlice::from_matrix(&b, [p, n - q - 2], n - q - p - 2, 2);
                c = x.into_matrix().transpose() * x + y.transpose() * y;
            } else {
                c = y.transpose() * y;
            }
        }

        let c_eigs = try!(c.clone().eigenvalues());

        // Choose eigenvalue closes to c[1,1].
        let lambda: T;
        if (c_eigs[0] - *c.get_unchecked([1, 1])).abs() <
           (c_eigs[1] - *c.get_unchecked([1, 1])).abs() {
            lambda = c_eigs[0];
        } else {
            lambda = c_eigs[1];
        }

        let b_pp = *b.get_unchecked([p, p]);
        let mut alpha = (b_pp * b_pp) - lambda;
        let mut beta = b_pp * *b.get_unchecked([p, p + 1]);
        for k in p..n - q - 1 {
            // Givens rot on columns k and k + 1
            let (c, s) = Matrix::<T>::givens_rot(alpha, beta);
            let givens_mat = Matrix::new(2, 2, vec![c, s, -s, c]);

            {
                // Pick the rows from b to be zerod.
                let b_block = MatrixSliceMut::from_matrix(b,
                                                          [k.saturating_sub(1), k],
                                                          cmp::min(3, n - k.saturating_sub(1)),
                                                          2);
                let transformed = &b_block * &givens_mat;
                b_block.set_to(transformed.as_slice());

                let v_block = MatrixSliceMut::from_matrix(v, [0, k], n, 2);
                let transformed = &v_block * &givens_mat;
                v_block.set_to(transformed.as_slice());
            }

            alpha = *b.get_unchecked([k, k]);
            beta = *b.get_unchecked([k + 1, k]);

            let (c, s) = Matrix::<T>::givens_rot(alpha, beta);
            let givens_mat = Matrix::new(2, 2, vec![c, -s, s, c]);

            {
                // Pick the columns from b to be zerod.
                let b_block = MatrixSliceMut::from_matrix(b, [k, k], 2, cmp::min(3, n - k));
                let transformed = &givens_mat * &b_block;
                b_block.set_to(transformed.as_slice());

                let m = u.rows();
                let u_block = MatrixSliceMut::from_matrix(u, [0, k], m, 2);
                let transformed = &u_block * givens_mat.transpose();
                u_block.set_to(transformed.as_slice());
            }

            if k + 2 < n - q {
                alpha = *b.get_unchecked([k, k + 1]);
                beta = *b.get_unchecked([k, k + 2]);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use matrix::{Matrix, BaseMatrix};
    use vector::Vector;
    use super::sort_svd;

    fn validate_svd(mat: &Matrix<f64>, b: &Matrix<f64>, u: &Matrix<f64>, v: &Matrix<f64>) {
        // b is diagonal (the singular values)
        for (idx, row) in b.row_iter().enumerate() {
            assert!(!row.iter().take(idx).any(|&x| x > 1e-10));
            assert!(!row.iter().skip(idx + 1).any(|&x| x > 1e-10));
            // Assert non-negativity of diagonal elements
            assert!(row[idx] >= 0.0);
        }

        let recovered = u * b * v.transpose();

        assert_eq!(recovered.rows(), mat.rows());
        assert_eq!(recovered.cols(), mat.cols());

        assert!(!mat.data()
            .iter()
            .zip(recovered.data().iter())
            .any(|(&x, &y)| (x - y).abs() > 1e-10));

        // The transposition is due to the fact that there does not exist
        // any column iterators at the moment, and we need to simultaneously iterate
        // over the columns. Once they do exist, we should rewrite
        // the below iterators to use iter_cols() or whatever instead.
        let ref u_transposed = u.transpose();
        let ref v_transposed = v.transpose();
        let ref mat_transposed = mat.transpose();

        let mut singular_triplets = u_transposed.row_iter().zip(b.diag()).zip(v_transposed.row_iter())
            // chained zipping results in nested tuple. Flatten it.
            .map(|((u_col, singular_value), v_col)| (Vector::new(u_col.raw_slice()), singular_value, Vector::new(v_col.raw_slice())));

        assert!(singular_triplets.by_ref()
            // For a matrix M, each singular value σ and left and right singular vectors u and v respectively
            // satisfy M v = σ u, so we take the difference
            .map(|(ref u, sigma, ref v)| mat * v - u * sigma)
            .flat_map(|v| v.into_vec().into_iter())
            .all(|x| x.abs() < 1e-10));

        assert!(singular_triplets.by_ref()
            // For a matrix M, each singular value σ and left and right singular vectors u and v respectively
            // satisfy M_transposed u = σ v, so we take the difference
            .map(|(ref u, sigma, ref v)| mat_transposed * u - v * sigma)
            .flat_map(|v| v.into_vec().into_iter())
            .all(|x| x.abs() < 1e-10));
    }

    #[test]
    fn test_sort_svd() {
        let u = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        let b = matrix![4.0, 0.0, 0.0;
                        0.0, 8.0, 0.0;
                        0.0, 0.0, 2.0];
        let v = matrix![21.0, 22.0, 23.0;
                        24.0, 25.0, 26.0;
                        27.0, 28.0, 29.0];

        let (b, u, v) = sort_svd(b, u, v);

        assert_eq!(b.data(), &vec![8.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 2.0]);
        assert_eq!(u.data(), &vec![2.0, 1.0, 3.0, 5.0, 4.0, 6.0]);
        assert_eq!(v.data(),
                   &vec![22.0, 21.0, 23.0, 25.0, 24.0, 26.0, 28.0, 27.0, 29.0]);

    }

    #[test]
    fn test_svd_tall_matrix() {
        // Note: This matrix is not arbitrary. It has been constructed specifically so that
        // the "natural" order of the singular values it not sorted by default.
        let mat = matrix![3.61833700244349288, -3.28382346228211697,  1.97968027781346501, -0.41869628192662156;
                          3.96046289599926427,  0.70730060716580723, -2.80552479438772817, -1.45283286109873933;
                          1.44435028724617442,  1.27749196276785826, -1.09858397535426366, -0.03159619816434689;
                          1.13455445826500667,  0.81521390274755756,  3.99123446373437263, -2.83025703359666192;
                          -3.30895752093770579, -0.04979044289857298,  3.03248594516832792,  3.85962479743330977];
        let (b, u, v) = mat.clone().svd().unwrap();

        let expected_values = vec![8.0, 6.0, 4.0, 2.0];

        validate_svd(&mat, &b, &u, &v);

        // Assert the singular values are what we expect
        assert!(expected_values.iter()
            .zip(b.diag())
            .all(|(expected, actual)| (expected - actual).abs() < 1e-14));
    }

    #[test]
    fn test_svd_short_matrix() {
        // Note: This matrix is not arbitrary. It has been constructed specifically so that
        // the "natural" order of the singular values it not sorted by default.
        let mat = matrix![3.61833700244349288,  3.96046289599926427,  1.44435028724617442,  1.13455445826500645, -3.30895752093770579;
                         -3.28382346228211697,  0.70730060716580723,  1.27749196276785826,  0.81521390274755756, -0.04979044289857298;
                          1.97968027781346545, -2.80552479438772817, -1.09858397535426366,  3.99123446373437263,  3.03248594516832792;
                         -0.41869628192662156, -1.45283286109873933, -0.03159619816434689, -2.83025703359666192,  3.85962479743330977];
        let (b, u, v) = mat.clone().svd().unwrap();

        let expected_values = vec![8.0, 6.0, 4.0, 2.0];

        validate_svd(&mat, &b, &u, &v);

        // Assert the singular values are what we expect
        assert!(expected_values.iter()
            .zip(b.diag())
            .all(|(expected, actual)| (expected - actual).abs() < 1e-14));
    }

    #[test]
    fn test_svd_square_matrix() {
        let mat = matrix![1.0,  2.0,  3.0,  4.0,  5.0;
                          2.0,  4.0,  1.0,  2.0,  1.0;
                          3.0,  1.0,  7.0,  1.0,  1.0;
                          4.0,  2.0,  1.0, -1.0,  3.0;
                          5.0,  1.0,  1.0,  3.0,  2.0];

        let expected_values = vec![12.1739747429271112,
                                   5.2681047320525831,
                                   4.4942269799769843,
                                   2.9279675877385123,
                                   2.8758200827412224];

        let (b, u, v) = mat.clone().svd().unwrap();
        validate_svd(&mat, &b, &u, &v);

        // Assert the singular values are what we expect
        assert!(expected_values.iter()
            .zip(b.diag())
            .all(|(expected, actual)| (expected - actual).abs() < 1e-12));
    }
}
