#![cfg_attr(not(target_vendor = "teaclave"), no_std)]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]

#[cfg(not(target_vendor = "teaclave"))]
#[macro_use]
extern crate sgx_tstd as std;

use std::io::{self, Write};
use std::slice;
use std::string::String;
use std::vec::Vec;

use sgx_new_edl::{ecall, In, Out};
use sgx_types::error::SgxStatus;

extern crate edl;

#[ecall]
pub fn foo(s: Out<'_, String>) -> SgxStatus {
    let mut os = String::from("Enclave Message");
    let arg0 = In::new(&mut os);
    let (status, retval) = edl::ocalls::bar::ocall(arg0);
    let s = s.get_mut();
    s.push_str(retval.as_str());
    s.push_str(format!("status: {:#x}", status).as_str());
    SgxStatus::Success
}
