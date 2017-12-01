use rulinalg::matrix::Matrix;
use rand;
use rand::{Rng, SeedableRng};

pub fn reproducible_random_matrix(rows: usize, cols: usize) -> Matrix<f64> {
    const STANDARD_SEED: [usize; 4] = [12, 2049, 4000, 33];
    let mut rng = rand::StdRng::from_seed(&STANDARD_SEED);
    let elements: Vec<_> = rng.gen_iter::<f64>().take(rows * cols).collect();
    Matrix::new(rows, cols, elements)
}
