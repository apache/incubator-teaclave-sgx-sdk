use crate::{ser::*, Update};
#[cfg(feature = "enclave")]
use sgx_trts::capi::{sgx_ocall, sgx_ocalloc, sgx_ocfree};
use sgx_types::error::SgxStatus;

pub type ExternOcallFn = unsafe extern "C" fn(*const u8) -> sgx_types::error::SgxStatus;

use std::vec::Vec;

pub trait OcallArg<Target> {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;

    // /// 将enclave内部的参数更新到外部
    // fn update(&mut self, other: Self);

    unsafe fn _clone(&mut self) -> Self;

    unsafe fn destory(self);
}

impl OcallArg<()> for () {
    fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }

    fn deserialize(_: &[u8]) -> Self {
        ()
    }

    unsafe fn _clone(&mut self) -> Self {
        ()
    }

    unsafe fn destory(self) {}
}

pub struct DynEntryTable<const N: usize> {
    pub nr_ocall: usize,
    pub entries: [EntryTable<N>; 1],
}

impl<const N: usize> DynEntryTable<N> {
    pub const fn new(entries: [u8; N]) -> Self {
        Self {
            nr_ocall: N,
            entries: [EntryTable { entries }],
        }
    }
}

#[repr(C)]
pub struct EntryTable<const N: usize> {
    pub entries: [u8; N],
}

#[repr(C)]
pub struct OcallTable<const N: usize> {
    pub nr_ocall: usize,
    pub entries: [OcallEntry; N],
}

impl<const N: usize> OcallTable<N> {
    pub const fn new(entries: [OcallEntry; N]) -> Self {
        Self {
            nr_ocall: N,
            entries,
        }
    }

    pub fn as_slice(&self) -> &[OcallEntry] {
        &self.entries
    }
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
    fn wrapper(&self, data: *const u8) -> SgxStatus;
}

impl<P, Target, Args> OcallWrapper<Args, Target> for P
where
    P: Ocall<Target, Args = Args>,
    Args: OcallArg<Target>,
    Target: 'static,
{
    fn wrapper(&self, data: *const u8) -> SgxStatus {
        let msg = unsafe { &mut *(data as *mut u8 as *mut OcallMsg) };
        let bytes = unsafe { std::slice::from_raw_parts(msg.addr as *const u8, msg.len) };

        // deserialize the arguments
        let mut raw_args = Args::deserialize(&bytes);

        let in_retval = P::call(self, raw_args);
        core::mem::replace(&mut msg.retval, in_retval);

        SgxStatus::Success
    }
}

#[cfg(feature = "enclave")]
pub fn enclave_ocall<Args, Target>(idx: usize, mut args: Args) -> (u32, SgxStatus)
where
    Args: OcallArg<Target>,
{
    use core::ffi::c_void;

    let mut retval = SgxStatus::default();
    let data = args.serialize();
    let size = core::mem::size_of::<OcallMsg>() + data.len();
    // allocate in untrusted memory
    let tmp = unsafe { sgx_ocalloc(size) };
    if tmp.is_null() {
        unsafe { sgx_ocfree() };
        return (SgxStatus::Unexpected.into(), SgxStatus::Unexpected);
    }

    // 由于序列化后的长度不确定，因此将Vec再进行一次序列化。
    let msg = unsafe { &mut *(tmp as *mut OcallMsg) };
    unsafe {
        *msg = OcallMsg {
            addr: tmp.add(core::mem::size_of::<OcallMsg>()) as usize,
            len: data.len(),
            retval: SgxStatus::default(),
        };
        core::slice::from_raw_parts_mut(msg.addr as *mut u8, msg.len).copy_from_slice(&data);
    }

    let status = unsafe { sgx_ocall(idx as i32, tmp as *mut c_void) };
    retval = msg.retval.clone();
    // if status == 0 {
    //     retval = msg.retval.clone();
    // } else {
    //     retval = SgxStatus::Unexpected;
    // }
    unsafe { sgx_ocfree() };
    (status, retval)
}

#[repr(C)]
struct OcallMsg {
    addr: usize,
    len: usize,
    retval: SgxStatus,
}
