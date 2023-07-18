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

use crate::arch::SE_PAGE_SHIFT;
use crate::arch::SE_PAGE_SIZE;
use crate::edmm::perm;
use crate::edmm::PageRange;
use crate::edmm::{PageInfo, PageType, ProtFlags};
use crate::enclave::is_within_enclave;
use alloc::alloc::Global;
use alloc::boxed::Box;
use intrusive_collections::intrusive_adapter;
use intrusive_collections::LinkedListLink;
use intrusive_collections::UnsafeRef;
use sgx_types::error::SgxResult;
use sgx_types::error::SgxStatus;

use crate::feature::SysFeatures;
use crate::trts::Version;
use crate::veh::{ExceptionHandler, ExceptionInfo};

use super::alloc::ResAlloc;
use super::alloc::StaticAlloc;
use super::bitmap::BitArray;
use super::flags::AllocFlags;
use super::interior::Alloc;

#[repr(C)]
#[derive(Clone)]
pub struct EMA {
    // starting address, page aligned
    start: usize,
    // bytes, or page may be more available
    length: usize,
    alloc_flags: AllocFlags,
    info: PageInfo,
    // bitmap for EACCEPT status
    eaccept_map: Option<BitArray>,
    // custom PF handler
    handler: Option<ExceptionHandler>,
    // private data for handler
    priv_data: Option<*mut ExceptionInfo>,
    alloc: Alloc,
    // intrusive linkedlist
    link: LinkedListLink,
}

// TODO: remove send and sync
unsafe impl Send for EMA {}
unsafe impl Sync for EMA {}

impl EMA {
    // start address must be page aligned
    pub fn new(
        start: usize,
        length: usize,
        alloc_flags: AllocFlags,
        info: PageInfo,
        handler: Option<ExceptionHandler>,
        priv_data: Option<*mut ExceptionInfo>,
        alloc: Alloc,
    ) -> SgxResult<Self> {
        // check flags' eligibility
        AllocFlags::try_from(alloc_flags.bits())?;

        if start != 0
            && length != 0
            && is_within_enclave(start as *const u8, length)
            && is_page_aligned!(start)
            && (length % crate::arch::SE_PAGE_SIZE) == 0
        {
            return Ok(Self {
                start,
                length,
                alloc_flags,
                info,
                eaccept_map: None,
                handler,
                priv_data,
                link: LinkedListLink::new(),
                alloc,
            });
        } else {
            return Err(SgxStatus::InvalidParameter);
        }
    }

    // Returns a newly allocated ema in charging of the memory in the range [addr, addr + len).
    // After the call, the original ema will be left containing the elements [start, addr)
    // with its previous capacity unchanged.
    pub fn split(&mut self, addr: usize) -> SgxResult<*mut EMA> {
        let l_start = self.start;
        let l_length = addr - l_start;

        let r_start = addr;
        let r_length = (self.start + self.length) - addr;

        let new_bitarray = match &mut self.eaccept_map {
            Some(bitarray) => {
                let pos = (addr - self.start) >> crate::arch::SE_PAGE_SHIFT;
                // split self.eaccept_map
                Some(bitarray.split(pos)?)
            }
            None => None,
        };

        let new_ema: *mut EMA = match self.alloc {
            Alloc::Reserve => {
                let mut ema = Box::new_in(self.clone(), ResAlloc);
                ema.start = r_start;
                ema.length = r_length;
                ema.eaccept_map = new_bitarray;
                Box::into_raw(ema)
            }
            Alloc::Static => {
                let mut ema = Box::new_in(self.clone(), StaticAlloc);
                ema.start = r_start;
                ema.length = r_length;
                ema.eaccept_map = new_bitarray;
                Box::into_raw(ema)
            }
        };

        self.start = l_start;
        self.length = l_length;

        return Ok(new_ema);
    }

    // FIXME: handling reserve ema node doesn't have ema eaccept
    /// Alloc the reserve / committed / vitual memory indeed
    pub fn alloc(&mut self) -> SgxResult {
        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Ok(());
        }

        // new self bitmap

        // COMMIT_ON_DEMAND and COMMIT_NOW both need to mmap memory in urts
        perm::alloc_ocall(self.start, self.length, self.info.typ, self.alloc_flags)?;

        if self.alloc_flags.contains(AllocFlags::COMMIT_NOW) {
            let grow_up: bool = if self.alloc_flags.contains(AllocFlags::GROWSDOWN) {
                false
            } else {
                true
            };
            self.eaccept(self.start, self.length, grow_up)?;
            // set eaccept map full
            match &mut self.eaccept_map {
                Some(map) => {
                    map.set_full();
                }
                None => {
                    // COMMIT_NOW must have eaccept_map
                    return Err(SgxStatus::Unexpected);
                }
            }
        } else {
            // clear eaccept map
            match &mut self.eaccept_map {
                Some(map) => {
                    map.clear();
                }
                None => {
                    // COMMIT_NOW must have eaccept_map
                    return Err(SgxStatus::Unexpected);
                }
            }
        }
        return Ok(());
    }

    /// do eaccept for targeted EPC page
    /// similiar to "apply_epc_pages(addr: usize, count: usize)" / intel emm do_commit()
    /// do not change eaccept map
    fn eaccept(&self, start: usize, length: usize, grow_up: bool) -> SgxResult {
        let info = PageInfo {
            typ: self.info.typ,
            prot: self.info.prot | ProtFlags::PENDING,
        };

        let pages = PageRange::new(start, length / crate::arch::SE_PAGE_SIZE, info)?;

        if grow_up {
            pages.accept_backward()
        } else {
            pages.accept_forward()
        }
    }

    // Attension, return EACCES SgxStatus may be more appropriate
    pub fn commit_check(&self) -> SgxResult {
        if self.info.prot.intersects(ProtFlags::R | ProtFlags::W) {
            return Err(SgxStatus::InvalidParameter);
        }

        if self.info.typ != PageType::Reg {
            return Err(SgxStatus::InvalidParameter);
        }

        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Err(SgxStatus::InvalidParameter);
        }

        Ok(())
    }

    /// commit all the memory in this ema
    pub fn commit_self(&mut self) -> SgxResult {
        self.commit(self.start, self.length)
    }

    /// ema_do_commit
    pub fn commit(&mut self, start: usize, length: usize) -> SgxResult {
        ensure!(
            length != 0
                && (length % crate::arch::SE_PAGE_SIZE) == 0
                && start >= self.start
                && start + length <= self.start + self.length,
            SgxStatus::InvalidParameter
        );

        let info = PageInfo {
            typ: PageType::Reg,
            prot: ProtFlags::R | ProtFlags::W | ProtFlags::PENDING,
        };

        let pages = PageRange::new(start, length / crate::arch::SE_PAGE_SIZE, info)?;

        // page index for parsing start address
        let init_idx = (start - self.start) >> crate::arch::SE_PAGE_SHIFT;
        let map = self.eaccept_map.as_mut().unwrap();

        for (idx, page) in pages.iter().enumerate() {
            let page_idx = idx + init_idx;
            if map.get(page_idx) {
                continue;
            } else {
                page.accept()?;
                map.set(page_idx, true);
            }
        }
        return Ok(());
    }

    pub fn uncommit_check(&self) -> SgxResult {
        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Err(SgxStatus::InvalidParameter);
        }
        Ok(())
    }

    pub fn uncommit_self(&mut self) -> SgxResult {
        let prot = self.info.prot;
        if prot == ProtFlags::NONE {
            self.modify_perm(ProtFlags::R)?
        }

        self.uncommit(self.start, self.length, prot)
    }

    /// uncommit EPC page
    pub fn uncommit(&mut self, start: usize, length: usize, prot: ProtFlags) -> SgxResult {
        // need READ for trimming
        ensure!(self.eaccept_map.is_some(), SgxStatus::InvalidParameter);

        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Ok(());
        }

        let trim_info = PageInfo {
            typ: PageType::Trim,
            prot: ProtFlags::MODIFIED,
        };

        let map = self.eaccept_map.as_mut().unwrap();
        let mut start = start;
        let end: usize = start + length;

        // TODO: optimized with [u8] slice
        while start < end {
            let mut block_start = start;
            while block_start < end {
                let pos = (block_start - self.start) >> crate::arch::SE_PAGE_SHIFT;
                if map.get(pos) {
                    break;
                } else {
                    block_start += crate::arch::SE_PAGE_SIZE;
                }
            }

            if block_start == end {
                break;
            }

            let mut block_end = block_start + crate::arch::SE_PAGE_SIZE;
            while block_end < end {
                let pos = (block_end - self.start) >> crate::arch::SE_PAGE_SHIFT;
                if map.get(pos) {
                    block_end += crate::arch::SE_PAGE_SIZE;
                } else {
                    break;
                }
            }

            let block_length = block_end - block_start;
            perm::modify_ocall(
                block_start,
                block_length,
                PageInfo {
                    typ: self.info.typ,
                    prot,
                },
                PageInfo {
                    typ: PageType::Trim,
                    prot,
                },
            )?;

            let pages = PageRange::new(
                block_start,
                block_length / crate::arch::SE_PAGE_SIZE,
                trim_info,
            )?;

            let init_idx = (block_start - self.start) >> crate::arch::SE_PAGE_SHIFT;
            for (idx, page) in pages.iter().enumerate() {
                page.accept()?;
                let pos = idx + init_idx;
                map.set(pos, false);
            }

            // eaccept trim notify
            perm::modify_ocall(
                block_start,
                block_length,
                PageInfo {
                    typ: PageType::Trim,
                    prot,
                },
                PageInfo {
                    typ: PageType::Trim,
                    prot,
                },
            )?;
            start = block_end;
        }
        Ok(())
    }

    pub fn modify_perm_check(&self) -> SgxResult {
        if self.info.typ != PageType::Reg {
            return Err(SgxStatus::InvalidParameter);
        }

        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Err(SgxStatus::InvalidParameter);
        }

        match &self.eaccept_map {
            Some(bitmap) => {
                if !bitmap.all_true() {
                    return Err(SgxStatus::InvalidParameter);
                }
            }
            None => {
                return Err(SgxStatus::InvalidParameter);
            }
        }

        Ok(())
    }

    pub fn modify_perm(&mut self, new_prot: ProtFlags) -> SgxResult {
        if self.info.prot == new_prot {
            return Ok(());
        }

        if SysFeatures::get().version() == Version::Sdk2_0 {
            perm::modify_ocall(
                self.start,
                self.length,
                self.info,
                PageInfo {
                    typ: self.info.typ,
                    prot: new_prot,
                },
            )?;
        }

        let info = PageInfo {
            typ: PageType::Reg,
            prot: new_prot | ProtFlags::PR,
        };

        let pages = PageRange::new(self.start, self.length / crate::arch::SE_PAGE_SIZE, info)?;

        for page in pages.iter() {
            // If new_prot is the subset of self.info.prot, no need to apply modpe.
            // So we can't use new_prot != self.info.prot as determination
            if (new_prot | self.info.prot) != self.info.prot {
                page.modpe()?;
            }

            // new permission is RWX, no EMODPR needed in untrusted part, hence no
            // EACCEPT
            if (new_prot & (ProtFlags::W | ProtFlags::X)) != (ProtFlags::W | ProtFlags::X) {
                page.accept()?;
            }
        }

        self.info = PageInfo {
            typ: self.info.typ,
            prot: new_prot,
        };

        if new_prot == ProtFlags::NONE && SysFeatures::get().version() == Version::Sdk2_0 {
            perm::modify_ocall(
                self.start,
                self.length,
                PageInfo {
                    typ: self.info.typ,
                    prot: ProtFlags::NONE,
                },
                PageInfo {
                    typ: self.info.typ,
                    prot: ProtFlags::NONE,
                },
            )?;
        }

        Ok(())
    }

    pub fn change_to_tcs(&mut self) -> SgxResult {
        // the ema has and only has one page
        if self.length != SE_PAGE_SIZE {
            return Err(SgxStatus::InvalidParameter);
        }

        // page must be committed
        if !self.is_page_committed(self.start) {
            return Err(SgxStatus::InvalidParameter);
        }

        let info = self.info;

        // page has been changed to tcs
        if info.typ == PageType::Tcs {
            return Ok(());
        }

        if (info.prot != (ProtFlags::R | ProtFlags::W)) || (info.typ != PageType::Reg) {
            return Err(SgxStatus::InvalidParameter);
        }

        perm::modify_ocall(
            self.start,
            self.length,
            info,
            PageInfo {
                typ: PageType::Tcs,
                prot: info.prot,
            },
        )?;

        let eaccept_info = PageInfo {
            typ: PageType::Tcs,
            prot: ProtFlags::MODIFIED,
        };

        let pages = PageRange::new(
            self.start,
            self.length / crate::arch::SE_PAGE_SIZE,
            eaccept_info,
        )?;

        for page in pages.iter() {
            page.accept()?;
        }

        self.info = PageInfo {
            typ: PageType::Tcs,
            prot: ProtFlags::NONE,
        };
        Ok(())
    }

    fn is_page_committed(&self, addr: usize) -> bool {
        assert!(addr % SE_PAGE_SIZE == 0);
        if self.eaccept_map.is_none() {
            return false;
        }
        let pos = (addr - self.start) >> SE_PAGE_SHIFT;
        self.eaccept_map.as_ref().unwrap().get(pos)
    }

    pub fn dealloc(&mut self) -> SgxResult {
        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Ok(());
        }

        if self.info.prot == ProtFlags::NONE {
            self.modify_perm(ProtFlags::R)?;
        }
        self.uncommit(self.start, self.length, ProtFlags::NONE)?;
        Ok(())
    }

    pub fn aligned_end(&self, align: usize) -> usize {
        let curr_end = self.start + self.length;
        round_to!(curr_end, align)
    }

    pub fn end(&self) -> usize {
        self.start + self.length
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn lower_than_addr(&self, addr: usize) -> bool {
        self.end() <= addr
    }

    pub fn higher_than_addr(&self, addr: usize) -> bool {
        self.start >= addr
    }

    pub fn set_flags(&mut self, flags: AllocFlags) {
        self.alloc_flags = flags;
    }
    pub fn set_prot(&mut self, info: PageInfo) {
        self.info = info;
    }
    fn flags(&self) -> AllocFlags {
        self.alloc_flags
    }
    fn info(&self) -> PageInfo {
        self.info
    }
    fn handler(&self) -> Option<ExceptionHandler> {
        self.handler
    }
}

intrusive_adapter!(pub EmaAda = UnsafeRef<EMA>: EMA { link: LinkedListLink });
