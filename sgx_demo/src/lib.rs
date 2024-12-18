use sgx_edl::{
    args::{In, Out},
    ecalls,
};
use sgx_edl_macro::ecall;

ecalls![add];

#[ecall]
pub fn add<'a>(left: In<'a, String>, right: Out<'a, String>) -> u64 {
    // left + right
    todo!()
}
