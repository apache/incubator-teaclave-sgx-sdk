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

use crate::arch::{self, Global, Tcs, TcsPolicy, Tds};
use crate::enclave::parse;
use crate::error;
use crate::feature::SysFeatures;
use crate::rand::Rng;
use crate::sync::OnceCell;
use core::convert::From;
use core::mem;
use core::num::NonZeroUsize;
use core::ptr;
use sgx_types::error::SgxResult;

#[link_section = ".data.rel.ro"]
static mut STACK_CHK_GUARD: OnceCell<NonZeroUsize> = OnceCell::new();

pub const STATIC_STACK_SIZE: usize = 4096;
const CANARY_OFFSET: usize = arch::SE_GUARD_PAGE_SIZE + STATIC_STACK_SIZE - mem::size_of::<usize>();

extern "C" {
    pub fn get_tds() -> *mut Tds;
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TcsId(NonZeroUsize);

impl TcsId {
    pub fn as_usize(self) -> usize {
        self.0.get()
    }
}

impl<'a> From<TcsId> for ThreadControl<'a> {
    fn from(h: TcsId) -> ThreadControl<'a> {
        let tcs = h.0.get() as *const Tcs;
        unsafe { ThreadControl::from_raw(tcs) }
    }
}

#[derive(Debug)]
pub struct ThreadControl<'a> {
    tcs: &'a Tcs,
    tds: &'a mut Tds,
    policy: TcsPolicy,
}

impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub struct TdFlags: usize {
        const UTILITY_THREAD = 0x0001;
        const INIT_THREAD = 0x0002;
        const PTHREAD_CREATE = 0x0004;
    }
}

impl<'a> ThreadControl<'a> {
    #[inline]
    pub fn tcs_policy(&self) -> TcsPolicy {
        self.policy
    }

    #[inline]
    pub fn id(&self) -> TcsId {
        TcsId(NonZeroUsize::new(self.tcs as *const _ as usize).unwrap())
    }

    #[inline]
    pub(crate) fn td_flags(&self) -> TdFlags {
        TdFlags::from_bits(self.tds.flags).unwrap()
    }

    pub(crate) fn from_tcs(tcs: &'a Tcs) -> ThreadControl<'a> {
        let tds = Tds::from_tcs_mut(tcs);
        let policy = if Global::get().tcs_policy != arch::TCS_POLICY_BIND {
            TcsPolicy::Unbind
        } else {
            TcsPolicy::Bind
        };
        ThreadControl { tcs, tds, policy }
    }

    #[inline]
    pub(crate) unsafe fn from_raw(tcs: *const Tcs) -> ThreadControl<'a> {
        ThreadControl::from_tcs(Tcs::from_raw(tcs))
    }

    #[inline]
    pub(crate) fn tcs(&self) -> &Tcs {
        self.tcs
    }

    #[inline]
    pub(crate) fn tds(&self) -> &Tds {
        self.tds
    }

    #[inline]
    pub(crate) fn tds_mut(&mut self) -> &mut Tds {
        self.tds
    }

    #[inline]
    pub(crate) fn is_utility(&self) -> bool {
        self.td_flags().contains(TdFlags::UTILITY_THREAD)
    }

    #[inline]
    pub(crate) fn is_init(&self) -> bool {
        self.td_flags().contains(TdFlags::INIT_THREAD)
    }

    #[allow(unused_variables)]
    #[allow(clippy::ptr_offset_with_cast)]
    pub(crate) fn init(&mut self, tidx: usize, enclave_init: bool) -> SgxResult {
        let tcs_base = self.tcs as *const _ as *const u8;
        let flags = self.tds.flags;

        cfg_if! {
            if #[cfg(not(any(feature = "sim", feature = "hyper")))] {
                let saved_stack_commit = self.tds.stack_commit;
                let first = saved_stack_commit == 0;
            } else if #[cfg(feature = "hyper")] {
                use sgx_types::error::SgxStatus;
                ensure!(tidx < Global::get().tcs_max_num, SgxStatus::Unexpected);
                ensure!(
                    self.tds.self_addr == 0 || self.tds.index == tidx,
                    SgxStatus::Unexpected
                );
            }
        }

        *self.tds = arch::Global::get().td_template;
        self.tds.last_sp = tcs_base.wrapping_offset(self.tds.last_sp as isize) as usize;
        self.tds.self_addr = tcs_base.wrapping_offset(self.tds.self_addr as isize) as usize;
        self.tds.stack_base = tcs_base.wrapping_offset(self.tds.stack_base as isize) as usize;
        self.tds.stack_limit = tcs_base.wrapping_offset(self.tds.stack_limit as isize) as usize;
        self.tds.stack_commit = self.tds.stack_limit;
        self.tds.first_ssa_gpr = tcs_base.wrapping_offset(self.tds.first_ssa_gpr as isize) as usize;
        self.tds.first_ssa_xsave =
            tcs_base.wrapping_offset(self.tds.first_ssa_xsave as isize) as usize;
        self.tds.tls_array = tcs_base.wrapping_offset(self.tds.tls_array as isize) as usize;
        self.tds.tls_addr = tcs_base.wrapping_offset(self.tds.tls_addr as isize) as usize;
        self.tds.last_sp -= STATIC_STACK_SIZE;
        self.tds.stack_base -= STATIC_STACK_SIZE;
        self.tds.stack_guard = Rng::new().next_usize();
        self.tds.flags = flags;
        #[cfg(feature = "hyper")]
        {
            self.tds.index = tidx;
        }
        init_static_stack_guard(self.tcs);

        let is_edmm = SysFeatures::get().is_edmm();
        if enclave_init {
            self.tds.flags |= TdFlags::INIT_THREAD.bits();
            if is_edmm {
                self.tds.flags |= TdFlags::UTILITY_THREAD.bits();
            }
        }

        #[cfg(not(any(feature = "sim", feature = "hyper")))]
        if first {
            // EDMM:
            // set stack_commit
            if is_edmm && (enclave_init || self.is_dyn_tcs()) {
                self.tds.stack_commit += stack_max_page() << arch::SE_PAGE_SHIFT;
            }
        } else {
            self.tds.stack_commit = saved_stack_commit;
        }

        if let Some((tls_addr, tls_size)) = parse::tls_info()? {
            if !tls_addr.is_null() {
                let addr = trim_to_page!(self.tds.tls_addr) as *mut u8;
                let size = round_to_page!(self.tds.self_addr - self.tds.tls_addr);
                unsafe {
                    ptr::write_bytes(addr, 0, size);
                    ptr::copy_nonoverlapping(tls_addr, self.tds.tls_addr as *mut u8, tls_size);
                }
            }
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn is_initialized(&self) -> bool {
        self.tds.self_addr != 0
    }

    #[cfg(not(any(feature = "sim", feature = "hyper")))]
    fn is_dyn_tcs(&self) -> bool {
        let table = crate::edmm::layout::LayoutTable::new();
        if let Some(attr) = table.check_dyn_range(self.tcs() as *const Tcs as usize, 1, None) {
            if attr.flags == arch::SI_FLAGS_TCS {
                return true;
            }
        }
        false
    }
}

impl<'a> PartialEq for ThreadControl<'a> {
    fn eq(&self, other: &ThreadControl) -> bool {
        ptr::eq(self.tcs, other.tcs) && ptr::eq(self.tds, other.tds)
    }
}

impl<'a> Eq for ThreadControl<'a> {}

impl Tds {
    pub fn is_stack_addr(&self, addr: usize, size: usize) -> bool {
        let stack_base = self.stack_base;
        let stack_limit = self.stack_limit;
        (addr <= (addr + size)) && (stack_base >= (addr + size)) && (stack_limit <= addr)
    }

    pub fn is_valid_sp(&self, sp: usize) -> bool {
        (sp & (mem::size_of::<usize>() - 1)) == 0 && self.is_stack_addr(sp, 0)
    }

    fn stack_size(&self) -> usize {
        self.stack_base - self.stack_limit + STATIC_STACK_SIZE
    }
}

pub fn current<'a>() -> ThreadControl<'a> {
    unsafe {
        let raw = get_tds();
        if raw.is_null() {
            error::abort();
        }
        let tds = Tds::from_raw(raw);
        let policy = if Global::get().tcs_policy != 0 {
            TcsPolicy::Unbind
        } else {
            TcsPolicy::Bind
        };
        ThreadControl {
            tcs: Tcs::from_td(tds),
            tds: Tds::from_raw_mut(raw),
            policy,
        }
    }
}

pub fn tcs_max_num() -> usize {
    if SysFeatures::get().is_edmm() {
        Global::get().tcs_max_num
    } else {
        Global::get().tcs_num
    }
}

pub fn tcs_policy() -> TcsPolicy {
    if Global::get().tcs_policy != arch::TCS_POLICY_BIND {
        TcsPolicy::Unbind
    } else {
        TcsPolicy::Bind
    }
}

pub fn stack_size() -> usize {
    current().tds().stack_size()
}

pub fn check_static_stack_guard(tcs: &Tcs) -> bool {
    let canary = (tcs as *const _ as usize - CANARY_OFFSET) as *const usize;
    unsafe { *canary == get_stack_guard().get() }
}

fn init_static_stack_guard(tcs: &Tcs) {
    let canary = (tcs as *const _ as usize - CANARY_OFFSET) as *mut usize;
    unsafe {
        *canary = get_stack_guard().get();
    }
}

pub fn get_stack_guard() -> NonZeroUsize {
    let guard = unsafe {
        STACK_CHK_GUARD.get_or_init(|| loop {
            let r = Rng::new().next_usize();
            if r != 0 {
                break NonZeroUsize::new(r).unwrap();
            }
        })
    };
    *guard
}

#[cfg(not(any(feature = "sim", feature = "hyper")))]
fn stack_max_page() -> usize {
    crate::edmm::layout::LayoutTable::new().dyn_stack_max_page()
}
