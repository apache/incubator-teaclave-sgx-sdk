use crate::{ser::*, Update};
#[cfg(feature = "enclave")]
use sgx_trts::capi::{sgx_ocalloc, sgx_ocfree};
use sgx_types::error::SgxStatus;

pub type ExternOcallFn = unsafe extern "C" fn(*const u8) -> sgx_types::error::SgxStatus;

use std::vec::Vec;

pub trait OcallArg<Target> {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;

    // /// 将enclave内部的参数更新到外部
    // fn update(&mut self, other: Self);

    unsafe fn _clone(&mut self) -> Self;
}

impl OcallArg<()> for () {
    fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }

    fn deserialize(_: &[u8]) -> Self {
        ()
    }

    // fn prepare(&self) -> () {
    //     ()
    // }

    // unsafe fn _from_mut(target: &mut ()) -> Self {
    //     ()
    // }

    // fn update(&mut self, _: ()) {}

    unsafe fn _clone(&mut self) -> Self {
        ()
    }
}

fn sgx_ocall(idx: usize, ms: *const u8) -> SgxStatus {
    panic!("exec sgx ocall")
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
        let bytes = unsafe {
            std::slice::from_raw_parts(data, core::mem::size_of::<((usize, usize), usize)>())
        };
        // ptr: arg address, len: arg bytes len, retval: sgx status address
        let ((ptr, len), retval) = deserialize::<((usize, usize), usize)>(bytes).unwrap();
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };

        // deserialize the arguments
        let mut raw_args = Args::deserialize(&bytes);
        let retval = unsafe { &mut *(retval as *mut SgxStatus) };

        let in_retval = Ocall::call(self, raw_args);
        *retval = in_retval;

        SgxStatus::Success
    }
}

#[cfg(feature = "enclave")]
pub fn enclave_ocall<Args, Target>(idx: usize, mut args: Args) -> SgxStatus
where
    Args: OcallArg<Target>,
{
    struct Head {
        addr: *mut u8,
        len: usize,
        retval: SgxStatus,
    }

    let mut retval = SgxStatus::default();
    let data = args.serialize();
    let size = core::mem::size_of::<Head>() + data.len();
    // allocate in untrusted memory
    let tmp = unsafe { sgx_ocalloc(size) };
    if tmp.is_null() {
        unsafe { sgx_ocfree() };
        return SgxStatus::Unexpected;
    }

    // 由于序列化后的长度不确定，因此将Vec再进行一次序列化。
    unsafe {
        *(tmp as *mut Head) = Head {
            addr: tmp.add(core::mem::size_of::<Head>()),
            len: data.len(),
            retval: SgxStatus::default(),
        };
        core::slice::from_raw_parts_mut(tmp, data.len()).copy_from_slice(&data);
    }

    let status = sgx_ocall(idx, tmp);
    if status == SgxStatus::Success {
        let head = unsafe { &*(tmp as *mut Head) };
        let data = unsafe { core::slice::from_raw_parts(head.addr, head.len) };
        let arg = Args::deserialize(data);
        retval = head.retval;
    }
    unsafe { sgx_ocfree() };
    retval
}
