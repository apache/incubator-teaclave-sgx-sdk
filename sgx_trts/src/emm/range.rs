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

use crate::{
    arch::SE_PAGE_SIZE,
    edmm::{PageInfo, PageType, ProtFlags},
    enclave::is_within_enclave,
};
use alloc::boxed::Box;
use intrusive_collections::{linked_list::CursorMut, LinkedList, UnsafeRef};
use sgx_types::error::{SgxResult, SgxStatus};
use spin::{Mutex, Once};

use super::{
    alloc::{ResAlloc, StaticAlloc},
    ema::{EmaAda, EMA},
    flags::AllocFlags,
    interior::Alloc,
    pfhandler::{PfHandler, PfInfo},
    user::{is_within_rts_range, is_within_user_range, USER_RANGE},
};

const ALLOC_FLAGS_SHIFT: usize = 0;
const ALLOC_FLAGS_MASK: usize = 0xFF << ALLOC_FLAGS_SHIFT;

const PAGE_TYPE_SHIFT: usize = 8;
const PAGE_TYPE_MASK: usize = 0xFF << PAGE_TYPE_SHIFT;

const ALLIGNMENT_SHIFT: usize = 24;
const ALLIGNMENT_MASK: usize = 0xFF << ALLIGNMENT_SHIFT;

pub static RM: Once<Mutex<RangeManage>> = Once::new();

/// Initialize range management
pub fn init_range_manage() {
    RM.call_once(|| Mutex::new(RangeManage::new()));
}

/// RangeManage manages virtual memory range
pub struct RangeManage {
    user: LinkedList<EmaAda>,
    rts: LinkedList<EmaAda>,
}

/// RangeType specifies using Rts or User range
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeType {
    Rts,
    User,
}

impl RangeManage {
    pub fn new() -> Self {
        Self {
            user: LinkedList::new(EmaAda::new()),
            rts: LinkedList::new(EmaAda::new()),
        }
    }

    /// Allocate a new memory region in enclave address space (ELRANGE).
    pub fn alloc(
        &mut self,
        addr: Option<usize>,
        size: usize,
        flags: usize,
        handler: Option<PfHandler>,
        priv_data: Option<*mut PfInfo>,
        typ: RangeType,
        alloc: Alloc,
    ) -> SgxResult<usize> {
        let addr = addr.unwrap_or(0);
        let alloc_flags =
            AllocFlags::try_from(((flags & ALLOC_FLAGS_MASK) >> ALLOC_FLAGS_SHIFT) as u32)?;
        let mut page_type =
            match PageType::try_from(((flags & PAGE_TYPE_MASK) >> PAGE_TYPE_SHIFT) as u8) {
                Ok(typ) => typ,
                Err(_) => return Err(SgxStatus::InvalidParameter),
            };

        if page_type == PageType::None {
            page_type = PageType::Reg;
        }

        if (size % SE_PAGE_SIZE) > 0 {
            return Err(SgxStatus::InvalidParameter);
        }

        let mut align_flag: u8 = ((flags & ALLIGNMENT_MASK) >> ALLIGNMENT_SHIFT) as u8;
        if align_flag == 0 {
            align_flag = 12;
        }
        if align_flag < 12 {
            return Err(SgxStatus::InvalidParameter);
        }
        let align_mask: usize = (1 << align_flag) - 1;

        if (addr & align_mask) > 0 {
            return Err(SgxStatus::InvalidParameter);
        }

        if (addr > 0) && !is_within_enclave(addr as *const u8, size) {
            return Err(SgxStatus::InvalidParameter);
        }

        let info = if alloc_flags.contains(AllocFlags::RESERVED) {
            PageInfo {
                prot: ProtFlags::NONE,
                typ: PageType::None,
            }
        } else {
            PageInfo {
                prot: ProtFlags::R | ProtFlags::W,
                typ: page_type,
            }
        };

        if addr > 0 {
            let is_fixed_alloc = alloc_flags.contains(AllocFlags::FIXED);
            // FIXME: search_ema_range implicitly contains splitting ema!
            let range = self.search_ema_range(addr, addr + size, typ, false);

            match range {
                // exist in emas list
                Ok(_) => {
                    // TODO: realloc EMA from reserve
                    if is_fixed_alloc {
                        return Err(SgxStatus::InvalidParameter);
                    }
                }
                // not exist in emas list
                Err(_) => {
                    let next_ema = self.find_free_region_at(addr, size, typ);
                    if next_ema.is_err() && is_fixed_alloc {
                        return Err(SgxStatus::InvalidParameter);
                    }
                }
            };
        };

        let (free_addr, mut next_ema) = self.find_free_region(size, 1 << align_flag, typ)?;

        let new_ema_ref = match alloc {
            Alloc::Reserve => {
                let mut new_ema = Box::<EMA, ResAlloc>::new_in(
                    EMA::new(
                        free_addr,
                        size,
                        alloc_flags,
                        info,
                        handler,
                        priv_data,
                        Alloc::Static,
                    )?,
                    ResAlloc,
                );
                new_ema.alloc()?;

                unsafe { UnsafeRef::from_raw(Box::into_raw(new_ema)) }
            }
            Alloc::Static => {
                let mut new_ema = Box::<EMA, StaticAlloc>::new_in(
                    EMA::new(
                        free_addr,
                        size,
                        alloc_flags,
                        info,
                        handler,
                        priv_data,
                        Alloc::Static,
                    )?,
                    StaticAlloc,
                );
                new_ema.alloc()?;

                unsafe { UnsafeRef::from_raw(Box::into_raw(new_ema)) }
            }
        };

        next_ema.insert_before(new_ema_ref);
        Ok(free_addr)
    }

    /// Commit a partial or full range of memory allocated previously with
    /// COMMIT_ON_DEMAND.
    pub fn commit(&mut self, addr: usize, size: usize, typ: RangeType) -> SgxResult {
        let (mut cursor, ema_num) = self.search_ema_range(addr, addr + size, typ, true)?;
        let start_ema_ptr = cursor.get().unwrap() as *const EMA;

        let mut count = ema_num;
        while count != 0 {
            cursor.get().unwrap().commit_check()?;
            cursor.move_next();
            count -= 1;
        }

        let mut cursor = match typ {
            RangeType::Rts => unsafe { self.rts.cursor_mut_from_ptr(start_ema_ptr) },
            RangeType::User => unsafe { self.user.cursor_mut_from_ptr(start_ema_ptr) },
        };

        count = ema_num;
        while count != 0 {
            unsafe { cursor.get_mut().unwrap().commit_self()? };
            cursor.move_next();
            count -= 1;
        }

        Ok(())
    }

    /// Deallocate the address range.
    pub fn dealloc(&mut self, addr: usize, size: usize, typ: RangeType) -> SgxResult {
        let (mut cursor, mut ema_num) = self.search_ema_range(addr, addr + size, typ, false)?;
        while ema_num != 0 {
            // Calling remove() implicitly moves cursor pointing to next ema
            let mut ema = cursor.remove().unwrap();
            ema.dealloc()?;

            // Drop inner EMA
            match ema.allocator() {
                Alloc::Reserve => {
                    let _ema_box = unsafe { Box::from_raw_in(UnsafeRef::into_raw(ema), ResAlloc) };
                }
                Alloc::Static => {
                    let _ema_box =
                        unsafe { Box::from_raw_in(UnsafeRef::into_raw(ema), StaticAlloc) };
                }
            }
            ema_num -= 1;
        }
        Ok(())
    }

    /// Change the page type of an allocated region.
    pub fn modify_type(
        &mut self,
        addr: usize,
        size: usize,
        new_page_typ: PageType,
        range_typ: RangeType,
    ) -> SgxResult {
        if new_page_typ != PageType::Tcs {
            return Err(SgxStatus::InvalidParameter);
        }

        if size != SE_PAGE_SIZE {
            return Err(SgxStatus::InvalidParameter);
        }

        let (mut cursor, ema_num) = self.search_ema_range(addr, addr + size, range_typ, true)?;
        assert!(ema_num == 1);
        unsafe { cursor.get_mut().unwrap().change_to_tcs()? };

        Ok(())
    }

    /// Change permissions of an allocated region.
    pub fn modify_perms(
        &mut self,
        addr: usize,
        size: usize,
        prot: ProtFlags,
        typ: RangeType,
    ) -> SgxResult {
        ensure!(
            (size != 0)
                && (size % SE_PAGE_SIZE == 0)
                && (addr % SE_PAGE_SIZE == 0)
                && (prot.contains(ProtFlags::X) && !prot.contains(ProtFlags::R)),
            SgxStatus::InvalidParameter
        );
        let (mut cursor, ema_num) = self.search_ema_range(addr, addr + size, typ, true)?;
        let start_ema_ptr = cursor.get().unwrap() as *const EMA;

        let mut count = ema_num;
        while count != 0 {
            cursor.get().unwrap().modify_perm_check()?;
            cursor.move_next();
            count -= 1;
        }

        let mut cursor = match typ {
            RangeType::Rts => unsafe { self.rts.cursor_mut_from_ptr(start_ema_ptr) },
            RangeType::User => unsafe { self.user.cursor_mut_from_ptr(start_ema_ptr) },
        };

        count = ema_num;
        while count != 0 {
            unsafe { cursor.get_mut().unwrap().modify_perm(prot)? };
            cursor.move_next();
            count -= 1;
        }

        Ok(())
    }

    /// Uncommit (trim) physical EPC pages in a previously committed range.
    pub fn uncommit(&mut self, addr: usize, size: usize, typ: RangeType) -> SgxResult {
        let (mut cursor, ema_num) = self.search_ema_range(addr, addr + size, typ, true)?;
        let start_ema_ptr = cursor.get().unwrap() as *const EMA;

        let mut count = ema_num;
        while count != 0 {
            cursor.get().unwrap().uncommit_check()?;
            cursor.move_next();
            count -= 1;
        }

        let mut cursor = match typ {
            RangeType::Rts => unsafe { self.rts.cursor_mut_from_ptr(start_ema_ptr) },
            RangeType::User => unsafe { self.user.cursor_mut_from_ptr(start_ema_ptr) },
        };

        count = ema_num;
        while count != 0 {
            unsafe { cursor.get_mut().unwrap().uncommit_self()? };
            cursor.move_next();
            count -= 1;
        }

        Ok(())
    }

    // search for a range of nodes containing addresses within [start, end)
    // 'ema_begin' will hold the fist ema that has address higher than /euqal to
    // 'start' 'ema_end' will hold the node immediately follow the last ema that has
    // address lower than / equal to 'end'
    // return ema_end and ema num
    fn search_ema_range(
        &mut self,
        start: usize,
        end: usize,
        typ: RangeType,
        continuous: bool,
    ) -> SgxResult<(CursorMut<'_, EmaAda>, usize)> {
        let mut cursor = match typ {
            RangeType::Rts => self.rts.front(),
            RangeType::User => self.user.front(),
        };

        while !cursor.is_null() && cursor.get().unwrap().lower_than_addr(start) {
            cursor.move_next();
        }

        if cursor.is_null() || cursor.get().unwrap().higher_than_addr(end) {
            return Err(SgxStatus::InvalidParameter);
        }

        let mut curr_ema = cursor.get().unwrap();

        let mut start_ema_ptr = curr_ema as *const EMA;
        let mut emas_num = 0;
        let mut prev_end = curr_ema.start();

        while !cursor.is_null() && !cursor.get().unwrap().higher_than_addr(end) {
            curr_ema = cursor.get().unwrap();
            // If continuity is required, there should
            // be no gaps in the specified range in the emas list.
            if continuous && prev_end != curr_ema.start() {
                return Err(SgxStatus::InvalidParameter);
            }

            emas_num += 1;
            prev_end = curr_ema.end();
            cursor.move_next();
        }

        let mut end_ema_ptr = curr_ema as *const EMA;

        // Found the overlapping emas with range [start, end)
        // needs to splitting emas

        // Spliting start ema
        let mut start_cursor = match typ {
            RangeType::Rts => unsafe { self.rts.cursor_mut_from_ptr(start_ema_ptr) },
            RangeType::User => unsafe { self.user.cursor_mut_from_ptr(start_ema_ptr) },
        };

        let curr_ema = unsafe { start_cursor.get_mut().unwrap() };
        let ema_start = curr_ema.start();

        // Problem may exist, need to check!!
        if ema_start < start {
            let right_ema = curr_ema.split(start).unwrap() as *const EMA;
            let right_ema_ref = unsafe { UnsafeRef::from_raw(right_ema) };
            start_cursor.insert_after(right_ema_ref);
            start_cursor.move_next();
            start_ema_ptr = start_cursor.get().unwrap() as *const EMA;
        }

        if emas_num == 1 {
            end_ema_ptr = start_ema_ptr;
        }

        // Spliting end ema
        let mut end_cursor = match typ {
            RangeType::Rts => unsafe { self.rts.cursor_mut_from_ptr(end_ema_ptr) },
            RangeType::User => unsafe { self.user.cursor_mut_from_ptr(end_ema_ptr) },
        };

        let end_ema = unsafe { end_cursor.get_mut().unwrap() };
        let ema_end = end_ema.end();

        if ema_end > end {
            let right_ema = end_ema.split(end).unwrap();
            let right_ema_ref = unsafe { UnsafeRef::from_raw(right_ema) };
            end_cursor.insert_after(right_ema_ref);
        }

        // Recover start ema and return it as range
        let start_cursor = match typ {
            RangeType::Rts => unsafe { self.rts.cursor_mut_from_ptr(start_ema_ptr) },
            RangeType::User => unsafe { self.user.cursor_mut_from_ptr(start_ema_ptr) },
        };

        Ok((start_cursor, emas_num))
    }

    // search for a ema node whose memory range contains address
    pub fn search_ema(&mut self, addr: usize, typ: RangeType) -> SgxResult<CursorMut<'_, EmaAda>> {
        let mut cursor = match typ {
            RangeType::Rts => self.rts.front_mut(),
            RangeType::User => self.user.front_mut(),
        };

        while !cursor.is_null() {
            let ema = cursor.get().unwrap();
            if ema.overlap_addr(addr) {
                return Ok(cursor);
            }
            cursor.move_next();
        }

        Err(SgxStatus::InvalidParameter)
    }

    // Find a free space at addr with 'len' bytes in reserve region,
    // the request space mustn't intersect with existed ema node.
    // If success, return the next ema cursor.
    fn find_free_region_at(
        &mut self,
        addr: usize,
        len: usize,
        typ: RangeType,
    ) -> SgxResult<CursorMut<'_, EmaAda>> {
        if !is_within_enclave(addr as *const u8, len) {
            return Err(SgxStatus::InvalidParameter);
        }
        match typ {
            RangeType::Rts => {
                if !is_within_rts_range(addr, len) {
                    return Err(SgxStatus::InvalidParameter);
                }
            }
            RangeType::User => {
                if !is_within_user_range(addr, len) {
                    return Err(SgxStatus::InvalidParameter);
                }
            }
        }

        let mut cursor: CursorMut<'_, EmaAda> = match typ {
            RangeType::Rts => self.rts.front_mut(),
            RangeType::User => self.user.front_mut(),
        };

        while !cursor.is_null() {
            let start_curr = cursor.get().map(|ema| ema.start()).unwrap();
            let end_curr = start_curr + cursor.get().map(|ema| ema.len()).unwrap();
            if start_curr >= addr + len {
                return Ok(cursor);
            }

            if addr >= end_curr {
                cursor.move_next();
            } else {
                break;
            }
        }

        // means addr is larger than the end of the last ema node
        if cursor.is_null() {
            return Ok(cursor);
        }

        Err(SgxStatus::InvalidParameter)
    }

    // Find a free space of size at least 'size' bytes in reserve region,
    // return the start address
    fn find_free_region(
        &mut self,
        len: usize,
        align: usize,
        typ: RangeType,
    ) -> SgxResult<(usize, CursorMut<'_, EmaAda>)> {
        let user_range = USER_RANGE.get().unwrap();
        let user_base = user_range.start;
        let user_end = user_range.end;

        let mut addr;

        let mut cursor: CursorMut<'_, EmaAda> = match typ {
            RangeType::Rts => self.rts.front_mut(),
            RangeType::User => self.user.front_mut(),
        };

        // no ema in list
        if cursor.is_null() {
            match typ {
                RangeType::Rts => {
                    if user_base >= len {
                        addr = trim_to!(user_base - len, align);
                        if is_within_enclave(addr as *const u8, len) {
                            return Ok((addr, cursor));
                        }
                    } else {
                        addr = round_to!(user_end, align);
                        // no integer overflow
                        if addr + len >= addr && is_within_enclave(addr as *const u8, len) {
                            return Ok((addr, cursor));
                        }
                    }
                    return Err(SgxStatus::InvalidParameter);
                }
                RangeType::User => {
                    addr = round_to!(user_base, align);
                    if is_within_user_range(addr, len) {
                        return Ok((addr, cursor));
                    }
                    return Err(SgxStatus::InvalidParameter);
                }
            }
        }

        let mut cursor_next = cursor.peek_next();

        // ema is_null means pointing to the Null object, not means this ema is empty
        while !cursor_next.is_null() {
            let curr_end = cursor.get().map(|ema| ema.aligned_end(align)).unwrap();

            let next_start = cursor_next.get().map(|ema| ema.start()).unwrap();

            if curr_end <= next_start {
                let free_size = next_start - curr_end;
                if free_size >= len
                    && (typ == RangeType::User || is_within_rts_range(curr_end, len))
                {
                    cursor.move_next();
                    return Ok((curr_end, cursor));
                }
            }
            cursor.move_next();
            cursor_next = cursor.peek_next();
        }

        addr = cursor.get().map(|ema| ema.aligned_end(align)).unwrap();

        if is_within_enclave(addr as *const u8, len)
            && ((typ == RangeType::Rts && is_within_rts_range(addr, len))
                || (typ == RangeType::User && is_within_user_range(addr, len)))
        {
            cursor.move_next();
            return Ok((addr, cursor));
        }

        // Cursor moves to emas->front_mut.
        // Firstly cursor moves to None, then moves to linkedlist head
        cursor.move_next();
        cursor.move_next();

        // Back to the first ema to check rts region before user region
        let start_first = cursor.get().map(|ema| ema.start()).unwrap();
        if start_first < len {
            return Err(SgxStatus::InvalidParameter);
        }

        addr = trim_to!(start_first, align);

        match typ {
            RangeType::User => {
                if is_within_user_range(addr, len) {
                    return Ok((addr, cursor));
                }
            }
            RangeType::Rts => {
                if is_within_enclave(addr as *const u8, len) && is_within_rts_range(addr, len) {
                    return Ok((addr, cursor));
                }
            }
        }

        Err(SgxStatus::InvalidParameter)
    }
}
