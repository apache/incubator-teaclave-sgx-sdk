use test::Bencher;
use rulinalg::matrix::Matrix;
use rulinalg::matrix::BaseMatrix;

#[bench]
fn solve_l_triangular_100x100(b: &mut Bencher) {
    let n = 100;
    let x = Matrix::<f64>::identity(n);
    b.iter(|| {
        x.solve_l_triangular(vector![0.0; n])
    });
}

#[bench]
fn solve_l_triangular_1000x1000(b: &mut Bencher) {
    let n = 1000;
    let x = Matrix::<f64>::identity(n);
    b.iter(|| {
        x.solve_l_triangular(vector![0.0; n])
    });
}

#[bench]
fn solve_l_triangular_10000x10000(b: &mut Bencher) {
    let n = 10000;
    let x = Matrix::<f64>::identity(n);
    b.iter(|| {
        x.solve_l_triangular(vector![0.0; n])
    });
}

#[bench]
fn solve_u_triangular_100x100(b: &mut Bencher) {
    let n = 100;
    let x = Matrix::<f64>::identity(n);
    b.iter(|| {
        x.solve_u_triangular(vector![0.0; n])
    });
}

#[bench]
fn solve_u_triangular_1000x1000(b: &mut Bencher) {
    let n = 1000;
    let x = Matrix::<f64>::identity(n);
    b.iter(|| {
        x.solve_u_triangular(vector![0.0; n])
    });
}

#[bench]
fn solve_u_triangular_10000x10000(b: &mut Bencher) {
    let n = 10000;
    let x = Matrix::<f64>::identity(n);
    b.iter(|| {
        x.solve_u_triangular(vector![0.0; n])
    });
}

