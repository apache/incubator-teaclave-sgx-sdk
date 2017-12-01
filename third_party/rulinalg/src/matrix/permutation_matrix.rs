use std;
use std::vec::*;
use matrix::{Matrix, BaseMatrix, BaseMatrixMut};
use vector::Vector;
use error::{Error, ErrorKind};

use libnum::Num;

/// An efficient implementation of a permutation matrix.
///
/// # Examples
/// ```
/// # #[macro_use] extern crate rulinalg; fn main() {
/// use rulinalg::matrix::PermutationMatrix;
///
/// let ref x = matrix![1, 2, 3;
///                     4, 5, 6;
///                     7, 8, 9];
///
/// // Swap the two first rows of x by left-multiplying a permutation matrix
/// let expected = matrix![4, 5, 6;
///                        1, 2, 3;
///                        7, 8, 9];
/// let mut p = PermutationMatrix::identity(3);
/// p.swap_rows(0, 1);
/// assert_eq!(expected, p * x);
///
/// // Swap the two last columns of x by right-multiplying a permutation matrix
/// let expected = matrix![1, 3, 2;
///                        4, 6, 5;
///                        7, 9, 8];
/// let mut p = PermutationMatrix::identity(3);
/// p.swap_rows(1, 2);
/// assert_eq!(expected, x * p);
///
/// // One can also construct the same permutation matrix directly
/// // from an array representation.
/// let ref p = PermutationMatrix::from_array(vec![0, 2, 1]).unwrap();
/// assert_eq!(expected, x * p);
///
/// // One may also obtain a full matrix representation of the permutation
/// assert_eq!(p.as_matrix(), matrix![1, 0, 0;
///                                   0, 0, 1;
///                                   0, 1, 0]);
///
/// // The inverse of a permutation matrix can efficiently be obtained
/// let p_inv = p.inverse();
///
/// // And permutations can be composed through multiplication
/// assert_eq!(p * p_inv, PermutationMatrix::identity(3));
/// # }
/// ```
///
/// # Rationale and complexity
///
/// A [permutation matrix](https://en.wikipedia.org/wiki/Permutation_matrix)
/// is a very special kind of matrix. It is essentially a matrix representation
/// of the more general concept of a permutation. That is, an `n` x `n` permutation
/// matrix corresponds to a permutation of ordered sets whose cardinality is `n`.
/// In particular, given an `m` x `n` matrix `A` and an `m` x `m` permutation
/// matrix `P`, the action of left-multiplying `A` by `P`, `PA`, corresponds
/// to permuting the rows of `A` by the given permutation represented by `P`.
/// Conversely, right-multiplication corresponds to column permutation.
/// More precisely, given another permutation matrix `Q` of size `n` x `n`,
/// then `AQ` is the corresponding permutation of the columns of `A`.
///
/// Due to their unique structure, permutation matrices can be much more
/// efficiently represented and applied than general matrices. Recall that
/// for general matrices `X` and `Y` of size `m` x `m` and `n` x `n` respectively,
/// the storage of `X` requires O(`m`<sup>2</sup>) memory and the storage of
/// `Y` requires O(`n`<sup>2</sup>) memory. Ignoring for the moment the existence
/// of Strassen's matrix multiplication algorithm and more theoretical alternatives,
/// the multiplication `XA` requires O(`m`<sup>2</sup>`n`) operations, and
/// the multiplication `AY` requires O(`m``n`<sup>2</sup>) operations.
///
/// By contrast, the storage of `P` requires only O(`m`) memory, and
/// the storage of `K` requires O(`n`) memory. Moreover, the products
/// `PA` and `AK` both require merely O(`mn`) operations.
///
/// # Representation
/// A permutation of an ordered set of cardinality *n* is a map of the form
///
/// ```text
/// p: { 1, ..., n } -> { 1, ..., n }.
/// ```
///
/// That is, for any index `i`, the permutation `p` sends `i` to some
/// index `j = p(i)`, and hence the map may be represented as an array of integers
/// of length *n*.
///
/// By convention, an instance of `PermutationMatrix` represents row permutations.
/// That is, the indices referred to above correspond to *row indices*,
/// and the internal representation of a `PermutationMatrix` is an array
/// describing how the permutation sends a row index `i` to a new row index
/// `j` in the permuted matrix. Because of this internal representation, one can only
/// efficiently swap *rows* of a `PermutationMatrix`.
/// However, keep in mind that while this API only lets one swap individual rows
/// of the permutation matrix itself, the right-multiplication of a general
/// matrix with a permutation matrix will permute the columns of the general matrix,
/// and so in practice this restriction is insignificant.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PermutationMatrix<T> {
    // A permutation matrix of dimensions NxN is represented as a permutation of the rows
    // of an NxM matrix for any M.
    perm: Vec<usize>,

    // Currently, we need to let PermutationMatrix be generic over T,
    // because BaseMatrixMut is.
    marker: std::marker::PhantomData<T>
}

/// Parity is the fact of being even or odd.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Parity {
    /// Even parity.
    Even,
    /// Odd parity.
    Odd
}

impl<T> PermutationMatrix<T> {
    /// The identity permutation.
    pub fn identity(n: usize) -> Self {
        PermutationMatrix {
            perm: (0 .. n).collect(),
            marker: std::marker::PhantomData
        }
    }

    /// Swaps rows i and j in the permutation matrix.
    pub fn swap_rows(&mut self, i: usize, j: usize) {
        self.perm.swap(i, j);
    }

    /// The inverse of the permutation matrix.
    pub fn inverse(&self) -> PermutationMatrix<T> {
        let mut inv: Vec<usize> = vec![0; self.size()];

        for (source, target) in self.perm.iter().cloned().enumerate() {
            inv[target] = source;
        }

        PermutationMatrix {
            perm: inv,
            marker: std::marker::PhantomData
        }
    }

    /// The size of the permutation matrix.
    ///
    /// A permutation matrix is a square matrix, so `size()` is equal
    /// to both the number of rows, as well as the number of columns.
    pub fn size(&self) -> usize {
        self.perm.len()
    }

    /// Constructs a `PermutationMatrix` from an array.
    ///
    /// # Errors
    /// The supplied N-length array must satisfy the following:
    ///
    /// - Each element must be in the half-open range [0, N).
    /// - Each element must be unique.
    pub fn from_array<A: Into<Vec<usize>>>(array: A) -> Result<PermutationMatrix<T>, Error> {
        let p = PermutationMatrix {
            perm: array.into(),
            marker: std::marker::PhantomData
        };
        validate_permutation(&p.perm).map(|_| p)
    }

    /// Constructs a `PermutationMatrix` from an array, without checking the validity of
    /// the supplied permutation.
    ///
    /// # Safety
    /// The supplied N-length array must satisfy the following:
    ///
    /// - Each element must be in the half-open range [0, N).
    /// - Each element must be unique.
    ///
    /// Note that while this function *itself* is technically safe
    /// regardless of the input array, passing an incorrect permutation matrix
    /// may cause undefined behavior in other methods of `PermutationMatrix`.
    /// As such, it may be difficult to debug. The user is strongly
    /// encouraged to use the safe function `from_array`, which for
    /// most real world applications only incurs a minor
    /// or even insignificant performance hit as a result of the
    /// extra validation.
    pub unsafe fn from_array_unchecked<A: Into<Vec<usize>>>(array: A) -> PermutationMatrix<T> {
        let p = PermutationMatrix {
            perm: array.into(),
            marker: std::marker::PhantomData
        };
        p
    }

    /// Maps the given row index into the resulting row index in the permuted matrix.
    ///
    /// More specifically, if the permutation sends row `i` to `j`, then
    /// `map_row(i)` returns `j`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rulinalg::matrix::PermutationMatrix;
    /// let mut p = PermutationMatrix::<u32>::identity(3);
    /// p.swap_rows(1, 2);
    /// assert_eq!(p.map_row(1), 2);
    /// ```
    pub fn map_row(&self, row_index: usize) -> usize {
        self.perm[row_index]
    }

    /// Computes the parity of the permutation (even- or oddness).
    pub fn parity(mut self) -> Parity {
        // As it happens, permute_by_swap effectively decomposes
        // each disjoint cycle in the permutation into a series
        // of transpositions. The result is that the whole permutation
        // is effectively decomposed into a series of
        // transpositions.
        // Hence, if we start out by assuming that the permutation
        // is even and simply flip this variable every time a swap
        // (transposition) is performed, we'll have the result by
        // the end of the procedure.
        let mut is_even = true;
        permute_by_swap(&mut self.perm, |_, _| is_even = !is_even);

        if is_even {
            Parity::Even
        } else {
            Parity::Odd
        }
    }
}

impl<T: Num> PermutationMatrix<T> {
    /// The permutation matrix in an equivalent full matrix representation.
    pub fn as_matrix(&self) -> Matrix<T> {
        Matrix::from_fn(self.size(), self.size(), |i, j|
            if self.perm[i] == j {
                T::one()
            } else {
                T::zero()
            }
        )
    }

    /// Computes the determinant of the permutation matrix.
    ///
    /// The determinant of a permutation matrix is always
    /// +1 (if the permutation is even) or
    /// -1 (if the permutation is odd).
    pub fn det(self) -> T {
        let parity = self.parity();
        match parity {
            Parity::Even => T::one(),
            Parity::Odd  => T::zero() - T::one()
        }
    }
}

impl<T> PermutationMatrix<T> {
    /// Permutes the rows of the given matrix in-place.
    ///
    /// # Panics
    ///
    /// - The number of rows in the matrix is not equal to
    ///   the size of the permutation matrix.
    pub fn permute_rows_in_place<M>(mut self, matrix: &mut M) where M: BaseMatrixMut<T> {
        validate_permutation_left_mul_dimensions(&self, matrix);
        permute_by_swap(&mut self.perm, |i, j| matrix.swap_rows(i, j));
    }

    /// Permutes the columns of the given matrix in-place.
    ///
    /// # Panics
    ///
    /// - The number of columns in the matrix is not equal to
    ///   the size of the permutation matrix.
    pub fn permute_cols_in_place<M>(mut self, matrix: &mut M) where M: BaseMatrixMut<T> {
        validate_permutation_right_mul_dimensions(matrix, &self);
        // Note: it _may_ be possible to increase cache efficiency
        // of this routine by swapping elements in each row individually
        // (since matrices are row major), but this would mean augmenting
        // permute_by_swap in such a way that the original permutation can
        // be recovered, which includes a little bit of additional work.
        // Moreover, it would mean having to work with signed indices
        // instead of unsigned (although temporarily casting would be sufficient),
        // which may or may not complicate matters.
        // For now, it was deemed significantly simpler and probably good enough
        // to just swap whole columns instead.
        permute_by_swap(&mut self.perm, |i, j| matrix.swap_cols(i, j));
    }

    /// Permutes the elements of the given vector in-place.
    ///
    /// # Panics
    ///
    /// - The size of the vector is not equal to the size of
    ///   the permutation matrix.
    pub fn permute_vector_in_place(mut self, vector: &mut Vector<T>) {
        validate_permutation_vector_dimensions(&self, vector);
        permute_by_swap(&mut self.perm, |i, j| vector.mut_data().swap(i, j));
    }
}

impl<T: Clone> PermutationMatrix<T> {
    /// Permutes the rows of the given `source_matrix` and stores the
    /// result in `buffer`.
    ///
    /// # Panics
    ///
    /// - The number of rows in the source matrix is not equal to
    ///   the size of the permutation matrix.
    /// - The dimensions of the source matrix and the buffer
    ///   are not identical.
    pub fn permute_rows_into_buffer<X, Y>(&self, source_matrix: &X, buffer: &mut Y)
        where X: BaseMatrix<T>, Y: BaseMatrixMut<T> {
        assert!(source_matrix.rows() == buffer.rows()
                && source_matrix.cols() == buffer.cols(),
                "Source and target matrix must have equal dimensions.");
        validate_permutation_left_mul_dimensions(self, source_matrix);
        for (source_row, target_row_index) in source_matrix.row_iter()
                                                           .zip(self.perm.iter()
                                                                         .cloned()) {
            buffer.row_mut(target_row_index)
                  .raw_slice_mut()
                  .clone_from_slice(source_row.raw_slice());
        }
    }

    /// Permutes the columns of the given `source_matrix` and stores the
    /// result in `buffer`.
    ///
    /// # Panics
    ///
    /// - The number of columns in the source matrix is not equal to
    ///   the size of the permutation matrix.
    /// - The dimensions of the source matrix and the buffer
    ///   are not identical.
    pub fn permute_cols_into_buffer<X, Y>(&self, source_matrix: &X, target_matrix: &mut Y)
        where X: BaseMatrix<T>, Y: BaseMatrixMut<T> {
        assert!(source_matrix.rows() == target_matrix.rows()
                && source_matrix.cols() == target_matrix.cols(),
                "Source and target matrix must have equal dimensions.");
        validate_permutation_right_mul_dimensions(source_matrix, self);

        // Permute columns in one row at a time for (presumably) better cache performance
        for (row_index, source_row) in source_matrix.row_iter()
                                                           .map(|r| r.raw_slice())
                                                           .enumerate() {
            let target_row = target_matrix.row_mut(row_index).raw_slice_mut();
            for (source_element, target_col) in source_row.iter().zip(self.perm.iter().cloned()) {
                target_row[target_col] = source_element.clone();
            }
        }
    }

    /// Permutes the elements of the given `source_vector` and stores the
    /// result in `buffer`.
    ///
    /// # Panics
    ///
    /// - The size of the source vector is not equal to the
    ///   size of the permutation matrix.
    /// - The dimensions of the source vector and the buffer
    ///   are not identical.
    pub fn permute_vector_into_buffer(
        &self,
        source_vector: &Vector<T>,
        buffer: &mut Vector<T>
    ) {
        assert!(source_vector.size() == buffer.size(),
               "Source and target vector must have equal dimensions.");
        validate_permutation_vector_dimensions(self, buffer);
        for (source_element, target_index) in source_vector.data()
                                                           .iter()
                                                           .zip(self.perm.iter().cloned()) {
            buffer[target_index] = source_element.clone();
        }
    }

    /// Computes the composition of `self` with the given `source_perm`
    /// and stores the result in `buffer`.
    ///
    /// # Panics
    ///
    /// - The size of the permutation matrix (self) is not equal to the
    ///   size of the source permutation matrix.
    pub fn compose_into_buffer(
        &self,
        source_perm: &PermutationMatrix<T>,
        buffer: &mut PermutationMatrix<T>
    ) {
        assert!(source_perm.size() == buffer.size(),
            "Source and target permutation matrix must have equal dimensions.");
        let left = self;
        let right = source_perm;
        for i in 0 .. self.perm.len() {
            buffer.perm[i] = left.perm[right.perm[i]];
        }
    }
}

impl<T> From<PermutationMatrix<T>> for Vec<usize> {
    fn from(p: PermutationMatrix<T>) -> Vec<usize> {
        p.perm
    }
}

impl<'a, T> Into<&'a [usize]> for &'a PermutationMatrix<T> {
    fn into(self) -> &'a [usize] {
        &self.perm
    }
}

fn validate_permutation_vector_dimensions<T>(p: &PermutationMatrix<T>, v: &Vector<T>) {
    assert!(p.size() == v.size(),
            "Permutation matrix and Vector dimensions are not compatible.");
}


fn validate_permutation_left_mul_dimensions<T, M>(p: &PermutationMatrix<T>, rhs: &M)
    where M: BaseMatrix<T> {
     assert!(p.size() == rhs.rows(),
            "Permutation matrix and right-hand side matrix dimensions
             are not compatible.");
}

fn validate_permutation_right_mul_dimensions<T, M>(lhs: &M, p: &PermutationMatrix<T>)
    where M: BaseMatrix<T> {
     assert!(lhs.cols() == p.size(),
            "Left-hand side matrix and permutation matrix dimensions
             are not compatible.");
}

fn validate_permutation(perm: &[usize]) -> Result<(), Error> {
    // Recall that a permutation array of size n is valid if:
    // 1. All elements are in the range [0, n)
    // 2. All elements are unique

    let n = perm.len();

    // Here we use a vector of boolean. If memory usage or performance
    // is ever an issue, we could replace the vector with a bit vector
    // (from e.g. the bit-vec crate), which would cut memory usage
    // to 1/8, and likely improve performance due to more data
    // fitting in cache.
    let mut visited = vec![false; n];
    for p in perm.iter().cloned() {
        if p < n {
            visited[p] = true;
        }
        else {
            return Err(Error::new(ErrorKind::InvalidPermutation,
                "Supplied permutation array contains elements out of bounds."));
        }
    }
    let all_unique = visited.iter().all(|x| x.clone());
    if all_unique {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::InvalidPermutation,
            "Supplied permutation array contains duplicate elements."))
    }
}

/// Applies the permutation by swapping elements in an abstract
/// container.
///
/// The permutation is applied by calls to `swap(i, j)` for indices
/// `i` and `j`.
///
/// # Complexity
///
/// - O(1) memory usage.
/// - O(n) worst case number of calls to `swap`.
fn permute_by_swap<S>(perm: &mut [usize], mut swap: S) where S: FnMut(usize, usize) -> () {
    // Please see https://en.wikipedia.org/wiki/Cyclic_permutation
    // for some explanation to the terminology used here.
    // Some useful resources I found on the internet:
    //
    // https://blog.merovius.de/2014/08/12/applying-permutation-in-constant.html
    // http://stackoverflow.com/questions/16501424/algorithm-to-apply-permutation-in-constant-memory-space
    //
    // A fundamental property of permutations on finite sets is that
    // any such permutation can be decomposed into a collection of
    // cycles on disjoint orbits.
    //
    // An observation is thus that given a permutation P,
    // we can trace out the cycle that includes index i
    // by starting at i and moving to P[i] recursively.
    for i in 0 .. perm.len() {
        let mut target = perm[i];
        while i != target {
            // When resolving a cycle, we resolve each index in the cycle
            // by repeatedly moving the current item into the target position,
            // and item in the target position into the current position.
            // By repeating this until we hit the start index,
            // we effectively resolve the entire cycle.
            let new_target = perm[target];
            swap(i, target);
            perm[target] = target;
            target = new_target;
        }
        perm[i] = i;
    }
}

#[cfg(test)]
mod tests {
    use matrix::Matrix;
    use vector::Vector;
    use super::{PermutationMatrix, Parity};
    use super::{permute_by_swap, validate_permutation};

    use itertools::Itertools;

    #[test]
    fn swap_rows() {
        let mut p = PermutationMatrix::<u64>::identity(4);
        p.swap_rows(0, 3);
        p.swap_rows(1, 3);

        let expected_permutation = PermutationMatrix::from_array(vec![3, 0, 2, 1]).unwrap();
        assert_eq!(p, expected_permutation);
    }

    #[test]
    fn as_matrix() {
        let p = PermutationMatrix::from_array(vec![2, 1, 0, 3]).unwrap();
        let expected_matrix: Matrix<u32> = matrix![0, 0, 1, 0;
                                                   0, 1, 0, 0;
                                                   1, 0, 0, 0;
                                                   0, 0, 0, 1];

        assert_matrix_eq!(expected_matrix, p.as_matrix());
    }

    #[test]
    fn from_array() {
        let array = vec![1, 0, 3, 2];
        let p = PermutationMatrix::<u32>::from_array(array.clone()).unwrap();
        let p_as_array: Vec<usize> = p.into();
        assert_eq!(p_as_array, array);
    }

    #[test]
    fn from_array_unchecked() {
        let array = vec![1, 0, 3, 2];
        let p = unsafe { PermutationMatrix::<u32>::from_array_unchecked(array.clone()) };
        let p_as_array: Vec<usize> = p.into();
        assert_eq!(p_as_array, array);
    }

    #[test]
    fn from_array_invalid() {
        assert!(PermutationMatrix::<u32>::from_array(vec![0, 1, 3]).is_err());
        assert!(PermutationMatrix::<u32>::from_array(vec![0, 0]).is_err());
        assert!(PermutationMatrix::<u32>::from_array(vec![3, 0, 1]).is_err());
    }

    #[test]
    fn vec_from_permutation() {
        let source_vec = vec![0, 2, 1];
        let p = PermutationMatrix::<u32>::from_array(source_vec.clone()).unwrap();
        let vec = Vec::from(p);
        assert_eq!(&source_vec, &vec);
    }

    #[test]
    fn into_slice_ref() {
        let source_vec = vec![0, 2, 1];
        let ref p = PermutationMatrix::<u32>::from_array(source_vec.clone()).unwrap();
        let p_as_slice_ref: &[usize] = p.into();
        assert_eq!(source_vec.as_slice(), p_as_slice_ref);
    }

    #[test]
    fn map_row() {
        let p = PermutationMatrix::<u32>::from_array(vec![0, 2, 1]).unwrap();
        assert_eq!(p.map_row(0), 0);
        assert_eq!(p.map_row(1), 2);
        assert_eq!(p.map_row(2), 1);
    }

    #[test]
    fn inverse() {
        let p = PermutationMatrix::<u32>::from_array(vec![1, 2, 0]).unwrap();
        let expected_inverse = PermutationMatrix::<u32>::from_array(vec![2, 0, 1]).unwrap();
        assert_eq!(p.inverse(), expected_inverse);
    }

    #[test]
    fn parity() {
        {
            let p = PermutationMatrix::<u32>::from_array(vec![1, 0, 3, 2]).unwrap();
            assert_eq!(p.parity(), Parity::Even);
        }

        {
            let p = PermutationMatrix::<u32>::from_array(vec![4, 2, 3, 1, 0, 5]).unwrap();
            assert_eq!(p.parity(), Parity::Odd);
        }
    }

    #[test]
    fn det() {
        {
            let p = PermutationMatrix::<i32>::from_array(vec![1, 0, 3, 2]).unwrap();
            assert_eq!(p.det(), 1);
        }

        {
            let p = PermutationMatrix::<i32>::from_array(vec![4, 2, 3, 1, 0, 5]).unwrap();
            assert_eq!(p.det(), -1);
        }
    }

    #[test]
    fn permute_by_swap_on_empty_array() {
        let mut x = Vec::<char>::new();
        let mut permutation_array = Vec::new();
        permute_by_swap(&mut permutation_array, |i, j| x.swap(i, j));
    }

    #[test]
    fn permute_by_swap_on_arbitrary_array() {
        let mut x = vec!['a', 'b', 'c', 'd'];
        let mut permutation_array = vec![0, 2, 3, 1];

        permute_by_swap(&mut permutation_array, |i, j| x.swap(i, j));
        assert_eq!(x, vec!['a', 'd', 'b', 'c']);
    }

    #[test]
    fn permute_by_swap_identity_on_arbitrary_array() {
        let mut x = vec!['a', 'b', 'c', 'd'];
        let mut permutation_array = vec![0, 1, 2, 3];
        permute_by_swap(&mut permutation_array, |i, j| x.swap(i, j));
        assert_eq!(x, vec!['a', 'b', 'c', 'd']);
    }

    #[test]
    fn compose_into_buffer() {
        let p = PermutationMatrix::<u32>::from_array(vec![2, 1, 0]).unwrap();
        let q = PermutationMatrix::<u32>::from_array(vec![1, 0, 2]).unwrap();
        let pq_expected = PermutationMatrix::<u32>::from_array(vec![1, 2, 0]).unwrap();
        let qp_expected = PermutationMatrix::<u32>::from_array(vec![2, 0, 1]).unwrap();

        {
            let mut pq = PermutationMatrix::identity(3);
            p.compose_into_buffer(&q, &mut pq);
            assert_eq!(pq, pq_expected);
        }

        {
            let mut qp = PermutationMatrix::identity(3);
            q.compose_into_buffer(&p, &mut qp);
            assert_eq!(qp, qp_expected);
        }
    }

    #[test]
    fn compose_regression() {
        // At some point during development, this example failed to
        // give the expected results
        let p = PermutationMatrix::<u32>::from_array(vec![1, 2, 0]).unwrap();
        let q = PermutationMatrix::<u32>::from_array(vec![2, 0, 1]).unwrap();
        let pq_expected = PermutationMatrix::<u32>::from_array(vec![0, 1, 2]).unwrap();

        let mut pq = PermutationMatrix::identity(3);
        p.compose_into_buffer(&q, &mut pq);
        assert_eq!(pq, pq_expected);
    }

    #[test]
    fn permute_rows_into_buffer() {
        let x = matrix![ 0;
                         1;
                         2;
                         3];
        let p = PermutationMatrix::from_array(vec![2, 1, 3, 0]).unwrap();
        let mut output = Matrix::zeros(4, 1);
        p.permute_rows_into_buffer(&x, &mut output);
        assert_matrix_eq!(output, matrix![ 3; 1; 0; 2]);
    }

    #[test]
    fn permute_rows_in_place() {
        let mut x = matrix![ 0;
                         1;
                         2;
                         3];
        let p = PermutationMatrix::from_array(vec![2, 1, 3, 0]).unwrap();
        p.permute_rows_in_place(&mut x);
        assert_matrix_eq!(x, matrix![ 3; 1; 0; 2]);
    }

    #[test]
    fn permute_cols_into_buffer() {
        let x = matrix![ 0, 1, 2, 3];
        let p = PermutationMatrix::from_array(vec![2, 1, 3, 0]).unwrap();
        let mut output = Matrix::zeros(1, 4);
        p.permute_cols_into_buffer(&x, &mut output);
        assert_matrix_eq!(output, matrix![ 3, 1, 0, 2]);
    }

    #[test]
    fn permute_cols_in_place() {
        let mut x = matrix![ 0, 1, 2, 3];
        let p = PermutationMatrix::from_array(vec![2, 1, 3, 0]).unwrap();
        p.permute_cols_in_place(&mut x);
        assert_matrix_eq!(x, matrix![ 3, 1, 0, 2]);
    }

    #[test]
    fn permute_vector_into_buffer() {
        let x = vector![ 0, 1, 2, 3];
        let p = PermutationMatrix::from_array(vec![2, 1, 3, 0]).unwrap();
        let mut output = Vector::zeros(4);
        p.permute_vector_into_buffer(&x, &mut output);
        assert_vector_eq!(output, vector![ 3, 1, 0, 2]);
    }

    #[test]
    fn permute_vector_in_place() {
        let mut x = vector![ 0, 1, 2, 3];
        let p = PermutationMatrix::from_array(vec![2, 1, 3, 0]).unwrap();
        p.permute_vector_in_place(&mut x);
        assert_vector_eq!(x, vector![ 3, 1, 0, 2]);
    }

    use quickcheck::{Arbitrary, Gen};

    // In order to write property tests for the validation of a permutation,
    // we need to be able to generate arbitrary (valid) permutations.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ValidPermutationArray(pub Vec<usize>);

    impl Arbitrary for ValidPermutationArray {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let upper_bound = g.size();
            let mut array = (0 .. upper_bound).collect::<Vec<usize>>();
            g.shuffle(&mut array);
            ValidPermutationArray(array)
        }
    }

    // We also want to be able to generate invalid permutations for
    // the same reasons
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct InvalidPermutationArray(pub Vec<usize>);

    impl Arbitrary for InvalidPermutationArray {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            // Take an arbitrary valid permutation and mutate it so that
            // it is invalid
            let mut permutation_array = ValidPermutationArray::arbitrary(g).0;
            let n = permutation_array.len();

            // There are two essential sources of invalidity:
            // 1. Duplicate elements
            // 2. Indices out of bounds
            // We want to have either or both

            let should_have_duplicates = g.gen::<bool>();
            let should_have_out_of_bounds = !should_have_duplicates || g.gen::<bool>();
            assert!(should_have_duplicates || should_have_out_of_bounds);

            if should_have_out_of_bounds {
                let num_out_of_bounds_rounds = g.gen_range::<usize>(1, n);
                for _ in 0 .. num_out_of_bounds_rounds {
                    let interior_index = g.gen_range::<usize>(0, n);
                    let exterior_index = n + g.gen::<usize>();
                    permutation_array[interior_index] = exterior_index;
                }
            }

            if should_have_duplicates {
                let num_duplicates = g.gen_range::<usize>(1, n);
                for _ in 0 .. num_duplicates {
                    let interior_index = g.gen_range::<usize>(0, n);
                    let duplicate_value = permutation_array[interior_index];
                    permutation_array.push(duplicate_value);
                }
            }

            // The duplicates are placed at the end, so we perform
            // an additional shuffle to end up with a more or less
            // arbitrary invalid permutation
            g.shuffle(&mut permutation_array);
            InvalidPermutationArray(permutation_array)
        }
    }

    impl<T: Send + Clone + 'static> Arbitrary for PermutationMatrix<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let ValidPermutationArray(array) = ValidPermutationArray::arbitrary(g);
            PermutationMatrix::from_array(array)
                .expect("The generated permutation array should always be valid.")
        }
    }

    quickcheck! {
        fn property_validate_permutation_is_ok_for_valid_input(array: ValidPermutationArray) -> bool {
            validate_permutation(&array.0).is_ok()
        }
    }

    quickcheck! {
        fn property_validate_permutation_is_err_for_invalid_input(array: InvalidPermutationArray) -> bool {
            validate_permutation(&array.0).is_err()
        }
    }

    quickcheck! {
        fn property_identity_has_identity_array(size: usize) -> bool {
            let p = PermutationMatrix::<u64>::identity(size);
            let p_as_array: Vec<usize> = p.into();
            let expected = (0 .. size).collect::<Vec<usize>>();
            p_as_array == expected
        }
    }

    quickcheck! {
        fn property_dim_is_equal_to_array_dimensions(array: ValidPermutationArray) -> bool {
            let ValidPermutationArray(array) = array;
            let n = array.len();
            let p = PermutationMatrix::<u32>::from_array(array).unwrap();
            p.size() == n
        }
    }

    quickcheck! {
        fn property_inverse_of_inverse_is_original(p: PermutationMatrix<u32>) -> bool {
            p == p.inverse().inverse()
        }
    }

    quickcheck! {
        fn property_inverse_composes_to_identity(p: PermutationMatrix<u32>) -> bool {
            // Recall that P * P_inv = I and P_inv * P = I
            let n = p.size();
            let pinv = p.inverse();
            let mut p_pinv_composition = PermutationMatrix::identity(n);
            let mut pinv_p_composition = PermutationMatrix::identity(n);
            p.compose_into_buffer(&pinv, &mut p_pinv_composition);
            pinv.compose_into_buffer(&p, &mut pinv_p_composition);
            assert_eq!(p_pinv_composition, PermutationMatrix::identity(n));
            assert_eq!(pinv_p_composition, PermutationMatrix::identity(n));
            true
        }
    }

    quickcheck! {
        fn property_identity_parity_is_even(n: usize) -> bool {
            let p = PermutationMatrix::<u32>::identity(n);
            p.parity() ==  Parity::Even
        }
    }

    quickcheck! {
        fn property_parity_agrees_with_parity_of_inversions(p: PermutationMatrix<u32>) -> bool {
            let array: &[usize] = (&p).into();
            let num_inversions = array.iter().cloned().enumerate()
                                      .cartesian_product(array.iter().cloned().enumerate())
                                      .filter(|&((i, permuted_i), (j, permuted_j))|
                                        // This is simply the definition of an inversion
                                        i < j && permuted_i > permuted_j
                                      )
                                      .count();
            // Recall that the parity of the number of inversions in the
            // permutation is equal to the parity of the permutation
            let parity = if num_inversions % 2 == 0 {
                Parity::Even
            } else {
                Parity::Odd
            };

            parity == p.clone().parity()
        }
    }
}
