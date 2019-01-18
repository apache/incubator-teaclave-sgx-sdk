#![feature(nll)]
#![cfg_attr(not(all(target_env = "sgx", feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
pub extern crate hexagon;

pub use hexagon as vm;

pub mod ast_codegen;
pub mod ast;
pub mod codegen;
pub mod lua_types;
pub mod runtime;

#[cfg(test)]
mod test_programs;

#[cfg(test)]
mod ast_test;

#[cfg(test)]
mod codegen_test;
