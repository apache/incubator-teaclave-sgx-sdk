use crate::{
    args::{EcallArg, In, Out},
    ocall::OTabEntry,
};

#[derive(Debug)]
pub enum Error {}

pub fn sgx_ecall(eid: usize, idx: usize, otab: &[OTabEntry], data: *const u8) {
    todo!()
}

pub struct ETabEntry {}
pub struct EcallTable {}

pub trait Ecall<Target> {
    const IDX: usize;

    type Args: EcallArg<Target>;

    fn call(&self, args: Self::Args);
}

pub trait EcallWrapper<Args, Target> {
    fn wrapper_u(&self, eid: usize, otab: &[OTabEntry], args: Args);
    fn wrapper_t(&self, data: *const u8);
}

impl<P, Target, Args> EcallWrapper<Args, Target> for P
where
    P: Ecall<Target, Args = Args>,
    Args: EcallArg<Target>,
    Target: 'static,
{
    fn wrapper_u(&self, eid: usize, otab: &[OTabEntry], args: Args) {
        let data = args.serialize();
        // 由于序列化后的长度不确定，因此将Vec再进行一次序列化。
        let vec_data = bincode::serialize(&(data.as_ptr() as usize, data.len())).unwrap();
        sgx_ecall(
            eid,
            Self::IDX,
            otab,
            &vec_data as *const Vec<u8> as *const u8,
        );
    }

    fn wrapper_t(&self, data: *const u8) {
        let bytes =
            unsafe { std::slice::from_raw_parts(data, core::mem::size_of::<(usize, usize)>()) };
        let (ptr, len) = bincode::deserialize::<(usize, usize)>(bytes).unwrap();
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };

        let mut raw_args = Args::deserialize(&bytes);
        let mut arg = raw_args.prepare();
        let in_args = unsafe { Args::from_mut(&mut arg) };

        Ecall::call(self, in_args);

        raw_args.update(arg);
    }
}
