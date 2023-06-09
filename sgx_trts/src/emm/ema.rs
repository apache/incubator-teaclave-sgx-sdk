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

use core::alloc::Allocator;

use crate::arch::Secinfo;
use crate::arch::SecinfoFlags;
use crate::edmm::perm;
use crate::edmm::PageRange;
use crate::edmm::{PageInfo, PageType, ProtFlags};
use crate::enclave::is_within_enclave;
use alloc::boxed::Box;
use intrusive_collections::intrusive_adapter;
use intrusive_collections::LinkedListLink;
use sgx_types::error::SgxResult;
use sgx_types::error::SgxStatus;

use crate::feature::SysFeatures;
use crate::trts::Version;
use crate::veh::{ExceptionHandler, ExceptionInfo};

use super::alloc::ResAlloc;
use super::bitmap::BitArray;
use super::flags::AllocFlags;

// pub struct Box<T, A = Global>(_, _)
// where
//          A: Allocator,
//          T: ?Sized;

#[repr(C)]
#[derive(Clone)]
pub struct EMA<A>
where
    A: Allocator + Clone,
{
    // starting address, page aligned
    start: usize,
    // bytes, or page may be more available
    length: usize,
    alloc_flags: AllocFlags,
    info: PageInfo,
    // bitmap for EACCEPT status
    eaccept_map: Option<BitArray<A>>,
    // custom PF handler (for EACCEPTCOPY use)
    handler: Option<ExceptionHandler>,
    // private data for handler
    priv_data: Option<*mut ExceptionInfo>,
    alloc: A,
    // intrusive linkedlist
    link: LinkedListLink,
}

impl<A> EMA<A>
where
    A: Allocator + Clone,
{
    // start address must be page aligned
    pub fn new(
        start: usize,
        length: usize,
        alloc_flags: AllocFlags,
        info: PageInfo,
        handler: Option<ExceptionHandler>,
        priv_data: Option<*mut ExceptionInfo>,
        alloc: A,
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

    // Returns a newly allocated ema in charging of the memory in the range [addr, len). 
    // After the call, the original ema will be left containing the elements [0, addr) 
    // with its previous capacity unchanged.
    pub fn split(&mut self, addr: usize) -> SgxResult<Box<EMA<A>,A>> {
        let l_start = self.start;
        let l_length = addr - l_start;

        let r_start = addr;
        let r_length = (self.start + self.length) - addr;

        let new_bitarray = match &mut self.eaccept_map{
            Some(bitarray) => {
                let pos = (addr - self.start) >> crate::arch::SE_PAGE_SHIFT;
                // split self.eaccept_map
                Some(bitarray.split(pos)?)
            }
            None => {
                None
            }
        };
        
        // 这里之后可以优化
        // 1. self.clone() 会把原有的bitmap重新alloc并复制一份，但其实clone之后这里是None即可
        // 2. 使用Box::new_in 会把 self.clone() 这部分在栈上的数据再拷贝一份到Box新申请的内存区域
        let mut new_ema: Box<EMA<A>,A> = Box::new_in(
            self.clone(), 
            self.alloc.clone()
        );

        self.start = l_start;
        self.length = l_length;
        
        new_ema.start = r_start;
        new_ema.length = r_length;
        new_ema.eaccept_map = new_bitarray;

        return Ok(new_ema);
    }

    // If the previous ema is divided into three parts -> (left ema, middle ema, right ema), return (middle ema, right ema).
    // If the previous ema is divided into two parts -> (left ema, right ema)
    // end split: return (None, right ema), start split: return (left ema, None)
    fn split_into_three(&mut self, start: usize, length: usize) -> SgxResult<(Option<Box<EMA<A>,A>>, Option<Box<EMA<A>,A>>)> {
        if start > self.start {
            let mut new_ema = self.split(start)?;
            if new_ema.start + new_ema.length > start + length {
                let r_ema = new_ema.split(start + length)?;
                return Ok((Some(new_ema), Some(r_ema)));
            } else {
                return Ok((Some(new_ema), None));
            }
        } else {
            if self.start + self.length > start + length {
                let new_ema = self.split(start + length)?;
                return Ok((None, Some(new_ema)));
            } else {
                return Ok((None, None));
            }
        }
    }

    // 这里存在一个问题，如果是reserve ema node, 没有eaccept map怎么办
    /// Alloc the reserve / committed / vitual memory indeed
    pub fn alloc(&mut self) -> SgxResult {
        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Ok(());
        }

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

    /// uncommit EPC page
    pub fn uncommit(&mut self, start: usize, length: usize, prot: ProtFlags) -> SgxResult {
        // need READ for trimming
        ensure!(self.info.prot != ProtFlags::NONE && self.eaccept_map.is_some(), 
            SgxStatus::InvalidParameter);

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
            perm::modify_ocall(block_start, block_length,
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
                trim_info
            )?;

            let init_idx = (block_start - self.start) >> crate::arch::SE_PAGE_SHIFT;
            for (idx, page) in pages.iter().enumerate() {
                page.accept()?;
                let pos = idx + init_idx;
                map.set(pos, false);
            }

            // eaccept trim notify
            perm::modify_ocall(block_start, block_length,
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

    pub fn start(&self) -> usize {
        self.start
    }

    // get and set attributes
    pub fn set_flags(flags: AllocFlags) -> SgxResult<()> {
        todo!()
    }
    pub fn set_prot(info: PageInfo) -> SgxResult<()> {
        todo!()
    }
    fn flags() -> AllocFlags {
        todo!()
    }
    fn info(&self) -> PageInfo {
        self.info
    }
    fn handler(&self) -> Option<ExceptionHandler> {
        self.handler
    }
}

// 
// intrusive_adapter!(pub RegEmaAda = Box<EMA<ResAlloc>, ResAlloc>: EMA<ResAlloc> { link: LinkedListLink });

// regular ema adapter
intrusive_adapter!(pub RegEmaAda = Box<EMA<ResAlloc>>: EMA<ResAlloc> { link: LinkedListLink });

// reserve ema adapter
intrusive_adapter!(pub ResEmaAda = Box<EMA<ResAlloc>>: EMA<ResAlloc> { link: LinkedListLink });

