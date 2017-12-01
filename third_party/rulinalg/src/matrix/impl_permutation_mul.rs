use matrix::{PermutationMatrix, Matrix,
             MatrixSlice, MatrixSliceMut,
             BaseMatrix};
use vector::Vector;

use libnum::Zero;
use std::ops::Mul;

/// Left-multiply a vector by a permutation matrix.
impl<T> Mul<Vector<T>> for PermutationMatrix<T> {
    type Output = Vector<T>;

    fn mul(self, mut rhs: Vector<T>) -> Vector<T> {
        self.permute_vector_in_place(&mut rhs);
        rhs
    }
}

/// Left-multiply a vector by a permutation matrix.
impl<'a, T> Mul<Vector<T>> for &'a PermutationMatrix<T> where T: Clone + Zero {
    type Output = Vector<T>;

    fn mul(self, rhs: Vector<T>) -> Vector<T> {
        // Here we have the choice of using `permute_by_copy`
        // `permute_by_swap`, as we can reuse one of the existing
        // implementations.
        self * &rhs
    }
}

/// Left-multiply a vector by a permutation matrix.
impl<'a, 'b, T> Mul<&'a Vector<T>> for &'b PermutationMatrix<T> where T: Clone + Zero {
    type Output = Vector<T>;

    fn mul(self, rhs: &'a Vector<T>) -> Vector<T> {
        let mut target_vec = Vector::zeros(rhs.size());
        self.permute_vector_into_buffer(rhs, &mut target_vec);
        target_vec
    }
}

/// Left-multiply a vector by a permutation matrix.
impl<'a, T> Mul<&'a Vector<T>> for PermutationMatrix<T> where T: Clone + Zero {
    type Output = Vector<T>;

    fn mul(self, rhs: &'a Vector<T>) -> Vector<T> {
        &self * rhs
    }
}

/// Left-multiply a matrix by a permutation matrix.
impl<T> Mul<Matrix<T>> for PermutationMatrix<T> {
    type Output = Matrix<T>;

    fn mul(self, mut rhs: Matrix<T>) -> Matrix<T> {
        self.permute_rows_in_place(&mut rhs);
        rhs
    }
}

/// Left-multiply a matrix by a permutation matrix.
impl<'b, T> Mul<Matrix<T>> for &'b PermutationMatrix<T> where T: Clone {
    type Output = Matrix<T>;

    fn mul(self, mut rhs: Matrix<T>) -> Matrix<T> {
        self.clone().permute_rows_in_place(&mut rhs);
        rhs
    }
}

macro_rules! impl_permutation_matrix_left_multiply_reference_type {
    ($MatrixType:ty) => (

/// Left-multiply a matrix by a permutation matrix.
impl<'a, 'm, T> Mul<&'a $MatrixType> for PermutationMatrix<T> where T: Zero + Clone {
    type Output = Matrix<T>;

    fn mul(self, rhs: &'a $MatrixType) -> Matrix<T> {
        let mut output = Matrix::zeros(rhs.rows(), rhs.cols());
        self.permute_rows_into_buffer(rhs, &mut output);
        output
    }
}

/// Left-multiply a matrix by a permutation matrix.
impl<'a, 'b, 'm, T> Mul<&'a $MatrixType> for &'b PermutationMatrix<T> where T: Zero + Clone {
    type Output = Matrix<T>;

    fn mul(self, rhs: &'a $MatrixType) -> Matrix<T> {
        let mut output = Matrix::zeros(rhs.rows(), rhs.cols());
        self.permute_rows_into_buffer(rhs, &mut output);
        output
    }
}

    )
}

macro_rules! impl_permutation_matrix_slice_left_multiply_value_type {
    ($MatrixType:ty) => (
/// Left-multiply a matrix by a permutation matrix.
impl<'a, 'm, T> Mul<$MatrixType> for PermutationMatrix<T> where T: Zero + Clone {
    type Output = Matrix<T>;

    fn mul(self, rhs: $MatrixType) -> Matrix<T> {
        self * &rhs
    }
}

/// Left-multiply a matrix by a permutation matrix.
impl<'a, 'b, 'm, T> Mul<$MatrixType> for &'b PermutationMatrix<T> where T: Zero + Clone {
    type Output = Matrix<T>;

    fn mul(self, rhs: $MatrixType) -> Matrix<T> {
        self * &rhs
    }
}
    )
}

impl_permutation_matrix_left_multiply_reference_type!(Matrix<T>);
impl_permutation_matrix_left_multiply_reference_type!(MatrixSlice<'m, T>);
impl_permutation_matrix_left_multiply_reference_type!(MatrixSliceMut<'m, T>);

impl_permutation_matrix_slice_left_multiply_value_type!(MatrixSlice<'m, T>);
impl_permutation_matrix_slice_left_multiply_value_type!(MatrixSliceMut<'m, T>);

/// Right-multiply a matrix by a permutation matrix.
impl<T> Mul<PermutationMatrix<T>> for Matrix<T> {
    type Output = Matrix<T>;

    fn mul(mut self, rhs: PermutationMatrix<T>) -> Matrix<T> {
        rhs.permute_cols_in_place(&mut self);
        self
    }
}

/// Right-multiply a matrix by a permutation matrix.
impl<'a, T> Mul<&'a PermutationMatrix<T>> for Matrix<T> where T: Clone {
    type Output = Matrix<T>;

    fn mul(mut self, rhs: &'a PermutationMatrix<T>) -> Matrix<T> {
        rhs.clone().permute_cols_in_place(&mut self);
        self
    }
}

macro_rules! impl_permutation_matrix_right_multiply_reference_type {
    ($MatrixType:ty) => (

/// Right-multiply a matrix by a permutation matrix.
impl<'a, 'm, T> Mul<PermutationMatrix<T>> for &'a $MatrixType where T: Zero + Clone {
    type Output = Matrix<T>;

    fn mul(self, rhs: PermutationMatrix<T>) -> Matrix<T> {
        let mut output = Matrix::zeros(self.rows(), self.cols());
        rhs.permute_cols_into_buffer(self, &mut output);
        output
    }
}

/// Right-multiply a matrix by a permutation matrix.
impl<'a, 'b, 'm, T> Mul<&'b PermutationMatrix<T>> for &'a $MatrixType where T: Zero + Clone {
    type Output = Matrix<T>;

    fn mul(self, rhs: &'b PermutationMatrix<T>) -> Matrix<T> {
        let mut output = Matrix::zeros(self.rows(), self.cols());
        rhs.permute_cols_into_buffer(self, &mut output);
        output
    }
}

    )
}

macro_rules! impl_permutation_matrix_slice_right_multiply_value_type {
    ($MatrixType:ty) => (
/// Right-multiply a matrix by a permutation matrix.
impl<'a, 'm, T> Mul<PermutationMatrix<T>> for $MatrixType where T: Zero + Clone {
    type Output = Matrix<T>;

    fn mul(self, rhs: PermutationMatrix<T>) -> Matrix<T> {
        &self * rhs
    }
}

/// Right-multiply a matrix by a permutation matrix.
impl<'a, 'b, 'm, T> Mul<&'b PermutationMatrix<T>> for $MatrixType where T: Zero + Clone {
    type Output = Matrix<T>;

    fn mul(self, rhs: &'b PermutationMatrix<T>) -> Matrix<T> {
        &self * rhs
    }
}
    )
}

impl_permutation_matrix_right_multiply_reference_type!(Matrix<T>);
impl_permutation_matrix_right_multiply_reference_type!(MatrixSlice<'m, T>);
impl_permutation_matrix_right_multiply_reference_type!(MatrixSliceMut<'m, T>);

impl_permutation_matrix_slice_right_multiply_value_type!(MatrixSlice<'m, T>);
impl_permutation_matrix_slice_right_multiply_value_type!(MatrixSliceMut<'m, T>);

/// Multiply a permutation matrix by a permutation matrix.
impl<T: Clone> Mul<PermutationMatrix<T>> for PermutationMatrix<T> {
    type Output = PermutationMatrix<T>;

    fn mul(self, rhs: PermutationMatrix<T>) -> PermutationMatrix<T> {
        let mut output = PermutationMatrix::identity(rhs.size());
        self.compose_into_buffer(&rhs, &mut output);
        output
    }
}

/// Multiply a permutation matrix by a permutation matrix.
impl<'a, T: Clone> Mul<&'a PermutationMatrix<T>> for PermutationMatrix<T> {
    type Output = PermutationMatrix<T>;

    fn mul(self, rhs: &PermutationMatrix<T>) -> PermutationMatrix<T> {
        let mut output = PermutationMatrix::identity(rhs.size());
        self.compose_into_buffer(rhs, &mut output);
        output
    }
}

/// Multiply a permutation matrix by a permutation matrix.
impl<'a, T: Clone> Mul<PermutationMatrix<T>> for &'a PermutationMatrix<T> {
    type Output = PermutationMatrix<T>;

    fn mul(self, rhs: PermutationMatrix<T>) -> PermutationMatrix<T> {
        let mut output = PermutationMatrix::identity(rhs.size());
        self.compose_into_buffer(&rhs, &mut output);
        output
    }
}

/// Multiply a permutation matrix by a permutation matrix.
impl<'a, 'b, T: Clone> Mul<&'a PermutationMatrix<T>> for &'b PermutationMatrix<T> {
    type Output = PermutationMatrix<T>;

    fn mul(self, rhs: &'a PermutationMatrix<T>) -> PermutationMatrix<T> {
        let mut output = PermutationMatrix::identity(rhs.size());
        self.compose_into_buffer(rhs, &mut output);
        output
    }
}

#[cfg(test)]
mod tests {
    use matrix::{BaseMatrix, BaseMatrixMut};
    use matrix::PermutationMatrix;

    #[test]
    fn permutation_vector_mul() {
        let p = PermutationMatrix::from_array(vec![1, 2, 0]).unwrap();
        let x = vector![1, 2, 3];
        let expected = vector![3, 1, 2];

        {
            let y = p.clone() * x.clone();
            assert_eq!(y, expected);
        }

        {
            let y = p.clone() * &x;
            assert_eq!(y, expected);
        }

        {
            let y = &p * x.clone();
            assert_eq!(y, expected);
        }

        {
            let y = &p * &x;
            assert_eq!(y, expected);
        }
    }

    #[test]
    fn permutation_matrix_left_mul_for_matrix() {
        let p = PermutationMatrix::from_array(vec![1, 2, 0]).unwrap();
        let x = matrix![1, 2, 3;
                        4, 5, 6;
                        7, 8, 9];
        let expected = matrix![7, 8, 9;
                               1, 2, 3;
                               4, 5, 6];

        {
            // Consume p, consume rhs
            let y = p.clone() * x.clone();
            assert_eq!(y, expected);
        }

        {
            // Consume p, borrow rhs
            let y = p.clone() * &x;
            assert_eq!(y, expected);
        }

        {
            // Borrow p, consume rhs
            let y = &p * x.clone();
            assert_eq!(y, expected);
        }

        {
            // Borrow p, borrow rhs
            let y = &p * &x;
            assert_eq!(y, expected);
        }
    }

    #[test]
    fn permutation_matrix_left_mul_for_matrix_slice() {
        let p = PermutationMatrix::from_array(vec![1, 2, 0]).unwrap();
        let x_source = matrix![1, 2, 3;
                               4, 5, 6;
                               7, 8, 9];
        let expected = matrix![7, 8, 9;
                               1, 2, 3;
                               4, 5, 6];

        {
            // Consume immutable, consume p
            let x = x_source.sub_slice([0, 0], 3, 3);
            let y = p.clone() * x;
            assert_eq!(y, expected);
        }

        {
            // Borrow immutable, consume p
            let x = x_source.sub_slice([0, 0], 3, 3);
            let y = p.clone() * &x;
            assert_eq!(y, expected);
        }

        {
            // Consume immutable, borrow p
            let x = x_source.sub_slice([0, 0], 3, 3);
            let y = &p * x;
            assert_eq!(y, expected);
        }

        {
            // Borrow immutable, borrow p
            let x = x_source.sub_slice([0, 0], 3, 3);
            let y = &p * &x;
            assert_eq!(y, expected);
        }

        {
            // Consume mutable, consume p
            let mut x_source = x_source.clone();
            let x = x_source.sub_slice_mut([0, 0], 3, 3);
            let y = p.clone() * x;
            assert_eq!(y, expected);
        }

        {
            // Borrow mutable, consume p
            let mut x_source = x_source.clone();
            let x = x_source.sub_slice_mut([0, 0], 3, 3);
            let y = p.clone() * &x;
            assert_eq!(y, expected);
        }

        {
            // Consume mutable, borrow p
            let mut x_source = x_source.clone();
            let x = x_source.sub_slice_mut([0, 0], 3, 3);
            let y = &p * x;
            assert_eq!(y, expected);
        }

        {
            // Borrow mutable, borrow p
            let mut x_source = x_source.clone();
            let x = x_source.sub_slice_mut([0, 0], 3, 3);
            let y = &p * &x;
            assert_eq!(y, expected);
        }
    }

    #[test]
    fn permutation_matrix_right_mul_for_matrix() {
        let p = PermutationMatrix::from_array(vec![1, 2, 0]).unwrap();
        let x = matrix![1, 2, 3;
                        4, 5, 6;
                        7, 8, 9];
        let expected = matrix![3, 1, 2;
                               6, 4, 5;
                               9, 7, 8];

        {
            // Consume lhs, consume p
            let y = x.clone() * p.clone();
            assert_eq!(y, expected);
        }

        {
            // Consume lhs, borrow p
            let y = x.clone() * &p;
            assert_eq!(y, expected);
        }

        {
            // Borrow lhs, consume p
            let y = &x * p.clone();
            assert_eq!(y, expected);
        }

        {
            // Borrow lhs, borrow p
            let y = &x * &p;
            assert_eq!(y, expected);
        }
    }

    #[test]
    fn permutation_matrix_right_mul_for_matrix_slice() {
        let p = PermutationMatrix::from_array(vec![1, 2, 0]).unwrap();
        let x_source = matrix![1, 2, 3;
                               4, 5, 6;
                               7, 8, 9];
        let expected = matrix![3, 1, 2;
                               6, 4, 5;
                               9, 7, 8];

        {
            // Consume immutable lhs, consume p
            let x = x_source.sub_slice([0, 0], 3, 3);
            let y = x * p.clone();
            assert_eq!(y, expected);
        }

        {
            // Borrow immutable lhs, consume p
            let x = x_source.sub_slice([0, 0], 3, 3);
            let y = &x * p.clone();
            assert_eq!(y, expected);
        }

        {
            // Consume immutable lhs, consume p
            let x = x_source.sub_slice([0, 0], 3, 3);
            let y = x * &p;
            assert_eq!(y, expected);
        }

        {
            // Borrow immutable lhs, borrow p
            let x = x_source.sub_slice([0, 0], 3, 3);
            let y = &x * &p;
            assert_eq!(y, expected);
        }

        {
            // Consume mutable lhs, consume p
            let mut x_source = x_source.clone();
            let x = x_source.sub_slice_mut([0, 0], 3, 3);
            let y = x * p.clone();
            assert_eq!(y, expected);
        }

        {
            // Borrow mutable lhs, consume p
            let mut x_source = x_source.clone();
            let x = x_source.sub_slice_mut([0, 0], 3, 3);
            let y = &x * p.clone();
            assert_eq!(y, expected);
        }

        {
            // Consume mutable lhs, borrow p
            let mut x_source = x_source.clone();
            let x = x_source.sub_slice_mut([0, 0], 3, 3);
            let y = x * &p;
            assert_eq!(y, expected);
        }

        {
            // Borrow mutable lhs, borrow p
            let mut x_source = x_source.clone();
            let x = x_source.sub_slice_mut([0, 0], 3, 3);
            let y = &x * &p;
            assert_eq!(y, expected);
        }
    }

    #[test]
    fn permutation_matrix_self_multiply() {
        let p1 = PermutationMatrix::<u32>::from_array(vec![2, 0, 1, 3]).unwrap();
        let p2 = PermutationMatrix::<u32>::from_array(vec![0, 3, 2, 1]).unwrap();

        let p1p2 = PermutationMatrix::from_array(vec![2, 3, 1, 0]).unwrap();
        let p2p1 = PermutationMatrix::from_array(vec![2, 0, 3, 1]).unwrap();

        {
            // Consume p1, consume p2
            assert_eq!(p1p2, p1.clone() * p2.clone());
            assert_eq!(p2p1, p2.clone() * p1.clone());
        }

        {
            // Consume p1, borrow p2
            assert_eq!(p1p2, p1.clone() * &p2);
            assert_eq!(p2p1, &p2 * p1.clone());
        }

        {
            // Borrow p1, consume p2
            assert_eq!(p1p2, &p1 * p2.clone());
            assert_eq!(p2p1, p2.clone() * &p1);
        }

        {
            // Borrow p1, borrow p2
            assert_eq!(p1p2, &p1 * &p2);
            assert_eq!(p2p1, &p2 * &p1);
        }
    }
}
