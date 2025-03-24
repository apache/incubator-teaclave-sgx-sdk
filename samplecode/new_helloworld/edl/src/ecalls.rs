use sgx_new_edl::{ecalls, In, Out};
use sgx_types::error::SgxStatus;

ecalls! {
    fn foo(s: Out<'_, String>) -> SgxStatus;
}

// extern "Rust" {
//     fn foo(a: In<'_, String>, b: Out<'_, String>) -> SgxStatus;
// }
