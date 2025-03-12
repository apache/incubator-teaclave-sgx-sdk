use crate::ser::*;
use sgx_types::error::SgxStatus;

pub type ExternOcallFn = unsafe extern "C" fn(*const u8) -> sgx_types::error::SgxStatus;

pub trait OcallArg<Target> {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;

    fn prepare(&self) -> Target;

    /// Reset lifetime
    unsafe fn _from_mut(target: &mut Target) -> Self;

    /// 将enclave内部的参数更新到外部
    fn update(&mut self, other: Target);
}

fn sgx_ocall(idx: usize, ms: *const u8) -> SgxStatus {
    panic!("exec sgx ocall")
}

#[repr(C)]
pub struct OcallTable<const N: usize> {
    pub nr_ocall: usize,
    pub entries: [OcallEntry; N],
}

#[repr(C)]
pub struct OcallEntry {
    pub ocall_addr: ExternOcallFn,
}

impl OcallEntry {
    pub const fn new(ocall: ExternOcallFn) -> Self {
        Self { ocall_addr: ocall }
    }
}

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

pub fn enclave_ocall<Args, Target>(id: usize, args: Args) -> SgxStatus
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
