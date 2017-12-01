use matrix::{Matrix, MatrixSlice, MatrixSliceMut};
use std::marker::PhantomData;

impl<'a, T> MatrixSlice<'a, T> {
    /// Produce a `MatrixSlice` from a `Matrix`
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, MatrixSlice};
    ///
    /// let a = Matrix::new(3,3, (0..9).collect::<Vec<usize>>());
    /// let slice = MatrixSlice::from_matrix(&a, [1,1], 2, 2);
    /// ```
    pub fn from_matrix(mat: &'a Matrix<T>,
                       start: [usize; 2],
                       rows: usize,
                       cols: usize)
                       -> MatrixSlice<T> {
        assert!(start[0] + rows <= mat.rows,
                "View dimensions exceed matrix dimensions.");
        assert!(start[1] + cols <= mat.cols,
                "View dimensions exceed matrix dimensions.");
        unsafe {
            MatrixSlice {
                ptr: mat.data().get_unchecked(start[0] * mat.cols + start[1]) as *const T,
                rows: rows,
                cols: cols,
                row_stride: mat.cols,
                marker: PhantomData::<&'a T>,
            }
        }
    }

    /// Creates a `MatrixSlice` from raw parts.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::MatrixSlice;
    ///
    /// let mut a = vec![4.0; 16];
    ///
    /// unsafe {
    ///     // Create a matrix slice with 3 rows, and 3 cols
    ///     // The row stride of 4 specifies the distance between the start of each row in the data.
    ///     let b = MatrixSlice::from_raw_parts(a.as_ptr(), 3, 3, 4);
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// The pointer must be followed by a contiguous slice of data larger than `row_stride * rows`.
    /// If not then other operations will produce undefined behaviour.
    ///
    /// Additionally `cols` should be less than the `row_stride`. It is possible to use this
    /// function safely whilst violating this condition. So long as
    /// `max(cols, row_stride) * rows` is less than the data size.
    pub unsafe fn from_raw_parts(ptr: *const T,
                                 rows: usize,
                                 cols: usize,
                                 row_stride: usize)
                                 -> MatrixSlice<'a, T> {
        MatrixSlice {
            ptr: ptr,
            rows: rows,
            cols: cols,
            row_stride: row_stride,
            marker: PhantomData::<&'a T>,
        }
    }
}

impl<'a, T> MatrixSliceMut<'a, T> {
    /// Produce a `MatrixSliceMut` from a mutable `Matrix`
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, MatrixSliceMut};
    ///
    /// let mut a = Matrix::new(3,3, (0..9).collect::<Vec<usize>>());
    /// let slice = MatrixSliceMut::from_matrix(&mut a, [1,1], 2, 2);
    /// ```
    pub fn from_matrix(mat: &'a mut Matrix<T>,
                       start: [usize; 2],
                       rows: usize,
                       cols: usize)
                       -> MatrixSliceMut<T> {
        assert!(start[0] + rows <= mat.rows,
                "View dimensions exceed matrix dimensions.");
        assert!(start[1] + cols <= mat.cols,
                "View dimensions exceed matrix dimensions.");

        let mat_cols = mat.cols;

        unsafe {
            MatrixSliceMut {
                ptr: mat.mut_data().get_unchecked_mut(start[0] * mat_cols + start[1]) as *mut T,
                rows: rows,
                cols: cols,
                row_stride: mat_cols,
                marker: PhantomData::<&'a mut T>,
            }
        }
    }

    /// Creates a `MatrixSliceMut` from raw parts.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::MatrixSliceMut;
    ///
    /// let mut a = vec![4.0; 16];
    ///
    /// unsafe {
    ///     // Create a mutable matrix slice with 3 rows, and 3 cols
    ///     // The row stride of 4 specifies the distance between the start of each row in the data.
    ///     let b = MatrixSliceMut::from_raw_parts(a.as_mut_ptr(), 3, 3, 4);
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// The pointer must be followed by a contiguous slice of data larger than `row_stride * rows`.
    /// If not then other operations will produce undefined behaviour.
    ///
    /// Additionally `cols` should be less than the `row_stride`. It is possible to use this
    /// function safely whilst violating this condition. So long as
    /// `max(cols, row_stride) * rows` is less than the data size.
    pub unsafe fn from_raw_parts(ptr: *mut T,
                                 rows: usize,
                                 cols: usize,
                                 row_stride: usize)
                                 -> MatrixSliceMut<'a, T> {
        MatrixSliceMut {
            ptr: ptr,
            rows: rows,
            cols: cols,
            row_stride: row_stride,
            marker: PhantomData::<&'a mut T>,
        }
    }
}

#[cfg(test)]
mod tests {

    use matrix::{Matrix, MatrixSlice, MatrixSliceMut, BaseMatrix, Axes};

    #[test]
    #[should_panic]
    fn make_slice_bad_dim() {
        let a = Matrix::ones(3, 3) * 2.0;
        let _ = MatrixSlice::from_matrix(&a, [1, 1], 3, 2);
    }

    #[test]
    fn make_slice() {
        let a = Matrix::ones(3, 3) * 2.0;
        let b = MatrixSlice::from_matrix(&a, [1, 1], 2, 2);

        assert_eq!(b.rows(), 2);
        assert_eq!(b.cols(), 2);
    }

    #[test]
    fn make_slice_mut() {
        let mut a = Matrix::ones(3, 3) * 2.0;
        {
            let mut b = MatrixSliceMut::from_matrix(&mut a, [1, 1], 2, 2);
            assert_eq!(b.rows(), 2);
            assert_eq!(b.cols(), 2);
            b += 2.0;
        }
        let exp = matrix![2.0, 2.0, 2.0;
                          2.0, 4.0, 4.0;
                          2.0, 4.0, 4.0];
        assert_matrix_eq!(a, exp);

    }

    #[test]
    fn matrix_min_max() {
        let a = matrix![1., 3., 5., 4.;
                        2., 4., 7., 1.;
                        1., 1., 0., 0.];
        assert_eq!(a.min(Axes::Col), vector![1., 1., 0.]);
        assert_eq!(a.min(Axes::Row), vector![1., 1., 0., 0.]);

        assert_eq!(a.max(Axes::Col), vector![5., 7., 1.]);
        assert_eq!(a.max(Axes::Row), vector![2., 4., 7., 4.]);

        let r = matrix![1., 3., 5., 4.];
        assert_eq!(r.min(Axes::Col), vector![1.]);
        assert_eq!(r.min(Axes::Row), vector![1., 3., 5., 4.]);

        assert_eq!(r.max(Axes::Col), vector![5.]);
        assert_eq!(r.max(Axes::Row), vector![1., 3., 5., 4.]);

        let c = matrix![1.; 2.; 3.];
        assert_eq!(c.min(Axes::Col), vector![1., 2., 3.]);
        assert_eq!(c.min(Axes::Row), vector![1.]);

        assert_eq!(c.max(Axes::Col), vector![1., 2., 3.]);
        assert_eq!(c.max(Axes::Row), vector![3.]);

        let t = matrix![1., 2.; 0., 1.];
        assert_eq!(t.min(Axes::Col), vector![1., 0.]);
        assert_eq!(t.min(Axes::Row), vector![0., 1.]);

        assert_eq!(t.max(Axes::Col), vector![2., 1.]);
        assert_eq!(t.max(Axes::Row), vector![1., 2.]);
    }
}
