use matrix::{Matrix, MatrixSlice, BaseMatrix, BaseMatrixMut};
use vector::Vector;
use error::{Error, ErrorKind};
use matrix::decomposition::{
    Decomposition,
    HouseholderReflection,
    HouseholderComposition
};
use matrix::decomposition::householder;

use std::any::Any;
use std::vec::*;

use libnum::Float;

/// The result of unpacking a QR decomposition.
///
/// Let `A` denote the `m x n` matrix given by `A = QR`.
/// Then `Q` is an `m x m` orthogonal matrix, and `R`
/// is an `m x n` upper trapezoidal matrix .
///
/// More precisely, if `m > n`, then we have the decomposition
///
/// ```text
/// A = QR = Q [ R1 ]
///            [  0 ]
/// ```
/// where `R1` is an `n x n` upper triangular matrix.
/// On the other hand, if `m < n`, we have
///
/// ```text
/// A = QR = Q [ R1 R2 ]
/// ```
///
/// where `R1` is an `m x m` upper triangular matrix and
/// `R2` is an `m x (n - m)` general matrix.
///
/// To actually compute the QR decomposition, see
/// [Householder QR](struct.HouseholderQr.html).
#[derive(Debug, Clone)]
pub struct QR<T> {
    /// The orthogonal matrix `Q` in the decomposition `A = QR`.
    pub q: Matrix<T>,
    /// The upper-trapezoidal matrix `R` in the decomposition `A = QR`.
    pub r: Matrix<T>
}

/// The result of computing a *thin* (or *reduced*) QR decomposition.
///
/// Let `A` denote the `m x n` matrix given by `A = QR`.
/// Then `Q` is an `m x m` orthogonal matrix, and `R`
/// is an `m x n` upper trapezoidal matrix.
///
/// If `m > n`, we may write
///
/// ```text
/// A = QR = [ Q1 Q2 ] [ R1 ] = Q1 R1
///                    [  0 ]
/// ```
///
/// where `Q1` is an `m x n` matrix with orthogonal columns,
/// and `R1` is an `n x n` upper triangular matrix.
/// For some applications, the remaining (m - n) columns
/// of the full `Q` matrix are not needed, in which case
/// the thin QR decomposition is substantially cheaper if
/// `m >> n`.
///
/// If `m <= n`, then the thin QR decomposition coincides with
/// the usual decomposition. See [QR](struct.QR.html) for details.
///
/// To actually compute the QR decomposition, see
/// [Householder QR](struct.HouseholderQr.html).
#[derive(Debug, Clone)]
pub struct ThinQR<T> {
    /// The matrix `Q1` in the decomposition `A = Q1 R1`.
    pub q1: Matrix<T>,
    /// The upper-triangular matrix `R1` in the decomposition `A = Q1 R1`.
    pub r1: Matrix<T>
}

/// QR decomposition based on Householder reflections.
///
/// For any `m x n` matrix `A`, there exist an `m x m`
/// orthogonal matrix `Q` and an `m x n` upper trapezoidal
/// (triangular) matrix `R` such that
///
/// ```text
/// A = QR.
/// ```
///
/// `HouseholderQr` holds the result of a QR decomposition
/// procedure based on Householder reflections. The full
/// factors `Q` and `R` can be acquired by calling `unpack()`.
/// However, it turns out that the orthogonal factor `Q`
/// can be represented much more efficiently than as a
/// full `m x m` matrix. For this purpose, the [q()](#method.q)
/// method provides access to an instance of
/// [HouseholderComposition](struct.HouseholderComposition.html)
/// which allows the efficient application of the (implicit)
/// `Q` matrix.
///
/// For some applications, it is sufficient to compute a
/// *thin* (or *reduced*) QR decomposition. The thin QR decomposition
/// can be obtained by calling [unpack_thin()](#method.unpack_thin)
/// on the decomposition object.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rulinalg; fn main() {
/// use rulinalg::matrix::Matrix;
/// use rulinalg::matrix::decomposition::{
///     Decomposition, HouseholderQr, QR
/// };
///
/// let a = matrix![ 3.0,  2.0;
///                 -5.0,  1.0;
///                  4.0, -2.0 ];
/// let identity = Matrix::identity(3);
///
/// let qr = HouseholderQr::decompose(a.clone());
/// let QR { q, r } = qr.unpack();
///
/// // Check that `Q` is orthogonal
/// assert_matrix_eq!(&q * q.transpose(), identity, comp = float);
/// assert_matrix_eq!(q.transpose() * &q, identity, comp = float);
///
/// // Check that `A = QR`
/// assert_matrix_eq!(q * r, a, comp = float);
/// # }
/// ```
///
/// # Internal storage format
/// Upon decomposition, the `HouseholderQr` struct takes ownership
/// of the matrix and repurposes its storage to compactly
/// store the factors `Q` and `R`.
/// In addition, a vector `tau` of size `min(m, n)`
/// holds auxiliary information about the Householder reflectors
/// which together constitute the `Q` matrix.
///
/// Specifically, given an input matrix `A`,
/// the upper triangular factor `R` is stored in `A_ij` for
/// `j >= i`. The orthogonal factor `Q` is implicitly stored
/// as the composition of `p := min(m, n)` Householder reflectors
/// `Q_i`, such that
///
/// ```text
/// Q = Q_1 * Q_2 * ... * Q_p.
/// ```
///
/// Each such Householder reflection `Q_i` corresponds to a
/// transformation of the form (using MATLAB-like colon notation)
///
/// ```text
/// Q_i [1:(i-1), 1:(i-1)] = I
/// Q_i [i:m, i:m] = I - τ_i * v_i * transpose(v_i)
/// ```
///
/// where `I` denotes the identity matrix of appropriate size,
/// `v_i` is the *Householder vector* normalized in such a way that
/// its first element is implicitly `1` (and thus does not need to
/// be stored) and `τ_i` is an appropriate scale factor. Each vector
/// `v_i` has length `m - i + 1`, and since the first element does not
/// need to be stored, each `v_i` can be stored in column `i` of
/// the matrix `A`.
///
/// The scale factors `τ_i` are stored in a separate vector.
///
/// This storage scheme should be compatible with LAPACK, although
/// this has yet to be put to the test. For the same reason,
/// the internal storage is not exposed in the public API at this point.
#[derive(Debug, Clone)]
pub struct HouseholderQr<T> {
    qr: Matrix<T>,
    tau: Vec<T>
}

impl<T> HouseholderQr<T> where T: Float {
    /// Decomposes the given matrix into implicitly stored factors
    /// `Q` and `R` as described in the struct documentation.
    pub fn decompose(matrix: Matrix<T>) -> HouseholderQr<T> {
        use std::cmp::min;

        // The implementation here is based on
        // Algorithm 5.2.1 (Householder QR) from
        // Matrix Computations, 4th Ed,
        // by Golub & Van Loan
        let m = matrix.rows();
        let n = matrix.cols();
        let p = min(m, n);

        let mut qr = matrix;
        let mut tau = vec![T::zero(); p];

        // In order to avoid frequently allocating new vectors
        // to hold the householder reflections, we allocate a single
        // buffer which we can reuse for every iteration. We also
        // need one as work space when applying the Householder
        // transformations.
        let mut buffer = vec![T::zero(); m];
        let mut multiply_buffer = vec![T::zero(); n];

        for j in 0 .. p {
            let mut bottom_right = qr.sub_slice_mut([j, j], m - j, n - j);

            // The householder vector which we will hold in the buffer
            // gets shorter for each iteration, so we truncate the buffer
            // to the appropriate length.
            buffer.truncate(m - j);
            multiply_buffer.truncate(bottom_right.cols());
            bottom_right.col(0).clone_into_slice(&mut buffer);

            let house = HouseholderReflection::compute(Vector::new(buffer));
            house.buffered_left_multiply_into(&mut bottom_right,
                                              &mut multiply_buffer);
            house.store_in_col(&mut bottom_right.col_mut(0));
            tau[j] = house.tau();
            buffer = house.into_vector().into_vec();
        }
        HouseholderQr {
            qr: qr,
            tau: tau
        }
    }

    /// Returns the orthogonal factor `Q` as an instance of a
    /// [HouseholderComposition](struct.HouseholderComposition.html)
    /// operator.
    pub fn q(&self) -> HouseholderComposition<T> {
        householder::create_composition(&self.qr, &self.tau)
    }

    /// Computes the *thin* (or reduced) QR decomposition.
    ///
    /// If `m <= n`, the thin QR decomposition coincides with the
    /// usual QR decomposition. See [ThinQR](struct.ThinQR.html)
    /// for details.
    ///
    /// # Examples
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// # use rulinalg::matrix::Matrix;
    /// # let x: Matrix<f64> = matrix![];
    /// use rulinalg::matrix::decomposition::{HouseholderQr, ThinQR};
    /// let x = matrix![3.0, 2.0;
    ///                 1.0, 2.0;
    ///                 4.0, 5.0];
    /// let ThinQR { q1, r1 } = HouseholderQr::decompose(x).unpack_thin();
    /// # }
    /// ```
    pub fn unpack_thin(self) -> ThinQR<T> {
        // Note: currently, there is no need to take ownership of
        // `self`. However, it is actually possible to compute the
        // rectangular Q1 factor in-place, but it is not currently
        // done. By taking `self` now, we can make this change in
        // the future without breaking changes.
        let m = self.qr.rows();
        let n = self.qr.cols();

        if m <= n {
            // If m <= n, Thin QR coincides with regular QR
            let qr = self.unpack();
            ThinQR {
                q1: qr.q,
                r1: qr.r
            }
        } else {
            let composition = householder::create_composition(&self.qr, &self.tau);
            let q1 = composition.first_k_columns(n);
            let r1 = extract_r1(&self.qr);
            ThinQR {
                q1: q1,
                r1: r1
            }
        }
    }
}

impl<T: Float> Decomposition for HouseholderQr<T> {
    type Factors = QR<T>;
    fn unpack(self) -> QR<T> {
        use internal_utils;
        let q = assemble_q(&self.qr, &self.tau);
        let mut r = self.qr;
        internal_utils::nullify_lower_triangular_part(&mut r);
        QR {
            q: q,
            r: r
        }
    }
}

fn assemble_q<T: Float>(qr: &Matrix<T>, tau: &Vec<T>) -> Matrix<T> {
    let m = qr.rows();
    let q_operator = householder::create_composition(qr, tau);
    q_operator.first_k_columns(m)
}

fn extract_r1<T: Float>(qr: &Matrix<T>) -> Matrix<T> {
    let m = qr.rows();
    let n = qr.cols();
    let mut data = Vec::<T>::with_capacity(m * n);

    assert!(m > n, "We only want to extract r1 if m > n!");

    for (i, row) in qr.row_iter().take(n).enumerate() {
        for _ in 0 .. i {
            data.push(T::zero());
        }

        for element in row.raw_slice().iter().skip(i).cloned() {
            data.push(element);
        }
    }
    Matrix::new(n, n, data)
}

impl<T> Matrix<T>
    where T: Any + Float
{
    /// Compute the QR decomposition of the matrix.
    ///
    /// Returns the tuple (Q,R).
    ///
    /// Note: this function is deprecated in favor of
    /// [HouseholderQr](./decomposition/struct.HouseholderQr.html)
    /// and will be removed in a future release.
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
    /// let (q, r) = m.qr_decomp().unwrap();
    /// # }
    /// ```
    ///
    /// # Failures
    ///
    /// - Cannot compute the QR decomposition.
    #[deprecated]
    pub fn qr_decomp(self) -> Result<(Matrix<T>, Matrix<T>), Error> {
        let m = self.rows();
        let n = self.cols();

        let mut q = Matrix::<T>::identity(m);
        let mut r = self;

        for i in 0..(n - ((m == n) as usize)) {
            let holder_transform: Result<Matrix<T>, Error>;
            {
                let lower_slice = MatrixSlice::from_matrix(&r, [i, i], m - i, 1);
                holder_transform =
                    Matrix::make_householder(&lower_slice.iter().cloned().collect::<Vec<_>>());
            }

            if !holder_transform.is_ok() {
                return Err(Error::new(ErrorKind::DecompFailure,
                                      "Cannot compute QR decomposition."));
            } else {
                let mut holder_data = holder_transform.unwrap().into_vec();

                // This bit is inefficient
                // using for now as we'll swap to lapack eventually.
                let mut h_full_data = Vec::with_capacity(m * m);

                for j in 0..m {
                    let mut row_data: Vec<T>;
                    if j < i {
                        row_data = vec![T::zero(); m];
                        row_data[j] = T::one();
                        h_full_data.extend(row_data);
                    } else {
                        row_data = vec![T::zero(); i];
                        h_full_data.extend(row_data);
                        h_full_data.extend(holder_data.drain(..m - i));
                    }
                }

                let h = Matrix::new(m, m, h_full_data);

                q = q * &h;
                r = h * &r;
            }
        }

        Ok((q, r))
    }
}

#[cfg(test)]
mod tests {
    use super::HouseholderQr;
    use super::{QR, ThinQR};

    use matrix::{Matrix, BaseMatrix};
    use matrix::decomposition::Decomposition;

    use testsupport::is_upper_triangular;

    fn verify_qr(x: Matrix<f64>, qr: QR<f64>) {
        let m = x.rows();
        let QR { ref q, ref r } = qr;

        assert_matrix_eq!(q * r, x, comp = float, ulp = 100);
        assert!(is_upper_triangular(r));

        // check orthogonality
        assert_matrix_eq!(q.transpose() * q, Matrix::identity(m),
            comp = float, ulp = 100);
        assert_matrix_eq!(q * q.transpose(), Matrix::identity(m),
            comp = float, ulp = 100);
    }

    fn verify_thin_qr(x: Matrix<f64>, qr: ThinQR<f64>) {
        use std::cmp::min;

        let m = x.rows();
        let n = x.cols();
        let ThinQR { ref q1, ref r1 } = qr;

        assert_matrix_eq!(q1 * r1, x, comp = float, ulp = 100);
        assert!(is_upper_triangular(r1));

        // Check that q1 has orthogonal columns
        assert_matrix_eq!(q1.transpose() * q1, Matrix::identity(min(m, n)),
            comp = float, ulp = 100);
    }

    #[test]
    pub fn householder_qr_unpack_reconstruction() {
        {
            // 1x1
            let x = matrix![1.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack();
            verify_qr(x, qr);
        }

        {
            // 1x2
            let x = matrix![1.0, 2.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack();
            verify_qr(x, qr);
        }

        {
            // 2x1
            let x = matrix![1.0;
                            2.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack();
            verify_qr(x, qr);
        }

        {
            // 2x2
            let x = matrix![1.0, 2.0;
                            3.0, 4.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack();
            verify_qr(x, qr);
        }

        {
            // 3x2
            let x = matrix![1.0, 2.0;
                            3.0, 4.0;
                            4.0, 5.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack();
            verify_qr(x, qr);
        }

        {
            // 2x3
            let x = matrix![1.0, 2.0, 3.0;
                            4.0, 5.0, 6.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack();
            verify_qr(x, qr);
        }

        {
            // 3x3
            let x = matrix![1.0, 2.0, 3.0;
                            4.0, 5.0, 6.0;
                            7.0, 8.0, 9.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack();
            verify_qr(x, qr);
        }
    }

    #[test]
    fn householder_qr_unpack_square_reconstruction_corner_cases() {
        {
            let x = matrix![-1.0, 0.0;
                             0.0, 1.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack();
            verify_qr(x, qr);
        }

        {
            let x = matrix![1.0,  0.0,  0.0;
                            0.0,  1.0,  0.0;
                            0.0,  0.0, -1.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack();
            verify_qr(x, qr);
        }

        {
            let x = matrix![1.0,   0.0,  0.0;
                            0.0,  -1.0,  0.0;
                            0.0,   0.0, -1.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack();
            verify_qr(x, qr);
        }
    }

    #[test]
    fn householder_qr_unpack_thin_reconstruction() {
        {
            // 1x1
            let x = matrix![1.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack_thin();
            verify_thin_qr(x, qr);
        }

        {
            // 1x2
            let x = matrix![1.0, 2.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack_thin();
            verify_thin_qr(x, qr);
        }

        {
            // 2x1
            let x = matrix![1.0;
                            2.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack_thin();
            verify_thin_qr(x, qr);
        }

        {
            // 2x2
            let x = matrix![1.0, 2.0;
                            3.0, 4.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack_thin();
            verify_thin_qr(x, qr);
        }

        {
            // 3x2
            let x = matrix![1.0, 2.0;
                            3.0, 4.0;
                            4.0, 5.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack_thin();
            verify_thin_qr(x, qr);
        }

        {
            // 2x3
            let x = matrix![1.0, 2.0, 3.0;
                            4.0, 5.0, 6.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack_thin();
            verify_thin_qr(x, qr);
        }

        {
            // 3x3
            let x = matrix![1.0, 2.0, 3.0;
                            4.0, 5.0, 6.0;
                            7.0, 8.0, 9.0];
            let qr = HouseholderQr::decompose(x.clone()).unpack_thin();
            verify_thin_qr(x, qr);
        }
    }
}
