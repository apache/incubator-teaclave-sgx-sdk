use sgx_types::error::SgxStatus;

use crate::{arg::EcallArg, ocall::OTabEntry};

#[derive(Debug)]
pub enum Error {}

pub fn sgx_ecall(eid: usize, idx: usize, otab: &[OTabEntry], data: *const u8) -> SgxStatus {
    todo!()
}

pub struct ETabEntry {}
pub struct EcallTable {}

pub trait Ecall<Target> {
    type Args: EcallArg<Target>;

    fn call(&self, args: Self::Args) -> sgx_types::error::SgxStatus;
}

pub trait EcallWrapper<Args, Target> {
    fn wrapper_t(&self, data: *const u8) -> sgx_types::error::SgxStatus;
}

impl<P, Target, Args> EcallWrapper<Args, Target> for P
where
    P: Ecall<Target, Args = Args>,
    Args: EcallArg<Target>,
    Target: 'static,
{
    fn wrapper_t(&self, data: *const u8) -> sgx_types::error::SgxStatus {
        let bytes =
            unsafe { std::slice::from_raw_parts(data, core::mem::size_of::<(usize, usize)>()) };
        let (ptr, len) = bincode::deserialize::<(usize, usize)>(bytes).unwrap();
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };

        let mut raw_args = Args::deserialize(&bytes);
        let mut arg = raw_args.prepare();
        let in_args = unsafe { Args::from_mut(&mut arg) };

        Ecall::call(self, in_args);

        raw_args.update(arg);
        SgxStatus::Success
    }
}

pub fn untrust_ecall<Args, Target>(
    id: usize,
    eid: usize,
    otab: &[OTabEntry],
    args: Args,
) -> SgxStatus
where
    Args: EcallArg<Target>,
{
    let data = args.serialize();
    // 由于序列化后的长度不确定，因此将Vec再进行一次序列化。
    let vec_data = bincode::serialize(&(data.as_ptr() as usize, data.len())).unwrap();
    sgx_ecall(eid, id, otab, &vec_data as *const Vec<u8> as *const u8)
}
