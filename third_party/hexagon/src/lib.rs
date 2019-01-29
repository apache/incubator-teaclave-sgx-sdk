#![feature(nll)]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_trts;

extern crate smallvec;
extern crate byteorder;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

#[macro_use]
pub mod debug;

#[macro_use]
pub mod fixed_array;

pub mod hybrid;
pub mod builtin;

pub mod basic_block;
pub mod call_stack;
pub mod dynamic_trait;
pub mod errors;
pub mod executor;
pub mod function_optimizer;
pub mod function;
pub mod generic_arithmetic;
//pub mod hybrid_bridge;
pub mod object_info;
pub mod object_pool;
pub mod object;
pub mod opcode;
pub mod primitive;
pub mod static_root;
pub mod value;

#[cfg(test)]
mod executor_test;

#[cfg(test)]
mod bench;

#[cfg(test)]
mod optimizer_test;
