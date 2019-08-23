#![deny(clippy::all)]
#![allow(
    clippy::cast_lossless,
    clippy::new_without_default,
    clippy::new_without_default_derive,
    clippy::block_in_if_condition_stmt,
    clippy::map_entry,
    clippy::needless_range_loop
)]

#![allow(non_snake_case)]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

extern crate num;
extern crate num_bigint;
extern crate sgx_rand as extern_rand;
extern crate base_conversion; // local dependency

pub mod circuit;
pub mod garble;
pub mod high_level;
pub mod numbers;
pub mod rand;
pub mod wire;
