/// The `matrix!` macro enables easy construction of small matrices.
///
/// This is particularly useful when writing tests involving matrices.
/// Note that the macro is just a convenient wrapper around the Matrix
/// constructors, and as a result the matrix is still allocated on the
/// heap.
///
/// Rows are separated by semi-colons, while commas separate the columns.
/// Users of MATLAB will find this style familiar. If the dimensions
/// don't match, the macro will fail to compile.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rulinalg;
///
/// # fn main() {
/// // Construct a 3x3 matrix of f64
/// let mat = matrix![1.0, 2.0, 3.0;
///                   4.0, 5.0, 6.0;
///                   7.0, 8.0, 9.0];
/// # }
/// ```
///
/// To construct matrices of other types, specify the type by
/// the usual Rust syntax:
///
/// ```
/// # #[macro_use]
/// # extern crate rulinalg;
/// # fn main() {
/// use rulinalg::matrix::Matrix;
///
/// // Construct a 2x3 matrix of f32
/// let mat: Matrix<f32> = matrix![1.0, 2.0, 3.0;
///                                4.0, 5.0, 6.0];
/// // Or
/// let mat = matrix![1.0, 2.0, 3.0;
///                   4.0, 5.0, 6.0f32];
/// # }
/// ```
///

#[macro_export]
macro_rules! matrix {
    () => {
        {
            // Handle the case when called with no arguments, i.e. matrix![]
            use $crate::matrix::Matrix;
            Matrix::new(0, 0, vec![])
        }
    };
    ($( $( $x: expr ),*);*) => {
        {
            use $crate::matrix::Matrix;
            let data_as_nested_array = [ $( [ $($x),* ] ),* ];
            let rows = data_as_nested_array.len();
            let cols = data_as_nested_array[0].len();
            let data_as_flat_array: Vec<_> = data_as_nested_array.into_iter()
                .flat_map(|row| row.into_iter())
                .cloned()
                .collect();
            Matrix::new(rows, cols, data_as_flat_array)
        }
    }
}

#[cfg(test)]
mod tests {
    use matrix::{Matrix, BaseMatrix};

    #[test]
    fn matrix_macro() {
        {
            // An arbitrary rectangular matrix
            let mat = matrix![1, 2, 3;
                              4, 5, 6];
            assert_eq!(2, mat.rows());
            assert_eq!(3, mat.cols());
            assert_eq!(&vec![1, 2, 3, 4, 5, 6], mat.data());
        }

        {
            // A single row
            let mat = matrix![1, 2, 3];
            assert_eq!(1, mat.rows());
            assert_eq!(3, mat.cols());
            assert_eq!(&vec![1, 2, 3], mat.data());
        }

        {
            // A single element
            let mat = matrix![1];
            assert_eq!(1, mat.rows());
            assert_eq!(1, mat.cols());
            assert_eq!(&vec![1], mat.data());
        }

        {
            // A floating point matrix
            let mat = matrix![1.0, 2.0, 3.0;
                              4.0, 5.0, 6.0;
                              7.0, 8.0, 9.0];
            let ref expected_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
            assert_eq!(3, mat.rows());
            assert_eq!(3, mat.cols());
            assert_eq!(expected_data, mat.data());
        }
    }

    #[test]
    fn matrix_macro_empty_mat() {
        let mat: Matrix<f64> = matrix![];

        assert_eq!(0, mat.rows());
        assert_eq!(0, mat.cols());
    }

}
