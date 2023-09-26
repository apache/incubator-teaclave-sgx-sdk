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

use self::area::{MmArea, MmPerm, MmState, MmType};
use self::manager::{MmAllocAddr, MmManager};
use self::range::MmRange;
use crate::map::{Map, MapObject};
use sgx_sync::{Once, StaticMutex};
use sgx_trts::emm;
use sgx_trts::trts::{self, MmLayout};
use sgx_types::error::errno::*;
use sgx_types::error::OsResult;
use sgx_types::metadata::{SE_PAGE_SHIFT, SE_PAGE_SIZE};
use sgx_types::types::ProtectPerm;

#[macro_use]
mod range;
mod area;
pub(crate) mod manager;

static mut RSRV_MEM: Option<RsrvMem> = None;
static RSRV_MEM_INIT: Once = Once::new();

#[no_mangle]
static mut g_peak_rsrv_mem_committed: usize = 0;

macro_rules! round_to_page {
    ($num:expr) => {
        ($num + SE_PAGE_SIZE - 1) & (!(SE_PAGE_SIZE - 1))
    };
}

macro_rules! trim_to_page {
    ($num:expr) => {
        $num & (!(SE_PAGE_SIZE - 1))
    };
}

#[derive(Debug)]
pub struct RsrvMem {
    base: usize,
    size: usize,
    min_size: usize,
    perm: MmPerm,
    committed_size: usize,
    manager: MmManager,
}

impl RsrvMem {
    pub fn get_or_init() -> OsResult<&'static mut RsrvMem> {
        unsafe {
            RSRV_MEM_INIT.call_once(|| RSRV_MEM = RsrvMem::init().ok());
            RSRV_MEM.as_mut().ok_or(ENOMEM)
        }
    }

    fn init() -> OsResult<RsrvMem> {
        let base = MmLayout::rsrvmem_base();
        let size = MmLayout::rsrvmem_size();
        let min_size = MmLayout::rsrvmem_min_size();
        let perm = MmLayout::rsrvmm_default_perm().into();
        ensure!(base != 0 && size != 0, ENOMEM);

        Ok(RsrvMem {
            base,
            size,
            min_size,
            perm,
            committed_size: 0,
            manager: MmManager::new(base, size, perm)?,
        })
    }

    fn lock() -> impl Drop {
        static LOCK: StaticMutex = StaticMutex::new();
        unsafe { LOCK.lock() }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn default_perm(&self) -> ProtectPerm {
        self.perm.into()
    }

    #[allow(dead_code)]
    #[inline]
    pub fn base(&self) -> usize {
        self.base
    }

    #[allow(dead_code)]
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[allow(dead_code)]
    #[inline]
    pub fn committed_size(&self) -> usize {
        let _lock = Self::lock();
        self.committed_size
    }

    pub unsafe fn mmap<T: Map + 'static>(
        &mut self,
        addr: MmAllocAddr,
        size: usize,
        perm: Option<ProtectPerm>,
        map_object: Option<MapObject<T>>,
    ) -> OsResult<usize> {
        let align_size = round_to_page!(size);
        ensure!(align_size > 0 && align_size <= self.size, EINVAL);

        let align_addr = match addr {
            MmAllocAddr::Any => MmAllocAddr::Any,
            MmAllocAddr::Hint(addr_) => MmAllocAddr::Hint(trim_to_page!(addr_)),
            MmAllocAddr::Need(addr_) | MmAllocAddr::Force(addr_) => {
                ensure!(is_page_aligned!(addr_), EINVAL);
                addr
            }
        };

        if let Some(map_obj) = map_object.as_ref() {
            ensure!(map_obj.can_read(), EACCES);
        }

        let _lock = Self::lock();
        let (insert_idx, free_range) = self.manager.find_free_range(align_addr, align_size)?;
        ensure!(self.manager.check_range(&free_range), ENOMEM);

        let new_range = self
            .manager
            .alloc_range_from(align_addr, align_size, &free_range);

        if new_range.end() > self.base + self.committed_size {
            let pre_committed = self.committed_size;
            let offset = new_range.end() - (self.base + self.committed_size);
            self.committed_size += round_to_page!(offset);

            if trts::is_supported_edmm() && new_range.end() > self.base + self.min_size {
                let (start_addr, size) = if pre_committed > self.min_size {
                    (self.base + pre_committed, round_to_page!(offset))
                } else {
                    (
                        self.base + self.min_size,
                        self.committed_size - self.min_size,
                    )
                };

                let ret = emm::rts_mm_commit(start_addr, size >> SE_PAGE_SHIFT);
                if ret.is_err() {
                    self.committed_size = pre_committed;
                    bail!(ENOMEM);
                }
            }
        }

        g_peak_rsrv_mem_committed = if g_peak_rsrv_mem_committed < self.committed_size {
            self.committed_size
        } else {
            g_peak_rsrv_mem_committed
        };

        let mut vrd = MmArea::new(
            new_range,
            self.perm,
            MmState::Committed,
            MmType::Reg,
            map_object,
        )?;

        vrd.load()?;
        let perm = perm.map(|p| p.into()).unwrap_or(self.perm);
        vrd.apply_perm(perm, MmArea::check_perm)?;
        self.manager.insert_vrd(insert_idx, vrd);

        Ok(new_range.start())
    }

    pub unsafe fn munmap(&mut self, addr: usize, size: usize) -> OsResult {
        let align_size = round_to_page!(size);
        ensure!(
            addr != 0 && align_size > 0 && align_size <= self.size,
            EINVAL
        );
        ensure!(
            trts::is_within_enclave(addr as *const u8, align_size),
            EINVAL
        );

        let range = MmRange::new_with_size(addr, align_size)?;

        let _lock = Self::lock();
        self.manager.free_vrds(&range)
    }

    pub unsafe fn mprotect(&mut self, addr: usize, size: usize, perm: ProtectPerm) -> OsResult {
        let align_size = round_to_page!(size);
        ensure!(
            addr != 0 && align_size > 0 && align_size <= self.size,
            EINVAL
        );
        ensure!(
            trts::is_within_enclave(addr as *const u8, align_size),
            EINVAL
        );

        // if !is_supported_edmm() {
        //     let perm = MmPerm::from(perm);
        //     let _lock = Self::lock();
        //     let cur_perm = self
        //         .manager
        //         .find_vrd(addr)
        //         .map_or_else(|| MmPerm::None, |(_, vrd)| vrd.perm());
        //     if cur_perm == MmPerm::None || (cur_perm == MmPerm::RW && perm.can_execute()) {
        //         bail!(EINVAL);
        //     }
        //     return Ok(())
        // }

        let range = MmRange::new_with_size(addr, align_size)?;

        let _lock = Self::lock();
        self.manager.split_vrds(&range).map_err(|_| ENOMEM)?;
        self.manager.find_vrd(addr).ok_or(EFAULT)?;
        let result = self.manager.apply_perm_vrds(&range, MmPerm::from(perm));
        self.manager.combine_vrds(&range);

        result
    }

    pub unsafe fn msync(&mut self, addr: usize, size: usize) -> OsResult {
        let align_size = round_to_page!(size);
        ensure!(
            addr != 0 && align_size > 0 && align_size <= self.size,
            EINVAL
        );
        ensure!(
            trts::is_within_enclave(addr as *const u8, align_size),
            EINVAL
        );

        let range = MmRange::new_with_size(addr, align_size)?;

        let _lock = Self::lock();
        self.manager.flush_vrds(&range)
    }
}
