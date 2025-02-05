#![cfg_attr(not(target_vendor = "teaclave"), no_std)]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]

#[cfg(not(target_vendor = "teaclave"))]
#[macro_use]
extern crate sgx_types;
extern crate sgx_tstd as std;

use std::io::{self, Write};
use std::slice;
use std::string::String;
use std::vec::Vec;

use sgx_new_edl::{ecall, In, Out, SgxStatus};

// export ecall table
extern crate edl;

#[ecall]
pub fn foo(a0: In<'_, String>, a1: Out<'_, String>) -> SgxStatus {
    todo!()
}
