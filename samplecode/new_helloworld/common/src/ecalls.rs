use sgx_new_edl::{
    args::{In, Out},
    ecall, ecalls,
};

ecalls![hello_world];

#[ecall]
pub fn hello_world<'a>(name: In<'a, String>, out: Out<'a, String>) {
    todo!()
}
