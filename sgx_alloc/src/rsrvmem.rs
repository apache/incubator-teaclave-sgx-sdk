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

pub use self::platform::*;
use core::fmt;
use core::ptr::NonNull;

pub struct RsrvMemAlloc;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProtectAttr {
    Read,
    ReadWrite,
    ReadExec,
    ReadWriteExec,
}

impl RsrvMemAlloc {
    /// Allocate a range of EPC memory from the reserved memory area
    ///
    /// # Parameters
    ///
    /// **count**
    ///
    /// Count of pages to allocate region
    ///
    /// # Return value
    ///
    /// Starting address of the new allocated memory area on success;
    ///
    #[inline]
    pub unsafe fn alloc(&self, count: u32) -> Result<NonNull<u8>, RsrvMemAllocErr> {
        NonNull::new(platform::alloc(count)).ok_or(RsrvMemAllocErr)
    }

    /// Allocate a range of EPC memory with a fixed address from the reserved memory area
    ///
    /// # Parameters
    ///
    /// **addr**
    ///
    /// The desired starting address to allocate the reserved memory. Should be page aligned.
    ///
    /// **count**
    ///
    /// Count of pages to allocate region
    ///
    /// # Return value
    ///
    /// Starting address of the new allocated memory area on success;
    ///
    #[inline]
    pub unsafe fn alloc_with_addr(
        &self,
        addr: NonNull<u8>,
        count: u32,
    ) -> Result<NonNull<u8>, RsrvMemAllocErr> {
        NonNull::new(platform::alloc_with_addr(addr.as_ptr(), count)).ok_or(RsrvMemAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_zeroed(&self, count: u32) -> Result<NonNull<u8>, RsrvMemAllocErr> {
        NonNull::new(platform::alloc_zeroed(count)).ok_or(RsrvMemAllocErr)
    }

    /// Free a range of EPC memory from the reserved memory area
    ///
    /// # Parameters
    ///
    /// ** ptr**
    ///
    /// Starting address of region to be freed. Page aligned.
    ///
    /// **count**
    ///
    /// Count of pages to allocate region
    ///
    #[inline]
    pub unsafe fn dealloc(&self, addr: NonNull<u8>, count: u32) -> Result<(), RsrvMemAllocErr> {
        platform::dealloc(addr.as_ptr(), count)
    }

    /// Modify the access permissions of the pages in the reserved memory area.
    ///
    /// # Parameters
    ///
    /// # Parameters
    ///
    /// ** ptr**
    ///
    /// Starting address of region to be freed. Page aligned.
    ///
    /// **count**
    ///
    /// Count of pages to allocate region
    ///
    /// **port**
    ///
    /// The target memory protection.
    ///
    #[inline]
    pub unsafe fn protect(
        &self,
        addr: NonNull<u8>,
        count: u32,
        prot: ProtectAttr,
    ) -> Result<(), RsrvMemAllocErr> {
        platform::protect(addr.as_ptr(), count, prot)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RsrvMemAllocErr;

impl fmt::Display for RsrvMemAllocErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("reserves memory allocation failed")
    }
}

mod platform {
    use super::ProtectAttr;
    use super::RsrvMemAllocErr;
    use core::ffi::c_void;
    use core::ptr;

    const SGX_PROT_READ: u32 = 0x1;
    const SGX_PROT_WRITE: u32 = 0x2;
    const SGX_PROT_EXEC: u32 = 0x4;
    const SE_PAGE_SIZE: usize = 0x1000;

    type size_t = usize;
    type int32_t = i32;
    type sgx_status_t = u32;

    extern "C" {
        pub fn sgx_alloc_rsrv_mem(length: size_t) -> *mut c_void;
        pub fn sgx_alloc_rsrv_mem_ex(desired_addr: *const c_void, length: size_t) -> *mut c_void;
        pub fn sgx_free_rsrv_mem(addr: *const c_void, length: size_t) -> int32_t;
        pub fn sgx_tprotect_rsrv_mem(
            addr: *const c_void,
            length: size_t,
            prot: i32,
        ) -> sgx_status_t;
    }

    #[inline]
    pub unsafe fn alloc(count: u32) -> *mut u8 {
        sgx_alloc_rsrv_mem(count as usize * SE_PAGE_SIZE) as *mut u8
    }

    #[inline]
    pub unsafe fn alloc_with_addr(addr: *mut u8, count: u32) -> *mut u8 {
        sgx_alloc_rsrv_mem_ex(addr as *const c_void, count as usize * SE_PAGE_SIZE) as *mut u8
    }

    #[inline]
    pub unsafe fn alloc_zeroed(count: u32) -> *mut u8 {
        let raw = alloc(count);
        if !raw.is_null() {
            ptr::write_bytes(raw, 0, count as usize * SE_PAGE_SIZE);
        }
        raw
    }

    #[inline]
    pub unsafe fn dealloc(addr: *mut u8, count: u32) -> Result<(), RsrvMemAllocErr> {
        if sgx_free_rsrv_mem(addr as *const c_void, count as usize * SE_PAGE_SIZE) == 0 {
            Ok(())
        } else {
            Err(RsrvMemAllocErr)
        }
    }

    #[inline]
    pub unsafe fn protect(
        addr: *mut u8,
        count: u32,
        prot: ProtectAttr,
    ) -> Result<(), RsrvMemAllocErr> {
        let attr = match prot {
            ProtectAttr::Read => SGX_PROT_READ,
            ProtectAttr::ReadWrite => SGX_PROT_READ | SGX_PROT_WRITE,
            ProtectAttr::ReadExec => SGX_PROT_READ | SGX_PROT_EXEC,
            ProtectAttr::ReadWriteExec => SGX_PROT_READ | SGX_PROT_WRITE | SGX_PROT_EXEC,
        };
        if sgx_tprotect_rsrv_mem(
            addr as *const c_void,
            count as usize * SE_PAGE_SIZE,
            attr as i32,
        ) == 0
        {
            Ok(())
        } else {
            Err(RsrvMemAllocErr)
        }
    }
}
