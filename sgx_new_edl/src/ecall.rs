// #![cfg_attr(feature = "enclave", no_std)]
// extern crate sgx_tstd as std;

use sgx_types::error::SgxStatus;
use sgx_types::function::sgx_ecall;

use crate::{ocall::OcallEntry, ser::*, Update};

pub type ExternEcallFn = unsafe extern "C" fn(*const u8) -> sgx_types::error::SgxStatus;

pub trait EcallArg<Target>: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;

    fn prepare(&self) -> Target;

    /// Reset lifetime
    unsafe fn _from_mut(target: &mut Target) -> Self;

    /// 将enclave内部的参数更新到外部
    fn update(&mut self, other: Target);

    fn destory(self);
}

#[repr(C)]
pub struct EcallEntry {
    pub ecall_addr: ExternEcallFn,
    pub is_priv: u8,
    pub is_switchless: u8,
}

impl EcallEntry {
    pub const fn new(ecall: ExternEcallFn) -> Self {
        Self {
            ecall_addr: ecall,
            is_priv: 0,
            is_switchless: 0,
        }
    }
}

#[repr(C)]
pub struct EcallTable<const N: usize> {
    pub nr_ecall: usize,
    pub entries: [EcallEntry; N],
}

impl<const N: usize> EcallTable<N> {
    pub const fn new(tab: [EcallEntry; N]) -> Self {
        Self {
            nr_ecall: tab.len(),
            entries: tab,
        }
    }
}

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
        let bytes = unsafe {
            std::slice::from_raw_parts(data, core::mem::size_of::<((usize, usize), usize)>())
        };
        // ptr: arg address, len: arg bytes len, retval: sgx status address
        let ((ptr, len), retval) = deserialize::<((usize, usize), usize)>(bytes).unwrap();
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };

        // deserialize the arguments
        let mut raw_args = Args::deserialize(&bytes);
        let retval = unsafe { &mut *(retval as *mut SgxStatus) };
        let mut arg = raw_args.prepare();
        // TODO: fence

        let in_args = unsafe { Args::_from_mut(&mut arg) };

        let in_retval = Ecall::call(self, in_args);

        // update input arguments
        raw_args.update(arg);
        // update sgx_status
        retval.update(&in_retval);

        SgxStatus::Success
    }
}

pub fn app_ecall<Args, Target>(id: usize, eid: u64, otab: &[OcallEntry], args: Args) -> SgxStatus
where
    Args: EcallArg<Target>,
{
    let data = args.serialize();
    let status = SgxStatus::default();
    // 由于序列化后的长度不确定，因此将Vec再进行一次序列化。

    let arg = (
        (data.as_ptr() as usize, data.len()),
        &status as *const SgxStatus as usize,
    );
    let mut bytes = serialize(&arg).unwrap();

    // TODO: 序列化ocall表
    let otab_ptr = otab.as_ptr() as *const u8;

    unsafe {
        sgx_ecall(
            eid,
            id as i32,
            otab_ptr as *const std::os::raw::c_void,
            bytes.as_mut_ptr() as *mut std::os::raw::c_void,
        )
    }
}
