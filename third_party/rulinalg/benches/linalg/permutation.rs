use rulinalg::matrix::PermutationMatrix;
use rulinalg::vector::Vector;

use test::Bencher;

/// A perfect shuffle as defined at
/// https://en.wikipedia.org/wiki/Faro_shuffle
/// Note that it creates a permutation matrix of size
/// 2 * n and not n.
fn perfect_shuffle<T>(n: usize) -> PermutationMatrix<T> {
    let array = (0 .. 2 * n).map(|k|
            if (k + 1) % 2 == 0 {
                n + (k + 1) / 2 - 1
            } else {
                (k + 1) / 2
            }
        ).collect::<Vec<_>>();
    PermutationMatrix::from_array(array).unwrap()
}

#[bench]
fn identity_permutation_mul_vector_100(b: &mut Bencher) {
    let n = 100;
    let p = PermutationMatrix::identity(n);
    let x: Vector<f64> = Vector::zeros(n);

    b.iter(|| {
        &p * &x
    })
}

#[bench]
fn identity_permutation_as_matrix_mul_vector_100(b: &mut Bencher) {
    let n = 100;
    let p = PermutationMatrix::identity(n).as_matrix();
    let x: Vector<f64> = Vector::zeros(n);

    b.iter(|| {
        &p * &x
    })
}

#[bench]
fn identity_permutation_mul_vector_1000(b: &mut Bencher) {
    let n = 1000;
    let p = PermutationMatrix::identity(n);
    let x: Vector<f64> = Vector::zeros(n);

    b.iter(|| {
        &p * &x
    })
}

#[bench]
fn identity_permutation_as_matrix_mul_vector_1000(b: &mut Bencher) {
    let n = 1000;
    let p = PermutationMatrix::identity(n).as_matrix();
    let x: Vector<f64> = Vector::zeros(n);

    b.iter(|| {
        &p * &x
    })
}

#[bench]
fn perfect_shuffle_permutation_mul_vector_1000(b: &mut Bencher) {
    let n = 1000;
    let p = perfect_shuffle(500);
    let x: Vector<f64> = Vector::zeros(n);

    b.iter(|| {
        &p * &x
    })
}

#[bench]
fn perfect_shuffle_permutation_as_matrix_mul_vector_1000(b: &mut Bencher) {
    let n = 1000;
    let p = perfect_shuffle(500).as_matrix();
    let x: Vector<f64> = Vector::zeros(n);

    b.iter(|| {
        &p * &x
    })
}
