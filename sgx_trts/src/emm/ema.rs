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

use crate::arch::{SE_PAGE_SHIFT, SE_PAGE_SIZE};
use crate::emm::{PageInfo, PageRange, PageType, ProtFlags};
use crate::enclave::is_within_enclave;
use alloc::boxed::Box;
use intrusive_collections::{intrusive_adapter, LinkedListLink, UnsafeRef};
use sgx_tlibc_sys::{c_void, EACCES, EINVAL};
use sgx_types::error::OsResult;

use super::alloc::Alloc;
use super::alloc::{ResAlloc, StaticAlloc};
use super::bitmap::BitArray;
use super::ocall;
use super::page::AllocFlags;
use super::pfhandler::PfHandler;

/// Enclave Management Area
#[repr(C)]
pub struct EMA {
    // page aligned start address
    start: usize,
    // bytes, round to page bytes
    length: usize,
    alloc_flags: AllocFlags,
    info: PageInfo,
    // bitmap for EACCEPT status
    // FIXME: replace BitArray with pointer
    eaccept_map: Option<BitArray>,
    // custom PF handler
    handler: Option<PfHandler>,
    // private data for PF handler
    priv_data: Option<*mut c_void>,
    alloc: Alloc,
    // intrusive linkedlist
    link: LinkedListLink,
}

// TODO: remove send and sync
unsafe impl Send for EMA {}
unsafe impl Sync for EMA {}

impl EMA {
    /// Initialize EMA node with null eaccept map,
    /// and start address must be page aligned
    pub fn new(
        start: usize,
        length: usize,
        alloc_flags: AllocFlags,
        info: PageInfo,
        handler: Option<PfHandler>,
        priv_data: Option<*mut c_void>,
        alloc: Alloc,
    ) -> OsResult<Self> {
        // check alloc flags' eligibility
        // AllocFlags::try_from(alloc_flags.bits())?;

        if start != 0
            && length != 0
            && is_within_enclave(start as *const u8, length)
            && is_page_aligned!(start)
            && (length % crate::arch::SE_PAGE_SIZE) == 0
        {
            Ok(Self {
                start,
                length,
                alloc_flags,
                info,
                eaccept_map: None,
                handler,
                priv_data,
                link: LinkedListLink::new(),
                alloc,
            })
        } else {
            Err(EINVAL)
        }
    }

    /// Split current ema at specified address, return a new allocated ema
    /// corresponding to the memory at the range of [addr, end).
    /// And the current ema manages the memory at the range of [start, addr).
    pub fn split(&mut self, addr: usize) -> OsResult<*mut EMA> {
        let l_start = self.start;
        let l_length = addr - l_start;

        let r_start = addr;
        let r_length = (self.start + self.length) - addr;

        let new_bitarray = match &mut self.eaccept_map {
            Some(bitarray) => {
                let pos = (addr - self.start) >> crate::arch::SE_PAGE_SHIFT;
                Some(bitarray.split(pos)?)
            }
            None => None,
        };

        // Initialize EMA with same allocator
        let new_ema: *mut EMA = match self.alloc {
            Alloc::Reserve => {
                let mut ema = Box::new_in(
                    EMA::new(
                        self.start,
                        self.length,
                        self.alloc_flags,
                        self.info,
                        self.handler,
                        self.priv_data,
                        Alloc::Reserve,
                    )
                    .unwrap(),
                    ResAlloc,
                );
                ema.start = r_start;
                ema.length = r_length;
                ema.eaccept_map = new_bitarray;
                Box::into_raw(ema)
            }
            Alloc::Static => {
                let mut ema = Box::new_in(
                    EMA::new(
                        self.start,
                        self.length,
                        self.alloc_flags,
                        self.info,
                        self.handler,
                        self.priv_data,
                        Alloc::Static,
                    )
                    .unwrap(),
                    StaticAlloc,
                );
                ema.start = r_start;
                ema.length = r_length;
                ema.eaccept_map = new_bitarray;
                Box::into_raw(ema)
            }
        };

        self.start = l_start;
        self.length = l_length;

        Ok(new_ema)
    }

    /// Allocate the reserve / committed / virtual memory at corresponding memory
    pub fn alloc(&mut self) -> OsResult {
        // RESERVED region only occupy memory range, but no real allocation
        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Ok(());
        }

        // Allocate new eaccept_map for COMMIT_ON_DEMAND and COMMIT_NOW
        if self.eaccept_map.is_none() {
            let eaccept_map = match self.alloc {
                Alloc::Reserve => {
                    let page_num = self.length >> SE_PAGE_SHIFT;
                    BitArray::new(page_num, Alloc::Reserve)?
                }
                Alloc::Static => {
                    let page_num = self.length >> SE_PAGE_SHIFT;
                    BitArray::new(page_num, Alloc::Static)?
                }
            };
            self.eaccept_map = Some(eaccept_map);
        };

        // Ocall to mmap memory in urts
        ocall::alloc_ocall(self.start, self.length, self.info.typ, self.alloc_flags)?;

        // Set the corresponding bits of eaccept map
        if self.alloc_flags.contains(AllocFlags::COMMIT_NOW) {
            let grow_up: bool = !self.alloc_flags.contains(AllocFlags::GROWSDOWN);
            self.eaccept(self.start, self.length, grow_up)?;
            self.eaccept_map.as_mut().unwrap().set_full();
        } else {
            self.eaccept_map.as_mut().unwrap().clear();
        }
        Ok(())
    }

    /// Eaccept target EPC pages with cpu instruction
    fn eaccept(&self, start: usize, length: usize, grow_up: bool) -> OsResult {
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

    /// Check the prerequisites of ema commitment
    pub fn commit_check(&self) -> OsResult {
        if !self.info.prot.intersects(ProtFlags::R | ProtFlags::W) {
            return Err(EACCES);
        }

        if self.info.typ != PageType::Reg {
            return Err(EACCES);
        }

        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Err(EACCES);
        }

        Ok(())
    }

    /// Commit the corresponding memory of this ema
    pub fn commit_self(&mut self) -> OsResult {
        self.commit(self.start, self.length)
    }

    /// Commit the partial memory of this ema
    pub fn commit(&mut self, start: usize, length: usize) -> OsResult {
        ensure!(
            length != 0
                && (length % crate::arch::SE_PAGE_SIZE) == 0
                && start >= self.start
                && start + length <= self.start + self.length,
            EINVAL
        );

        let info = PageInfo {
            typ: PageType::Reg,
            prot: ProtFlags::R | ProtFlags::W | ProtFlags::PENDING,
        };

        let pages = PageRange::new(start, length / crate::arch::SE_PAGE_SIZE, info)?;

        // Page index of the start address
        let init_idx = (start - self.start) >> crate::arch::SE_PAGE_SHIFT;
        let map = self.eaccept_map.as_mut().unwrap();

        for (idx, page) in pages.iter().enumerate() {
            let page_idx = idx + init_idx;
            if map.get(page_idx).unwrap() {
                continue;
            } else {
                page.accept()?;
                map.set(page_idx, true)?;
            }
        }
        Ok(())
    }

    /// Check the prerequisites of ema uncommitment
    pub fn uncommit_check(&self) -> OsResult {
        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Err(EACCES);
        }
        Ok(())
    }

    /// Uncommit the corresponding memory of this ema
    pub fn uncommit_self(&mut self) -> OsResult {
        let prot = self.info.prot;
        if prot == ProtFlags::NONE && (self.info.typ != PageType::Tcs) {
            self.modify_perm(ProtFlags::R)?
        }

        self.uncommit(self.start, self.length, prot)
    }

    /// Uncommit the partial memory of this ema
    pub fn uncommit(&mut self, start: usize, length: usize, prot: ProtFlags) -> OsResult {
        assert!(self.eaccept_map.is_some());

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

        while start < end {
            let mut block_start = start;
            while block_start < end {
                let pos = (block_start - self.start) >> crate::arch::SE_PAGE_SHIFT;
                if map.get(pos).unwrap() {
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
                if map.get(pos).unwrap() {
                    block_end += crate::arch::SE_PAGE_SIZE;
                } else {
                    break;
                }
            }

            let block_length = block_end - block_start;
            ocall::modify_ocall(
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
                map.set(pos, false)?;
            }

            // Notify trimming
            ocall::modify_ocall(
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

    /// Check the prerequisites of modifying permissions
    pub fn modify_perm_check(&self) -> OsResult {
        if self.info.typ != PageType::Reg {
            return Err(EACCES);
        }

        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Err(EACCES);
        }

        match &self.eaccept_map {
            Some(bitmap) => {
                if !bitmap.all_true() {
                    return Err(EINVAL);
                }
            }
            None => {
                return Err(EINVAL);
            }
        }

        Ok(())
    }

    /// Modifying the permissions of corresponding memory of this ema
    pub fn modify_perm(&mut self, new_prot: ProtFlags) -> OsResult {
        if self.info.prot == new_prot {
            return Ok(());
        }

        // Notify modifying permissions
        ocall::modify_ocall(
            self.start,
            self.length,
            self.info,
            PageInfo {
                typ: self.info.typ,
                prot: new_prot,
            },
        )?;

        let info = PageInfo {
            typ: PageType::Reg,
            prot: new_prot | ProtFlags::PR,
        };

        let pages = PageRange::new(self.start, self.length / crate::arch::SE_PAGE_SIZE, info)?;

        // Modpe the EPC with cpu instruction
        for page in pages.iter() {
            if (new_prot | self.info.prot) != self.info.prot {
                page.modpe()?;
            }

            // If the new permission is RWX, no EMODPR needed in untrusted part (modify ocall)
            if (new_prot & (ProtFlags::W | ProtFlags::X)) != (ProtFlags::W | ProtFlags::X) {
                page.accept()?;
            }
        }

        self.info = PageInfo {
            typ: self.info.typ,
            prot: new_prot,
        };

        if new_prot == ProtFlags::NONE {
            ocall::modify_ocall(
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

    /// Changing the page type from Reg to Tcs
    pub fn change_to_tcs(&mut self) -> OsResult {
        // The ema must have and only have one page
        if self.length != SE_PAGE_SIZE {
            return Err(EINVAL);
        }

        // The page must be committed
        if !self.is_page_committed(self.start) {
            return Err(EACCES);
        }

        let info = self.info;
        if info.typ == PageType::Tcs {
            return Ok(());
        }

        if (info.prot != (ProtFlags::R | ProtFlags::W)) || (info.typ != PageType::Reg) {
            return Err(EACCES);
        }

        ocall::modify_ocall(
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

    pub fn is_page_committed(&self, addr: usize) -> bool {
        assert!(addr % SE_PAGE_SIZE == 0);
        if self.eaccept_map.is_none() {
            return false;
        }
        let pos = (addr - self.start) >> SE_PAGE_SHIFT;
        self.eaccept_map.as_ref().unwrap().get(pos).unwrap()
    }

    /// Deallocate the corresponding memory of this ema
    pub fn dealloc(&mut self) -> OsResult {
        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Ok(());
        }

        if self.info.prot == ProtFlags::NONE && (self.info.typ != PageType::Tcs) {
            self.modify_perm(ProtFlags::R)?;
        }
        self.uncommit(self.start, self.length, ProtFlags::NONE)?;
        Ok(())
    }

    /// Obtain the aligned end address
    pub fn aligned_end(&self, align: usize) -> usize {
        let curr_end = self.start + self.length;
        round_to!(curr_end, align)
    }

    /// Obtain the end address of ema
    pub fn end(&self) -> usize {
        self.start + self.length
    }

    /// Obtain the start address of ema
    pub fn start(&self) -> usize {
        self.start
    }

    /// Obtain the length of ema (bytes)
    pub fn len(&self) -> usize {
        self.length
    }

    /// Check if the ema range is lower than the address
    pub fn lower_than_addr(&self, addr: usize) -> bool {
        self.end() <= addr
    }

    /// Check if the ema range is higher than the address
    pub fn higher_than_addr(&self, addr: usize) -> bool {
        self.start >= addr
    }

    /// Check if the ema range contains the specified address
    pub fn overlap_addr(&self, addr: usize) -> bool {
        (addr >= self.start) && (addr < self.start + self.length)
    }

    pub fn set_eaccept_map_full(&mut self) -> OsResult {
        if self.eaccept_map.is_none() {
            let mut eaccept_map = match self.alloc {
                Alloc::Reserve => {
                    let page_num = self.length >> SE_PAGE_SHIFT;
                    BitArray::new(page_num, Alloc::Reserve)?
                }
                Alloc::Static => {
                    let page_num = self.length >> SE_PAGE_SHIFT;
                    BitArray::new(page_num, Alloc::Static)?
                }
            };
            eaccept_map.set_full();
            self.eaccept_map = Some(eaccept_map);
        } else {
            self.eaccept_map.as_mut().unwrap().set_full();
        }
        Ok(())
    }

    fn set_flags(&mut self, flags: AllocFlags) {
        self.alloc_flags = flags;
    }

    fn set_prot(&mut self, info: PageInfo) {
        self.info = info;
    }

    /// Obtain the allocator of ema
    pub fn allocator(&self) -> Alloc {
        self.alloc
    }

    pub fn flags(&self) -> AllocFlags {
        self.alloc_flags
    }

    pub fn info(&self) -> PageInfo {
        self.info
    }

    pub fn fault_handler(&self) -> (Option<PfHandler>, Option<*mut c_void>) {
        (self.handler, self.priv_data)
    }
}

// Implement ema adapter for the operations of intrusive linkedlist
intrusive_adapter!(pub EmaAda = UnsafeRef<EMA>: EMA { link: LinkedListLink });

// pub struct EmaRange<'a> {
//     pub cursor: CursorMut<'a, EmaAda>,
//     pub count : usize,
// }

// impl<'a> Iterator for EmaRange<'a> {
//     type Item = &'a mut EMA;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.count == 0 {
//             None
//         } else {
//             self.cursor.move_next();
//             self.count -= 1;

//             let ema = unsafe { self.cursor.get_mut().unwrap() };
//             Some(ema)
//         }
//     }
// }
