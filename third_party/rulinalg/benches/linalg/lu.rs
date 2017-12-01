use rulinalg::matrix::{BaseMatrix, BaseMatrixMut,
                       Matrix};
use rulinalg::matrix::decomposition::{PartialPivLu};
use rulinalg::vector::Vector;

use linalg::util::reproducible_random_matrix;
use libnum::{Zero, One};

use test::Bencher;

fn nullify_upper_triangular_part<T: Zero>(matrix: &mut Matrix<T>) {
    for i in 0 .. matrix.rows() {
        for j in (i + 1) .. matrix.cols() {
            matrix[[i, j]] = T::zero();
        }
    }

    // TODO: assert is_lower_triangular
}

fn nullify_lower_triangular_part<T: Zero>(matrix: &mut Matrix<T>) {
    for i in 0 .. matrix.rows() {
        for j in 0 .. i {
            matrix[[i, j]] = T::zero();
        }
    }

    // TODO: assert is_upper_triangular
}

fn set_diagonal_to_one<T: One>(matrix: &mut Matrix<T>) {
    use rulinalg::matrix::DiagOffset::Main;
    for d in matrix.diag_iter_mut(Main) {
        *d = T::one();
    }
}

fn well_conditioned_matrix(n: usize) -> Matrix<f64> {
    let mut l = reproducible_random_matrix(n, n);
    let mut u = reproducible_random_matrix(n, n);
    nullify_upper_triangular_part(&mut l);
    nullify_lower_triangular_part(&mut u);
    // By making the diagonal of both L and U consist
    // exclusively of 1, the eigenvalues of the resulting
    // matrix LU will also have eigenvalues of 1
    // (and thus be well-conditioned)
    set_diagonal_to_one(&mut l);
    set_diagonal_to_one(&mut u);
    return l * u;
}

#[bench]
fn partial_piv_lu_decompose_10x10(b: &mut Bencher) {
    let n = 10;
    let x = well_conditioned_matrix(n);
    b.iter(|| {
        // Assume that the cost of cloning x is roughly
        // negligible in comparison with the cost of LU
        PartialPivLu::decompose(x.clone()).expect("Matrix is well-conditioned")
    })
}

#[bench]
fn partial_piv_lu_decompose_100x100(b: &mut Bencher) {
    let n = 100;
    let x = well_conditioned_matrix(n);
    b.iter(|| {
        // Assume that the cost of cloning x is roughly
        // negligible in comparison with the cost of LU
        PartialPivLu::decompose(x.clone()).expect("Matrix is well-conditioned")
    })
}

#[bench]
fn partial_piv_lu_inverse_10x10(b: &mut Bencher) {
    let n = 10;
    let x = well_conditioned_matrix(n);
    let lu = PartialPivLu::decompose(x)
                     .expect("Matrix is well-conditioned.");
    b.iter(|| {
        lu.inverse()
    })
}

#[bench]
fn partial_piv_lu_inverse_100x100(b: &mut Bencher) {
    let n = 100;
    let x = well_conditioned_matrix(n);
    let lu = PartialPivLu::decompose(x)
                     .expect("Matrix is well-conditioned.");
    b.iter(|| {
        lu.inverse()
    })
}

#[bench]
fn partial_piv_lu_det_10x10(b: &mut Bencher) {
    let n = 10;
    let x = well_conditioned_matrix(n);
    let lu = PartialPivLu::decompose(x)
                     .expect("Matrix is well-conditioned.");
    b.iter(|| {
        lu.det()
    })
}

#[bench]
fn partial_piv_lu_det_100x100(b: &mut Bencher) {
    let n = 100;
    let x = well_conditioned_matrix(n);
    let lu = PartialPivLu::decompose(x)
                     .expect("Matrix is well-conditioned.");
    b.iter(|| {
        lu.det()
    })
}

#[bench]
fn partial_piv_lu_solve_10x10(b: &mut Bencher) {
    let n = 10;
    let x = well_conditioned_matrix(n);
    let lu = PartialPivLu::decompose(x)
                     .expect("Matrix is well-conditioned.");
    b.iter(|| {
        let b = Vector::ones(n);
        lu.solve(b).unwrap()
    })
}

#[bench]
fn partial_piv_lu_solve_100x100(b: &mut Bencher) {
    let n = 100;
    let x = well_conditioned_matrix(n);
    let lu = PartialPivLu::decompose(x)
                     .expect("Matrix is well-conditioned.");
    b.iter(|| {
        let b = Vector::ones(n);
        lu.solve(b).unwrap()
    })
}
