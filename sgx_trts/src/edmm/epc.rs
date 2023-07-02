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

use crate::arch::{SecInfo, SE_PAGE_SHIFT, SE_PAGE_SIZE};
use crate::enclave::is_within_enclave;
use crate::inst::EncluInst;
use core::num::NonZeroUsize;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::ContiguousMemory;

impl_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum PageType {
        // Secs = 0,
        None = 0,
        Tcs  = 1,
        Reg  = 2,
        // Va   = 3,
        Trim = 4,
        Frist = 5,
        Rest = 6,
    }
}

// ProtFlags may have richer meaning compared to ProtFlags
// ProtFlags and AllocFlags are confused to developer
// PageInfo->flags should change to PageInfo->prot
impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ProtFlags: u8 {
        const NONE     = 0x00;
        const R        = 0x01;
        const W        = 0x02;
        const X        = 0x04;
        const PENDING  = 0x08;
        const MODIFIED = 0x10;
        const PR       = 0x20;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PageInfo {
    pub typ: PageType,
    pub prot: ProtFlags,
}

impl Into<u32> for PageInfo {
    fn into(self) -> u32 {
        (Into::<u8>::into(self.typ) as u32) << 8 | (self.prot.bits() as u32)
    }
}

unsafe impl ContiguousMemory for PageInfo {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRange {
    addr: NonZeroUsize,
    count: usize,
    info: PageInfo,
}

unsafe impl ContiguousMemory for PageRange {}

impl PageRange {
    pub fn new(addr: usize, count: usize, info: PageInfo) -> SgxResult<PageRange> {
        if addr != 0
            && count != 0
            && is_within_enclave(addr as *const u8, count * SE_PAGE_SIZE)
            && is_page_aligned!(addr)
        {
            Ok(PageRange {
                addr: unsafe { NonZeroUsize::new_unchecked(addr) },
                count,
                info,
            })
        } else {
            Err(SgxStatus::InvalidParameter)
        }
    }

    pub fn accept_forward(&self) -> SgxResult {
        for page in self.iter() {
            page.accept()?;
        }
        Ok(())
    }

    pub fn accept_backward(&self) -> SgxResult {
        for page in self.iter().rev() {
            page.accept()?;
        }
        Ok(())
    }

    pub fn modpe(&self) -> SgxResult {
        for page in self.iter() {
            page.modpe()?;
        }
        Ok(())
    }

    pub(crate) fn modify(&self) -> SgxResult {
        for page in self.iter() {
            let _ = page.modpe();
            if !page.info.prot.contains(ProtFlags::W | ProtFlags::X) {
                page.accept()?;
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> Iter {
        Iter {
            head: self.addr.get(),
            tail: self.addr.get() + ((self.count - 1) << SE_PAGE_SHIFT),
            count: self.count,
            info: self.info,
        }
    }
}

impl IntoIterator for &PageRange {
    type Item = Page;
    type IntoIter = Iter;

    fn into_iter(self) -> Iter {
        self.iter()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Iter {
    head: usize,
    tail: usize,
    count: usize,
    info: PageInfo,
}

impl Iterator for Iter {
    type Item = Page;

    #[inline]
    fn next(&mut self) -> Option<Page> {
        if self.count == 0 {
            None
        } else {
            let cur = self.head;
            self.head += SE_PAGE_SIZE;
            self.count -= 1;
            Some(unsafe { Page::new_unchecked(cur, self.info) })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }

    #[inline]
    fn last(mut self) -> Option<Page> {
        self.next_back()
    }
}

impl DoubleEndedIterator for Iter {
    #[inline]
    fn next_back(&mut self) -> Option<Page> {
        if self.count == 0 {
            None
        } else {
            let cur = self.tail;
            self.tail -= SE_PAGE_SIZE;
            self.count -= 1;
            Some(unsafe { Page::new_unchecked(cur, self.info) })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Page {
    addr: usize,
    info: PageInfo,
}

unsafe impl ContiguousMemory for Page {}

impl Page {
    pub fn new(addr: usize, info: PageInfo) -> SgxResult<Page> {
        ensure!(
            addr != 0
                && is_within_enclave(addr as *const u8, SE_PAGE_SIZE)
                && is_page_aligned!(addr),
            SgxStatus::InvalidParameter
        );
        Ok(Page { addr, info })
    }

    pub unsafe fn new_unchecked(addr: usize, info: PageInfo) -> Page {
        Page { addr, info }
    }

    pub fn accept(&self) -> SgxResult {
        let secinfo: SecInfo = self.info.into();
        EncluInst::eaccept(&secinfo, self.addr).map_err(|_| SgxStatus::Unexpected)
    }

    pub fn modpe(&self) -> SgxResult {
        let secinfo: SecInfo = self.info.into();
        EncluInst::emodpe(&secinfo, self.addr).map_err(|_| SgxStatus::Unexpected)
    }
}
