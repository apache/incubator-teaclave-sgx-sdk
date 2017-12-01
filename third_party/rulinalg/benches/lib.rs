#![feature(test)]

#[macro_use]
extern crate rulinalg;
extern crate num as libnum;
extern crate test;
extern crate rand;

pub mod linalg {
	mod iter;
	mod matrix;
	mod svd;
	mod lu;
	mod cholesky;
	mod qr;
	mod norm;
	mod triangular;
	mod permutation;
	pub mod util;
}
