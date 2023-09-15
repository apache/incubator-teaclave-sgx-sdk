// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

use crate::arch::{OCallContext, Tcs, TcsPolicy};
use crate::call::ocall;
use crate::enclave;
use crate::enclave::state::{self, State};
use crate::error;
use crate::fence;
use crate::sync::Once;
use crate::tcs::{self, ThreadControl};
use core::convert::{Into, TryFrom};
use core::ffi::c_void;
use core::mem;
use core::num;
use core::slice;
use sgx_types::error::{SgxResult, SgxStatus};

pub static FIRST_ECALL: Once = Once::new();
pub type FnEcall = extern "C" fn(ms: *mut c_void) -> SgxStatus;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ECallIndex {
    ECall(i32),
    RtInit,
    ORet,
    Except,
    MkTcs,
    RtUninit,
    Thread,
    GlobalInit,
    GlobalExit,
}

impl ECallIndex {
    #[link_section = ".nipx"]
    pub fn is_builtin_index(index: i32) -> bool {
        (-6..=-1).contains(&index) || (i32::MAX - 1..=i32::MAX).contains(&index)
    }

    #[link_section = ".nipx"]
    pub fn is_builtin(&self) -> bool {
        !matches!(*self, ECallIndex::ECall(_))
    }

    #[link_section = ".nipx"]
    pub fn is_ecall(&self) -> bool {
        match self {
            ECallIndex::ECall(n) => (0..i32::MAX - 1).contains(n),
            _ => false,
        }
    }

    #[link_section = ".nipx"]
    pub fn is_enclave_init(&self) -> bool {
        matches!(*self, ECallIndex::RtInit)
    }
}

#[allow(clippy::manual_range_contains)]
impl TryFrom<i32> for ECallIndex {
    type Error = num::TryFromIntError;
    #[link_section = ".nipx"]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            v if v >= 0 && v < i32::MAX - 1 => Ok(ECallIndex::ECall(v)),
            -1 => Ok(ECallIndex::RtInit),
            -2 => Ok(ECallIndex::ORet),
            -3 => Ok(ECallIndex::Except),
            -4 => Ok(ECallIndex::MkTcs),
            -5 => Ok(ECallIndex::RtUninit),
            -6 => Ok(ECallIndex::Thread),
            0x7FFFFFFF => Ok(ECallIndex::GlobalInit),
            0x7FFFFFFE => Ok(ECallIndex::GlobalExit),
            _ => Err(u8::try_from(256_u16).unwrap_err()),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<i32> for ECallIndex {
    #[link_section = ".nipx"]
    fn into(self) -> i32 {
        match self {
            ECallIndex::ECall(n) => n,
            ECallIndex::RtInit => -1,
            ECallIndex::ORet => -2,
            ECallIndex::Except => -3,
            ECallIndex::MkTcs => -4,
            ECallIndex::RtUninit => -5,
            ECallIndex::Thread => -6,
            ECallIndex::GlobalInit => i32::MAX,
            ECallIndex::GlobalExit => i32::MAX - 1,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ECallAddr {
    addr: usize,
    is_priv: u8,
    is_switchless: u8,
}

#[derive(Debug)]
pub struct ECallTable {
    ecall_table: [ECallAddr],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RawECallTable {
    nr_ecall: usize,
    ecall_table: [ECallAddr; 1],
}

impl ECallTable {
    pub fn ecall_addr(&self, idx: ECallIndex) -> SgxResult<FnEcall> {
        ensure!(idx.is_ecall(), SgxStatus::InvalidFunction);

        let index = Into::<i32>::into(idx) as usize;
        ensure!(index < self.nr_ecall(), SgxStatus::InvalidFunction);

        if is_root_ecall(&tcs::current()) && self.ecall_table[index].is_priv != 0 {
            bail!(SgxStatus::ECallNotAllowed);
        }

        let addr = self.ecall_table[index].addr;
        ensure!(
            enclave::is_within_enclave(addr as *const u8, 0),
            SgxStatus::Unexpected
        );

        Ok(unsafe { mem::transmute::<usize, FnEcall>(addr) })
    }

    #[inline]
    pub fn get<'a>() -> &'a ECallTable {
        unsafe { Self::from_raw(&g_ecall_table) }
    }

    #[inline]
    pub fn table(&self) -> &[ECallAddr] {
        &self.ecall_table
    }

    #[inline]
    pub fn nr_ecall(&self) -> usize {
        self.ecall_table.len()
    }

    #[inline]
    unsafe fn from_raw(raw: &RawECallTable) -> &ECallTable {
        let bytes = slice::from_raw_parts(&raw.ecall_table[0], raw.nr_ecall);
        &*(bytes as *const [ECallAddr] as *const ECallTable)
    }
}

#[derive(Debug)]
pub struct EntryTable {
    entry_table: [u8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RawEntryTable {
    nr_ocall: usize,
    entry_table: [u8; 1],
}

impl EntryTable {
    pub fn is_allow(&self, idx: ECallIndex, nr_ecall: usize) -> SgxResult {
        ensure!(idx.is_ecall(), SgxStatus::InvalidFunction);

        let index = Into::<i32>::into(idx) as usize;
        ensure!(index < nr_ecall, SgxStatus::InvalidFunction);

        let tc = tcs::current();

        fence::lfence();
        if is_root_ecall(&tc) {
            Ok(())
        } else {
            let context = unsafe { &*(tc.tds().last_sp as *const OCallContext) };
            if context.ocall_flag != ocall::OCALL_FLAG {
                error::abort();
            }

            let ocall_index = context.ocall_index;
            ensure!(ocall_index < self.nr_ocall(), SgxStatus::InvalidFunction);

            if self.entry_table[ocall_index * nr_ecall + index] != 0 {
                Ok(())
            } else {
                Err(SgxStatus::ECallNotAllowed)
            }
        }
    }

    #[inline]
    pub fn get<'a>() -> &'a EntryTable {
        unsafe { Self::from_raw(&g_dyn_entry_table) }
    }

    #[inline]
    pub fn table(&self) -> &[u8] {
        &self.entry_table
    }

    #[inline]
    pub fn nr_ocall(&self) -> usize {
        self.entry_table.len()
    }

    #[inline]
    unsafe fn from_raw(raw: &RawEntryTable) -> &EntryTable {
        let bytes = slice::from_raw_parts(&raw.entry_table[0], raw.nr_ocall);
        &*(bytes as *const [u8] as *const EntryTable)
    }
}

extern "C" {
    static g_ecall_table: RawECallTable;
    static g_dyn_entry_table: RawEntryTable;
}

pub fn ecall<T>(idx: ECallIndex, tcs: &mut Tcs, ms: *mut T, tidx: usize) -> SgxResult {
    ensure!(state::get_state() == State::InitDone, SgxStatus::Unexpected);

    let mut tc = ThreadControl::from_tcs(tcs);
    let is_root_ecall = is_root_ecall(&tc);

    if !tc.is_initialized()
        || (is_root_ecall
            && (tc.tcs_policy() == TcsPolicy::Unbind
                || idx == ECallIndex::Thread
                || thread_is_exit()))
    {
        tc.init(tidx, false)?;
    }

    #[cfg(not(any(feature = "sim", feature = "hyper")))]
    {
        let tds = tc.tds_mut();
        if tds.aex_notify_flag == 1 {
            tds.aex_notify_flag = 0;
            let _ = crate::aexnotify::AEXNotify::set(true);
        }
    }

    #[cfg(not(feature = "hyper"))]
    if is_root_ecall {
        let _ = crate::pkru::Pkru::write(0);
    }

    if !FIRST_ECALL.is_completed() {
        ensure!(is_root_ecall, SgxStatus::ECallNotAllowed);

        FIRST_ECALL.call_once(|| {
            debug_call_once();
            // EDMM:
            #[cfg(not(any(feature = "sim", feature = "hyper")))]
            {
                if crate::feature::SysFeatures::get().is_edmm() {
                    // save all the static tcs into the tcs table. These TCS would be trimmed in the uninit flow.
                    crate::edmm::tcs::add_static_tcs()?;

                    // change back the page permission
                    crate::edmm::mem::change_perm().map_err(|e| {
                        let _ = crate::edmm::tcs::clear_static_tcs();
                        e
                    })?;
                }
            }

            let _ = enclave::ctors();
            Ok(())
        })?;
    }

    let ecall_fn = if idx == ECallIndex::Thread {
        cfg_if! {
            if #[cfg(feature = "thread")] {
                use crate::thread;
                thread::thread_run
            } else {
                bail!(SgxStatus::InvalidFunction);
            }
        }
    } else {
        let ecall_table = ECallTable::get();
        let entry_table = EntryTable::get();
        entry_table.is_allow(idx, ecall_table.nr_ecall())?;
        ecall_table.ecall_addr(idx)?
    };

    fence::lfence();

    cfg_if! {
        if #[cfg(feature = "thread")] {
            use crate::thread::tls::Tls;

            let status =
                if is_root_ecall && idx != ECallIndex::Thread && tcs::tcs_policy() == TcsPolicy::Unbind
            {
                Tls::init();
                let status = ecall_fn(ms.cast());

                let active_tls = Tls::activate();
                drop(active_tls);
                status
            } else {
                ecall_fn(ms.cast())
            };
        } else {
            let status = ecall_fn(ms.cast());
        }
    }

    if status.is_success() {
        Ok(())
    } else {
        Err(status)
    }
}

fn is_root_ecall(tc: &ThreadControl) -> bool {
    tc.tds().stack_base == tc.tds().last_sp
}

#[inline]
pub fn thread_is_exit() -> bool {
    cfg_if! {
        if #[cfg(feature = "thread")] {
            use crate::thread;
            thread::is_exit()
        } else {
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn debug_call_once() {}
