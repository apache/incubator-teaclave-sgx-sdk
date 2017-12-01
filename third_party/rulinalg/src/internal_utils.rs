use matrix::{BaseMatrix, BaseMatrixMut};
use libnum::{Zero, Num};
use utils::in_place_vec_bin_op;

pub fn nullify_lower_triangular_part<T, M>(matrix: &mut M)
    where T: Zero, M: BaseMatrixMut<T> {
    for (i, mut row) in matrix.row_iter_mut().enumerate() {
        for element in row.raw_slice_mut().iter_mut().take(i) {
            *element = T::zero();
        }
    }
}

pub fn nullify_upper_triangular_part<T, M>(matrix: &mut M)
    where T: Zero, M: BaseMatrixMut<T> {
    for (i, mut row) in matrix.row_iter_mut().enumerate() {
        for element in row.raw_slice_mut().iter_mut().skip(i + 1) {
            *element = T::zero();
        }
    }
}

/// Given a vector `x` and a `m x n` matrix `A`, compute
/// `y = A^T x`.
///
/// This is a stopgap solution until we have a more proper
/// BLIS/BLAS-like API.
pub fn transpose_gemv<T, M>(a: &M, x: &[T], y: &mut [T])
    where M: BaseMatrix<T>, T: Num + Copy
{
    let m = a.rows();
    let n = a.cols();

    assert!(x.len() == m, "A and x must be dimensionally compatible.");
    assert!(y.len() == n, "A and y must be dimensionally compatible.");

    for element in y.iter_mut() {
        *element = T::zero();
    }

    for j in 0 .. m {
        let a_j = a.row(j).raw_slice();
        axpy(x[j], a_j, y);
    }
}

// Given scalar `a` and vectors `x` and `y` of same length, computes the
// scalar-vector product `y <- y + a * x`.
//
/// This function is a stopgap solution until we have a more proper
/// BLIS/BLAS-like API.
pub fn axpy<T>(a: T, x: &[T], y: &mut [T])
    where T: Num + Copy
{
    assert!(x.len() == y.len());
    in_place_vec_bin_op(y, x, |y, x| {
        *y = y.clone() + a * x.clone();
    });
}

/// Given a `m x n` matrix `A` and vectors `x` and `y` and
/// a scalar `alpha`, perform the rank-1 update
/// `A <- A + alpha * x y^T`.
///
/// This function is a stopgap solution until we have a more proper
/// BLIS/BLAS-like API.
pub fn ger<T, M>(a: &mut M, alpha: T, x: &[T], y: &[T])
    where M: BaseMatrixMut<T>, T: Num + Copy
{
    let m = a.rows();
    let n = a.cols();

    assert!(x.len() == m);
    assert!(y.len() == n);

    for i in 0 .. m {
        let mut a_i = a.row_mut(i).raw_slice_mut();
        // Let a_i be the ith row. Then
        // a_i <- a_i + alpha * x[i] * y
        // is just an axpy operation
        axpy(alpha * x[i], &y, &mut a_i);
    }
}

#[cfg(test)]
mod tests {
    use vector::Vector;

    use super::nullify_lower_triangular_part;
    use super::nullify_upper_triangular_part;
    use super::transpose_gemv;
    use super::ger;

    #[test]
    fn nullify_lower_triangular_part_examples() {
        let mut x = matrix![1.0, 2.0, 3.0;
                            4.0, 5.0, 6.0;
                            7.0, 8.0, 9.0];
        nullify_lower_triangular_part(&mut x);
        assert_matrix_eq!(x, matrix![
            1.0, 2.0, 3.0;
            0.0, 5.0, 6.0;
            0.0, 0.0, 9.0
        ]);
    }

    #[test]
    fn nullify_upper_triangular_part_examples() {
        let mut x = matrix![1.0, 2.0, 3.0;
                            4.0, 5.0, 6.0;
                            7.0, 8.0, 9.0];
        nullify_upper_triangular_part(&mut x);
        assert_matrix_eq!(x, matrix![
            1.0, 0.0, 0.0;
            4.0, 5.0, 0.0;
            7.0, 8.0, 9.0
        ]);
    }

    #[test]
    fn transpose_gemv_examples() {
        {
            let a = matrix![3.0, 4.0, 5.0;
                            2.0, 3.0, 1.0];
            let x = vec![2.0, 3.0];
            let mut y = vec![0.0; 3];
            transpose_gemv(&a, &x, &mut y);

            let y = Vector::new(y);
            assert_vector_eq!(y, vector![12.0, 17.0, 13.0]);
        }
    }

    #[test]
    fn ger_examples() {
        {
            let mut a = matrix![3.0, 4.0, 5.0;
                            2.0, 3.0, 1.0];
            let x = vec![3.0, 4.0];
            let y = vec![2.0, 1.0, 3.0];
            let alpha = 3.0;

            ger(&mut a, alpha, &x, &y);

            let expected = matrix![21.0, 13.0, 32.0;
                                   26.0, 15.0, 37.0];
            assert_matrix_eq!(a, expected, comp = float);
        }
    }
}
