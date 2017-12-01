use rulinalg::matrix::{Matrix, BaseMatrix};
use rulinalg::norm::*;
use libnum::Float;
use test::Bencher;
use test::black_box;

#[bench]
fn lp_1_mat_10_50(b: &mut Bencher) {
    let a = Matrix::new(10, 50, vec![2.0;500]);
    let lp = Lp::Integer(1);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&lp, &a));
    });
}

#[bench]
fn lp_1_mat_100_50(b: &mut Bencher) {
    let a = Matrix::new(100, 50, vec![2.0;5000]);
    let lp = Lp::Integer(1);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&lp, &a));
    });
}

#[bench]
fn sum_abs_mat_10_50(b: &mut Bencher) {
    let a = Matrix::new(10, 50, vec![2.0;500]);

    b.iter(|| {
        let mut s = 0.0;
        for x in a.iter() {
            black_box(s = s + x.abs());
        }
    });
}

#[bench]
fn sum_abs_mat_100_50(b: &mut Bencher) {
    let a = Matrix::new(100, 50, vec![2.0;5000]);

    b.iter(|| {
        let mut s = 0.0;
        for x in a.iter() {
            black_box(s = s + x.abs());
        }
    });
}

#[bench]
fn lp_2_mat_10_50(b: &mut Bencher) {
    let a = Matrix::new(10, 50, vec![2.0;500]);
    let lp = Lp::Integer(2);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&lp, &a));
    });
}

#[bench]
fn lp_2_mat_100_50(b: &mut Bencher) {
    let a = Matrix::new(100, 50, vec![2.0;5000]);
    let lp = Lp::Integer(2);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&lp, &a));
    });
}

#[bench]
fn lp_3_mat_10_50(b: &mut Bencher) {
    let a = Matrix::new(10, 50, vec![2.0;500]);
    let lp = Lp::Integer(3);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&lp, &a));
    });
}

#[bench]
fn lp_3_mat_100_50(b: &mut Bencher) {
    let a = Matrix::new(100, 50, vec![2.0;5000]);
    let lp = Lp::Integer(3);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&lp, &a));
    });
}

#[bench]
fn lp_float_2_mat_10_50(b: &mut Bencher) {
    let a = Matrix::new(10, 50, vec![2.0;500]);
    let lp = Lp::Float(2.0);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&lp, &a));
    });
}

#[bench]
fn lp_float_2_mat_100_50(b: &mut Bencher) {
    let a = Matrix::new(100, 50, vec![2.0;5000]);
    let lp = Lp::Float(2.0);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&lp, &a));
    });
}

#[bench]
fn lp_float_3_mat_10_50(b: &mut Bencher) {
    let a = Matrix::new(10, 50, vec![2.0;500]);
    let lp = Lp::Float(3.0);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&lp, &a));
    });
}

#[bench]
fn lp_float_3_mat_100_50(b: &mut Bencher) {
    let a = Matrix::new(100, 50, vec![2.0;5000]);
    let lp = Lp::Float(3.0);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&lp, &a));
    });
}

#[bench]
fn euclidean_mat_10_50(b: &mut Bencher) {
    let a = Matrix::new(10, 50, vec![2.0;500]);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&Euclidean, &a));
    });
}

#[bench]
fn euclidean_mat_100_50(b: &mut Bencher) {
    let a = Matrix::new(100, 50, vec![2.0;5000]);

    b.iter(|| {
    	let _ = black_box(MatrixNorm::norm(&Euclidean, &a));
    });
}
