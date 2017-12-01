use rulinalg::matrix::{Matrix, DiagOffset};
use rulinalg::matrix::BaseMatrix;
use test::Bencher;
use test::black_box;

#[bench]
fn empty(b: &mut Bencher) {
    b.iter(|| 1)
}

#[bench]
fn mat_diag_iter_10_50(b: &mut Bencher) {

    let a = Matrix::new(10, 50, vec![2.0;500]);

    b.iter(|| {
        let _ = black_box(a.diag_iter(DiagOffset::Main).cloned().collect::<Vec<_>>());
    });
}

#[bench]
fn mat_diag_manual_10_50(b: &mut Bencher) {

    let a = Matrix::new(10, 50, vec![2.0;500]);

    b.iter(|| {
        let mut d = black_box(Vec::with_capacity(10));
        for i in 0..10 {
            unsafe {
                black_box(d.push(*a.get_unchecked([i, i])));
            }
        }
    });
}

#[bench]
fn mat_diag_iter_100_500(b: &mut Bencher) {

    let a = Matrix::new(100, 500, vec![2.0;50000]);

    b.iter(|| {
        let _ = black_box(a.diag_iter(DiagOffset::Main).cloned().collect::<Vec<_>>());
    });
}

#[bench]
fn mat_diag_manual_100_500(b: &mut Bencher) {

    let a = Matrix::new(100, 500, vec![2.0;50000]);

    b.iter(|| {
        let mut d = black_box(Vec::with_capacity(100));
        for i in 0..100 {
            unsafe {
                black_box(d.push(*a.get_unchecked([i, i])));
            }
        }
    });
}
