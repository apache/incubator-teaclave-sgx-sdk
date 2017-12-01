//! Decompositions for matrices.
//!
//! This module houses the decomposition API of `rulinalg`.
//! A decomposition - or factorization - of a matrix is an
//! ordered set of *factors* such that when multiplied reconstructs
//! the original matrix. The [Decomposition](trait.Decomposition.html)
//! trait encodes this property.
//!
//! # The decomposition API
//!
//! Decompositions in `rulinalg` are in general modeled after
//! the following:
//!
//! 1. Given an appropriate matrix, an opaque decomposition object
//!    may be computed which internally stores the factors
//!    in an efficient and appropriate format.
//! 2. In general, the factors may not be immediately available
//!    as distinct matrices after decomposition. If the user
//!    desires the explicit matrix factors involved in the
//!    decomposition, the user must `unpack` the decomposition.
//! 3. Before unpacking the decomposition, the decomposition
//!    data structure in question may offer an API that provides
//!    efficient implementations for some of the most common
//!    applications of the decomposition. The user is encouraged
//!    to use the decomposition-specific API rather than unpacking
//!    the decompositions whenever possible.
//!
//! For a motivating example that explains the rationale behind
//! this design, let us consider the typical LU decomposition with
//! partial pivoting. In this case, given a square invertible matrix
//! `A`, one may find matrices `P`, `L` and `U` such that
//! `PA = LU`. Here `P` is a permutation matrix, `L` is a lower
//! triangular matrix and `U` is an upper triangular matrix.
//!
//! Once the decomposition has been obtained, one of its applications
//! is the efficient solution of multiple similar linear systems.
//! Consider that while computing the LU decomposition requires
//! O(n<sup>3</sup>) floating point operations, the solution to
//! the system `Ax = b` can be computed in O(n<sup>2</sup>) floating
//! point operations if the LU decomposition has already been obtained.
//! Since the right-hand side `b` has no bearing on the LU decomposition,
//! it follows that one can efficiently solve this system for any `b`.
//!
//! It turns out that the matrices `L` and `U` can be stored compactly
//! in the space of a single matrix. Indeed, this is how `PartialPivLu`
//! stores the LU decomposition internally. This allows `rulinalg` to
//! provide the user with efficient implementations of common applications
//! for the LU decomposition. However, the full matrix factors are easily
//! available to the user by unpacking the decomposition.
//!
//! # Available decompositions
//!
//! **The decompositions API is a work in progress.**
//!
//! Currently, only a portion of the available decompositions in `rulinalg`
//! are available through the decomposition API. Please see the
//! [Matrix](../struct.Matrix.html) API for the old decomposition
//! implementations that have yet not been implemented within
//! this framework.
//!
//! <table>
//! <thead>
//! <tr>
//! <th>Decomposition</th>
//! <th>Matrix requirements</th>
//! <th>Supported features</th>
//! </tr>
//! <tbody>
//!
//! <tr>
//! <td><a href="struct.PartialPivLu.html">PartialPivLu</a></td>
//! <td>Square, invertible</td>
//! <td>
//!     <ul>
//!     <li>Linear system solving</li>
//!     <li>Matrix inverse</li>
//!     <li>Determinant computation</li>
//!     </ul>
//! </td>
//! </tr>
//!
//! <tr>
//! <td><a href="struct.FullPivLu.html">FullPivLu</a></td>
//! <td>Square matrices</td>
//! <td>
//!     <ul>
//!     <li>Linear system solving</li>
//!     <li>Matrix inverse</li>
//!     <li>Determinant computation</li>
//!     <li>Rank computation</li>
//!     </ul>
//! </td>
//! </tr>
//!
//! <tr>
//! <td><a href="struct.Cholesky.html">Cholesky</a></td>
//! <td>Square, symmetric positive definite</td>
//! <td>
//!     <ul>
//!     <li>Linear system solving</li>
//!     <li>Matrix inverse</li>
//!     <li>Determinant computation</li>
//!     </ul>
//! </td>
//! </tr>
//!
//! <tr>
//! <td><a href="struct.HouseholderQr.html">HouseholderQr</a></td>
//! <td>Any matrix</td>
//! <td></td>
//! </tr>
//!
//! </tbody>
//! </table>

// References:
//
// 1. [On Matrix Balancing and EigenVector computation]
// (http://arxiv.org/pdf/1401.5766v1.pdf), James, Langou and Lowery
//
// 2. [The QR algorithm for eigen decomposition]
// (http://people.inf.ethz.ch/arbenz/ewp/Lnotes/chapter4.pdf)
//
// 3. [Computation of the SVD]
// (http://www.cs.utexas.edu/users/inderjit/public_papers/HLA_SVD.pdf)
use std::vec::*;

mod qr;
mod cholesky;
mod bidiagonal;
mod svd;
mod hessenberg;
mod lu;
mod eigen;
mod householder;

use std::any::Any;

use matrix::{Matrix, BaseMatrix};
use norm::Euclidean;
use vector::Vector;
use utils;
use error::{Error, ErrorKind};

use self::householder::HouseholderReflection;

pub use self::householder::HouseholderComposition;
pub use self::lu::{PartialPivLu, LUP, FullPivLu, LUPQ};
pub use self::cholesky::Cholesky;
pub use self::qr::{HouseholderQr, QR, ThinQR};

use libnum::{Float};

/// Base trait for decompositions.
///
/// A matrix decomposition, or factorization,
/// is a procedure which takes a matrix `X` and returns
/// a set of `k` factors `X_1, X_2, ..., X_k` such that
/// `X = X_1 * X_2 * ... * X_k`.
pub trait Decomposition {
    /// The type representing the ordered set of factors
    /// that when multiplied yields the decomposed matrix.
    type Factors;

    /// Extract the individual factors from this decomposition.
    fn unpack(self) -> Self::Factors;
}

impl<T> Matrix<T>
    where T: Any + Float
{
    /// Compute the cos and sin values for the givens rotation.
    ///
    /// Returns a tuple (c, s).
    fn givens_rot(a: T, b: T) -> (T, T) {
        let r = a.hypot(b);

        (a / r, -b / r)
    }

    fn make_householder(column: &[T]) -> Result<Matrix<T>, Error> {
        let size = column.len();

        if size == 0 {
            return Err(Error::new(ErrorKind::InvalidArg,
                                  "Column for householder transform cannot be empty."));
        }

        let denom = column[0] + column[0].signum() * utils::dot(column, column).sqrt();

        if denom == T::zero() {
            return Err(Error::new(ErrorKind::DecompFailure,
                                  "Cannot produce househoulder transform from column as first \
                                   entry is 0."));
        }

        let mut v = column.into_iter().map(|&x| x / denom).collect::<Vec<T>>();
        // Ensure first element is fixed to 1.
        v[0] = T::one();
        let v = Vector::new(v);
        let v_norm_sq = v.dot(&v);

        let v_vert = Matrix::new(size, 1, v.data().clone());
        let v_hor = Matrix::new(1, size, v.into_vec());
        Ok(Matrix::<T>::identity(size) - (v_vert * v_hor) * ((T::one() + T::one()) / v_norm_sq))
    }

    fn make_householder_vec(column: &[T]) -> Result<Matrix<T>, Error> {
        let size = column.len();

        if size == 0 {
            return Err(Error::new(ErrorKind::InvalidArg,
                                  "Column for householder transform cannot be empty."));
        }

        let denom = column[0] + column[0].signum() * utils::dot(column, column).sqrt();

        if denom == T::zero() {
            return Err(Error::new(ErrorKind::DecompFailure,
                                  "Cannot produce househoulder transform from column as first \
                                   entry is 0."));
        }

        let mut v = column.into_iter().map(|&x| x / denom).collect::<Vec<T>>();
        // Ensure first element is fixed to 1.
        v[0] = T::one();
        let v = Matrix::new(size, 1, v);

        Ok(&v / v.norm(Euclidean))
    }
}
