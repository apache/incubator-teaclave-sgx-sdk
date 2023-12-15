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
use crate::emm::alloc::EmmAllocator;
use crate::emm::{PageInfo, PageRange, PageType, ProtFlags};
use crate::enclave::is_within_enclave;
use alloc::boxed::Box;
use intrusive_collections::{intrusive_adapter, LinkedListLink, UnsafeRef};
use sgx_tlibc_sys::{c_void, EACCES, EFAULT, EINVAL};
use sgx_types::error::OsResult;

use super::alloc::AllocType;
use super::bitmap::BitArray;
use super::ocall;
use super::page::AllocFlags;
use super::pfhandler::PfHandler;

/// Enclave Management Area
///
/// Question: should we replace BitArray with pointer
/// to split struct into two pieces of 80 bytes and 32 bytes or an entity of 104 bytes?
pub(crate) struct Ema {
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
    alloc: AllocType,
    // intrusive linkedlist
    link: LinkedListLink,
}

// Implement ema adapter for the operations of intrusive linkedlist
intrusive_adapter!(pub(crate) EmaAda = UnsafeRef<Ema>: Ema { link: LinkedListLink });

#[derive(Clone, Copy)]
/// Options for allocating Emas.
pub struct EmaOptions {
    pub addr: Option<usize>,
    pub length: usize,
    pub alloc_flags: AllocFlags,
    pub alloc: AllocType,
    info: PageInfo,
    handler: Option<PfHandler>,
    priv_data: Option<*mut c_void>,
}

// TODO: remove send and sync
unsafe impl Send for Ema {}
unsafe impl Sync for Ema {}

impl Ema {
    /// Initialize Emanode with null eaccept map,
    /// and start address must be page aligned
    pub fn new(options: &EmaOptions) -> OsResult<Self> {
        ensure!(options.addr.is_some(), EINVAL);

        Ok(Self {
            start: options.addr.unwrap(),
            length: options.length,
            alloc_flags: options.alloc_flags,
            info: options.info,
            eaccept_map: None,
            handler: options.handler,
            priv_data: options.priv_data,
            link: LinkedListLink::new(),
            alloc: options.alloc,
        })
    }

    /// Split current ema at specified address, return a new allocated ema
    /// corresponding to the memory at the range of [addr, end).
    /// And the current ema manages the memory at the range of [start, addr).
    pub fn split(&mut self, addr: usize) -> OsResult<UnsafeRef<Ema>> {
        let laddr = self.start;
        let lsize = addr - laddr;

        let raddr = addr;
        let rsize = (self.start + self.length) - addr;

        let rarray = match &mut self.eaccept_map {
            Some(bitarray) => {
                let pos = (addr - self.start) >> crate::arch::SE_PAGE_SHIFT;
                Some(bitarray.split(pos)?)
            }
            None => None,
        };

        let mut rema = self.clone_ema();
        rema.start = raddr;
        rema.length = rsize;
        rema.eaccept_map = rarray;

        self.start = laddr;
        self.length = lsize;

        Ok(rema)
    }

    /// Employ same allocator to Clone Ema without eaccept map
    pub(crate) fn clone_ema(&self) -> UnsafeRef<Ema> {
        let mut ema_options = EmaOptions::new(Some(self.start), self.length, self.alloc_flags);
        ema_options
            .info(self.info)
            .handle(self.handler, self.priv_data)
            .alloc(self.alloc);

        let allocator = self.alloc.alloctor();
        let ema = Box::new_in(Ema::new(&ema_options).unwrap(), allocator);
        unsafe { UnsafeRef::from_raw(Box::into_raw(ema)) }
    }

    /// Allocate the reserve / committed / virtual memory at corresponding memory
    pub fn alloc(&mut self) -> OsResult {
        // RESERVED region only occupy memory range with no real allocation
        if self.alloc_flags.contains(AllocFlags::RESERVED) {
            return Ok(());
        }

        // Allocate new eaccept_map for COMMIT_ON_DEMAND and COMMIT_NOW
        if self.eaccept_map.is_none() {
            self.init_eaccept_map()?;
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

    /// Eaccept target EPC pages with cpu eaccept instruction
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
        ensure!(
            self.info.prot.intersects(ProtFlags::RW)
                && self.info.typ == PageType::Reg
                && !self.alloc_flags.contains(AllocFlags::RESERVED),
            EACCES
        );

        Ok(())
    }

    /// Commit the corresponding memory of this ema
    pub fn commit_all(&mut self) -> OsResult {
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
            prot: ProtFlags::RW | ProtFlags::PENDING,
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
        ensure!(!self.alloc_flags.contains(AllocFlags::RESERVED), EACCES);
        Ok(())
    }

    /// Uncommit the corresponding memory of this ema
    pub fn uncommit_all(&mut self) -> OsResult {
        self.uncommit(self.start, self.length)
    }

    /// Uncommit the partial memory of this ema
    pub fn uncommit(&mut self, start: usize, length: usize) -> OsResult {
        // Question: there exists a problem:
        // If developers trim partial pages of the ema with none protection flag,
        // the protection flag of left committed pages would be modified to Read implicitly.
        let prot = self.info.prot;
        if prot == ProtFlags::NONE && (self.info.typ != PageType::Tcs) {
            self.modify_perm(ProtFlags::R)?
        }

        self.uncommit_inner(start, length, prot)
    }

    #[inline]
    fn uncommit_inner(&mut self, start: usize, length: usize, prot: ProtFlags) -> OsResult {
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
        ensure!(
            (self.info.typ == PageType::Reg && !self.alloc_flags.contains(AllocFlags::RESERVED)),
            EACCES
        );

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

        ensure!(self.length == SE_PAGE_SIZE, EINVAL);
        ensure!(self.is_page_committed(self.start), EACCES);

        let info = self.info;
        if info.typ == PageType::Tcs {
            return Ok(());
        }

        ensure!(
            (info.prot == ProtFlags::RW) && (info.typ == PageType::Reg),
            EACCES
        );

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
        self.uncommit_inner(self.start, self.length, ProtFlags::NONE)?;
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
            self.init_eaccept_map()?;
            self.eaccept_map.as_mut().unwrap().set_full();
        } else {
            self.eaccept_map.as_mut().unwrap().set_full();
        }
        Ok(())
    }

    fn init_eaccept_map(&mut self) -> OsResult {
        let page_num = self.length >> SE_PAGE_SHIFT;
        let eaccept_map = BitArray::new(page_num, self.alloc)?;
        self.eaccept_map = Some(eaccept_map);
        Ok(())
    }

    fn set_flags(&mut self, flags: AllocFlags) {
        self.alloc_flags = flags;
    }

    fn set_prot(&mut self, info: PageInfo) {
        self.info = info;
    }

    pub fn alloc_type(&self) -> AllocType {
        self.alloc
    }

    /// Obtain the allocator of ema
    pub fn allocator(&self) -> &'static dyn EmmAllocator {
        self.alloc.alloctor()
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

impl EmaOptions {
    /// Creates new options for allocating the Emas
    pub fn new(addr: Option<usize>, length: usize, alloc_flags: AllocFlags) -> Self {
        Self {
            addr,
            length,
            alloc_flags,
            info: PageInfo {
                typ: PageType::Reg,
                prot: ProtFlags::RW,
            },
            handler: None,
            priv_data: None,
            alloc: AllocType::Reserve,
        }
    }

    /// Resets the base address of allocated Emas.
    pub fn addr(&mut self, addr: usize) -> &mut Self {
        self.addr = Some(addr);
        self
    }

    /// Sets the page info of allocated Emas.
    ///
    /// The default value is `PageInfo { typ: PageType::Reg, prot: ProtFlags::RW }`.
    pub fn info(&mut self, info: PageInfo) -> &mut Self {
        self.info = info;
        self
    }

    /// Sets the customized page fault handler and private data of allocated Emas.
    ///
    /// The default value is `handler: None, priv_data: None`.
    pub fn handle(
        &mut self,
        handler: Option<PfHandler>,
        priv_data: Option<*mut c_void>,
    ) -> &mut Self {
        self.handler = handler;
        self.priv_data = priv_data;
        self
    }

    /// The method can not be exposed to User.
    /// Sets the inner allocate method of allocated Emas.
    ///
    /// If `alloc` is set as `AllocType::Reserve`, the Ema will be allocated
    /// at reserve memory region (commited pages in user region).
    /// If `alloc` is set as `AllocType::Static`, the Ema will be allocated
    /// at static memory region (a small static memory).
    ///
    /// The default value is `AllocType::Reserve`.
    pub(crate) fn alloc(&mut self, alloc: AllocType) -> &mut Self {
        self.alloc = alloc;
        self
    }
}

impl EmaOptions {
    pub(crate) fn check(options: &EmaOptions) -> OsResult {
        let addr = options.addr.unwrap_or(0);
        let size = options.length;

        if addr > 0 {
            ensure!(
                is_page_aligned!(addr) && is_within_enclave(addr as *const u8, size),
                EINVAL
            );
        }
        ensure!(size != 0 && ((size % SE_PAGE_SIZE) == 0), EINVAL);

        Ok(())
    }
}

impl Ema {
    pub fn allocate(options: &EmaOptions, apply_now: bool) -> OsResult<UnsafeRef<Ema>> {
        ensure!(options.addr.is_some(), EFAULT);
        let mut new_ema = {
            let allocator = options.alloc.alloctor();
            let new_ema = Box::new_in(Ema::new(options)?, allocator);
            unsafe { UnsafeRef::from_raw(Box::into_raw(new_ema)) }
        };
        if apply_now {
            new_ema.alloc()?;
        }
        Ok(new_ema)
    }
}
