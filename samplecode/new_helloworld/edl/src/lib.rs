#![cfg_attr(all(not(target_vendor = "teaclave"), feature = "enclave"), no_std)]

#[cfg(not(target_vendor = "teaclave"))]
#[cfg(not(feature = "app"))]
extern crate sgx_tstd as std;

pub mod ecalls;
pub mod ocalls;
