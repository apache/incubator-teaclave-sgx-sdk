use crate::ser::*;
use sgx_types::error::SgxStatus;

use crate::In;

/* C function
sgx_status_t SGXAPI sgx_ocall(const unsigned int index,
                              void* ms);
*/
fn sgx_ocall(idx: usize, ms: *const u8) -> SgxStatus {
    panic!("exec sgx ocall")
}

pub struct OcallTable;

pub struct OTabEntry;

pub trait Ocall<Target> {
    type Args: OcallArg<Target>;

    fn call(&self, args: Self::Args) -> SgxStatus;
}

pub trait OcallWrapper<Args, Target> {
    fn wrapper_u(&self, data: *const u8) -> SgxStatus;
}

impl<P, Target, Args> OcallWrapper<Args, Target> for P
where
    P: Ocall<Target, Args = Args>,
    Args: OcallArg<Target>,
    Target: 'static,
{
    fn wrapper_u(&self, data: *const u8) -> SgxStatus {
        todo!()
    }
}

pub fn trust_ecall<Args, Target>(id: usize, args: Args) -> SgxStatus
where
    Args: OcallArg<Target>,
{
    let data = args.serialize();
    let status = SgxStatus::default();
    //// 由于序列化后的长度不确定，因此将Vec再进行一次序列化。
    let arg = (
        (data.as_ptr() as usize, data.len()),
        &status as *const SgxStatus as usize,
    );
    let bytes = serialize(&arg).unwrap();
    sgx_ocall(id, bytes.as_ptr());
    //sgx_ecall(eid, id, otab, bytes.as_ptr())
    todo!()
}

pub fn OtabTou8Ptr(otab: &[OTabEntry]) -> *const u8 {
    otab.as_ptr() as *const u8
}
