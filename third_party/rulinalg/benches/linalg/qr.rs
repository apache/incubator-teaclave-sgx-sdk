use rulinalg::matrix::decomposition::{Decomposition, HouseholderQr};

use linalg::util::reproducible_random_matrix;

use test::Bencher;

#[bench]
fn householder_qr_decompose_100x100(b: &mut Bencher) {
    let x = reproducible_random_matrix(100, 100);
    b.iter(|| {
        HouseholderQr::decompose(x.clone())
    })
}

#[bench]
fn householder_qr_decompose_500x100(b: &mut Bencher) {
    let x = reproducible_random_matrix(500, 100);
    b.iter(|| {
        HouseholderQr::decompose(x.clone())
    })
}

#[bench]
fn householder_qr_decompose_100x500(b: &mut Bencher) {
    let x = reproducible_random_matrix(100, 500);
    b.iter(|| {
        HouseholderQr::decompose(x.clone())
    })
}

#[bench]
fn householder_qr_decompose_unpack_100x100(b: &mut Bencher) {
    let x = reproducible_random_matrix(100, 100);
    b.iter(|| {
        HouseholderQr::decompose(x.clone()).unpack()
    })
}

#[bench]
fn householder_qr_decompose_unpack_500x100(b: &mut Bencher) {
    let x = reproducible_random_matrix(500, 100);
    b.iter(|| {
        HouseholderQr::decompose(x.clone()).unpack()
    })
}

#[bench]
fn householder_qr_decompose_unpack_100x500(b: &mut Bencher) {
    let x = reproducible_random_matrix(100, 500);
    b.iter(|| {
        HouseholderQr::decompose(x.clone()).unpack()
    })
}

#[bench]
fn householder_qr_decompose_unpack_thin_100x100(b: &mut Bencher) {
    let x = reproducible_random_matrix(100, 100);
    b.iter(|| {
        HouseholderQr::decompose(x.clone()).unpack_thin()
    })
}

#[bench]
fn householder_qr_decompose_unpack_thin_500x100(b: &mut Bencher) {
    let x = reproducible_random_matrix(500, 100);
    b.iter(|| {
        HouseholderQr::decompose(x.clone()).unpack_thin()
    })
}

#[bench]
fn householder_qr_decompose_unpack_thin_100x500(b: &mut Bencher) {
    let x = reproducible_random_matrix(100, 500);
    b.iter(|| {
        HouseholderQr::decompose(x.clone()).unpack_thin()
    })
}

#[bench]
#[allow(deprecated)]
fn qr_decomp_100x100(b: &mut Bencher) {
    let x = reproducible_random_matrix(100, 100);
    b.iter(|| {
        x.clone().qr_decomp()
    })
}

#[bench]
#[allow(deprecated)]
fn qr_decomp_100x500(b: &mut Bencher) {
    let x = reproducible_random_matrix(100, 500);
    b.iter(|| {
        x.clone().qr_decomp()
    })
}
