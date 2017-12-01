//! The rulinalg prelude.
//!
//! This module alleviates some common imports used within rulinalg.

pub use matrix::{Axes, Matrix, MatrixSlice, MatrixSliceMut};
pub use vector::Vector;
pub use matrix::slice::BaseSlice;

#[cfg(test)]
mod tests {
    use super::super::prelude::*;

    #[test]
    fn create_mat_from_prelude() {
        let _ = Matrix::new(2, 2, vec![4.0;4]);
    }
}
