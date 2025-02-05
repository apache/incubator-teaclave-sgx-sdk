use serde::{Deserialize, Serialize};
use sgx_types::error::SgxStatus;

use bincode as ser;

use crate::In;

pub trait OcallArg<Target> {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;

    fn prepare(&self) -> Target;

    /// Reset lifetime
    unsafe fn _from_mut(target: &mut Target) -> Self;

    /// 将enclave内部的参数更新到外部
    fn update(&mut self, other: Target);

    fn destory(self);
}

impl<'a, Target: Serialize + for<'de> Deserialize<'de>> OcallArg<Target> for In<'a, Target> {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(data: &[u8]) -> Self {
        todo!()
    }

    fn prepare(&self) -> Target {
        todo!()
    }

    unsafe fn _from_mut(target: &mut Target) -> Self {
        todo!()
    }

    fn update(&mut self, other: Target) {
        todo!()
    }

    fn destory(self) {
        todo!()
    }
}

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
    let bytes = ser::serialize(&arg).unwrap();
    sgx_ocall(id, bytes.as_ptr());
    //sgx_ecall(eid, id, otab, bytes.as_ptr())
    todo!()
}
