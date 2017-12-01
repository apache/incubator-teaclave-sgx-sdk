use super::{BaseMatrix, BaseMatrixMut};
use matrix::{Matrix, MatrixSlice, MatrixSliceMut};
use matrix::{Row, RowMut, Column, ColumnMut};

use utils;
use libnum::Zero;

use std::ops::{Add, Mul, Div};

impl<T> BaseMatrix<T> for Matrix<T> {
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn row_stride(&self) -> usize {
        self.cols
    }
    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    fn as_ptr(&self) -> *const T {
        self.data.as_ptr()
    }

    fn into_matrix(self) -> Matrix<T>
        where T: Copy
    {
        self // for Matrix, this is a no-op
    }

    fn sum(&self) -> T
        where T: Copy + Zero + Add<T, Output = T>
    {
        utils::unrolled_sum(&self.data[..])
    }

    fn elemul(&self, m: &Self) -> Matrix<T>
        where T: Copy + Mul<T, Output = T>
    {
        assert!(self.rows() == m.rows(), "Matrix row counts not equal.");
        assert!(self.cols() == m.cols(), "Matrix column counts not equal.");

        let data = utils::vec_bin_op(self.data(), m.data(), T::mul);
        Matrix::new(self.rows(), self.cols(), data)
    }

    fn elediv(&self, m: &Self) -> Matrix<T>
        where T: Copy + Div<T, Output = T>
    {
        assert!(self.rows() == m.rows(), "Matrix row counts not equal.");
        assert!(self.cols() == m.cols(), "Matrix column counts not equal.");

        let data = utils::vec_bin_op(self.data(), m.data(), T::div);
        Matrix::new(self.rows(), self.cols(), data)
    }

    fn vcat<S>(&self, m: &S) -> Matrix<T>
        where T: Copy,
              S: BaseMatrix<T>
    {
        assert!(self.cols() == m.cols(),
                "Matrix column counts are not equal.");

        let mut new_data = self.data.clone();
        new_data.reserve(m.rows() * m.cols());

        for row in m.row_iter() {
            new_data.extend_from_slice(row.raw_slice());
        }

        Matrix {
            cols: self.cols(),
            rows: (self.rows() + m.rows()),
            data: new_data,
        }
    }
}

impl<'a, T> BaseMatrix<T> for MatrixSlice<'a, T> {
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn row_stride(&self) -> usize {
        self.row_stride
    }
    fn as_ptr(&self) -> *const T {
        self.ptr
    }
}

impl<'a, T> BaseMatrix<T> for MatrixSliceMut<'a, T> {
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn row_stride(&self) -> usize {
        self.row_stride
    }
    fn as_ptr(&self) -> *const T {
        self.ptr as *const T
    }
}

impl<T> BaseMatrixMut<T> for Matrix<T> {
    /// Top left index of the slice.
    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr()
    }
}

impl<'a, T> BaseMatrixMut<T> for MatrixSliceMut<'a, T> {
    /// Top left index of the slice.
    fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

impl<'a, T> BaseMatrix<T> for Row<'a, T> {
    fn rows(&self) -> usize {
        1
    }
    fn cols(&self) -> usize {
        self.row.cols()
    }
    fn row_stride(&self) -> usize {
        self.row.row_stride()
    }

    fn as_ptr(&self) -> *const T {
        self.row.as_ptr()
    }
}

impl<'a, T> BaseMatrix<T> for RowMut<'a, T> {
    fn rows(&self) -> usize {
        1
    }
    fn cols(&self) -> usize {
        self.row.cols()
    }
    fn row_stride(&self) -> usize {
        self.row.row_stride()
    }

    fn as_ptr(&self) -> *const T {
        self.row.as_ptr()
    }
}

impl<'a, T> BaseMatrixMut<T> for RowMut<'a, T> {
    /// Top left index of the slice.
    fn as_mut_ptr(&mut self) -> *mut T {
        self.row.as_mut_ptr()
    }
}

impl<'a, T> BaseMatrix<T> for Column<'a, T> {
    fn rows(&self) -> usize {
        self.col.rows()
    }
    fn cols(&self) -> usize {
        1
    }
    fn row_stride(&self) -> usize {
        self.col.row_stride()
    }

    fn as_ptr(&self) -> *const T {
        self.col.as_ptr()
    }
}

impl<'a, T> BaseMatrix<T> for ColumnMut<'a, T> {
    fn rows(&self) -> usize {
        self.col.rows()
    }
    fn cols(&self) -> usize {
        1
    }
    fn row_stride(&self) -> usize {
        self.col.row_stride()
    }

    fn as_ptr(&self) -> *const T {
        self.col.as_ptr()
    }
}

impl<'a, T> BaseMatrixMut<T> for ColumnMut<'a, T> {
    /// Top left index of the slice.
    fn as_mut_ptr(&mut self) -> *mut T {
        self.col.as_mut_ptr()
    }
}

#[cfg(test)]
mod tests {
    use matrix::{Matrix, MatrixSlice, MatrixSliceMut};
    use matrix::{BaseMatrix, BaseMatrixMut};
    use matrix::{Axes, DiagOffset};

    #[test]
    fn test_sub_slice() {
        let mut a = Matrix::new(4, 4, (0..16).collect::<Vec<_>>());
        {
            let slice = a.sub_slice([1, 1], 3, 2);
            assert_eq!(&slice.iter().cloned().collect::<Vec<_>>(), &vec![5, 6, 9, 10, 13, 14]);

            let slice = slice.sub_slice([1, 1], 2, 1);
            assert_eq!(&slice.iter().cloned().collect::<Vec<_>>(), &vec![10, 14]);
        }
        {
            let mut slice_mut = a.sub_slice_mut([3, 1], 1, 1);
            unsafe {
                *slice_mut.get_unchecked_mut([0, 0]) = 25;
                assert_eq!(*a.get_unchecked([3, 1]), 25);
            }
        }
    }

    #[test]
    fn slice_into_matrix() {
        let mut a = Matrix::ones(3, 3) * 2.0;

        {
            let b = MatrixSlice::from_matrix(&a, [1, 1], 2, 2);
            let c = b.into_matrix();
            assert_eq!(c.rows(), 2);
            assert_eq!(c.cols(), 2);
        }

        let d = MatrixSliceMut::from_matrix(&mut a, [1, 1], 2, 2);
        let e = d.into_matrix();
        assert_eq!(e.rows(), 2);
        assert_eq!(e.cols(), 2);
    }

    #[test]
    fn test_split_matrix() {
        let a = Matrix::new(3, 3, (0..9).collect::<Vec<_>>());

        let (b, c) = a.split_at(1, Axes::Row);

        assert_eq!(b.rows(), 1);
        assert_eq!(b.cols(), 3);
        assert_eq!(c.rows(), 2);
        assert_eq!(c.cols(), 3);

        assert_eq!(b[[0, 0]], 0);
        assert_eq!(b[[0, 1]], 1);
        assert_eq!(b[[0, 2]], 2);
        assert_eq!(c[[0, 0]], 3);
        assert_eq!(c[[0, 1]], 4);
        assert_eq!(c[[0, 2]], 5);
        assert_eq!(c[[1, 0]], 6);
        assert_eq!(c[[1, 1]], 7);
        assert_eq!(c[[1, 2]], 8);
    }

    #[test]
    fn test_split_matrix_mut() {
        let mut a = Matrix::new(3, 3, (0..9).collect::<Vec<_>>());

        {
            let (mut b, mut c) = a.split_at_mut(1, Axes::Row);

            assert_eq!(b.rows(), 1);
            assert_eq!(b.cols(), 3);
            assert_eq!(c.rows(), 2);
            assert_eq!(c.cols(), 3);

            assert_eq!(b[[0, 0]], 0);
            assert_eq!(b[[0, 1]], 1);
            assert_eq!(b[[0, 2]], 2);
            assert_eq!(c[[0, 0]], 3);
            assert_eq!(c[[0, 1]], 4);
            assert_eq!(c[[0, 2]], 5);
            assert_eq!(c[[1, 0]], 6);
            assert_eq!(c[[1, 1]], 7);
            assert_eq!(c[[1, 2]], 8);

            b[[0, 0]] = 4;
            c[[0, 0]] = 5;
        }

        assert_eq!(a[[0, 0]], 4);
        assert_eq!(a[[0, 1]], 1);
        assert_eq!(a[[0, 2]], 2);
        assert_eq!(a[[1, 0]], 5);
        assert_eq!(a[[1, 1]], 4);
        assert_eq!(a[[1, 2]], 5);
        assert_eq!(a[[2, 0]], 6);
        assert_eq!(a[[2, 1]], 7);
        assert_eq!(a[[2, 2]], 8);
    }

    #[test]
    #[should_panic]
    fn test_diag_iter_too_high() {
        let a = matrix![0.0, 1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0, 7.0;
                        8.0, 9.0, 10.0, 11.0];

        for _ in a.diag_iter(DiagOffset::Above(4)) {

        }
    }

    #[test]
    #[should_panic]
    fn test_diag_iter_too_low() {
        let a = matrix![0.0, 1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0, 7.0;
                        8.0, 9.0, 10.0, 11.0];

        for _ in a.diag_iter(DiagOffset::Below(3)) {

        }
    }

    #[test]
    fn test_swap_rows() {
        let mut a = Matrix::new(4, 3, (0..12).collect::<Vec<usize>>());
        a.swap_rows(0, 1);

        assert_eq!(a.data(), &[3, 4, 5, 0, 1, 2, 6, 7, 8, 9, 10, 11]);

        {
            let mut b = MatrixSliceMut::from_matrix(&mut a, [0, 0], 4, 2);
            b.swap_rows(0, 1);
        }

        assert_eq!(a.into_vec(), vec![0, 1, 5, 3, 4, 2, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn test_matrix_swap_cols() {
        let mut a = Matrix::new(4, 3, (0..12).collect::<Vec<usize>>());
        a.swap_cols(0, 1);

        assert_eq!(a.data(), &[1, 0, 2, 4, 3, 5, 7, 6, 8, 10, 9, 11]);

        {
            let mut b = MatrixSliceMut::from_matrix(&mut a, [0, 0], 3, 3);
            b.swap_cols(0, 2);
        }

        assert_eq!(a.into_vec(), vec![2, 0, 1, 5, 3, 4, 8, 6, 7, 10, 9, 11]);
    }

    #[test]
    fn test_matrix_swap_same_rows() {
        let mut a = Matrix::new(4, 2, (0..8).collect::<Vec<usize>>());
        a.swap_rows(0, 0);

        assert_eq!(a.into_vec(), (0..8).collect::<Vec<usize>>());
    }

    #[test]
    fn test_matrix_swap_same_cols() {
        let mut a = Matrix::new(4, 2, (0..8).collect::<Vec<usize>>());
        a.swap_cols(0, 0);

        assert_eq!(a.into_vec(), (0..8).collect::<Vec<usize>>());
    }

    #[test]
    #[should_panic]
    fn test_matrix_swap_row_high_first() {
        let mut a = Matrix::new(4, 2, (0..8).collect::<Vec<usize>>());
        a.swap_rows(5, 0);
    }

    #[test]
    #[should_panic]
    fn test_matrix_swap_row_high_second() {
        let mut a = Matrix::new(4, 2, (0..8).collect::<Vec<usize>>());
        a.swap_rows(0, 5);
    }

    #[test]
    #[should_panic]
    fn test_matrix_swap_col_high_first() {
        let mut a = Matrix::new(4, 2, (0..8).collect::<Vec<usize>>());
        a.swap_cols(2, 1);
    }

    #[test]
    #[should_panic]
    fn test_matrix_swap_col_high_second() {
        let mut a = Matrix::new(4, 2, (0..8).collect::<Vec<usize>>());
        a.swap_cols(1, 2);
    }

    #[test]
    fn test_matrix_select_rows() {
        let a = Matrix::new(4, 2, (0..8).collect::<Vec<usize>>());

        let b = a.select_rows(&[0, 2, 3]);

        assert_eq!(b.into_vec(), vec![0, 1, 4, 5, 6, 7]);
    }

    #[test]
    fn test_matrix_select_cols() {
        let a = Matrix::new(4, 2, (0..8).collect::<Vec<usize>>());

        let b = a.select_cols(&[1]);

        assert_eq!(b.into_vec(), vec![1, 3, 5, 7]);
    }

    #[test]
    fn test_matrix_select() {
        let a = Matrix::new(4, 2, (0..8).collect::<Vec<usize>>());

        let b = a.select(&[0, 2], &[1]);

        assert_eq!(b.into_vec(), vec![1, 5]);
    }

    #[test]
    fn matrix_diag() {
        let a = matrix![1., 3., 5.;
                        2., 4., 7.;
                        1., 1., 0.];

        let b = a.is_diag();

        assert!(!b);

        let c = matrix![1., 0., 0.;
                        0., 2., 0.;
                        0., 0., 3.];
        let d = c.is_diag();

        assert!(d);
    }

    #[test]
    fn transpose_mat() {
        let a = matrix![1., 2.;
                        3., 4.;
                        5., 6.;
                        7., 8.;
                        9., 10.];

        let c = a.transpose();

        assert_eq!(c.cols(), a.rows());
        assert_eq!(c.rows(), a.cols());

        assert_eq!(a[[0, 0]], c[[0, 0]]);
        assert_eq!(a[[1, 0]], c[[0, 1]]);
        assert_eq!(a[[2, 0]], c[[0, 2]]);
        assert_eq!(a[[3, 0]], c[[0, 3]]);
        assert_eq!(a[[4, 0]], c[[0, 4]]);
        assert_eq!(a[[0, 1]], c[[1, 0]]);
        assert_eq!(a[[1, 1]], c[[1, 1]]);
        assert_eq!(a[[2, 1]], c[[1, 2]]);
        assert_eq!(a[[3, 1]], c[[1, 3]]);
        assert_eq!(a[[4, 1]], c[[1, 4]]);
    }
}
