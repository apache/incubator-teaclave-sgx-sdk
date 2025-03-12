#![cfg_attr(all(not(target_vendor = "teaclave"), feature = "enclave"), no_std)]

#[cfg(not(target_vendor = "teaclave"))]
#[cfg(feature = "enclave")]
extern crate sgx_tstd as std;

mod arg;
mod ecall;
mod ocall;
mod ser;

pub use arg::{In, Out, Update};
pub use ecall::{app_ecall, Ecall, EcallEntry, EcallTable, EcallWrapper};
pub use ocall::{OcallEntry, OcallTable};
pub use ser::*;
pub use sgx_edl_macros::{ecall, ecalls, ocall, ocalls};
pub use sgx_types::error::SgxStatus;

impl Update for String {
    fn update(&mut self, other: &Self) {}
}

impl Update for SgxStatus {
    fn update(&mut self, other: &Self) {
        let _ = core::mem::replace(self, *other);
    }
}
