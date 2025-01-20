use sgx_new_edl::{ecall, In, Out, SgxStatus};

// export ecall table
extern crate edl;

#[ecall]
pub fn foo(a0: In<'_, String>, a1: Out<'_, String>) -> SgxStatus {
    todo!()
}
