use std::iter::{ExactSizeIterator, FromIterator};
use std::mem;
use std::vec::*;

use super::{Matrix, MatrixSlice, MatrixSliceMut};
use super::{Column, ColumnMut, Cols, ColsMut, Row, RowMut, Rows, RowsMut, Diagonal, DiagonalMut};
use super::{BaseMatrix, BaseMatrixMut, SliceIter, SliceIterMut};

macro_rules! impl_slice_iter (
    ($slice_iter:ident, $data_type:ty) => (
/// Iterates over the matrix slice data in row-major order.
impl<'a, T> Iterator for $slice_iter<'a, T> {
    type Item = $data_type;

    fn next(&mut self) -> Option<$data_type> {
        let offset = self.row_pos * self.row_stride + self.col_pos;
        let end = self.slice_rows * self.row_stride;
        // Set the position of the next element
        if offset < end {
            unsafe {
                let iter_ptr = self.slice_start.offset(offset as isize);

                // If end of row, set to start of next row
                if self.col_pos + 1 == self.slice_cols {
                    self.row_pos += 1usize;
                    self.col_pos = 0usize;
                } else {
                    self.col_pos += 1usize;
                }

                Some(mem::transmute(iter_ptr))
            }
        } else {
            None
        }
    }
}
    );
);

impl_slice_iter!(SliceIter, &'a T);
impl_slice_iter!(SliceIterMut, &'a mut T);

macro_rules! impl_diag_iter (
    ($diag:ident, $diag_base:ident, $diag_type:ty, $as_ptr:ident) => (

/// Iterates over the diagonals in the matrix.
impl<'a, T, M: $diag_base<T>> Iterator for $diag<'a, T, M> {
    type Item = $diag_type;

    fn next(&mut self) -> Option<Self::Item> {
        if self.diag_pos < self.diag_end {
            let pos = self.diag_pos as isize;
            self.diag_pos += self.matrix.row_stride() + 1;
            unsafe {
                Some(mem::transmute(self.matrix.$as_ptr()
                            .offset(pos)))
            }
        } else {
            None
        }
    }

    fn last(self) -> Option<Self::Item> {
        if self.diag_pos < self.diag_end {
            unsafe {
                Some(mem::transmute(self.matrix.$as_ptr()
                            .offset(self.diag_end as isize - 1)))
            }
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.diag_pos += n * (self.matrix.row_stride() + 1);
        if self.diag_pos < self.diag_end {
            let pos = self.diag_pos as isize;
            self.diag_pos += self.matrix.row_stride() + 1;
            unsafe {
                Some(mem::transmute(self.matrix.$as_ptr()
                            .offset(pos)))
            }
        } else {
            None
        }
    }

    fn count(self) -> usize {
        self.size_hint().0
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.diag_pos < self.diag_end {
            let s = (self.diag_end - self.diag_pos) / (self.matrix.row_stride() + 1) + 1;
            (s, Some(s))
        } else {
            (0, Some(0))
        }
    }
}

impl<'a, T, M: $diag_base<T>> ExactSizeIterator for $diag<'a, T, M> {}
    );
);

impl_diag_iter!(Diagonal, BaseMatrix, &'a T, as_ptr);
impl_diag_iter!(DiagonalMut, BaseMatrixMut, &'a mut T, as_mut_ptr);

macro_rules! impl_col_iter (
    ($cols:ident, $col_type:ty, $col_base:ident, $slice_base:ident) => (

/// Iterates over the columns in the matrix.
impl<'a, T> Iterator for $cols<'a, T> {
    type Item = $col_type;

    fn next(&mut self) -> Option<Self::Item> {
        if self.col_pos >= self.slice_cols {
            return None;
        }

        let column: $col_type;
        unsafe {
            let ptr = self.slice_start.offset(self.col_pos as isize);
            column  = $col_base {
                col: $slice_base::from_raw_parts(ptr, self.slice_rows, 1, self.row_stride as usize)
            };
        }
        self.col_pos += 1;
        Some(column)
    }

    fn last(self) -> Option<Self::Item> {
        if self.col_pos >= self.slice_cols {
            return None;
        }

        unsafe {
            let ptr = self.slice_start.offset((self.slice_cols - 1) as isize);
            Some($col_base {
                col: $slice_base::from_raw_parts(ptr, self.slice_rows, 1, self.row_stride as usize)
            })
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.col_pos + n >= self.slice_cols {
            return None;
        }

        let column: $col_type;
        unsafe {
            let ptr = self.slice_start.offset((self.col_pos + n) as isize);
            column = $col_base {
                col: $slice_base::from_raw_parts(ptr, self.slice_rows, 1, self.row_stride as usize)
            }
        }
        self.col_pos += n + 1;
        Some(column)
    }

    fn count(self) -> usize {
        self.slice_cols - self.col_pos
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.slice_cols - self.col_pos, Some(self.slice_cols - self.col_pos))
    }
}
    );
);

impl_col_iter!(Cols, Column<'a, T>, Column, MatrixSlice);
impl_col_iter!(ColsMut, ColumnMut<'a, T>, ColumnMut, MatrixSliceMut);

impl<'a, T> ExactSizeIterator for Cols<'a, T> {}
impl<'a, T> ExactSizeIterator for ColsMut<'a, T> {}

macro_rules! impl_row_iter (
    ($rows:ident, $row_type:ty, $row_base:ident, $slice_base:ident) => (

/// Iterates over the rows in the matrix.
impl<'a, T> Iterator for $rows<'a, T> {
    type Item = $row_type;

    fn next(&mut self) -> Option<Self::Item> {
// Check if we have reached the end
        if self.row_pos < self.slice_rows {
            let row: $row_type;
            unsafe {
// Get pointer and create a slice from raw parts
                let ptr = self.slice_start.offset(self.row_pos as isize * self.row_stride);
                row = $row_base {
                    row: $slice_base::from_raw_parts(ptr, 1, self.slice_cols, self.row_stride as usize)
                };
            }

            self.row_pos += 1;
            Some(row)
        } else {
            None
        }
    }

    fn last(self) -> Option<Self::Item> {
// Check if already at the end
        if self.row_pos < self.slice_rows {
            unsafe {
// Get pointer to last row and create a slice from raw parts
                let ptr = self.slice_start.offset((self.slice_rows - 1) as isize * self.row_stride);
                Some($row_base {
                    row: $slice_base::from_raw_parts(ptr, 1, self.slice_cols, self.row_stride as usize)
                })
            }
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.row_pos + n < self.slice_rows {
            let row: $row_type;
            unsafe {
                let ptr = self.slice_start.offset((self.row_pos + n) as isize * self.row_stride);
                row = $row_base {
                    row: $slice_base::from_raw_parts(ptr, 1, self.slice_cols, self.row_stride as usize)
                }
            }

            self.row_pos += n + 1;
            Some(row)
        } else {
            None
        }
    }

    fn count(self) -> usize {
        self.slice_rows - self.row_pos
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.slice_rows - self.row_pos, Some(self.slice_rows - self.row_pos))
    }
}
    );
);

impl_row_iter!(Rows, Row<'a, T>, Row, MatrixSlice);
impl_row_iter!(RowsMut, RowMut<'a, T>, RowMut, MatrixSliceMut);

impl<'a, T> ExactSizeIterator for Rows<'a, T> {}
impl<'a, T> ExactSizeIterator for RowsMut<'a, T> {}

/// Creates a `Matrix` from an iterator over slices.
///
/// Each of the slices produced by the iterator will become a row in the matrix.
///
/// # Panics
///
/// Will panic if the iterators items do not have constant length.
///
/// # Examples
///
/// We can create a new matrix from some data.
///
/// ```
/// use rulinalg::matrix::{Matrix, BaseMatrix};
///
/// let a : Matrix<f64> = vec![4f64; 16].chunks(4).collect();
///
/// assert_eq!(a.rows(), 4);
/// assert_eq!(a.cols(), 4);
/// ```
///
/// We can also do more interesting things.
///
/// ```
/// use rulinalg::matrix::{Matrix, BaseMatrix};
///
/// let a = Matrix::new(4,2, (0..8).collect::<Vec<usize>>());
///
/// // Here we skip the first row and take only those
/// // where the first entry is less than 6.
/// let b = a.row_iter()
///          .skip(1)
///          .filter(|x| x[0] < 6)
///          .collect::<Matrix<usize>>();
///
/// // We take the middle rows
/// assert_eq!(b.into_vec(), vec![2,3,4,5]);
/// ```
impl<'a, T: 'a + Copy> FromIterator<&'a [T]> for Matrix<T> {
    fn from_iter<I: IntoIterator<Item = &'a [T]>>(iterable: I) -> Self {
        let mut mat_data: Vec<T>;
        let cols: usize;
        let mut rows = 0;

        let mut iterator = iterable.into_iter();

        match iterator.next() {
            None => {
                return Matrix {
                    data: Vec::new(),
                    rows: 0,
                    cols: 0,
                }
            }
            Some(row) => {
                rows += 1;
                // Here we set the capacity - get iterator size and the cols
                let (lower_rows, _) = iterator.size_hint();
                cols = row.len();

                mat_data = Vec::with_capacity(lower_rows.saturating_add(1).saturating_mul(cols));
                mat_data.extend_from_slice(row);
            }
        }

        for row in iterator {
            assert!(row.len() == cols, "Iterator slice length must be constant.");
            mat_data.extend_from_slice(row);
            rows += 1;
        }

        mat_data.shrink_to_fit();

        Matrix {
            data: mat_data,
            rows: rows,
            cols: cols,
        }
    }
}

macro_rules! impl_from_iter_row(
    ($row_type:ty) => (
impl<'a, T: 'a + Copy> FromIterator<$row_type> for Matrix<T> {
    fn from_iter<I: IntoIterator<Item = $row_type>>(iterable: I) -> Self {
        let mut mat_data: Vec<T>;
        let cols: usize;
        let mut rows = 0;

        let mut iterator = iterable.into_iter();

        match iterator.next() {
            None => {
                return Matrix {
                    data: Vec::new(),
                    rows: 0,
                    cols: 0,
                }
            }
            Some(row) => {
                rows += 1;
                // Here we set the capacity - get iterator size and the cols
                let (lower_rows, _) = iterator.size_hint();
                cols = row.row.cols();

                mat_data = Vec::with_capacity(lower_rows.saturating_add(1).saturating_mul(cols));
                mat_data.extend_from_slice(row.raw_slice());
            }
        }

        for row in iterator {
            assert!(row.row.cols() == cols, "Iterator row size must be constant.");
            mat_data.extend_from_slice(row.raw_slice());
            rows += 1;
        }

        mat_data.shrink_to_fit();

        Matrix {
            data: mat_data,
            rows: rows,
            cols: cols,
        }
    }
}
    );
);

impl_from_iter_row!(Row<'a, T>);
impl_from_iter_row!(RowMut<'a, T>);


impl<'a, T> IntoIterator for MatrixSlice<'a, T> {
    type Item = &'a T;
    type IntoIter = SliceIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a MatrixSlice<'a, T> {
    type Item = &'a T;
    type IntoIter = SliceIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut MatrixSlice<'a, T> {
    type Item = &'a T;
    type IntoIter = SliceIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for MatrixSliceMut<'a, T> {
    type Item = &'a mut T;
    type IntoIter = SliceIterMut<'a, T>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> IntoIterator for &'a MatrixSliceMut<'a, T> {
    type Item = &'a T;
    type IntoIter = SliceIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut MatrixSliceMut<'a, T> {
    type Item = &'a mut T;
    type IntoIter = SliceIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{DiagOffset, Matrix, MatrixSlice, MatrixSliceMut};
    use super::super::{BaseMatrix, BaseMatrixMut};

    #[test]
    fn test_diag_offset_equivalence() {
        // This test will check that `Main`,
        // `Below(0)`, and `Above(0)` are all equivalent.
        let a = matrix![0.0, 1.0, 2.0;
                        3.0, 4.0, 5.0;
                        6.0, 7.0, 8.0];

        // Collect each diagonal and compare them
        let d1 = a.diag_iter(DiagOffset::Main).collect::<Vec<_>>();
        let d2 = a.diag_iter(DiagOffset::Above(0)).collect::<Vec<_>>();
        let d3 = a.diag_iter(DiagOffset::Below(0)).collect::<Vec<_>>();
        assert_eq!(d1, d2);
        assert_eq!(d2, d3);

        let b = MatrixSlice::from_matrix(&a, [0, 0], 2, 3);
        let d1 = b.diag_iter(DiagOffset::Main).collect::<Vec<_>>();
        let d2 = b.diag_iter(DiagOffset::Above(0)).collect::<Vec<_>>();
        let d3 = b.diag_iter(DiagOffset::Below(0)).collect::<Vec<_>>();
        assert_eq!(d1, d2);
        assert_eq!(d2, d3);
    }

    #[test]
    fn test_matrix_diag() {
        let mut a = matrix![0.0, 1.0, 2.0;
                            3.0, 4.0, 5.0;
                            6.0, 7.0, 8.0];

        let diags = vec![0.0, 4.0, 8.0];
        assert_eq!(a.diag_iter(DiagOffset::Main).cloned().collect::<Vec<_>>(), diags);
        let diags = vec![1.0, 5.0];
        assert_eq!(a.diag_iter(DiagOffset::Above(1)).cloned().collect::<Vec<_>>(), diags);
        let diags = vec![3.0, 7.0];
        assert_eq!(a.diag_iter(DiagOffset::Below(1)).cloned().collect::<Vec<_>>(), diags);
        let diags = vec![2.0];
        assert_eq!(a.diag_iter(DiagOffset::Above(2)).cloned().collect::<Vec<_>>(), diags);
        let diags = vec![6.0];
        assert_eq!(a.diag_iter(DiagOffset::Below(2)).cloned().collect::<Vec<_>>(), diags);

        {
            let diags_iter_mut = a.diag_iter_mut(DiagOffset::Main);
            for d in diags_iter_mut {
                *d = 1.0;
            }
        }

        for i in 0..3 {
            assert_eq!(a[[i,i]], 1.0);
        }
    }

    #[test]
    fn test_empty_matrix_diag() {
        let a: Matrix<f32> = matrix![];

        assert_eq!(None, a.diag_iter(DiagOffset::Main).next());
    }

    #[test]
    fn test_matrix_slice_diag() {
        let mut a = matrix![0.0, 1.0, 2.0, 3.0;
                            4.0, 5.0, 6.0, 7.0;
                            8.0, 9.0, 10.0, 11.0];
        {
            let b = MatrixSlice::from_matrix(&a, [0, 0], 2, 4);

            let diags = vec![0.0, 5.0];
            assert_eq!(b.diag_iter(DiagOffset::Main).cloned().collect::<Vec<_>>(), diags);
            let diags = vec![1.0, 6.0];
            assert_eq!(b.diag_iter(DiagOffset::Above(1)).cloned().collect::<Vec<_>>(), diags);
            let diags = vec![2.0, 7.0];
            assert_eq!(b.diag_iter(DiagOffset::Above(2)).cloned().collect::<Vec<_>>(), diags);
            let diags = vec![3.0];
            assert_eq!(b.diag_iter(DiagOffset::Above(3)).cloned().collect::<Vec<_>>(), diags);
            let diags = vec![4.0];
            assert_eq!(b.diag_iter(DiagOffset::Below(1)).cloned().collect::<Vec<_>>(), diags);
        }

        {
            let diags_iter_mut = a.diag_iter_mut(DiagOffset::Main);
            for d in diags_iter_mut {
                *d = 1.0;
            }
        }

        for i in 0..3 {
            assert_eq!(a[[i,i]], 1.0);
        }
    }

    #[test]
    fn test_matrix_diag_nth() {
        let a = matrix![0.0, 1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0, 7.0;
                        8.0, 9.0, 10.0, 11.0];

        let mut diags_iter = a.diag_iter(DiagOffset::Main);
        assert_eq!(0.0, *diags_iter.nth(0).unwrap());
        assert_eq!(10.0, *diags_iter.nth(1).unwrap());
        assert_eq!(None, diags_iter.next());

        let mut diags_iter = a.diag_iter(DiagOffset::Above(1));
        assert_eq!(6.0, *diags_iter.nth(1).unwrap());
        assert_eq!(11.0, *diags_iter.next().unwrap());
        assert_eq!(None, diags_iter.next());

        let mut diags_iter = a.diag_iter(DiagOffset::Below(1));
        assert_eq!(9.0, *diags_iter.nth(1).unwrap());
        assert_eq!(None, diags_iter.next());
    }

    #[test]
    fn test_matrix_slice_diag_nth() {
        let a = matrix![0.0, 1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0, 7.0;
                        8.0, 9.0, 10.0, 11.0];
        let b = MatrixSlice::from_matrix(&a, [0, 0], 2, 4);

        let mut diags_iter = b.diag_iter(DiagOffset::Main);
        assert_eq!(5.0, *diags_iter.nth(1).unwrap());;
        assert_eq!(None, diags_iter.next());

        let mut diags_iter = b.diag_iter(DiagOffset::Above(1));
        assert_eq!(6.0, *diags_iter.nth(1).unwrap());
        assert_eq!(None, diags_iter.next());

        let mut diags_iter = b.diag_iter(DiagOffset::Below(1));
        assert_eq!(4.0, *diags_iter.nth(0).unwrap());
        assert_eq!(None, diags_iter.next());
    }

    #[test]
    fn test_matrix_diag_last() {
        let a = matrix![0.0, 1.0, 2.0;
                        3.0, 4.0, 5.0;
                        6.0, 7.0, 8.0];

        let diags_iter = a.diag_iter(DiagOffset::Main);
        assert_eq!(8.0, *diags_iter.last().unwrap());

        let diags_iter = a.diag_iter(DiagOffset::Above(2));
        assert_eq!(2.0, *diags_iter.last().unwrap());

        let diags_iter = a.diag_iter(DiagOffset::Below(2));
        assert_eq!(6.0, *diags_iter.last().unwrap());
    }

    #[test]
    fn test_matrix_slice_diag_last() {
        let a = matrix![0.0, 1.0, 2.0;
                        3.0, 4.0, 5.0;
                        6.0, 7.0, 8.0];
        let b = MatrixSlice::from_matrix(&a, [0, 0], 3, 2);

        {
            let diags_iter = b.diag_iter(DiagOffset::Main);
            assert_eq!(4.0, *diags_iter.last().unwrap());
        }

        {
            let diags_iter = b.diag_iter(DiagOffset::Above(1));
            assert_eq!(1.0, *diags_iter.last().unwrap());
        }

        {
            let diags_iter = b.diag_iter(DiagOffset::Below(2));
            assert_eq!(6.0, *diags_iter.last().unwrap());
        }
    }

    #[test]
    fn test_matrix_diag_count() {
        let a = matrix![0.0, 1.0, 2.0;
                        3.0, 4.0, 5.0;
                        6.0, 7.0, 8.0];

        assert_eq!(3, a.diag_iter(DiagOffset::Main).count());
        assert_eq!(2, a.diag_iter(DiagOffset::Above(1)).count());
        assert_eq!(1, a.diag_iter(DiagOffset::Above(2)).count());
        assert_eq!(2, a.diag_iter(DiagOffset::Below(1)).count());
        assert_eq!(1, a.diag_iter(DiagOffset::Below(2)).count());

        let mut diags_iter = a.diag_iter(DiagOffset::Main);
        diags_iter.next();
        assert_eq!(2, diags_iter.count());
    }

    #[test]
    fn test_matrix_diag_size_hint() {
        let a = matrix![0.0, 1.0, 2.0;
                        3.0, 4.0, 5.0;
                        6.0, 7.0, 8.0];

        let mut diags_iter = a.diag_iter(DiagOffset::Main);
        assert_eq!((3, Some(3)), diags_iter.size_hint());
        diags_iter.next();

        assert_eq!((2, Some(2)), diags_iter.size_hint());
        diags_iter.next();
        diags_iter.next();

        assert_eq!((0, Some(0)), diags_iter.size_hint());
        assert_eq!(None, diags_iter.next());
        assert_eq!((0, Some(0)), diags_iter.size_hint());
    }

    #[test]
    fn test_matrix_cols() {
        let mut a = matrix![0, 1, 2, 3;
                            4, 5, 6, 7;
                            8, 9, 10, 11];
        let data = [[0, 4, 8], [1, 5, 9], [2, 6, 10], [3, 7, 11]];

        for (i, col) in a.col_iter().enumerate() {
            for (j, value) in col.iter().enumerate() {
                assert_eq!(data[i][j], *value);
            }
        }

        for (i, mut col) in a.col_iter_mut().enumerate() {
            for (j, value) in col.iter_mut().enumerate() {
                assert_eq!(data[i][j], *value);
            }
        }

        for mut col in a.col_iter_mut() {
            for r in col.iter_mut() {
                *r = 0;
            }
        }

        assert_eq!(a.into_vec(), vec![0; 12]);
    }

    #[test]
    fn test_matrix_slice_cols() {
        let a = matrix![0, 1, 2, 3;
                        4, 5, 6, 7;
                        8, 9, 10, 11];

        let b = MatrixSlice::from_matrix(&a, [0, 0], 3, 2);

        let data = [[0, 4, 8], [1, 5, 9]];

        for (i, col) in b.col_iter().enumerate() {
            for (j, value) in col.iter().enumerate() {
                assert_eq!(data[i][j], *value);
            }
        }
    }

    #[test]
    fn test_matrix_slice_mut_cols() {
        let mut a = matrix![0, 1, 2, 3;
                            4, 5, 6, 7;
                            8, 9, 10, 11];

        {
            let mut b = MatrixSliceMut::from_matrix(&mut a, [0, 0], 3, 2);

            let data = [[0, 4, 8], [1, 5, 9]];

            for (i, col) in b.col_iter().enumerate() {
                for (j, value) in col.iter().enumerate() {
                    assert_eq!(data[i][j], *value);
                }
            }

            for (i, mut col) in b.col_iter_mut().enumerate() {
                for (j, value) in col.iter_mut().enumerate() {
                    assert_eq!(data[i][j], *value);
                }
            }

            for mut col in b.col_iter_mut() {
                for r in col.iter_mut() {
                    *r = 0;
                }
            }
        }

        assert_eq!(a.into_vec(), vec![0, 0, 2, 3, 0, 0, 6, 7, 0, 0, 10, 11]);
    }

    #[test]
    fn test_matrix_cols_nth() {
        let a = matrix![0, 1, 2, 3;
                        4, 5, 6, 7;
                        8, 9, 10, 11];

        let mut col_iter = a.col_iter();

        let mut nth0 = col_iter.nth(0).unwrap().into_iter();

        assert_eq!(0, *nth0.next().unwrap());
        assert_eq!(4, *nth0.next().unwrap());
        assert_eq!(8, *nth0.next().unwrap());

        let mut nth1 = col_iter.nth(2).unwrap().into_iter();

        assert_eq!(3, *nth1.next().unwrap());
        assert_eq!(7, *nth1.next().unwrap());
        assert_eq!(11, *nth1.next().unwrap());

        assert!(col_iter.next().is_none());
    }

    #[test]
    fn test_matrix_cols_last() {
        let a = matrix![0, 1, 2, 3;
                        4, 5, 6, 7;
                        8, 9, 10, 11];

        let mut col_iter = a.col_iter().last().unwrap().into_iter();

        assert_eq!(3, *col_iter.next().unwrap());
        assert_eq!(7, *col_iter.next().unwrap());
        assert_eq!(11, *col_iter.next().unwrap());

        let mut col_iter = a.col_iter();

        col_iter.next();

        let mut last_col_iter = col_iter.last().unwrap().into_iter();

        assert_eq!(3, *last_col_iter.next().unwrap());
        assert_eq!(7, *last_col_iter.next().unwrap());
        assert_eq!(11, *last_col_iter.next().unwrap());

        let mut col_iter = a.col_iter();

        col_iter.next();
        col_iter.next();
        col_iter.next();
        col_iter.next();

        assert!(col_iter.last().is_none());
    }

    #[test]
    fn test_matrix_cols_count() {
        let a = matrix![0, 1, 2;
                        3, 4, 5;
                        6, 7, 8];

        let col_iter = a.col_iter();

        assert_eq!(3, col_iter.count());

        let mut col_iter_2 = a.col_iter();
        col_iter_2.next();
        assert_eq!(2, col_iter_2.count());
    }

    #[test]
    fn test_matrix_cols_size_hint() {
        let a = matrix![0, 1, 2;
                        3, 4, 5;
                        6, 7, 8];

        let mut col_iter = a.col_iter();

        assert_eq!((3, Some(3)), col_iter.size_hint());

        col_iter.next();

        assert_eq!((2, Some(2)), col_iter.size_hint());
        col_iter.next();
        col_iter.next();

        assert_eq!((0, Some(0)), col_iter.size_hint());

        assert!(col_iter.next().is_none());
        assert_eq!((0, Some(0)), col_iter.size_hint());
    }

    #[test]
    fn test_matrix_rows() {
        let mut a = matrix![0, 1, 2;
                            3, 4, 5;
                            6, 7, 8];

        let data = [[0, 1, 2], [3, 4, 5], [6, 7, 8]];

        for (i, row) in a.row_iter().enumerate() {
            assert_eq!(data[i], *row.raw_slice());
        }

        for (i, row) in a.row_iter_mut().enumerate() {
            assert_eq!(data[i], *row.raw_slice());
        }

        for mut row in a.row_iter_mut() {
            for r in row.raw_slice_mut() {
                *r = 0;
            }
        }

        assert_eq!(a.into_vec(), vec![0; 9]);
    }

    #[test]
    fn test_matrix_slice_rows() {
        let a = matrix![0, 1, 2;
                        3, 4, 5;
                        6, 7, 8];

        let b = MatrixSlice::from_matrix(&a, [0, 0], 2, 2);

        let data = [[0, 1], [3, 4]];

        for (i, row) in b.row_iter().enumerate() {
            assert_eq!(data[i], *row.raw_slice());
        }
    }

    #[test]
    fn test_matrix_slice_mut_rows() {
        let mut a = matrix![0, 1, 2;
                            3, 4, 5;
                            6, 7, 8];

        {
            let mut b = MatrixSliceMut::from_matrix(&mut a, [0, 0], 2, 2);

            let data = [[0, 1], [3, 4]];

            for (i, row) in b.row_iter().enumerate() {
                assert_eq!(data[i], *row.raw_slice());
            }

            for (i, row) in b.row_iter_mut().enumerate() {
                assert_eq!(data[i], *row.raw_slice());
            }

            for mut row in b.row_iter_mut() {
                for r in row.raw_slice_mut() {
                    *r = 0;
                }
            }
        }

        assert_eq!(a.into_vec(), vec![0, 0, 2, 0, 0, 5, 6, 7, 8]);
    }

    #[test]
    fn test_matrix_rows_nth() {
        let a = matrix![0, 1, 2;
                        3, 4, 5;
                        6, 7, 8];

        let mut row_iter = a.row_iter();

        assert_eq!([0, 1, 2], *row_iter.nth(0).unwrap().raw_slice());
        assert_eq!([6, 7, 8], *row_iter.nth(1).unwrap().raw_slice());

        assert!(row_iter.next().is_none());
    }

    #[test]
    fn test_matrix_rows_last() {
        let a = matrix![0, 1, 2;
                        3, 4, 5;
                        6, 7, 8];

        let row_iter = a.row_iter();

        assert_eq!([6, 7, 8], *row_iter.last().unwrap().raw_slice());

        let mut row_iter = a.row_iter();

        row_iter.next();
        assert_eq!([6, 7, 8], *row_iter.last().unwrap().raw_slice());

        let mut row_iter = a.row_iter();

        row_iter.next();
        row_iter.next();
        row_iter.next();
        row_iter.next();

        assert!(row_iter.last().is_none());
    }

    #[test]
    fn test_matrix_rows_count() {
        let a = matrix![0, 1, 2;
                        3, 4, 5;
                        6, 7, 8];

        let row_iter = a.row_iter();

        assert_eq!(3, row_iter.count());

        let mut row_iter_2 = a.row_iter();
        row_iter_2.next();
        assert_eq!(2, row_iter_2.count());
    }

    #[test]
    fn test_matrix_rows_size_hint() {
        let a = matrix![0, 1, 2;
                        3, 4, 5;
                        6, 7, 8];

        let mut row_iter = a.row_iter();

        assert_eq!((3, Some(3)), row_iter.size_hint());

        row_iter.next();

        assert_eq!((2, Some(2)), row_iter.size_hint());
        row_iter.next();
        row_iter.next();

        assert_eq!((0, Some(0)), row_iter.size_hint());

        assert!(row_iter.next().is_none());
        assert_eq!((0, Some(0)), row_iter.size_hint());
    }

    #[test]
    fn into_iter_compile() {
        let a = Matrix::ones(3, 3) * 2.;
        let mut b = MatrixSlice::from_matrix(&a, [1, 1], 2, 2);

        for _ in b {
        }

        for _ in &b {
        }

        for _ in &mut b {
        }
    }

    #[test]
    fn into_iter_mut_compile() {
        let mut a = Matrix::<f32>::ones(3, 3) * 2.;

        {
            let b = MatrixSliceMut::from_matrix(&mut a, [1, 1], 2, 2);

            for v in b {
                *v = 1.0;
            }
        }

        {
            let b = MatrixSliceMut::from_matrix(&mut a, [1, 1], 2, 2);

            for _ in &b {
            }
        }

        {
            let mut b = MatrixSliceMut::from_matrix(&mut a, [1, 1], 2, 2);

            for v in &mut b {
                *v = 1.0;
            }
        }
    }

    #[test]
    fn iter_matrix_small_matrices() {
        {
            let x = matrix![ 1 ];
            let mut i = x.iter();
            assert_eq!(i.next(), Some(&1));
            assert_eq!(i.next(), None);
        }

        {
            let x = matrix![ 1, 2 ];
            let mut i = x.iter();
            assert_eq!(i.next(), Some(&1));
            assert_eq!(i.next(), Some(&2));
            assert_eq!(i.next(), None);
        }

        {
            let x = matrix![ 1; 2 ];
            let mut i = x.iter();
            assert_eq!(i.next(), Some(&1));
            assert_eq!(i.next(), Some(&2));
            assert_eq!(i.next(), None);
        }

        {
            let x = matrix![ 1, 2;
                             3, 4 ];
            let mut i = x.iter();
            assert_eq!(i.next(), Some(&1));
            assert_eq!(i.next(), Some(&2));
            assert_eq!(i.next(), Some(&3));
            assert_eq!(i.next(), Some(&4));
            assert_eq!(i.next(), None);
        }
    }

    #[test]
    fn iter_matrix_slice() {
        let x = matrix![1, 2, 3;
                        4, 5, 6;
                        7, 8, 9];

        // Helper to simplify writing the below tests.
        // Note that .collect() is an implicit test of .next(),
        // including checking that None is returned when there
        // are no more elements.
        let collect_slice = |(i, j), rows, cols| {
            x.sub_slice([i, j], rows, cols)
             .iter()
             .cloned()
             .collect::<Vec<_>>()
        };

        {
            // Zero elements
            for i in 0 .. 2 {
                for j in 0 .. 2 {
                    let y = x.sub_slice([i, j], 0, 0);
                    assert!(y.iter().next().is_none());
                }
            }

        }

        {
            // One element
            for i in 0 .. 2 {
                for j in 0 .. 2 {
                    let y = x.sub_slice([i, j], 1, 1);
                    assert_eq!(y.iter().next(), Some(&x[[i, j]]));
                }
            }
        }

        {
            // 1x2 sub slices
            assert_eq!(collect_slice((0, 0), 1, 2), vec![1, 2]);
            assert_eq!(collect_slice((0, 1), 1, 2), vec![2, 3]);
            assert_eq!(collect_slice((1, 0), 1, 2), vec![4, 5]);
            assert_eq!(collect_slice((1, 1), 1, 2), vec![5, 6]);
            assert_eq!(collect_slice((2, 0), 1, 2), vec![7, 8]);
            assert_eq!(collect_slice((2, 1), 1, 2), vec![8, 9]);
        }

        {
            // 2x1 sub slices
            assert_eq!(collect_slice((0, 0), 2, 1), vec![1, 4]);
            assert_eq!(collect_slice((1, 0), 2, 1), vec![4, 7]);
            assert_eq!(collect_slice((0, 1), 2, 1), vec![2, 5]);
            assert_eq!(collect_slice((1, 1), 2, 1), vec![5, 8]);
            assert_eq!(collect_slice((0, 2), 2, 1), vec![3, 6]);
            assert_eq!(collect_slice((1, 2), 2, 1), vec![6, 9]);
        }

        {
            // 2x2 sub slices
            assert_eq!(collect_slice((0, 0), 2, 2), vec![1, 2, 4, 5]);
            assert_eq!(collect_slice((0, 1), 2, 2), vec![2, 3, 5, 6]);
            assert_eq!(collect_slice((1, 0), 2, 2), vec![4, 5, 7, 8]);
            assert_eq!(collect_slice((1, 1), 2, 2), vec![5, 6, 8, 9]);
        }
    }

    #[test]
    fn iter_empty_matrix() {
        {
            let x = Matrix::<u32>::zeros(0, 0);
            assert!(x.iter().next().is_none());
        }

        {
            let x = Matrix::<u32>::zeros(1, 0);
            assert!(x.iter().next().is_none());
        }

        {
            let x = Matrix::<u32>::zeros(0, 1);
            assert!(x.iter().next().is_none());
        }
    }
}
