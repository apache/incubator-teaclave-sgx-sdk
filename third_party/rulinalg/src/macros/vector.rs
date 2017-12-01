/// The `vector!` macro enables easy construction of small vectors.
///
/// This is particularly useful when writing tests involving vectors.
/// Note that the macro is just a convenient wrapper around the Vector
/// constructors, and as a result the vector is still allocated on the
/// heap.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rulinalg;
///
/// # fn main() {
/// use rulinalg::vector::Vector;
///
/// // Construct a vector of f64
/// let vec = vector![1.0, 2.0, 3.0];
/// # }
/// ```
///
/// To construct vectors of other types, specify the type by
/// the usual Rust syntax:
///
/// ```
/// #[macro_use]
/// extern crate rulinalg;
///
/// # fn main() {
/// use rulinalg::vector::Vector;
///
/// // Construct a vector of f32
/// let vec: Vector<f32> = vector![1.0, 2.0, 3.0];
/// // Or
/// let vec = vector![1.0, 2.0, 3.0f32];
/// # }
/// ```
///

#[macro_export]
macro_rules! vector {
    () => {
        {
            // Handle the case when called with no arguments, i.e. vector![]
            use $crate::vector::Vector;
            Vector::new(vec![])
        }
    };
    ($($x:expr),*) => {
        {
            use $crate::vector::Vector;
            Vector::new(vec![$($x),*])
        }
    };
    ($x:expr; $n:expr) => {
        {
            use $crate::vector::Vector;
            Vector::new(vec![$x; $n])
        }
    }
}

#[cfg(test)]
mod tests {
    use vector::Vector;

    #[test]
    fn vector_macro() {
        {
            // An arbitrary vector
            let vec = vector![1, 2, 3, 4, 5, 6];
            assert_eq!(6, vec.size());
            assert_eq!(&vec![1, 2, 3, 4, 5, 6], vec.data());
        }

        {
            // A floating point vector
            let vec = vector![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
            let ref expected_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
            assert_eq!(6, vec.size());
            assert_eq!(expected_data, vec.data());
        }
    }

    #[test]
    fn vector_macro_constant_size() {
        // A constant size vector
        let vec = vector![1.0; 5];
        let ref expected_data = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        assert_eq!(5, vec.size());
        assert_eq!(expected_data, vec.data());
    }

    #[test]
    fn vector_macro_empty_vec() {
        let vec: Vector<f64> = vector![];

        assert_eq!(0, vec.size());
    }

}
