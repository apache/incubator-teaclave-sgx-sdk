extern crate sgx_tstd as std;


use sgx_new_edl::{ecalls, In, Out};
use sgx_types::error::SgxStatus;

ecalls! {
    fn foo(a: In<'_, String>, b: Out<'_, String>) -> SgxStatus;
    fn bar(a: In<'_, String>, b: Out<'_, String>) -> SgxStatus;
}
