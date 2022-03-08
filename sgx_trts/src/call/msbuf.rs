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

use crate::arch::SE_PAGE_SIZE;
use crate::enclave::{is_within_enclave, is_within_host, EnclaveRange};
use crate::inst::GlobalHyper;
use crate::tcs;
use core::alloc::Layout;
use core::mem;
use core::ptr::NonNull;
use sgx_types::error::{SgxResult, SgxStatus};

//
//    Buffer Layout
//    ------------  <- base (ms_buf_metadata_t)
//    | base     |
//    | sp       |
//    ------------  <- old_sp_1
//    | ms_buf_1 |
//    |          |
//    ------------
//    | old_sp_1 |
//    ------------  <- old_sp_2
//    | ms_buf_2 |
//    |          |
//    ------------
//    | old_sp_2 |
//    ------------  <- sp
//    |   ...    |
//

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MsbufInfo {
    pub base: usize,
    pub num: usize,
    pub size: usize,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Header {
    base: usize,
    sp: usize,
}

const ALLOC_ALIGNMENT: usize = 0x10;

macro_rules! round_to {
    ($num:expr, $align:expr) => {
        ($num + $align - 1) & (!($align - 1))
    };
}

macro_rules! trim_to {
    ($num:expr, $align:expr) => {
        $num & (!($align - 1))
    };
}

macro_rules! is_aligned {
    ($num:expr, $align:expr) => {
        $num & ($align - 1) == 0
    };
}

impl MsbufInfo {
    #[inline]
    pub fn get<'a>() -> &'a MsbufInfo {
        GlobalHyper::get().get_msbuf_info()
    }

    pub fn alloc(&self, index: usize, layout: Layout) -> SgxResult<NonNull<u8>> {
        ensure!(layout.size() != 0, SgxStatus::Unexpected);

        let (header_ref, header, initial_sp) = self.header(index)?;
        header.is_sp_valid(self.size, initial_sp)?;

        let addr = round_to!(header.sp, layout.align());
        let new_sp = addr + round_to!(layout.size() + mem::size_of::<usize>(), ALLOC_ALIGNMENT);
        ensure!(new_sp <= header.base + self.size, SgxStatus::Unexpected);

        unsafe {
            (new_sp as *mut usize).sub(1).write(header.sp);
        }
        header_ref.sp = new_sp;

        NonNull::new(addr as *mut u8).ok_or(SgxStatus::Unexpected)
    }

    pub fn free(&self, index: usize) -> SgxResult {
        let (header_ref, header, initial_sp) = self.header(index)?;

        header.is_sp_valid(self.size, initial_sp)?;
        ensure!(header.sp > initial_sp, SgxStatus::Unexpected);

        let old_sp = unsafe { (header.sp as *mut usize).sub(1).read() };
        ensure!(
            (initial_sp..header.sp).contains(&old_sp),
            SgxStatus::Unexpected
        );
        ensure!(is_aligned!(old_sp, ALLOC_ALIGNMENT), SgxStatus::Unexpected);

        header_ref.sp = old_sp;
        Ok(())
    }

    pub fn remain_size(&self, index: usize) -> SgxResult<usize> {
        let (_, header, initial_sp) = self.header(index)?;
        header.is_sp_valid(self.size, initial_sp)?;

        let bottom = header.base + (self.size - ALLOC_ALIGNMENT);
        if bottom > header.sp {
            Ok(trim_to!(bottom - header.sp, ALLOC_ALIGNMENT))
        } else {
            Ok(0)
        }
    }

    pub fn reset(&self, index: usize) -> SgxResult {
        let (header_ref, _, initial_sp) = self.header(index)?;
        header_ref.sp = initial_sp;
        Ok(())
    }

    #[inline]
    pub fn is_valid(&self) -> SgxResult {
        ensure!(self.base != 0, SgxStatus::Unexpected);
        ensure!(
            self.num > 0 && self.num <= tcs::tcs_max_num(),
            SgxStatus::Unexpected
        );
        ensure!(is_aligned!(self.size, SE_PAGE_SIZE), SgxStatus::Unexpected);
        ensure!(
            self.size.checked_mul(self.num).is_some(),
            SgxStatus::Unexpected
        );
        ensure!(self.is_host_range(), SgxStatus::Unexpected);
        Ok(())
    }

    #[inline]
    fn header(&self, index: usize) -> SgxResult<(&mut Header, Header, usize)> {
        ensure!(index < self.num, SgxStatus::Unexpected);

        let hbase = self.base + index * self.size;
        let header_ref = unsafe { &mut *(hbase as *mut Header) };

        let header = Header {
            base: hbase,
            sp: header_ref.sp,
        };
        let initial_sp = header.initial_sp();

        Ok((header_ref, header, initial_sp))
    }
}

impl Header {
    #[inline]
    fn is_valid(&self, msbuf_info: &MsbufInfo, index: usize) -> SgxResult {
        ensure!(
            self.base == msbuf_info.base + index * msbuf_info.size,
            SgxStatus::Unexpected
        );
        let initial_sp = self.initial_sp();
        self.is_sp_valid(msbuf_info.size, initial_sp)
    }

    #[inline]
    fn is_sp_valid(&self, msbuf_size: usize, initial_sp: usize) -> SgxResult {
        ensure!(
            (initial_sp..=self.base + msbuf_size).contains(&self.sp),
            SgxStatus::Unexpected
        );
        ensure!(is_aligned!(self.sp, ALLOC_ALIGNMENT), SgxStatus::Unexpected);
        Ok(())
    }

    #[inline]
    fn initial_sp(&self) -> usize {
        round_to!(self.base + mem::size_of::<Header>(), ALLOC_ALIGNMENT)
    }
}

impl EnclaveRange for MsbufInfo {
    fn is_enclave_range(&self) -> bool {
        is_within_enclave(
            self.base as *const u8,
            self.size.checked_mul(self.num).unwrap_or(0),
        )
    }

    fn is_host_range(&self) -> bool {
        is_within_host(
            self.base as *const u8,
            self.size.checked_mul(self.num).unwrap_or(0),
        )
    }
}
