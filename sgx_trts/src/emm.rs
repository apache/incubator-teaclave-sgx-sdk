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

use core::mem;
use core::ptr::{self, NonNull};
use sgx_types::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocAddr {
    /// Free to choose any address
    Any,
    /// Prefer the address, but can use other address
    Hint(NonNull<u8>),
    /// Need to use the address, otherwise report error
    Need(NonNull<u8>),
}

impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct AllocFlags: u32 {
        const RESERVE           = 0x0000_0001;
        const COMMIT_NOW        = 0x0000_0002;
        const COMMIT_ON_DEMAND  = 0x0000_0004;
        const GROWSDOWN         = 0x0000_0010;
        const GROWSUP           = 0x0000_0020;
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum PageType {
        TCS = 0x0000_0100,
        REG = 0x0000_0200,
        TRIM = 0x0000_0400,
        SS_FIRST = 0x0000_0500,
        SS_REST = 0x0000_0600,
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum Align {
        A4KB = 12,
        A8KB = 13,
        A16KB = 14,
        A32KB = 15,
        A64KB = 16,
        A128KB = 17,
        A256KB = 18,
        A512KB = 19,
        A1MB = 20,
        A2MB = 21,
        A4MB = 22,
        A8MB = 23,
        A16MB = 24,
        A32MB = 25,
        A64MB = 26,
        A128MB = 27,
        A256MB = 28,
        A512MB = 29,
        A1GB = 30,
        A2GB = 31,
        A4GB = 32,
    }
}

impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Perm: u32 {
        const NONE  = 0x0000_0000;
        const READ  = 0x0000_0001;
        const WRITE  = 0x0000_0002;
        const EXEC  = 0x0000_0004;
        const DEFAULT = Self::READ.bits() | Self::WRITE.bits();
        const ALL = Self::DEFAULT.bits() | Self::EXEC.bits();
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum HandleResult {
        Search = 0,
        Execution = 0xFFFFFFFF,
    }
}

pub type PageFaultHandler = extern "C" fn(pfinfo: &sgx_pfinfo, private: usize) -> HandleResult;

pub struct AllocOptions {
    flags: AllocFlags,
    page_type: PageType,
    align: Align,
    handler: Option<PageFaultHandler>,
    private: usize,
}

impl AllocOptions {
    #[inline]
    pub const fn new() -> Self {
        AllocOptions {
            flags: AllocFlags::RESERVE,
            page_type: PageType::REG,
            align: Align::A4KB,
            handler: None,
            private: 0,
        }
    }

    #[inline]
    pub fn set_flags(mut self, flags: AllocFlags) -> Self {
        self.flags = flags;
        self
    }

    #[inline]
    pub fn set_page_types(mut self, page_type: PageType) -> Self {
        self.page_type = page_type;
        self
    }

    #[inline]
    pub fn set_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    #[inline]
    pub fn set_handler(mut self, handler: PageFaultHandler, private: usize) -> Self {
        self.handler.replace(handler);
        self.private = private;
        self
    }
}

impl Default for AllocOptions {
    #[inline]
    fn default() -> Self {
        AllocOptions::new()
    }
}

pub struct EmmAlloc;

impl EmmAlloc {
    /// Allocate a new memory region in enclave address space (ELRANGE).
    #[inline]
    pub unsafe fn alloc(
        &self,
        addr: AllocAddr,
        length: usize,
        options: AllocOptions,
    ) -> SysResult<NonNull<u8>> {
        let mut out_addr: *mut c_void = ptr::null_mut();

        let flags = options.flags.bits()
            | options.page_type as u32
            | (options.align as u32) << SGX_EMA_ALIGNMENT_SHIFT;
        let (addr, flags) = match addr {
            AllocAddr::Any => (ptr::null_mut(), flags),
            AllocAddr::Hint(addr) => (addr.as_ptr(), flags),
            AllocAddr::Need(addr) => (addr.as_ptr(), flags | SGX_EMA_FIXED),
        };

        let ret = sgx_mm_alloc(
            addr as *const _,
            length,
            flags as i32,
            mem::transmute(options.handler),
            options.private as *mut _,
            &mut out_addr as *mut *mut _,
        );
        if ret == 0 {
            Ok(NonNull::new_unchecked(out_addr as *mut _))
        } else {
            Err(ret)
        }
    }

    // Commit a partial or full range of memory allocated previously with
    // AllocFlags::COMMIT_ON_DEMAND.
    #[inline]
    pub unsafe fn commit(&self, addr: NonNull<u8>, length: usize) -> SysError {
        let ret = sgx_mm_commit(addr.as_ptr() as *const _, length);
        if ret == 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }

    /// Load data into target pages within a region previously allocated by
    /// alloc. This can be called to load data and set target permissions
    /// at the same time, e.g., dynamic code loading. The caller has verified
    /// data to be trusted and expected to be loaded to the target address range.
    // Calling this API on pages already committed will fail.
    #[inline]
    pub unsafe fn commit_with_data(addr: NonNull<u8>, data: &[u8], perm: Perm) -> SysError {
        let ret = sgx_mm_commit_data(
            addr.as_ptr() as *const _,
            data.len(),
            data.as_ptr() as *const _,
            perm.bits() as _,
        );
        if ret == 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }

    /// Uncommit (trim) physical EPC pages in a previously committed range.
    /// The pages in the allocation are freed, but the address range is still
    /// reserved.
    #[inline]
    pub unsafe fn uncommit(&self, addr: NonNull<u8>, length: usize) -> SysError {
        let ret = sgx_mm_uncommit(addr.as_ptr() as *const _, length);
        if ret == 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }

    /// Deallocate the address range.
    /// The pages in the allocation are freed and the address range is released
    /// for future allocation.
    #[inline]
    pub unsafe fn dealloc(&self, addr: NonNull<u8>, length: usize) -> SysError {
        let ret = sgx_mm_dealloc(addr.as_ptr() as *const _, length);
        if ret == 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }

    /// Change permissions of an allocated region.
    #[inline]
    pub unsafe fn modify_permissions(
        &self,
        addr: NonNull<u8>,
        length: usize,
        perm: Perm,
    ) -> SysError {
        let ret = sgx_mm_modify_permissions(addr.as_ptr() as *const _, length, perm.bits() as _);
        if ret == 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }

    /// Change the page type of an allocated region.
    #[inline]
    pub unsafe fn modify_type(
        &self,
        addr: NonNull<u8>,
        length: usize,
        page_type: PageType,
    ) -> SysError {
        let ret = sgx_mm_modify_type(addr.as_ptr() as *const _, length, page_type as _);
        if ret == 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }
}
