//! Traits for matrices operations.
//!
//! These traits defines operations for structs representing matrices arranged in row-major order.
//!
//! Implementations are provided for
//! - `Matrix`: an owned matrix
//! - `MatrixSlice`: a borrowed immutable block of `Matrix`
//! - `MatrixSliceMut`: a borrowed mutable block of `Matrix`
//!
//! ```
//! use rulinalg::matrix::{Matrix, BaseMatrix};
//!
//! let a = Matrix::new(3,3, (0..9).collect::<Vec<usize>>());
//!
//! // Manually create our slice - [[4,5],[7,8]].
//! let mat_slice = a.sub_slice([0,1], 3, 2);
//!
//! // We can perform arithmetic with mixing owned and borrowed versions
//! let _new_mat = &mat_slice.transpose() * &a;
//! ```
use std::vec::*;
use matrix::{Matrix, MatrixSlice, MatrixSliceMut};
use matrix::{Cols, ColsMut, Row, RowMut, Column, ColumnMut, Rows, RowsMut, Axes};
use matrix::{DiagOffset, Diagonal, DiagonalMut};
use matrix::{back_substitution, forward_substitution};
use matrix::{SliceIter, SliceIterMut};
use norm::{MatrixNorm, MatrixMetric};
use vector::Vector;
use utils;
use libnum::{Zero, Float};
use error::Error;

use std::any::Any;
use std::cmp::min;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Add, Mul, Div};
use std::ptr;
use std::slice;

mod impl_base;

/// Trait for immutable matrix structs.
pub trait BaseMatrix<T>: Sized {
    /// Rows in the matrix.
    fn rows(&self) -> usize;

    /// Columns in the matrix.
    fn cols(&self) -> usize;

    /// Row stride in the matrix.
    fn row_stride(&self) -> usize;

    /// Returns true if the matrix contais no elements
    fn is_empty(&self) -> bool {
        self.rows() == 0 || self.cols() == 0
    }

    /// Top left index of the matrix.
    fn as_ptr(&self) -> *const T;

    /// Returns a `MatrixSlice` over the whole matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = Matrix::new(3, 3, vec![2.0; 9]);
    /// let b = a.as_slice();
    /// ```
    fn as_slice(&self) -> MatrixSlice<T> {
        unsafe {
            MatrixSlice::from_raw_parts(self.as_ptr(), self.rows(), self.cols(), self.row_stride())
        }
    }

    /// Get a reference to an element in the matrix without bounds checking.
    unsafe fn get_unchecked(&self, index: [usize; 2]) -> &T {
        &*(self.as_ptr().offset((index[0] * self.row_stride() + index[1]) as isize))
    }

    /// Get a reference to an element in the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let mat = matrix![0, 1;
    ///                   3, 4;
    ///                   6, 7];
    ///
    /// assert_eq!(mat.get([0, 2]), None);
    /// assert_eq!(mat.get([3, 0]), None);
    ///
    /// assert_eq!( *mat.get([0, 0]).unwrap(), 0)
    /// # }
    /// ```
    fn get(&self, index: [usize; 2]) -> Option<&T> {
        let row_ind = index[0];
        let col_ind = index[1];

        if row_ind >= self.rows() || col_ind >= self.cols() {
          None
        } else {
          unsafe { Some(self.get_unchecked(index)) }
        }
    }

    /// Returns the column of a matrix at the given index.
    /// `None` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let mat = matrix![0, 1, 2;
    ///                   3, 4, 5;
    ///                   6, 7, 8];
    /// let col = mat.col(1);
    /// let expected = matrix![1usize; 4; 7];
    /// assert_matrix_eq!(*col, expected);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Will panic if the column index is out of bounds.
    fn col(&self, index: usize) -> Column<T> {
        if index < self.cols() {
            unsafe { self.col_unchecked(index) }
        } else {
            panic!("Column index out of bounds.")
        }
    }

    /// Returns the column of a matrix at the given
    /// index without doing a bounds check.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let mat = matrix![0, 1, 2;
    ///                   3, 4, 5;
    ///                   6, 7, 8];
    /// let col = unsafe { mat.col_unchecked(2) };
    /// let expected = matrix![2usize; 5; 8];
    /// assert_matrix_eq!(*col, expected);
    /// # }
    /// ```
    unsafe fn col_unchecked(&self, index: usize) -> Column<T> {
        let ptr = self.as_ptr().offset(index as isize);
        Column { col: MatrixSlice::from_raw_parts(ptr, self.rows(), 1, self.row_stride()) }
    }

    /// Returns the row of a matrix at the given index.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let mat = matrix![0, 1, 2;
    ///                   3, 4, 5;
    ///                   6, 7, 8];
    /// let row = mat.row(1);
    /// let expected = matrix![3usize, 4, 5];
    /// assert_matrix_eq!(*row, expected);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Will panic if the row index is out of bounds.
    fn row(&self, index: usize) -> Row<T> {
        if index < self.rows() {
            unsafe { self.row_unchecked(index) }
        } else {
            panic!("Row index out of bounds.")
        }
    }

    /// Returns the row of a matrix at the given index without doing unbounds checking
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let mat = matrix![0, 1, 2;
    ///                   3, 4, 5;
    ///                   6, 7, 8];
    /// let row = unsafe { mat.row_unchecked(2) };
    /// let expected = matrix![6usize, 7, 8];
    /// assert_matrix_eq!(*row, expected);
    /// # }
    /// ```
    unsafe fn row_unchecked(&self, index: usize) -> Row<T> {
        let ptr = self.as_ptr().offset((self.row_stride() * index) as isize);
        Row { row: MatrixSlice::from_raw_parts(ptr, 1, self.cols(), self.row_stride()) }
    }

    /// Returns an iterator over the matrix data.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let mat = matrix![0, 1, 2;
    ///                   3, 4, 5;
    ///                   6, 7, 8];
    /// let slice = mat.sub_slice([1, 1], 2, 2);
    ///
    /// let slice_data = slice.iter().map(|v| *v).collect::<Vec<usize>>();
    /// assert_eq!(slice_data, vec![4, 5, 7, 8]);
    /// # }
    /// ```
    fn iter<'a>(&self) -> SliceIter<'a, T>
        where T: 'a
    {
        SliceIter {
            slice_start: self.as_ptr(),
            row_pos: 0,
            col_pos: 0,
            slice_rows: self.rows(),
            slice_cols: self.cols(),
            row_stride: self.row_stride(),
            _marker: PhantomData::<&T>,
        }
    }

    /// Iterate over the columns of the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = matrix![0, 1;
    ///                 2, 3;
    ///                 4, 5];
    ///
    /// let mut iter = a.col_iter();
    ///
    /// assert_matrix_eq!(*iter.next().unwrap(), matrix![ 0; 2; 4 ]);
    /// assert_matrix_eq!(*iter.next().unwrap(), matrix![ 1; 3; 5 ]);
    /// assert!(iter.next().is_none());
    /// # }
    /// ```
    fn col_iter(&self) -> Cols<T> {
        Cols {
            _marker: PhantomData::<&T>,
            col_pos: 0,
            row_stride: self.row_stride() as isize,
            slice_cols: self.cols(),
            slice_rows: self.rows(),
            slice_start: self.as_ptr(),
        }
    }

    /// Iterate over the rows of the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    /// let a = matrix![0, 1, 2;
    ///                 3, 4, 5;
    ///                 6, 7, 8];
    ///
    /// let mut iter = a.row_iter();
    ///
    /// assert_matrix_eq!(*iter.next().unwrap(), matrix![ 0, 1, 2 ]);
    /// assert_matrix_eq!(*iter.next().unwrap(), matrix![ 3, 4, 5 ]);
    /// assert_matrix_eq!(*iter.next().unwrap(), matrix![ 6, 7, 8 ]);
    /// assert!(iter.next().is_none());
    /// # }
    /// ```
    fn row_iter(&self) -> Rows<T> {
        Rows {
            slice_start: self.as_ptr(),
            row_pos: 0,
            slice_rows: self.rows(),
            slice_cols: self.cols(),
            row_stride: self.row_stride() as isize,
            _marker: PhantomData::<&T>,
        }
    }

    /// Iterate over diagonal entries
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg;
    ///
    /// # fn main() {
    /// use rulinalg::matrix::{DiagOffset, Matrix, BaseMatrix};
    ///
    /// let a = matrix![0, 1, 2;
    ///                 3, 4, 5;
    ///                 6, 7, 8];
    /// // Print super diag [1, 5]
    /// for d in a.diag_iter(DiagOffset::Above(1)) {
    ///     println!("{}", d);
    /// }
    ///
    /// // Print sub diag [3, 7]
    /// // Equivalent to `diag_iter(DiagOffset::Below(1))`
    /// for d in a.diag_iter(DiagOffset::from(-1)) {
    ///     println!("{}", d);
    /// }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// If using an `Above` or `Below` offset which is
    /// out-of-bounds this function will panic.
    ///
    /// This function will never panic if the `Main` diagonal
    /// offset is used.
    fn diag_iter(&self, k: DiagOffset) -> Diagonal<T, Self> {
        let (diag_len, diag_start) = match k.into() {
            DiagOffset::Main => (min(self.rows(), self.cols()), 0),
            DiagOffset::Above(m) => {
                assert!(m < self.cols(),
                        "Offset diagonal is not within matrix dimensions.");
                (min(self.rows(), self.cols() - m), m)
            }
            DiagOffset::Below(m) => {
                assert!(m < self.rows(),
                        "Offset diagonal is not within matrix dimensions.");
                (min(self.rows() - m, self.cols()), m * self.row_stride())
            }
        };

        Diagonal {
            matrix: self,
            diag_pos: diag_start,
            diag_end: diag_start + diag_len.saturating_sub(1) * self.row_stride() + diag_len,
            _marker: PhantomData::<&T>,
        }
    }

    /// The sum of the rows of the matrix.
    ///
    /// Returns a Vector equal to the sums of elements over the matrices rows.
    ///
    /// Note that the resulting vector is identical to the sums of
    /// elements along each column of the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    ///
    /// let c = a.sum_rows();
    /// assert_eq!(c, vector![4.0, 6.0]);
    /// # }
    /// ```
    fn sum_rows(&self) -> Vector<T>
        where T: Copy + Zero + Add<T, Output = T>
    {
        let mut sum_rows = vec![T::zero(); self.cols()];
        for row in self.row_iter() {
            utils::in_place_vec_bin_op(&mut sum_rows, row.raw_slice(), |sum, &r| *sum = *sum + r);
        }
        Vector::new(sum_rows)
    }

    /// The sum of the columns of the matrix.
    ///
    /// Returns a Vector equal to the sums of elements over the matrices columns.
    ///
    /// Note that the resulting vector is identical to the sums of
    /// elements along each row of the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    ///
    /// let c = a.sum_cols();
    /// assert_eq!(c, vector![3.0, 7.0]);
    /// # }
    /// ```
    fn sum_cols(&self) -> Vector<T>
        where T: Copy + Zero + Add<T, Output = T>
    {
        let mut col_sum = Vec::with_capacity(self.rows());
        col_sum.extend(self.row_iter().map(|row| utils::unrolled_sum(row.raw_slice())));
        Vector::new(col_sum)
    }

    /// Compute given matrix norm for matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::BaseMatrix;
    /// use rulinalg::norm::Euclidean;
    ///
    /// let a = matrix![3.0, 4.0];
    /// let c = a.norm(Euclidean);
    ///
    /// assert_eq!(c, 5.0);
    /// # }
    /// ```
    fn norm<N: MatrixNorm<T, Self>>(&self, norm: N) -> T
        where T: Float
    {
        norm.norm(self)
    }

    /// Compute the metric distance between two matrices.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::BaseMatrix;
    /// use rulinalg::norm::Euclidean;
    ///
    /// let a = matrix![3.0, 4.0;
    ///                 1.0, 2.0];
    /// let b = matrix![2.0, 5.0;
    ///                 0.0, 3.0];
    ///
    /// // Compute the square root of the sum of
    /// // elementwise squared-differences
    /// let c = a.metric(&b, Euclidean);
    ///
    /// assert_eq!(c, 2.0);
    /// # }
    /// ```
    fn metric<'a, 'b, B, M>(&'a self, mat: &'b B, metric: M) -> T
        where B: 'b + BaseMatrix<T>,
              M: MatrixMetric<'a, 'b, T, Self, B>
    {
        metric.metric(self, mat)
    }

    /// The sum of all elements in the matrix
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::BaseMatrix;
    ///
    /// let a = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    ///
    /// let c = a.sum();
    /// assert_eq!(c, 10.0);
    /// # }
    /// ```
    fn sum(&self) -> T
        where T: Copy + Zero + Add<T, Output = T>
    {
        self.row_iter()
            .fold(T::zero(),
                  |sum, row| sum + utils::unrolled_sum(row.raw_slice()))
    }

    /// The min of the specified axis of the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix, Axes};
    ///
    /// let a = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    ///
    /// let cmin = a.min(Axes::Col);
    /// assert_eq!(cmin, vector![1.0, 3.0]);
    ///
    /// let rmin = a.min(Axes::Row);
    /// assert_eq!(rmin, vector![1.0, 2.0]);
    /// # }
    /// ```
    fn min(&self, axis: Axes) -> Vector<T>
        where T: Copy + PartialOrd
    {
        match axis {
            Axes::Col => {
                let mut mins: Vec<T> = Vec::with_capacity(self.rows());
                for row in self.row_iter() {
                    let min = row.iter()
                        .skip(1)
                        .fold(row[0], |m, &v| if v < m { v } else { m });
                    mins.push(min);
                }
                Vector::new(mins)
            }
            Axes::Row => {
                let mut mins: Vec<T> = self.row(0).raw_slice().into();
                for row in self.row_iter().skip(1) {
                    utils::in_place_vec_bin_op(&mut mins, row.raw_slice(), |min, &r| if r < *min {
                        *min = r;
                    });
                }
                Vector::new(mins)
            }
        }
    }

    /// The max of the specified axis of the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{BaseMatrix, Axes};
    ///
    /// let a = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    ///
    /// let cmax = a.max(Axes::Col);
    /// assert_eq!(cmax, vector![2.0, 4.0]);
    ///
    /// let rmax = a.max(Axes::Row);
    /// assert_eq!(rmax, vector![3.0, 4.0]);
    /// # }
    /// ```
    fn max(&self, axis: Axes) -> Vector<T>
        where T: Copy + PartialOrd
    {
        match axis {
            Axes::Col => {
                let mut maxs: Vec<T> = Vec::with_capacity(self.rows());
                for row in self.row_iter() {
                    let max = row.iter()
                        .skip(1)
                        .fold(row[0], |m, &v| if v > m { v } else { m });
                    maxs.push(max);
                }
                Vector::new(maxs)
            }
            Axes::Row => {
                let mut maxs: Vec<T> = self.row(0).raw_slice().into();
                for row in self.row_iter().skip(1) {
                    utils::in_place_vec_bin_op(&mut maxs, row.raw_slice(), |max, &r| if r > *max {
                        *max = r;
                    });
                }
                Vector::new(maxs)
            }
        }
    }

    /// Convert the matrix struct into a owned Matrix.
    fn into_matrix(self) -> Matrix<T>
        where T: Copy
    {
        self.row_iter().collect()
    }

    /// Select rows from matrix
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = Matrix::<f64>::ones(3,3);
    ///
    /// let b = &a.select_rows(&[2]);
    /// assert_eq!(b.rows(), 1);
    /// assert_eq!(b.cols(), 3);
    ///
    /// let c = &a.select_rows(&[1,2]);
    /// assert_eq!(c.rows(), 2);
    /// assert_eq!(c.cols(), 3);
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if row indices exceed the matrix dimensions.
    fn select_rows<'a, I>(&self, rows: I) -> Matrix<T>
        where T: Copy,
              I: IntoIterator<Item = &'a usize>,
              I::IntoIter: ExactSizeIterator + Clone
    {
        let row_iter = rows.into_iter();
        let mut mat_vec = Vec::with_capacity(row_iter.len() * self.cols());

        for row in row_iter.clone() {
            assert!(*row < self.rows(),
                    "Row index is greater than number of rows.");
        }

        for row_idx in row_iter.clone() {
            unsafe {
                let row = self.row_unchecked(*row_idx);
                mat_vec.extend_from_slice(row.raw_slice());
            }
        }

        Matrix {
            cols: self.cols(),
            rows: row_iter.len(),
            data: mat_vec,
        }
    }

    /// Select columns from matrix
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = Matrix::<f64>::ones(3,3);
    /// let b = &a.select_cols(&[2]);
    /// assert_eq!(b.rows(), 3);
    /// assert_eq!(b.cols(), 1);
    ///
    /// let c = &a.select_cols(&[1,2]);
    /// assert_eq!(c.rows(), 3);
    /// assert_eq!(c.cols(), 2);
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if column indices exceed the matrix dimensions.
    fn select_cols<'a, I>(&self, cols: I) -> Matrix<T>
        where T: Copy,
              I: IntoIterator<Item = &'a usize>,
              I::IntoIter: ExactSizeIterator + Clone
    {
        let col_iter = cols.into_iter();
        let mut mat_vec = Vec::with_capacity(col_iter.len() * self.rows());

        for col in col_iter.clone() {
            assert!(*col < self.cols(),
                    "Column index is greater than number of columns.");
        }

        unsafe {
            for i in 0..self.rows() {
                for col in col_iter.clone() {
                    mat_vec.push(*self.get_unchecked([i, *col]));
                }
            }
        }

        Matrix {
            cols: col_iter.len(),
            rows: self.rows(),
            data: mat_vec,
        }
    }

    /// The elementwise product of two matrices.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    /// let b = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    ///
    /// let c = &a.elemul(&b);
    /// assert_matrix_eq!(c, &matrix![1.0, 4.0; 9.0, 16.0]);
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrices have different row counts.
    /// - The matrices have different column counts.
    fn elemul(&self, m: &Self) -> Matrix<T>
        where T: Copy + Mul<T, Output = T>
    {
        assert!(self.rows() == m.rows(), "Matrix row counts not equal.");
        assert!(self.cols() == m.cols(), "Matrix column counts not equal.");

        let mut data = Vec::with_capacity(self.rows() * self.cols());
        for (self_r, m_r) in self.row_iter().zip(m.row_iter()) {
            data.extend_from_slice(&utils::vec_bin_op(self_r.raw_slice(), m_r.raw_slice(), T::mul));
        }
        Matrix::new(self.rows(), self.cols(), data)
    }

    /// The elementwise division of two matrices.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    /// let b = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    ///
    /// let c = &a.elediv(&b);
    /// assert_matrix_eq!(c, &matrix![1.0, 1.0; 1.0, 1.0]);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrices have different row counts.
    /// - The matrices have different column counts.
    fn elediv(&self, m: &Self) -> Matrix<T>
        where T: Copy + Div<T, Output = T>
    {
        assert!(self.rows() == m.rows(), "Matrix row counts not equal.");
        assert!(self.cols() == m.cols(), "Matrix column counts not equal.");

        let mut data = Vec::with_capacity(self.rows() * self.cols());
        for (self_r, m_r) in self.row_iter().zip(m.row_iter()) {
            data.extend_from_slice(&utils::vec_bin_op(self_r.raw_slice(), m_r.raw_slice(), T::div));
        }
        Matrix::new(self.rows(), self.cols(), data)
    }

    /// Select block matrix from matrix
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = Matrix::<f64>::identity(3);
    /// let b = &a.select(&[0,1], &[1,2]);
    ///
    /// // We get the 2x2 block matrix in the upper right corner.
    /// assert_eq!(b.rows(), 2);
    /// assert_eq!(b.cols(), 2);
    ///
    /// // Prints [0,0, 1,0]
    /// println!("{:?}", b.data());
    /// ```
    ///
    /// # Panics
    ///
    /// - Panics if row or column indices exceed the matrix dimensions.
    fn select(&self, rows: &[usize], cols: &[usize]) -> Matrix<T>
        where T: Copy
    {

        let mut mat_vec = Vec::with_capacity(cols.len() * rows.len());

        for col in cols {
            assert!(*col < self.cols(),
                    "Column index is greater than number of columns.");
        }

        for row in rows {
            assert!(*row < self.rows(),
                    "Row index is greater than number of columns.");
        }

        unsafe {
            for row in rows {
                for col in cols {
                    mat_vec.push(*self.get_unchecked([*row, *col]));
                }
            }
        }

        Matrix {
            cols: cols.len(),
            rows: rows.len(),
            data: mat_vec,
        }
    }

    /// Horizontally concatenates two matrices. With self on the left.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = matrix![1.0, 2.0;
    ///                 3.0, 4.0;
    ///                 5.0, 6.0];
    /// let b = matrix![4.0;
    ///                 5.0;
    ///                 6.0];
    ///
    /// let c = &a.hcat(&b);
    /// assert_eq!(c.cols(), a.cols() + b.cols());
    /// assert_eq!(c[[1, 2]], 5.0);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - Self and m have different row counts.
    fn hcat<S>(&self, m: &S) -> Matrix<T>
        where T: Copy,
              S: BaseMatrix<T>
    {
        assert!(self.rows() == m.rows(), "Matrix row counts are not equal.");

        let mut new_data = Vec::with_capacity((self.cols() + m.cols()) * self.rows());

        for (self_row, m_row) in self.row_iter().zip(m.row_iter()) {
            new_data.extend_from_slice(self_row.raw_slice());
            new_data.extend_from_slice(m_row.raw_slice());
        }

        Matrix {
            cols: (self.cols() + m.cols()),
            rows: self.rows(),
            data: new_data,
        }
    }

    /// Vertically concatenates two matrices. With self on top.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = matrix![1.0, 2.0, 3.0;
    ///                 4.0, 5.0, 6.0];
    /// let b = matrix![4.0, 5.0, 6.0];;
    ///
    /// let c = &a.vcat(&b);
    /// assert_eq!(c.rows(), a.rows() + b.rows());
    /// assert_eq!(c[[2, 2]], 6.0);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - Self and m have different column counts.
    fn vcat<S>(&self, m: &S) -> Matrix<T>
        where T: Copy,
              S: BaseMatrix<T>
    {
        assert!(self.cols() == m.cols(),
                "Matrix column counts are not equal.");

        let mut new_data = Vec::with_capacity((self.rows() + m.rows()) * self.cols());

        for row in self.row_iter().chain(m.row_iter()) {
            new_data.extend_from_slice(row.raw_slice());
        }

        Matrix {
            cols: self.cols(),
            rows: (self.rows() + m.rows()),
            data: new_data,
        }
    }

    /// Extract the diagonal of the matrix
    ///
    /// Examples
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate rulinalg;
    ///
    /// use rulinalg::matrix::BaseMatrix;
    ///
    /// # fn main() {
    /// let a = matrix![1, 2, 3;
    ///                 4, 5, 6;
    ///                 7, 8, 9].diag().cloned().collect::<Vec<_>>();
    /// let b = matrix![1, 2;
    ///                 3, 4;
    ///                 5, 6].diag().cloned().collect::<Vec<_>>();
    ///
    /// assert_eq!(a, vec![1, 5, 9]);
    /// assert_eq!(b, vec![1, 4]);
    /// # }
    /// ```
    fn diag(&self) -> Diagonal<T, Self> {
        self.diag_iter(DiagOffset::Main)
    }

    /// Tranposes the given matrix
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let mat = matrix![1.0, 2.0, 3.0;
    ///                   4.0, 5.0, 6.0];
    ///
    /// let expected = matrix![1.0, 4.0;
    ///                        2.0, 5.0;
    ///                        3.0, 6.0];
    /// assert_matrix_eq!(mat.transpose(), expected);
    /// # }
    /// ```
    fn transpose(&self) -> Matrix<T>
        where T: Copy
    {
        let mut new_data = Vec::with_capacity(self.rows() * self.cols());

        unsafe {
            new_data.set_len(self.rows() * self.cols());
            for i in 0..self.cols() {
                for j in 0..self.rows() {
                    *new_data.get_unchecked_mut(i * self.rows() + j) = *self.get_unchecked([j, i]);
                }
            }
        }

        Matrix {
            cols: self.rows(),
            rows: self.cols(),
            data: new_data,
        }
    }

    /// Checks if matrix is diagonal.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let a = matrix![1.0, 0.0;
    ///                 0.0, 1.0];
    /// let a_diag = a.is_diag();
    ///
    /// assert_eq!(a_diag, true);
    ///
    /// let b = matrix![1.0, 0.0;
    ///                 1.0, 0.0];
    /// let b_diag = b.is_diag();
    ///
    /// assert_eq!(b_diag, false);
    /// # }
    /// ```
    fn is_diag(&self) -> bool
        where T: Zero + PartialEq
    {
        let mut next_diag = 0usize;
        self.iter().enumerate().all(|(i, data)| if i == next_diag {
            next_diag += self.cols() + 1;
            true
        } else {
            data == &T::zero()
        })
    }

    /// Solves an upper triangular linear system.
    ///
    /// Given a matrix `A` and a vector `b`, this function returns the
    /// solution of the upper triangular system `Ux = b`, where `U` is
    /// the upper triangular part of `A`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::BaseMatrix;
    /// use std::f32;
    ///
    /// let u = matrix![1.0, 2.0;
    ///                 0.0, 1.0];
    /// let y = vector![3.0, 1.0];
    ///
    /// let x = u.solve_u_triangular(y).expect("A solution should exist!");
    /// assert!((x[0] - 1.0) < f32::EPSILON);
    /// assert!((x[1] - 1.0) < f32::EPSILON);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - Vector size and matrix column count are not equal.
    ///
    /// # Failures
    ///
    /// - There is no valid solution to the system (matrix is singular).
    /// - The matrix is empty.
    fn solve_u_triangular(&self, y: Vector<T>) -> Result<Vector<T>, Error>
        where T: Any + Float
    {
        assert!(self.cols() == y.size(),
                format!("Vector size {0} != {1} Matrix column count.",
                        y.size(),
                        self.cols()));

        back_substitution(self, y)
    }

    /// Solves a lower triangular linear system.
    ///
    /// Given a matrix `A` and a vector `b`, this function returns the
    /// solution of the lower triangular system `Lx = b`, where `L` is
    /// the lower triangular part of `A`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::BaseMatrix;
    /// use std::f32;
    ///
    /// let l = matrix![1.0, 0.0;
    ///                 2.0, 1.0];
    /// let y = vector![1.0, 3.0];
    ///
    /// let x = l.solve_l_triangular(y).expect("A solution should exist!");
    /// println!("{:?}", x);
    /// assert!((x[0] - 1.0) < f32::EPSILON);
    /// assert!((x[1] - 1.0) < f32::EPSILON);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - Vector size and matrix column count are not equal.
    ///
    /// # Failures
    ///
    /// - There is no valid solution to the system (matrix is singular).
    /// - The matrix is empty.
    fn solve_l_triangular(&self, y: Vector<T>) -> Result<Vector<T>, Error>
        where T: Any + Float
    {
        assert!(self.cols() == y.size(),
                format!("Vector size {0} != {1} Matrix column count.",
                        y.size(),
                        self.cols()));

        forward_substitution(self, y)
    }

    /// Split the matrix at the specified axis returning two `MatrixSlice`s.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Axes, Matrix, BaseMatrix};
    ///
    /// let a = Matrix::new(3,3, vec![2.0; 9]);
    /// let (b,c) = a.split_at(1, Axes::Row);
    /// ```
    fn split_at(&self, mid: usize, axis: Axes) -> (MatrixSlice<T>, MatrixSlice<T>) {
        let slice_1: MatrixSlice<T>;
        let slice_2: MatrixSlice<T>;

        match axis {
            Axes::Row => {
                assert!(mid < self.rows());
                unsafe {
                    slice_1 = MatrixSlice::from_raw_parts(self.as_ptr(),
                                                          mid,
                                                          self.cols(),
                                                          self.row_stride());
                    slice_2 = MatrixSlice::from_raw_parts(self.as_ptr()
                                                              .offset((mid * self.row_stride()) as
                                                                      isize),
                                                          self.rows() - mid,
                                                          self.cols(),
                                                          self.row_stride());
                }
            }
            Axes::Col => {
                assert!(mid < self.cols());
                unsafe {
                    slice_1 = MatrixSlice::from_raw_parts(self.as_ptr(),
                                                          self.rows(),
                                                          mid,
                                                          self.row_stride());
                    slice_2 = MatrixSlice::from_raw_parts(self.as_ptr().offset(mid as isize),
                                                          self.rows(),
                                                          self.cols() - mid,
                                                          self.row_stride());
                }
            }
        }

        (slice_1, slice_2)
    }

    /// Produce a `MatrixSlice` from an existing matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, BaseMatrix, MatrixSlice};
    ///
    /// let a = Matrix::new(3,3, (0..9).collect::<Vec<usize>>());
    /// let slice = MatrixSlice::from_matrix(&a, [1,1], 2, 2);
    /// let new_slice = slice.sub_slice([0,0], 1, 1);
    /// ```
    fn sub_slice<'a>(&self, start: [usize; 2], rows: usize, cols: usize) -> MatrixSlice<'a, T>
        where T: 'a
    {
        assert!(start[0] + rows <= self.rows(),
                "View dimensions exceed matrix dimensions.");
        assert!(start[1] + cols <= self.cols(),
                "View dimensions exceed matrix dimensions.");

        unsafe {
            MatrixSlice::from_raw_parts(self.as_ptr()
                                            .offset((start[0] * self.row_stride() + start[1]) as
                                                    isize),
                                        rows,
                                        cols,
                                        self.row_stride())
        }
    }
}

/// Trait for mutable matrices.
pub trait BaseMatrixMut<T>: BaseMatrix<T> {
    /// Top left index of the slice.
    fn as_mut_ptr(&mut self) -> *mut T;

    /// Returns a `MatrixSliceMut` over the whole matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut a = Matrix::new(3, 3, vec![2.0; 9]);
    /// let b = a.as_mut_slice();
    /// ```
    fn as_mut_slice(&mut self) -> MatrixSliceMut<T> {
        unsafe {
            MatrixSliceMut::from_raw_parts(self.as_mut_ptr(),
                                           self.rows(),
                                           self.cols(),
                                           self.row_stride())
        }
    }

    /// Get a mutable reference to an element in the matrix without bounds checks.
    unsafe fn get_unchecked_mut(&mut self, index: [usize; 2]) -> &mut T {
        &mut *(self.as_mut_ptr().offset((index[0] * self.row_stride() + index[1]) as isize))
    }

    /// Get a mutable reference to an element in the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrix, BaseMatrixMut};
    ///
    /// let mut mat = matrix![0, 1;
    ///                       3, 4;
    ///                       6, 7];
    ///
    /// assert_eq!(mat.get_mut([0, 2]), None);
    /// assert_eq!(mat.get_mut([3, 0]), None);
    ///
    /// assert_eq!(*mat.get_mut([0, 0]).unwrap(), 0);
    /// *mat.get_mut([0,0]).unwrap() = 2;
    /// assert_eq!(*mat.get_mut([0, 0]).unwrap(), 2);
    /// # }
    /// ```
    fn get_mut(&mut self, index: [usize; 2]) -> Option<&mut T> {
        let row_ind = index[0];
        let col_ind = index[1];

        if row_ind >= self.rows() || col_ind >= self.cols() {
          None
        } else {
          unsafe { Some(self.get_unchecked_mut(index)) }
        }
    }

    /// Returns a mutable iterator over the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut a = Matrix::new(3,3, (0..9).collect::<Vec<usize>>());
    ///
    /// {
    ///     let mut slice = a.sub_slice_mut([1,1], 2, 2);
    ///
    ///     for d in slice.iter_mut() {
    ///         *d = *d + 2;
    ///     }
    /// }
    ///
    /// // Only the matrix slice is updated.
    /// assert_matrix_eq!(a, matrix![0, 1, 2; 3, 6, 7; 6, 9, 10]);
    /// # }
    /// ```
    fn iter_mut<'a>(&mut self) -> SliceIterMut<'a, T>
        where T: 'a
    {
        SliceIterMut {
            slice_start: self.as_mut_ptr(),
            row_pos: 0,
            col_pos: 0,
            slice_rows: self.rows(),
            slice_cols: self.cols(),
            row_stride: self.row_stride(),
            _marker: PhantomData::<&mut T>,
        }
    }

    /// Returns a mutable reference to the column of a matrix at the given index.
    /// `None` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate rulinalg;
    ///
    /// # fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut mat = matrix![0, 1, 2;
    ///                       3, 4, 5;
    ///                       6, 7, 8];
    /// let mut slice = mat.sub_slice_mut([1,1], 2, 2);
    /// {
    ///     let col = slice.col_mut(1);
    ///     let mut expected = matrix![5usize; 8];
    ///     assert_matrix_eq!(*col, expected);
    /// }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Will panic if the column index is out of bounds.
    fn col_mut(&mut self, index: usize) -> ColumnMut<T> {
        if index < self.cols() {
            unsafe { self.col_unchecked_mut(index) }
        } else {
            panic!("Column index out of bounds.")
        }
    }

    /// Returns a mutable reference to the column of a matrix at the given index
    /// without doing a bounds check.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate rulinalg;
    ///
    /// # fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut mat = matrix![0, 1, 2;
    ///                       3, 4, 5;
    ///                       6, 7, 8];
    /// let mut slice = mat.sub_slice_mut([1,1], 2, 2);
    /// let col = unsafe { slice.col_unchecked_mut(1) };
    /// let mut expected = matrix![5usize; 8];
    /// assert_matrix_eq!(*col, expected);
    /// # }
    /// ```
    unsafe fn col_unchecked_mut(&mut self, index: usize) -> ColumnMut<T> {
        let ptr = self.as_mut_ptr().offset(index as isize);
        ColumnMut { col: MatrixSliceMut::from_raw_parts(ptr, self.rows(), 1, self.row_stride()) }
    }

    /// Returns a mutable reference to the row of a matrix at the given index.
    /// `None` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate rulinalg;
    ///
    /// # fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut mat = matrix![0, 1, 2;
    ///                       3, 4, 5;
    ///                       6, 7, 8];
    /// let mut slice = mat.sub_slice_mut([1,1], 2, 2);
    /// {
    ///     let row = slice.row_mut(1);
    ///     let mut expected = matrix![7usize, 8];
    ///     assert_matrix_eq!(*row, expected);
    /// }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Will panic if the row index is out of bounds.
    fn row_mut(&mut self, index: usize) -> RowMut<T> {
        if index < self.rows() {
            unsafe { self.row_unchecked_mut(index) }
        } else {
            panic!("Row index out of bounds.")
        }
    }

    /// Returns a mutable reference to the row of a matrix at the given index
    /// without doing a bounds check.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate rulinalg;
    ///
    /// # fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut mat = matrix![0, 1, 2;
    ///                       3, 4, 5;
    ///                       6, 7, 8];
    /// let mut slice = mat.sub_slice_mut([1,1], 2, 2);
    /// let row = unsafe { slice.row_unchecked_mut(1) };
    /// let mut expected = matrix![7usize, 8];
    /// assert_matrix_eq!(*row, expected);
    /// # }
    /// ```
    unsafe fn row_unchecked_mut(&mut self, index: usize) -> RowMut<T> {
        let ptr = self.as_mut_ptr().offset((self.row_stride() * index) as isize);
        RowMut { row: MatrixSliceMut::from_raw_parts(ptr, 1, self.cols(), self.row_stride()) }
    }

    /// Swaps two rows in a matrix.
    ///
    /// If `a == b`, this method does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg;
    /// # fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut x = matrix![0, 1;
    ///                     2, 3;
    ///                     4, 5;
    ///                     6, 7];
    ///
    /// x.swap_rows(1, 3);
    /// let expected = matrix![0, 1;
    ///                        6, 7;
    ///                        4, 5;
    ///                        2, 3];
    ///
    /// assert_matrix_eq!(x, expected);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` are out of bounds.
    fn swap_rows(&mut self, a: usize, b: usize) {
        assert!(a < self.rows(),
                format!("Row index {0} larger than row count {1}", a, self.rows()));
        assert!(b < self.rows(),
                format!("Row index {0} larger than row count {1}", b, self.rows()));

        if a != b {
            unsafe {
                let row_a = slice::from_raw_parts_mut(self.as_mut_ptr()
                                                          .offset((self.row_stride() * a) as
                                                                  isize),
                                                      self.cols());
                let row_b = slice::from_raw_parts_mut(self.as_mut_ptr()
                                                          .offset((self.row_stride() * b) as
                                                                  isize),
                                                      self.cols());

                for (x, y) in row_a.into_iter().zip(row_b.into_iter()) {
                    mem::swap(x, y);
                }
            }
        }

    }

    /// Swaps two columns in a matrix.
    ///
    /// If `a == b`, this method does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg;
    /// # fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut x = matrix![0, 1;
    ///                     2, 3;
    ///                     4, 5];
    ///
    /// x.swap_cols(0, 1);
    /// let expected = matrix![1, 0;
    ///                        3, 2;
    ///                        5, 4];
    ///
    /// assert_matrix_eq!(x, expected);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` are out of bounds.
    fn swap_cols(&mut self, a: usize, b: usize) {
        assert!(a < self.cols(),
                format!("Row index {0} larger than row count {1}", a, self.rows()));
        assert!(b < self.cols(),
                format!("Row index {0} larger than row count {1}", b, self.rows()));

        if a != b {
            unsafe {
                for i in 0..self.rows() {
                    let a_ptr: *mut T = self.get_unchecked_mut([i, a]);
                    let b_ptr: *mut T = self.get_unchecked_mut([i, b]);
                    ptr::swap(a_ptr, b_ptr);
                }
            }
        }

    }

    /// Iterate over the mutable columns of the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut a = matrix![0, 1;
    ///                     2, 3;
    ///                     4, 5];
    ///
    /// for mut col in a.col_iter_mut() {
    ///     *col += 1;
    /// }
    ///
    /// // Now contains the range 1..7
    /// println!("{}", a);
    /// # }
    /// ```
    fn col_iter_mut(&mut self) -> ColsMut<T> {
        ColsMut {
            _marker: PhantomData::<&mut T>,
            col_pos: 0,
            row_stride: self.row_stride() as isize,
            slice_cols: self.cols(),
            slice_rows: self.rows(),
            slice_start: self.as_mut_ptr(),
        }
    }

    /// Iterate over the mutable rows of the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut a = matrix![0, 1;
    ///                     2, 3;
    ///                     4, 5];
    ///
    /// for mut row in a.row_iter_mut() {
    ///     *row += 1;
    /// }
    ///
    /// // Now contains the range 1..7
    /// println!("{}", a);
    /// # }
    /// ```
    fn row_iter_mut(&mut self) -> RowsMut<T> {
        RowsMut {
            slice_start: self.as_mut_ptr(),
            row_pos: 0,
            slice_rows: self.rows(),
            slice_cols: self.cols(),
            row_stride: self.row_stride() as isize,
            _marker: PhantomData::<&mut T>,
        }
    }

    /// Iterate over diagonal entries mutably
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg;
    ///
    /// # fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut, DiagOffset};
    ///
    /// let mut a = matrix![0, 1, 2;
    ///                     3, 4, 5;
    ///                     6, 7, 8];
    ///
    /// // Increment super diag
    /// for d in a.diag_iter_mut(DiagOffset::Above(1)) {
    ///     *d = *d + 1;
    /// }
    ///
    /// // Zero the sub-diagonal (sets 3 and 7 to 0)
    /// // Equivalent to `diag_iter(DiagOffset::Below(1))`
    /// for sub_d in a.diag_iter_mut(DiagOffset::from(-1)) {
    ///     *sub_d = 0;
    /// }
    ///
    /// println!("{}", a);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// If using an `Above` or `Below` offset which is
    /// out-of-bounds this function will panic.
    ///
    /// This function will never panic if the `Main` diagonal
    /// offset is used.
    fn diag_iter_mut(&mut self, k: DiagOffset) -> DiagonalMut<T, Self> {
        let (diag_len, diag_start) = match k.into() {
            DiagOffset::Main => (min(self.rows(), self.cols()), 0),
            DiagOffset::Above(m) => {
                assert!(m < self.cols(),
                        "Offset diagonal is not within matrix dimensions.");
                (min(self.rows(), self.cols() - m), m)
            }
            DiagOffset::Below(m) => {
                assert!(m < self.rows(),
                        "Offset diagonal is not within matrix dimensions.");
                (min(self.rows() - m, self.cols()), m * self.row_stride())
            }
        };


        let diag_end = diag_start + (diag_len - 1) * self.row_stride() + diag_len;
        DiagonalMut {
            matrix: self,
            diag_pos: diag_start,
            diag_end: diag_end,
            _marker: PhantomData::<&mut T>,
        }
    }

    /// Sets the underlying matrix data to the target data.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// let mut mat = Matrix::<f32>::zeros(4,4);
    /// let one_block = Matrix::<f32>::ones(2,2);
    ///
    /// // Get a mutable slice of the upper left 2x2 block.
    /// let mat_block = mat.sub_slice_mut([0,0], 2, 2);
    ///
    /// // Set the upper left 2x2 block to be ones.
    /// mat_block.set_to(one_block);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the dimensions of `self` and `target` are not the same.
    fn set_to<M: BaseMatrix<T>>(mut self, target: M)
        where T: Copy
    {
        assert!(self.rows() == target.rows(),
                "Target has different row count to self.");
        assert!(self.cols() == target.cols(),
                "Target has different column count to self.");
        for (mut s, t) in self.row_iter_mut().zip(target.row_iter()) {
            // Vectorized assignment per row.
            utils::in_place_vec_bin_op(s.raw_slice_mut(), t.raw_slice(), |x, &y| *x = y);
        }
    }

    /// Applies a function to each element in the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, BaseMatrixMut};
    ///
    /// fn add_two(a: f64) -> f64 {
    ///     a + 2f64
    /// }
    ///
    /// let a = Matrix::new(2, 2, vec![0.;4]);
    ///
    /// let b = a.apply(&add_two);
    ///
    /// assert_eq!(b, matrix![2.0, 2.0; 2.0, 2.0]);
    /// # }
    /// ```
    fn apply(mut self, f: &Fn(T) -> T) -> Self
        where T: Copy
    {
        for val in self.iter_mut() {
            *val = f(*val);
        }
        self
    }

    /// Split the matrix at the specified axis returning two `MatrixSliceMut`s.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Axes, Matrix, BaseMatrixMut};
    ///
    /// let mut a = Matrix::new(3,3, vec![2.0; 9]);
    /// let (b, c) = a.split_at_mut(1, Axes::Col);
    /// ```
    fn split_at_mut(&mut self, mid: usize, axis: Axes) -> (MatrixSliceMut<T>, MatrixSliceMut<T>) {

        let slice_1: MatrixSliceMut<T>;
        let slice_2: MatrixSliceMut<T>;

        match axis {
            Axes::Row => {
                assert!(mid < self.rows());
                unsafe {
                    slice_1 = MatrixSliceMut::from_raw_parts(self.as_mut_ptr(),
                                                             mid,
                                                             self.cols(),
                                                             self.row_stride());
                    slice_2 = MatrixSliceMut::from_raw_parts(self.as_mut_ptr()
                                                                 .offset((mid *
                                                                          self.row_stride()) as
                                                                         isize),
                                                             self.rows() - mid,
                                                             self.cols(),
                                                             self.row_stride());
                }
            }
            Axes::Col => {
                assert!(mid < self.cols());
                unsafe {
                    slice_1 = MatrixSliceMut::from_raw_parts(self.as_mut_ptr(),
                                                             self.rows(),
                                                             mid,
                                                             self.row_stride());
                    slice_2 = MatrixSliceMut::from_raw_parts(self.as_mut_ptr()
                                                                 .offset(mid as isize),
                                                             self.rows(),
                                                             self.cols() - mid,
                                                             self.row_stride());
                }
            }
        }

        (slice_1, slice_2)
    }

    /// Produce a `MatrixSliceMut` from an existing matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, MatrixSliceMut, BaseMatrixMut};
    ///
    /// let mut a = Matrix::new(3,3, (0..9).collect::<Vec<usize>>());
    /// let mut slice = MatrixSliceMut::from_matrix(&mut a, [1,1], 2, 2);
    /// let new_slice = slice.sub_slice_mut([0,0], 1, 1);
    /// ```
    fn sub_slice_mut<'a>(&mut self,
                         start: [usize; 2],
                         rows: usize,
                         cols: usize)
                         -> MatrixSliceMut<'a, T>
        where T: 'a
    {
        assert!(start[0] + rows <= self.rows(),
                "View dimensions exceed matrix dimensions.");
        assert!(start[1] + cols <= self.cols(),
                "View dimensions exceed matrix dimensions.");

        unsafe {
            MatrixSliceMut::from_raw_parts(self.as_mut_ptr()
                                               .offset((start[0] * self.row_stride() + start[1]) as
                                                       isize),
                                           rows,
                                           cols,
                                           self.row_stride())
        }
    }
}
