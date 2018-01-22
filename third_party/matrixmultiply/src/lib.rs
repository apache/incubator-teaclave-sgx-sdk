// Copyright 2016 bluss
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//! 
//! General matrix multiplication for f32, f64 matrices.
//! 
//! Allows arbitrary row, column strided matrices.
//! 
//! Uses the same microkernel algorithm as [BLIS][bl], but in a much simpler
//! and less featureful implementation.
//! See their [multithreading][mt] page for a very good diagram over how
//! the algorithm partitions the matrix (*Note:* this crate does not implement
//! multithreading).
//! 
//! [bl]: https://github.com/flame/blis
//! 
//! [mt]: https://github.com/flame/blis/wiki/Multithreading
//!
//! ## Matrix Representation
//!
//! **matrixmultiply** supports matrices with general stride, so a matrix
//! is passed using a pointer and four integers:
//!
//! - `a: *const f32`, pointer to the first element in the matrix
//! - `m: usize`, number of rows
//! - `k: usize`, number of columns
//! - `rsa: isize`, row stride
//! - `csa: isize`, column stride
//!
//! In this example, A is a m by k matrix. `a` is a pointer to the element at
//! index *0, 0*.
//!
//! The *row stride* is the pointer offset (in number of elements) to the
//! element on the next row. It’s the distance from element *i, j* to *i + 1,
//! j*.
//!
//! The *column stride* is the pointer offset (in number of elements) to the
//! element in the next column. It’s the distance from element *i, j* to *i,
//! j + 1*.
//!
//! For example for a contiguous matrix, row major strides are *rsa=k,
//! csa=1* and column major strides are *rsa=1, csa=m*.
//!
//! Stides can be negative or even zero, but for a mutable matrix elements
//! may not alias each other.

#![doc(html_root_url = "https://docs.rs/matrixmultiply/0.1/")]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
extern crate sgx_tstd as std;

extern crate rawpointer;

#[macro_use] mod debugmacros;
#[macro_use] mod loopmacros;
mod archparam;
mod kernel;
mod gemm;
mod sgemm_kernel;
mod dgemm_kernel;
mod util;

pub use gemm::sgemm;
pub use gemm::dgemm;
