#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

extern crate byteorder;
extern crate num_traits;
extern crate num_bigint;

mod common;
mod deserializer;
mod serializer;
mod utils;

pub use deserializer::{Deserializer, DeserializationError, DeserializationResult, from_vec};
pub use serializer::{Serializer, to_vec};
pub use utils::{BitString, ObjectIdentifier};
