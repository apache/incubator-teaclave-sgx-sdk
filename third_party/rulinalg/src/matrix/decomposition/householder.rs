use matrix::{Matrix, BaseMatrix, BaseMatrixMut, Column, ColumnMut};
use vector::Vector;
use utils;
use std::vec::*;

use libnum::Float;

/// An efficient representation of a Householder reflection,
/// also known as Householder matrix or elementary reflector.
///
/// Mathematically, it has the form
/// H := I - τ v vᵀ,
/// with τ = 2 / (vᵀv).
///
/// Given a vector `x`, it is possible to choose `v` such that
/// Hx = a e1,
/// where a is a constant and e1 is the standard unit vector
/// whose elements are zero except the first, which is 1.
///
/// The implementation here is largely based upon the contents
/// of Chapter 5.1 (Householder and Givens Transformations)
/// in Matrix Computations, 4th Ed, Golub and Van Loan,
/// but with modifications that among other things makes
/// the implementation compliant with LAPACK.
pub struct HouseholderReflection<T> {
    v: Vector<T>,
    tau: T
}

impl<T: Float> HouseholderReflection<T> {
    /// Compute the Householder reflection which will zero out
    /// all elements in the vector `x` except the first.
    pub fn compute(x: Vector<T>) -> HouseholderReflection<T> {
        // The following code is loosely based on notes in
        // Applied Numerical Linear Algebra by Demmel,
        // Matrix Computations 4th Ed by Golub & Van Loan,
        // as well as LAPACK documentation.
        //
        // From Demmel, we have that we can choose the vector
        // v = [ x1 + sign(x1) norm(x) ]
        //     [ x[2:] ]
        // as our Householder vector (the choice of sign in v(1) avoids
        // cancellation issues which would lead to reduced accuracy in
        // certain corner cases). However, we must divide v by
        // v1 so that the first element of v is 1. Propagating these
        // changes into τ leads to the below code.
        // Note that if x[2:] == 0 (norm is identically zero),
        // we explicitly set τ = 0 since x is already a multiple of
        // the unit vector e1 (and we avoid potential division by zero).
        let m = x.size();

        if m > 0 {
            let sigma = utils::dot(&x.data()[1 ..], &x.data()[1 ..]);
            let x0 = x[0];
            let tau;
            let mut v = x;

            if sigma == T::zero() {
                // The vector is already a multiple of e1, the unit vector for which
                // 1 is the first element and all other elements are zero.
                tau = T::zero();
            } else {
                let x_norm = T::sqrt(x0 * x0 + sigma);
                // This choice avoids accuracy issues related
                // to cancellation
                // (see e.g. Demmel, Applied Numerical Linear Algebra).
                let v0 = if x0 > T::zero() { x0 + x_norm }
                         else { x0 - x_norm };

                // Normalize the Householder vector v so that
                // its first element is 1.
                let two = T::from(2).unwrap();
                tau = two * v0 * v0 / (v0 * v0 + sigma);
                v[0] = v0;
                v = v / v0;
            }

            HouseholderReflection {
                v: v,
                tau: tau
            }
        } else {
            // x is an empty vector, so just use it as the
            // Householder vector
            HouseholderReflection {
                v: x,
                tau: T::zero()
            }
        }
    }

    /// Left-multiplies the given matrix by this Householder reflection.
    ///
    /// More precisely, let `H` denote this Householder reflection matrix,
    /// and let `A` be a dimensionally compatible matrix. Then
    /// this function computes the product `HA` and stores the result
    /// back in `A`.
    ///
    /// The user must provide a buffer of size `A.cols()` which is used
    /// to store intermediate results.
    pub fn buffered_left_multiply_into<M>(&self, matrix: &mut M, buffer: &mut [T])
        where M: BaseMatrixMut<T>
    {
        use internal_utils::{transpose_gemv, ger};
        assert!(buffer.len() == matrix.cols());

        // Recall that the Householder reflection is represented by
        // H = I - τ v vᵀ,
        //
        // which means that the product HA can be computed as
        //
        // HA = A - (τ v) (vᵀ A) = A - (τ v) (Aᵀ v)ᵀ,
        //
        // which constitutes a (transposed) matrix-vector product`
        // u = Aᵀ v and a rank-1 update A <- A - τ v uᵀ
        //
        // Performing both the matrix-vector product and the
        // rank-1 update can actually be performed without
        // allocating any additional memory, but this would access
        // the data in the matrix column-by-column, which is inefficient.
        // Instead, we will use the provided buffer to hold the result of the
        // matrix-vector product.
        let ref v = self.v.data();
        let u = buffer;

        // u = A^T v
        transpose_gemv(matrix, v, u);

        // A <- A - τ v uᵀ
        ger(matrix, - self.tau, v, u);
    }

    pub fn as_vector(&self) -> &Vector<T> {
        &self.v
    }

    pub fn into_vector(self) -> Vector<T> {
        self.v
    }

    pub fn from_parameters(v: Vector<T>, tau: T) -> HouseholderReflection<T> {
        HouseholderReflection {
            v: v,
            tau: tau
        }
    }

    pub fn tau(&self) -> T {
        self.tau
    }

    pub fn store_in_col(&self, col: &mut ColumnMut<T>) {
        let m = col.rows();
        assert!(m == self.v.size());

        if m > 0 {
            // The first element is implicitly 1, so make sure we don't
            // touch it
            let mut slice_after_first =  col.sub_slice_mut([1, 0], m - 1, 1);
            let mut col_after_first = slice_after_first.col_mut(0);
            col_after_first.clone_from_slice(&self.as_vector().data()[1..]);
        }
    }
}

/// An efficient representation for a composition of
/// Householder transformations.
///
/// This means that `HouseholderComposition` represents
/// an operator `Q` of the form
///
/// ```text
/// Q = Q_1 * Q_2 * ... * Q_p
/// ```
///
/// as explained in the documentation for
/// [HouseholderQr](struct.HouseholderQr.html).
#[derive(Debug, Clone)]
pub struct HouseholderComposition<'a, T> where T: 'a {
    storage: &'a Matrix<T>,
    tau: &'a [T]
}

/// Instantiates a HouseholderComposition with the given
/// storage and vector of tau values.
///
/// Note: This function is deliberately not exported to
/// the public API. This means that users cannot create
/// a HouseholderComposition by themselves, which is desirable
/// because we want to have the freedom to change details
/// of the internal representation if necessary.
pub fn create_composition<'a, T>(storage: &'a Matrix<T>, tau: &'a [T])
    -> HouseholderComposition<'a, T>
{
    HouseholderComposition {
            storage: storage,
            tau: tau
    }
}

impl<'a, T> HouseholderComposition<'a, T> where T: Float {
    /// Given a matrix `A` of compatible dimensions, computes
    /// the product `A <- QA`, storing the result in `A`.
    pub fn left_multiply_into<X>(&self, matrix: &mut X)
        where X: BaseMatrixMut<T>
    {
        use std::cmp::min;

        let m = self.storage.rows();
        let n = self.storage.cols();
        let p = min(m, n);
        let q = matrix.cols();

        assert!(matrix.rows() == m, "Matrix does not have compatible dimensions.");

        let mut house_buffer = Vec::with_capacity(m);
        let mut multiply_buffer = vec![T::zero(); q];
        for j in (0 .. p).rev() {
            house_buffer.resize(m - j, T::zero());
            let storage_block = self.storage.sub_slice([j, j], m - j, n - j);
            let mut matrix_block = matrix.sub_slice_mut([j, 0], m - j, q);
            let house = load_house_from_col(&storage_block.col(0),
                                            self.tau[j], house_buffer);
            house.buffered_left_multiply_into(&mut matrix_block,
                                              &mut multiply_buffer);
            house_buffer = house.into_vector().into_vec();
        }
    }

    /// Computes the first k columns of the implicitly
    /// stored matrix `Q`.
    ///
    /// # Panics
    /// - `k` must be less than or equal to `m`, the number
    ///   of rows of `Q`.
    pub fn first_k_columns(&self, k: usize) -> Matrix<T> {
        use std::cmp::min;
        let m = self.storage.rows();
        let n = self.storage.cols();
        let p = min(m, n);

        assert!(k <= self.storage.rows(),
            "k cannot exceed m, the number of rows of Q");

        // Let Q_k = Q[:, 1:k], the first k rows of Q
        let mut q_k = Matrix::from_fn(m, k, |row, col| {
            if row == col { T::one()}
            else { T::zero() }
        });

        // This is almost identical to left_multiply_into,
        // but we can use the sparsity of the identity matrix
        // to reduce the number of operations
        // (note the size of the "q_k_block")
        let mut buffer = Vec::with_capacity(m);
        let mut multiply_buffer = Vec::with_capacity(k);
        for j in (0 .. min(p, k)).rev() {
            buffer.resize(m - j, T::zero());
            multiply_buffer.resize(k - j, T::zero());
            let storage_block = self.storage.sub_slice([j, j], m - j, n - j);
            let mut q_k_block = q_k.sub_slice_mut([j, j], m - j, k - j);
            let house = load_house_from_col(&storage_block.col(0),
                                            self.tau[j], buffer);
            house.buffered_left_multiply_into(&mut q_k_block,
                                              &mut multiply_buffer);
            buffer = house.into_vector().into_vec();
        }
        q_k
    }
}

fn load_house_from_col<T: Float>(col: &Column<T>, tau: T, buffer: Vec<T>)
    -> HouseholderReflection<T> {
    let mut v = buffer;

    col.clone_into_slice(&mut v);

    // First element is implicitly 1 regardless of
    // whatever is stored in the column.
    if let Some(first_element) = v.get_mut(0) {
        *first_element = T::one();
    }

    HouseholderReflection::from_parameters(Vector::new(v), tau)
}

#[cfg(test)]
mod tests {
    use vector::Vector;
    use matrix::{Matrix, BaseMatrix};
    use super::HouseholderReflection;
    use super::create_composition;

    pub fn house_as_matrix(house: HouseholderReflection<f64>)
        -> Matrix<f64>
    {
        let m = house.v.size();
        let v = Matrix::new(m, 1, house.v.into_vec());
        let v_t = v.transpose();
        Matrix::identity(m) - v * v_t * house.tau
    }

    fn verify_house(x: Vector<f64>, house: HouseholderReflection<f64>) {
        let m = x.size();
        assert!(m > 0);

        let house = house_as_matrix(house);
        let y = house.clone() * x.clone();

        // Check that y[1 ..] is approximately zero
        let z = Vector::new(y.data().iter().skip(1).cloned().collect::<Vec<_>>());
        assert_vector_eq!(z, Vector::zeros(m - 1), comp = float, eps = 1e-12);

        // Check that applying the Householder transformation again
        // recovers the original vector (since H = H^T = inv(H))
        let w = house * y;
        assert_vector_eq!(x, w, comp = float);
    }

    #[test]
    fn compute_empty_vector() {
        let x: Vector<f64> = vector![];
        let house = HouseholderReflection::compute(x.clone());
        assert_scalar_eq!(house.tau, 0.0);
        assert_vector_eq!(house.v, x.clone());
    }

    #[test]
    fn compute_single_element_vector() {
        let x = vector![2.0];
        let house = HouseholderReflection::compute(x.clone());
        assert_scalar_eq!(house.tau, 0.0);
    }

    #[test]
    fn compute_examples() {
        {
            let x = vector![1.0, 0.0, 0.0];
            let house = HouseholderReflection::compute(x.clone());
            verify_house(x, house);
        }

        {
            let x = vector![-1.0, 0.0, 0.0];
            let house = HouseholderReflection::compute(x.clone());
            verify_house(x, house);
        }

        {
            let x = vector![3.0, -2.0, 5.0];
            let house = HouseholderReflection::compute(x.clone());
            verify_house(x, house);
        }
    }

    #[test]
    fn householder_reflection_left_multiply() {
        let mut x = matrix![ 0.0,  1.0,  2.0,  3.0;
                             4.0,  5.0,  6.0,  7.0;
                             8.0,  9.0, 10.0, 11.0;
                            12.0, 13.0, 14.0, 15.0 ];

        // The provided data is rather rubbish, but
        // the result should still hold
        let h = HouseholderReflection {
            tau: 0.06666666666666667,
            v: vector![1.0, 2.0, 3.0, 4.0]
        };

        let mut buffer = vec![0.0; 4];

        h.buffered_left_multiply_into(&mut x, &mut buffer);

        let expected = matrix![ -5.3333,  -5.0000, -4.6667,  -4.3333;
                                -6.6667,  -7.0000, -7.3333,  -7.6667;
                                -8.0000,  -9.0000,-10.0000, -11.0000;
                                -9.3333, -11.0000,-12.6667, -14.3333];
        assert_matrix_eq!(x, expected, comp = abs, tol = 1e-3);
    }

    #[test]
    fn householder_composition_left_multiply() {
        let storage = matrix![ 5.0,  3.0,  2.0;
                               2.0,  1.0,  3.0;
                              -2.0,  3.0, -2.0];
        let tau = vec![2.0/9.0, 1.0 / 5.0, 2.0];

        // `q` is a manually computed matrix representation
        // of the Householder composition stored implicitly in
        // `storage` and `tau. We leave it here to make writing
        // further tests easier
        // let q = matrix![7.0/9.0, -28.0/45.0,   4.0/45.0;
        //                -4.0/9.0, - 4.0/ 9.0,   7.0/ 9.0;
        //                 4.0/9.0,  29.0/45.0,  28.0/45.0];
        let composition = create_composition(&storage, &tau);

        {
            // Square
            let mut x = matrix![4.0,  5.0, -3.0;
                                2.0, -1.0, -3.0;
                                1.0,  3.0,  5.0];
            composition.left_multiply_into(&mut x);

            let expected = matrix![ 88.0/45.0, 43.0/9.0, -1.0/45.0;
                                   -17.0/ 9.0,  5.0/9.0, 59.0/ 9.0;
                                   166.0/45.0, 31.0/9.0, -7.0/45.0];
            assert_matrix_eq!(x, expected, comp = float, eps = 1e-15);
        }

        {
            // Tall
            let mut x = matrix![ 4.0, 5.0;
                                 3.0, 2.0;
                                -1.0,-2.0];
            composition.left_multiply_into(&mut x);
            let expected = matrix![52.0/45.0,  37.0/15.0;
                                  -35.0/ 9.0, -14.0/ 3.0;
                                  139.0/45.0,  34.0/15.0];
            assert_matrix_eq!(x, expected, comp = float, eps = 1e-15);
        }

        {
            // Short
            let mut x = matrix![ 4.0,  5.0,  2.0, -5.0;
                                 3.0,  2.0,  1.0,  1.0;
                                -1.0, -2.0,  0.0, -5.0];
            composition.left_multiply_into(&mut x);
            let expected = matrix![52.0/45.0,  37.0/15.0, 14.0/15.0, -223.0/45.0;
                                  -35.0/ 9.0, -14.0/ 3.0, -4.0/ 3.0,  -19.0/ 9.0;
                                  139.0/45.0,  34.0/15.0, 23.0/15.0, -211.0/45.0];
            assert_matrix_eq!(x, expected, comp = float, eps = 1e-15);
        }
    }

    #[test]
    fn householder_composition_first_k_columns() {
        let storage = matrix![ 5.0,  3.0,  2.0;
                               2.0,  1.0,  3.0;
                              -2.0,  3.0, -2.0];
        let tau = vec![2.0/9.0, 1.0 / 5.0, 2.0];
        let composition = create_composition(&storage, &tau);

        // This corresponds to the following `Q` matrix
        let q = matrix![7.0/9.0, -28.0/45.0,   4.0/45.0;
                       -4.0/9.0, - 4.0/ 9.0,   7.0/ 9.0;
                        4.0/9.0,  29.0/45.0,  28.0/45.0];
        {
            // First 0 columns
            let q_k = composition.first_k_columns(0);
            assert_eq!(q_k.rows(), 3);
            assert_eq!(q_k.cols(), 0);
        }

        {
            // First column
            let q_k = composition.first_k_columns(1);
            assert_matrix_eq!(q_k, q.sub_slice([0, 0], 3, 1),
                              comp = float);
        }

        {
            // First 2 columns
            let q_k = composition.first_k_columns(2);
            assert_matrix_eq!(q_k, q.sub_slice([0, 0], 3, 2),
                              comp = float);
        }

        {
            // First 3 columns
            let q_k = composition.first_k_columns(3);
            assert_matrix_eq!(q_k, q.sub_slice([0, 0], 3, 3),
                              comp = float);
        }
    }


}
