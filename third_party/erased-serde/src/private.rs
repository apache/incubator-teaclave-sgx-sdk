//! Not public API. Used as `$crate::export` by macros.

pub extern crate serde;
pub use std::marker::{Send, Sync};
pub use std::result::Result;
