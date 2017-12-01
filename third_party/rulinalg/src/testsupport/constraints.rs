use matrix::BaseMatrix;

use libnum::Zero;

use std::iter::Iterator;

/// Returns true if the matrix is lower triangular, otherwise false.
/// This generalizes to rectangular matrices, in which case
/// it returns true if the matrix is lower trapezoidal.
pub fn is_lower_triangular<T, M>(m: &M) -> bool
    where T: Zero + PartialEq<T>, M: BaseMatrix<T> {

    m.row_iter()
     .enumerate()
     .all(|(i, row)| row.iter()
                        .skip(i + 1)
                        .all(|x| x == &T::zero()))
}

/// Returns true if the matrix is upper triangular, otherwise false.
/// This generalizes to rectangular matrices, in which case
/// it returns true if the matrix is upper trapezoidal.
pub fn is_upper_triangular<T, M>(m: &M) -> bool
    where T: Zero + PartialEq<T>, M: BaseMatrix<T> {

    m.row_iter()
     .enumerate()
     .all(|(i, row)| row.iter()
                        .take(i)
                        .all(|x| x == &T::zero()))
}

#[cfg(test)]
mod tests {
    use super::is_lower_triangular;
    use super::is_upper_triangular;
    use matrix::Matrix;

    #[test]
    fn is_lower_triangular_empty_matrix() {
        let x: Matrix<u32> = matrix![];
        assert!(is_lower_triangular(&x));
    }

    #[test]
    fn is_lower_triangular_1x1() {
        let x = matrix![1];
        assert!(is_lower_triangular(&x));
    }

    #[test]
    fn is_lower_triangular_square() {
        {
            let x = matrix![3, 0;
                            4, 5];
            assert!(is_lower_triangular(&x));
        }

        {
            let x = matrix![1, 0, 0;
                            3, 3, 0;
                            0, 4, 6];
            assert!(is_lower_triangular(&x));
        }

        {
            // Diagonal is an important special case
            let x = matrix![1, 0;
                            0, 2];
            assert!(is_lower_triangular(&x));
        }
    }

    #[test]
    fn is_upper_triangular_empty_matrix() {
        let x: Matrix<u32> = matrix![];
        assert!(is_upper_triangular(&x));
    }

    #[test]
    fn is_upper_triangular_1x1() {
        let x = matrix![1];
        assert!(is_upper_triangular(&x));
    }

    #[test]
    fn is_upper_triangular_square() {
        {
            let x = matrix![3, 4;
                            0, 5];
            assert!(is_upper_triangular(&x));
        }

        {
            let x = matrix![1, 3, 0;
                            0, 3, 4;
                            0, 0, 6];
            assert!(is_upper_triangular(&x));
        }

        {
            // Diagonal is an important special case
            let x = matrix![1, 0;
                            0, 2];
            assert!(is_upper_triangular(&x));
        }
    }

    #[test]
    fn is_upper_triangular_rectangular() {
        {
            let x = matrix![1, 2;
                            0, 3;
                            0, 0];
            assert!(is_upper_triangular(&x));
        }

        {
            let x = matrix![1, 2, 3;
                            0, 4, 5];
            assert!(is_upper_triangular(&x));
        }
    }

    quickcheck! {
        fn property_zero_is_lower_triangular(m: usize, n: usize) -> bool {
            let x = Matrix::<u32>::zeros(m, n);
            is_lower_triangular(&x)
        }
    }

    quickcheck! {
        fn property_zero_is_upper_triangular(m: usize, n: usize) -> bool {
            let x = Matrix::<u32>::zeros(m, n);
            is_upper_triangular(&x)
        }
    }
}

