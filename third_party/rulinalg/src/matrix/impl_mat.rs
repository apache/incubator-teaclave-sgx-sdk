use std::any::Any;
use std::fmt;
use libnum::{One, Zero, Float, FromPrimitive};
use std::vec::*;

use super::{Matrix};
use super::{Axes};
use super::base::BaseMatrix;
use error::{Error, ErrorKind};
use vector::Vector;
use matrix::decomposition::PartialPivLu;

impl<T> Matrix<T> {
    /// Constructor for Matrix struct.
    ///
    /// Requires both the row and column dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// let mat = Matrix::new(2,2, vec![1.0, 2.0, 3.0, 4.0]);
    ///
    /// assert_eq!(mat.rows(), 2);
    /// assert_eq!(mat.cols(), 2);
    /// ```
    ///
    /// # Panics
    ///
    /// - The input data does not match the given dimensions.
    pub fn new<U: Into<Vec<T>>>(rows: usize, cols: usize, data: U) -> Matrix<T> {
        let our_data = data.into();

        assert!(cols * rows == our_data.len(),
                "Data does not match given dimensions.");
        Matrix {
            cols: cols,
            rows: rows,
            data: our_data,
        }
    }

    /// Constructor for Matrix struct that takes a function `f`
    /// and constructs a new matrix such that `A_ij = f(j, i)`,
    /// where `i` is the row index and `j` the column index.
    ///
    /// Requires both the row and column dimensions
    /// as well as a generating function.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::{Matrix, BaseMatrix};
    ///
    /// // Let's assume you have an array of "things" for
    /// // which you want to generate a distance matrix:
    /// let things: [i32; 3] = [1, 2, 3];
    /// let distances: Matrix<f64> = Matrix::from_fn(things.len(), things.len(), |col, row| {
    ///     (things[col] - things[row]).abs().into()
    /// });
    ///
    /// assert_eq!(distances.rows(), 3);
    /// assert_eq!(distances.cols(), 3);
    /// assert_eq!(distances.data(), &vec![
    ///     0.0, 1.0, 2.0,
    ///     1.0, 0.0, 1.0,
    ///     2.0, 1.0, 0.0,
    /// ]);
    /// ```
    /// # Warning
    ///
    /// _This function will be changed in a future release so that `A_ij = f(i, j)` - to be consistent
    /// with the rest of the library._
    pub fn from_fn<F>(rows: usize, cols: usize, mut f: F) -> Matrix<T>
        where F: FnMut(usize, usize) -> T
    {
        let mut data = Vec::with_capacity(rows * cols);
        for row in 0..rows {
            for col in 0..cols {
                data.push(f(col, row));
            }
        }
        Matrix::new(rows, cols, data)
    }

    /// Returns a non-mutable reference to the underlying data.
    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    /// Returns a mutable slice of the underlying data.
    pub fn mut_data(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Consumes the Matrix and returns the Vec of data.
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }
}

impl<T: Clone + Zero> Matrix<T> {
    /// Constructs matrix of all zeros.
    ///
    /// Requires both the row and the column dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::Matrix;
    ///
    /// let mat = Matrix::<f64>::zeros(2,3);
    /// ```
    pub fn zeros(rows: usize, cols: usize) -> Matrix<T> {
        Matrix {
            cols: cols,
            rows: rows,
            data: vec![T::zero(); cols*rows],
        }
    }

    /// Constructs matrix with given diagonal.
    ///
    /// Requires slice of diagonal elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::Matrix;
    ///
    /// let mat = Matrix::from_diag(&vec![1.0,2.0,3.0,4.0]);
    /// ```
    pub fn from_diag(diag: &[T]) -> Matrix<T> {
        let size = diag.len();
        let mut data = vec![T::zero(); size * size];

        for (i, item) in diag.into_iter().enumerate().take(size) {
            data[i * (size + 1)] = item.clone();
        }

        Matrix {
            cols: size,
            rows: size,
            data: data,
        }
    }
}

impl<T: Clone + One> Matrix<T> {
    /// Constructs matrix of all ones.
    ///
    /// Requires both the row and the column dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::Matrix;
    ///
    /// let mat = Matrix::<f64>::ones(2,3);
    /// ```
    pub fn ones(rows: usize, cols: usize) -> Matrix<T> {
        Matrix {
            cols: cols,
            rows: rows,
            data: vec![T::one(); cols*rows],
        }
    }
}

impl<T: Clone + Zero + One> Matrix<T> {
    /// Constructs the identity matrix.
    ///
    /// Requires the size of the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// use rulinalg::matrix::Matrix;
    ///
    /// let I = Matrix::<f64>::identity(4);
    /// ```
    pub fn identity(size: usize) -> Matrix<T> {
        let mut data = vec![T::zero(); size * size];

        for i in 0..size {
            data[(i * (size + 1)) as usize] = T::one();
        }

        Matrix {
            cols: size,
            rows: size,
            data: data,
        }
    }
}

impl<T: Float + FromPrimitive> Matrix<T> {
    /// The mean of the matrix along the specified axis.
    ///
    /// - Axis Row - Arithmetic mean of rows.
    /// - Axis Col - Arithmetic mean of columns.
    ///
    /// Calling `mean()` on an empty matrix will return an empty matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, Axes};
    ///
    /// let a = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    ///
    /// let c = a.mean(Axes::Row);
    /// assert_eq!(c, vector![2.0, 3.0]);
    ///
    /// let d = a.mean(Axes::Col);
    /// assert_eq!(d, vector![1.5, 3.5]);
    /// # }
    /// ```
    pub fn mean(&self, axis: Axes) -> Vector<T> {
        if self.data.len() == 0 {
            // If the matrix is empty, there are no means to calculate.
            return Vector::new(vec![]);
        }

        let m: Vector<T>;
        let n: T;
        match axis {
            Axes::Row => {
                m = self.sum_rows();
                n = FromPrimitive::from_usize(self.rows).unwrap();
            }
            Axes::Col => {
                m = self.sum_cols();
                n = FromPrimitive::from_usize(self.cols).unwrap();
            }
        }
        m / n
    }

    /// The variance of the matrix along the specified axis.
    ///
    /// - Axis Row - Sample variance of rows.
    /// - Axis Col - Sample variance of columns.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::{Matrix, Axes};
    ///
    /// let a = matrix![1.0, 2.0;
    ///                 3.0, 4.0];
    ///
    /// let c = a.variance(Axes::Row).unwrap();
    /// assert_eq!(c, vector![2.0, 2.0]);
    ///
    /// let d = a.variance(Axes::Col).unwrap();
    /// assert_eq!(d, vector![0.5, 0.5]);
    /// # }
    /// ```
    ///
    /// # Failures
    ///
    /// - There are one or fewer row/columns in the working axis.
    pub fn variance(&self, axis: Axes) -> Result<Vector<T>, Error> {
        let mean = self.mean(axis);

        let n: usize;
        let m: usize;

        match axis {
            Axes::Row => {
                n = self.rows;
                m = self.cols;
            }
            Axes::Col => {
                n = self.cols;
                m = self.rows;
            }
        }

        if n < 2 {
            return Err(Error::new(ErrorKind::InvalidArg,
                                  "There must be at least two rows or columns in the working \
                                   axis."));
        }

        let mut variance = Vector::zeros(m);

        for i in 0..n {
            let mut t = Vec::<T>::with_capacity(m);

            unsafe {
                t.set_len(m);

                for j in 0..m {
                    t[j] = match axis {
                        Axes::Row => *self.data.get_unchecked(i * m + j),
                        Axes::Col => *self.data.get_unchecked(j * n + i),
                    }

                }
            }

            let diff = Vector::new(t) - &mean;

            variance = variance + diff.elemul(&diff);
        }

        let var_size: T = FromPrimitive::from_usize(n - 1).unwrap();
        Ok(variance / var_size)
    }
}

impl<T: Any + Float> Matrix<T> {
    /// Solves the equation `Ax = y`.
    ///
    /// Requires a Vector `y` as input.
    ///
    /// The method performs an LU decomposition internally,
    /// consuming the matrix in the process. If solving
    /// the same system for multiple right-hand sides
    /// is desired, see
    /// [PartialPivLu](decomposition/struct.PartialPivLu.html).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::Matrix;
    /// use rulinalg::vector::Vector;
    ///
    /// let a = matrix![2.0, 3.0;
    ///                 1.0, 2.0];
    /// let y = vector![13.0, 8.0];
    ///
    /// let x = a.solve(y).unwrap();
    ///
    /// assert_eq!(x, vector![2.0, 3.0]);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrix column count and vector size are different.
    /// - The matrix is not square.
    ///
    /// # Failures
    ///
    /// - The matrix cannot be decomposed into an LUP form to solve.
    /// - There is no valid solution as the matrix is singular.
    pub fn solve(self, y: Vector<T>) -> Result<Vector<T>, Error> {
        PartialPivLu::decompose(self)?.solve(y)
    }

    /// Computes the inverse of the matrix.
    ///
    /// Internally performs an LU decomposition.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// use rulinalg::matrix::Matrix;
    ///
    /// let a = matrix![2., 3.;
    ///                 1., 2.];
    /// let inv = a.clone().inverse().expect("This matrix should have an inverse!");
    ///
    /// let I = a * inv;
    ///
    /// assert_matrix_eq!(I, matrix![1.0, 0.0; 0.0, 1.0]);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrix is not square.
    ///
    /// # Failures
    ///
    /// - The matrix could not be LUP decomposed.
    /// - The matrix has zero determinant.
    pub fn inverse(self) -> Result<Matrix<T>, Error> {
        PartialPivLu::decompose(self)?.inverse()
    }

    /// Computes the determinant of the matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate rulinalg; fn main() {
    /// let a = matrix![1.0, 2.0, 0.0;
    ///                 0.0, 3.0, 4.0;
    ///                 5.0, 1.0, 2.0];
    ///
    /// let det = a.det();
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// - The matrix is not square.
    pub fn det(self) -> T {
        assert!(self.rows == self.cols, "Matrix is not square.");

        let n = self.cols;

        if self.is_diag() {
            self.diag().cloned().fold(T::one(), |d, entry| d * entry)
        } else if n == 2 {
            (self[[0, 0]] * self[[1, 1]]) - (self[[0, 1]] * self[[1, 0]])
        } else if n == 3 {
            (self[[0, 0]] * self[[1, 1]] * self[[2, 2]]) +
            (self[[0, 1]] * self[[1, 2]] * self[[2, 0]]) +
            (self[[0, 2]] * self[[1, 0]] * self[[2, 1]]) -
            (self[[0, 0]] * self[[1, 2]] * self[[2, 1]]) -
            (self[[0, 1]] * self[[1, 0]] * self[[2, 2]]) -
            (self[[0, 2]] * self[[1, 1]] * self[[2, 0]])
        } else {
            PartialPivLu::decompose(self).map(|lu| lu.det())
                                         .unwrap_or(T::zero())
        }
    }
}

impl<T: fmt::Display> fmt::Display for Matrix<T> {
    /// Formats the Matrix for display.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut max_datum_width = 0;
        for datum in &self.data {
            let datum_width = match f.precision() {
                Some(places) => format!("{:.1$}", datum, places).len(),
                None => format!("{}", datum).len(),
            };
            if datum_width > max_datum_width {
                max_datum_width = datum_width;
            }
        }
        let width = max_datum_width;

        fn write_row<T>(f: &mut fmt::Formatter,
                        row: &[T],
                        left_delimiter: &str,
                        right_delimiter: &str,
                        width: usize)
                        -> Result<(), fmt::Error>
            where T: fmt::Display
        {
            try!(write!(f, "{}", left_delimiter));
            for (index, datum) in row.iter().enumerate() {
                match f.precision() {
                    Some(places) => {
                        try!(write!(f, "{:1$.2$}", datum, width, places));
                    }
                    None => {
                        try!(write!(f, "{:1$}", datum, width));
                    }
                }
                if index < row.len() - 1 {
                    try!(write!(f, " "));
                }
            }
            write!(f, "{}", right_delimiter)
        }

        match self.rows {
            1 => write_row(f, &self.data, "[", "]", width),
            _ => {
                try!(write_row(f,
                               &self.data[0..self.cols],
                               "⎡", // \u{23a1} LEFT SQUARE BRACKET UPPER CORNER
                               "⎤", // \u{23a4} RIGHT SQUARE BRACKET UPPER CORNER
                               width));
                try!(f.write_str("\n"));
                for row_index in 1..self.rows - 1 {
                    try!(write_row(f,
                                   &self.data[row_index * self.cols..(row_index + 1) * self.cols],
                                   "⎢", // \u{23a2} LEFT SQUARE BRACKET EXTENSION
                                   "⎥", // \u{23a5} RIGHT SQUARE BRACKET EXTENSION
                                   width));
                    try!(f.write_str("\n"));
                }
                write_row(f,
                          &self.data[(self.rows - 1) * self.cols..self.rows * self.cols],
                          "⎣", // \u{23a3} LEFT SQUARE BRACKET LOWER CORNER
                          "⎦", // \u{23a6} RIGHT SQUARE BRACKET LOWER CORNER
                          width)
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use matrix::{Axes, BaseMatrix, Matrix};

    #[test]
    fn test_new_mat() {
        let a = vec![2.0; 9];
        let b = Matrix::new(3, 3, a);

        assert_eq!(b.rows(), 3);
        assert_eq!(b.cols(), 3);
        assert_eq!(b.into_vec(), vec![2.0; 9]);
    }

    #[test]
    #[should_panic]
    fn test_new_mat_bad_data() {
        let a = vec![2.0; 7];
        let _ = Matrix::new(3, 3, a);
    }

    #[test]
    fn test_new_mat_from_fn() {
        let mut counter = 0;
        let m: Matrix<usize> = Matrix::from_fn(3, 2, |_, _| {
            let value = counter;
            counter += 1;
            value
        });
        assert!(m.rows() == 3);
        assert!(m.cols() == 2);
        assert!(m.data == vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_equality() {
        // well, "PartialEq", at least
        let a = matrix![1., 2., 3.;
                        4., 5., 6.];
        let a_redux = a.clone();
        assert_eq!(a, a_redux);
    }

    #[test]
    fn test_new_from_slice() {
        let data_vec: Vec<u32> = vec![1, 2, 3, 4, 5, 6];
        let data_slice: &[u32] = &data_vec[..];
        let from_vec = Matrix::new(3, 2, data_vec.clone());
        let from_slice = Matrix::new(3, 2, data_slice);
        assert_eq!(from_vec, from_slice);
    }

    #[test]
    fn test_display_formatting() {
        let first_matrix = matrix![1, 2, 3;
                                   4, 5, 6];
        let first_expectation = "⎡1 2 3⎤\n⎣4 5 6⎦";
        assert_eq!(first_expectation, format!("{}", first_matrix));

        let second_matrix = matrix![3.14, 2.718, 1.414;
                                    2.503, 4.669, 1.202;
                                    1.618, 0.5772, 1.3;
                                    2.68545, 1.282, 10000.];
        let second_exp = "⎡   3.14   2.718   1.414⎤\n⎢  2.503   4.669   1.202⎥\n⎢  \
                        1.618  0.5772     1.3⎥\n⎣2.68545   1.282   10000⎦";
        assert_eq!(second_exp, format!("{}", second_matrix));
    }

    #[test]
    fn test_single_row_display_formatting() {
        let one_row_matrix = matrix![1, 2, 3, 4];
        assert_eq!("[1 2 3 4]", format!("{}", one_row_matrix));
    }

    #[test]
    fn test_display_formatting_precision() {
        let our_matrix = matrix![1.2, 1.23, 1.234;
                                 1.2345, 1.23456, 1.234567];
        let expectations = vec!["⎡1.2 1.2 1.2⎤\n⎣1.2 1.2 1.2⎦",

                                "⎡1.20 1.23 1.23⎤\n⎣1.23 1.23 1.23⎦",

                                "⎡1.200 1.230 1.234⎤\n⎣1.234 1.235 1.235⎦",

                                "⎡1.2000 1.2300 1.2340⎤\n⎣1.2345 1.2346 1.2346⎦"];

        for (places, &expectation) in (1..5).zip(expectations.iter()) {
            assert_eq!(expectation, format!("{:.1$}", our_matrix, places));
        }
    }

    #[test]
    fn test_matrix_index_mut() {
        let mut a = Matrix::ones(3, 3) * 2.0;

        a[[0, 0]] = 13.0;

        for i in 1..9 {
            assert_eq!(a.data()[i], 2.0);
        }

        assert_eq!(a[[0, 0]], 13.0);
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
    fn matrix_det() {
        let a = matrix![2., 3.;
                        1., 2.];
        let b = a.det();

        assert_eq!(b, 1.);

        let c = matrix![1., 2., 3.;
                        4., 5., 6.;
                        7., 8., 9.];
        let d = c.det();

        assert_eq!(d, 0.);

        let e: Matrix<f64> = matrix![1., 2., 3., 4., 5.;
                                     3., 0., 4., 5., 6.;
                                     2., 1., 2., 3., 4.;
                                     0., 0., 0., 6., 5.;
                                     0., 0., 0., 5., 6.];

        let f = e.det();

        assert_scalar_eq!(f, 99.0, comp = float);

        let g: Matrix<f64> = matrix![1., 2., 3., 4.;
                                     0., 0., 0., 0.;
                                     0., 0., 0., 0.;
                                     0., 0., 0., 0.];
        let h = g.det();
        assert_eq!(h, 0.);
    }

    #[test]
    fn matrix_solve() {
        let a = matrix![2., 3.;
                        1., 2.];

        let y = vector![8., 5.];

        let x = a.solve(y).unwrap();

        assert_eq!(x.size(), 2);

        assert_eq!(x[0], 1.);
        assert_eq!(x[1], 2.);
    }

    #[test]
    fn create_mat_zeros() {
        let a = Matrix::<f32>::zeros(10, 10);

        assert_eq!(a.rows(), 10);
        assert_eq!(a.cols(), 10);

        for i in 0..10 {
            for j in 0..10 {
                assert_eq!(a[[i, j]], 0.0);
            }
        }
    }

    #[test]
    fn create_mat_identity() {
        let a = Matrix::<f32>::identity(4);

        assert_eq!(a.rows(), 4);
        assert_eq!(a.cols(), 4);

        assert_eq!(a[[0, 0]], 1.0);
        assert_eq!(a[[1, 1]], 1.0);
        assert_eq!(a[[2, 2]], 1.0);
        assert_eq!(a[[3, 3]], 1.0);

        assert_eq!(a[[0, 1]], 0.0);
        assert_eq!(a[[2, 1]], 0.0);
        assert_eq!(a[[3, 0]], 0.0);
    }

    #[test]
    fn create_mat_diag() {
        let a = Matrix::from_diag(&[1.0, 2.0, 3.0, 4.0]);

        assert_eq!(a.rows(), 4);
        assert_eq!(a.cols(), 4);

        assert_eq!(a[[0, 0]], 1.0);
        assert_eq!(a[[1, 1]], 2.0);
        assert_eq!(a[[2, 2]], 3.0);
        assert_eq!(a[[3, 3]], 4.0);

        assert_eq!(a[[0, 1]], 0.0);
        assert_eq!(a[[2, 1]], 0.0);
        assert_eq!(a[[3, 0]], 0.0);
    }

    #[test]
    fn test_empty_mean() {
        let a: Matrix<f64> = matrix![];

        let c = a.mean(Axes::Row);
        assert_eq!(*c.data(), vec![]);

        let d = a.mean(Axes::Col);
        assert_eq!(*d.data(), vec![]);
    }

    #[test]
    fn test_invalid_variance() {
        // Only one row
        let a: Matrix<f32> = matrix![1.0, 2.0];

        let a_row = a.variance(Axes::Row);
        assert!(a_row.is_err());

        let a_col = a.variance(Axes::Col).unwrap();
        assert_eq!(*a_col.data(), vec![0.5]);

        // Only one column
        let b: Matrix<f32> = matrix![1.0; 2.0];

        let b_row = b.variance(Axes::Row).unwrap();
        assert_eq!(*b_row.data(), vec![0.5]);

        let b_col = b.variance(Axes::Col);
        assert!(b_col.is_err());

        // Empty matrix
        let d: Matrix<f32> = matrix![];

        let d_row = d.variance(Axes::Row);
        assert!(d_row.is_err());

        let d_col = d.variance(Axes::Col);
        assert!(d_col.is_err());
    }
}
