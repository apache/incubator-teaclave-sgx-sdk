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

use crate::rsrvmm::manager::MmAllocAddr;
use crate::rsrvmm::RsrvMem;
use core::fmt;
use core::num::NonZeroUsize;
use core::ptr;
use core::ptr::NonNull;
use sgx_types::types::ProtectPerm;

pub struct RsrvMemAlloc;

impl RsrvMemAlloc {
    /// Allocate a range of EPC memory from the reserved memory area
    pub unsafe fn alloc(&self, size: NonZeroUsize) -> Result<NonNull<[u8]>, RsrvMemAllocErr> {
        let rsrvmem = RsrvMem::get_or_init().map_err(|_| RsrvMemAllocErr)?;
        let addr = rsrvmem
            .mmap::<()>(MmAllocAddr::Any, size.get(), None, None)
            .map_err(|_| RsrvMemAllocErr)?;

        let ptr = NonNull::new(addr as *mut u8).ok_or(RsrvMemAllocErr)?;
        Ok(NonNull::slice_from_raw_parts(ptr, size.get()))
    }

    /// Allocate a range of EPC memory with a fixed address from the reserved memory area
    pub unsafe fn alloc_with_addr(
        &self,
        addr: NonNull<u8>,
        size: NonZeroUsize,
    ) -> Result<NonNull<[u8]>, RsrvMemAllocErr> {
        let rsrvmem = RsrvMem::get_or_init().map_err(|_| RsrvMemAllocErr)?;
        let addr = rsrvmem
            .mmap::<()>(
                MmAllocAddr::Hint(addr.as_ptr() as usize),
                size.get(),
                None,
                None,
            )
            .map_err(|_| RsrvMemAllocErr)?;

        let ptr = NonNull::new(addr as *mut u8).ok_or(RsrvMemAllocErr)?;
        Ok(NonNull::slice_from_raw_parts(ptr, size.get()))
    }

    /// Allocate a range of EPC memory from the reserved memory area
    #[inline]
    pub unsafe fn alloc_zeroed(
        &self,
        size: NonZeroUsize,
    ) -> Result<NonNull<[u8]>, RsrvMemAllocErr> {
        let raw = self.alloc(size)?;
        ptr::write_bytes(raw.as_mut_ptr(), 0, size.get());
        Ok(raw)
    }

    /// Allocate a range of EPC memory with a fixed address from the reserved memory area
    #[inline]
    pub unsafe fn alloc_zeroed_with_addr(
        &self,
        addr: NonNull<u8>,
        size: NonZeroUsize,
    ) -> Result<NonNull<[u8]>, RsrvMemAllocErr> {
        let raw = self.alloc_with_addr(addr, size)?;
        ptr::write_bytes(raw.as_mut_ptr(), 0, size.get());
        Ok(raw)
    }

    /// Free a range of EPC memory from the reserved memory area
    pub unsafe fn dealloc(
        &self,
        addr: NonNull<u8>,
        size: NonZeroUsize,
    ) -> Result<(), RsrvMemAllocErr> {
        let rsrvmem = RsrvMem::get_or_init().map_err(|_| RsrvMemAllocErr)?;
        rsrvmem
            .munmap(addr.as_ptr() as usize, size.get())
            .map_err(|_| RsrvMemAllocErr)
    }

    /// Modify the access permissions of the pages in the reserved memory area.
    pub unsafe fn protect(
        &self,
        addr: NonNull<u8>,
        size: NonZeroUsize,
        perm: ProtectPerm,
    ) -> Result<(), RsrvMemAllocErr> {
        let rsrvmem = RsrvMem::get_or_init().map_err(|_| RsrvMemAllocErr)?;
        rsrvmem
            .mprotect(addr.as_ptr() as usize, size.get(), perm)
            .map_err(|_| RsrvMemAllocErr)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RsrvMemAllocErr;

impl fmt::Display for RsrvMemAllocErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("reserves memory allocation failed")
    }
}
