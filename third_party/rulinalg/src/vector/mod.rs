//! The vector module.
//!
//! Currently contains all code
//! relating to the vector linear algebra struct.
use std::vec::*;
mod impl_ops;
mod impl_vec;

/// The Vector struct.
///
/// Can be instantiated with any type.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Vector<T> {
    size: usize,
    data: Vec<T>,
}
