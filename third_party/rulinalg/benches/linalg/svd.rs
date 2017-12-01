use test::Bencher;
use linalg::util::reproducible_random_matrix;

#[bench]
fn svd_10_10(b: &mut Bencher) {
    let mat = reproducible_random_matrix(10, 10);

    b.iter(||
        mat.clone().svd()
    )
}

#[bench]
fn svd_100_100(b: &mut Bencher) {
    let mat = reproducible_random_matrix(100, 100);

    b.iter(||
        mat.clone().svd()
    )
}
