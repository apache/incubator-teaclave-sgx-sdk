//! # The rulinalg crate.
//!
//! A crate that provides high-dimensional linear algebra
//! implemented entirely in Rust.
//!
//! ---
//!
//! This crate provides two core data structures: `Matrix` and
//! `Vector`. These structs are designed to behave as you would expect
//! with relevant operator overloading.
//!
//! The library currently contains (at least) the following linear algebra
//! methods:
//!
//! - Matrix inversion
//! - LUP decomposition
//! - QR decomposition
//! - SVD decomposition
//! - Cholesky decomposition
//! - Eigenvalue decomposition
//! - Upper Hessenberg decomposition
//! - Linear system solver
//! - Other standard transformations, e.g. Transposing, concatenation, etc.
//!
//! ---
//!
//! ## Usage
//!
//! Specific usage of modules is described within the modules themselves. This section
//! will highlight the basic usage.
//!
//! We can create new matrices.
//!
//! ```
//! use rulinalg::matrix::Matrix;
//!
//! // A new matrix with 3 rows and 2 columns.
//! let a = Matrix::new(3, 2, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
//! ```
//!
//! The matrices are stored in row-major order. This means in the example above the top
//! row will be [1, 2].
//!
//! We can perform operations on matrices.
//!
//! ```
//! use rulinalg::matrix::Matrix;
//!
//! // A new matrix with 3 rows and 2 columns.
//! let a = Matrix::new(3, 2, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
//! let b = Matrix::new(3, 2, vec![6.0, 5.0, 4.0, 3.0, 2.0, 1.0]);
//!
//! // Produces a 3x2 matrix filled with sevens.
//! let c = a + b;
//! ```
//!
//! Sometimes we want to construct small matrices by hand, usually for writing unit tests
//! or examples. For this purpose, `rulinalg` provides the `matrix!` macro:
//!
//! ```
//! // Remember to enable macro usage in rulinalg!
//! #[macro_use]
//! extern crate rulinalg;
//!
//! # fn main() {
//! // Construct a 3x3 matrix of f64
//! // Commas separate columns and semi-colons separate rows
//! let mat = matrix![1.0, 2.0, 3.0;
//!                   4.0, 5.0, 6.0;
//!                   7.0, 8.0, 9.0];
//! # }
//! ```
//!
//! Of course the library can support more complex operations but you should check the individual
//! modules for more information.
//!
//! # Matrix Slices
//!
//! Often times it is desirable to operate on only a sub-section of a `Matrix` without copying this block.
//! Rulinalg allows this via the `MatrixSlice` and `MatrixSliceMut` structs. These structs can be created
//! from `Matrix` structs and follow all of the borrowing rules of Rust.
//!
//! Note finally that much of the `Matrix`/`MatrixSlice`/`MatrixSliceMut` functionality is contained behind
//! the `BaseMatrix`/`BaseMatrixMut` traits. This allows us to be generic over matrices or slices.

#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

extern crate num as libnum;
extern crate matrixmultiply;

// macros should be at the top in order for macros to be accessible in subsequent modules
#[macro_use]
pub mod macros;
pub mod matrix;
pub mod convert;
pub mod error;
pub mod utils;
pub mod vector;
pub mod ulp;
pub mod norm;

mod internal_utils;

#[cfg(test)]
mod testsupport;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
extern crate itertools;

pub use norm::{VectorNorm, MatrixNorm};
pub use norm::{VectorMetric, MatrixMetric};

#[cfg(feature = "io")]
extern crate csv as libcsv;

#[cfg(feature = "io")]
extern crate rustc_serialize;

#[cfg(feature = "io")]
pub mod io;
