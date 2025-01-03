use sgx_new_edl::{ecall, In, Out, SgxStatus};

#[ecall]
pub fn foo(a0: In<'_, String>, a1: Out<'_, String>) -> SgxStatus {
    todo!()
}
