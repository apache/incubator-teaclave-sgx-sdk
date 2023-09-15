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

cfg_if! {
    if #[cfg(not(any(feature = "sim", feature = "hyper")))] {
        pub use hw::*;
    } else {
        pub use sw::*;
    }
}

#[cfg(not(any(feature = "sim", feature = "hyper")))]
mod hw {
    use crate::arch::SE_PAGE_SHIFT;
    use crate::call::{ocall, OCallIndex, OcAlloc};
    use crate::emm::page::AllocFlags;
    use crate::emm::{PageInfo, PageType};
    use alloc::boxed::Box;
    use core::convert::Into;
    use sgx_types::error::{SgxResult, SgxStatus};
    use sgx_types::types::ProtectPerm;
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct EmmAllocOcall {
        retval: i32,
        addr: usize,
        size: usize,
        page_properties: u32,
        alloc_flags: u32,
    }

    pub fn alloc_ocall(
        addr: usize,
        length: usize,
        page_type: PageType,
        alloc_flags: AllocFlags,
    ) -> SgxResult {
        let mut change = Box::try_new_in(
            EmmAllocOcall {
                retval: 0,
                addr,
                size: length,
                page_properties: Into::<u8>::into(page_type) as u32,
                alloc_flags: alloc_flags.bits(),
            },
            OcAlloc,
        )
        .map_err(|_| SgxStatus::OutOfMemory)?;

        ocall(OCallIndex::Alloc, Some(change.as_mut()))
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct EmmModifyOcall {
        retval: i32,
        addr: usize,
        size: usize,
        flags_from: u32,
        flags_to: u32,
    }

    pub fn modify_ocall(
        addr: usize,
        length: usize,
        info_from: PageInfo,
        info_to: PageInfo,
    ) -> SgxResult {
        let mut change = Box::try_new_in(
            EmmModifyOcall {
                retval: 0,
                addr,
                size: length,
                flags_from: Into::<u32>::into(info_from),
                flags_to: Into::<u32>::into(info_to),
            },
            OcAlloc,
        )
        .map_err(|_| SgxStatus::OutOfMemory)?;

        ocall(OCallIndex::Modify, Some(change.as_mut()))
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct ChangePermOcall {
        addr: usize,
        size: usize,
        perm: u64,
    }

    pub fn modpr_ocall(addr: usize, count: usize, perm: ProtectPerm) -> SgxResult {
        let mut change = Box::try_new_in(
            ChangePermOcall {
                addr,
                size: count << SE_PAGE_SHIFT,
                perm: Into::<u8>::into(perm) as u64,
            },
            OcAlloc,
        )
        .map_err(|_| SgxStatus::OutOfMemory)?;

        ocall(OCallIndex::Modpr, Some(change.as_mut()))
    }

    pub fn mprotect_ocall(addr: usize, count: usize, perm: ProtectPerm) -> SgxResult {
        let mut change = Box::try_new_in(
            ChangePermOcall {
                addr,
                size: count << SE_PAGE_SHIFT,
                perm: Into::<u8>::into(perm) as u64,
            },
            OcAlloc,
        )
        .map_err(|_| SgxStatus::OutOfMemory)?;

        ocall(OCallIndex::Mprotect, Some(change.as_mut()))
    }
}

#[cfg(any(feature = "sim", feature = "hyper"))]
mod sw {
    use sgx_types::error::SgxResult;
    use sgx_types::types::ProtectPerm;

    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    pub fn alloc_ocall(
        _addr: usize,
        _length: usize,
        _page_type: PageType,
        _alloc_flags: AllocFlags,
    ) -> SgxResult {
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    pub fn modify_ocall(
        _addr: usize,
        _length: usize,
        _info_from: PageInfo,
        _info_to: PageInfo,
    ) -> SgxResult {
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    pub fn modpr_ocall(_addr: usize, _count: usize, _perm: ProtectPerm) -> SgxResult {
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    pub fn mprotect_ocall(_addr: usize, _count: usize, _perm: ProtectPerm) -> SgxResult {
        Ok(())
    }
}
