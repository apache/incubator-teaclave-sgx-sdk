use crate::ocall::OTabEntry;

#[derive(Debug)]
pub enum Error {}

pub fn sgx_ecall(eid: usize, idx: usize, otab: &[OTabEntry], data: *const u8) {
    todo!()
}

pub struct ETabEntry {}
pub struct EcallTable {}

pub trait EcallArg: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;

    /// 生成真正传入函数的参数
    ///
    /// Ok表示生成成功，Err表示生成失败需要退出
    ///
    /// None表示使用原本的参数, Some表示使用新的参数
    fn prepare(&self) -> Result<Self, Error>;

    // fn is_out() -> bool;

    /// 将enclave内部的参数更新到外部
    fn update(&mut self, other: Self);

    /// 释放位于enclave内部的参数。
    ///
    /// 由于enclave内部的参数在反序列化时，可能会通过解裸指针或者Box::leak等方式获取引用。
    /// 因此需要手动delete这些数据，释放内存空间。
    ///
    /// 简单来说，每个prepare()生成的新参数都需要调用一次destory()。
    fn destory(self);
}

pub trait Ecall {
    const IDX: usize;

    type Args;

    fn call(&self, args: Self::Args) -> Self::Args;
}

//pub trait EcallWrapperU<Args> {
//    fn wrapper_u(eid: usize, otab: &[OTabEntry]), args
//}

pub trait EcallWrapper<Args> {
    fn wrapper_u(&self, eid: usize, otab: &[OTabEntry], args: Args);
    fn wrapper_t(&self, data: *const u8);
}

impl<T, Args: EcallArg> EcallWrapper<Args> for T
where
    T: Ecall<Args = Args>,
{
    fn wrapper_u(&self, eid: u64 , otab: &[OTabEntry], args: Args) {
        let data = args.serialize();
        // 由于序列化后的长度不确定，因此将Vec再进行一次序列化。
        let vec_data = bincode::serialize(&(data.as_ptr() as usize, data.len())).unwrap();
        sgx_ecall(
            eid as usize,
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
        let mut args = Args::deserialize(&bytes);
        let mut in_args = args.prepare().unwrap();
        in_args = self.call(in_args);
        args.update(in_args);
    }
}
