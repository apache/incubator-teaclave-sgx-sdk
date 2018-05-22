//! WebAssembly format library
#![warn(missing_docs)]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

extern crate byteorder;

pub mod elements;
pub mod builder;

pub use elements::{
	Error as SerializationError,
	deserialize_buffer,
	deserialize_file,
	serialize,
	serialize_to_file,
	peek_size,
};

