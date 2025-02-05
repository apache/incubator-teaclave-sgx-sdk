mod arg;
mod ecall;
mod ocall;

pub use arg::{In, Out, Update};
pub use ecall::{untrust_ecall, Ecall, EcallEntry, EcallTable, EcallWrapper};
pub use ocall::{OTabEntry, OcallTable};
pub use sgx_edl_macros::{ecall, ecalls};
pub use sgx_types::error::SgxStatus;

impl Update for String {
    fn update(&mut self, other: &Self) {}
}

impl Update for SgxStatus {
    fn update(&mut self, other: &Self) {
        let _ = core::mem::replace(self, *other);
    }
}
