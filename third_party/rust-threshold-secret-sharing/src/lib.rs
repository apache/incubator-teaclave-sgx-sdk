// Copyright (c) 2016 rust-threshold-secret-sharing developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! # Threshold Secret Sharing
//! Pure-Rust library for [secret sharing](https://en.wikipedia.org/wiki/Secret_sharing),
//! offering efficient share generation and reconstruction for both
//! traditional Shamir sharing and its packet (or ramp) variant.
//! For now, secrets and shares are fixed as prime field elements
//! represented by `i64` values.
#![no_std]

extern crate sgx_rand as rand;
#[macro_use]
extern crate sgx_tstd as std;

mod fields;
mod numtheory;
pub use numtheory::positivise;

pub mod shamir;
pub mod packed;
