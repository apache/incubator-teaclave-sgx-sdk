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

use core::alloc::{Allocator, Layout};
use core::mem::ManuallyDrop;
use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::ptr::Unique;
use sgx_types::error::{SgxResult, SgxStatus};

cfg_if! {
    if #[cfg(not(feature = "hyper"))] {
        pub use stack::OcAlloc;
    } else {
        pub use msbuf::OcAlloc;
    }
}

#[derive(Debug)]
pub struct OcBuffer {
    buf: Unique<[u8]>,
}

impl !Send for OcBuffer {}

impl OcBuffer {
    #[inline]
    pub fn alloc(len: NonZeroUsize) -> SgxResult<OcBuffer> {
        Self::alloc_aligned(len, NonZeroUsize::new(1).unwrap())
    }

    #[inline]
    pub fn alloc_aligned(len: NonZeroUsize, align: NonZeroUsize) -> SgxResult<OcBuffer> {
        let layout = Layout::from_size_align(len.get(), align.get())
            .map_err(|_| SgxStatus::InvalidParameter)?;
        let mut buf = OcAlloc
            .allocate(layout)
            .map_err(|_| SgxStatus::Unexpected)?;
        Ok(OcBuffer {
            buf: unsafe { Unique::from(buf.as_mut()) },
        })
    }

    #[inline]
    pub fn alloc_zeroed(len: NonZeroUsize) -> SgxResult<OcBuffer> {
        Self::alloc_aligned_zeroed(len, NonZeroUsize::new(1).unwrap())
    }

    #[inline]
    pub fn alloc_aligned_zeroed(len: NonZeroUsize, align: NonZeroUsize) -> SgxResult<OcBuffer> {
        let mut host_buf = Self::alloc_aligned(len, align)?;
        host_buf.fill(0);
        Ok(host_buf)
    }

    #[inline]
    pub fn into_raw(b: Self) -> *mut [u8] {
        ManuallyDrop::new(b).as_mut() as *mut [u8]
    }

    #[inline]
    pub unsafe fn from_raw(mut ptr: NonNull<[u8]>) -> OcBuffer {
        OcBuffer {
            buf: Unique::from(ptr.as_mut()),
        }
    }

    #[inline]
    pub(crate) unsafe fn free() -> SgxResult {
        OcAlloc.oc_free()
    }

    #[inline]
    pub fn remain_size() -> usize {
        OcAlloc.oc_remain_size()
    }
}

impl Drop for OcBuffer {
    #[inline]
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.len(), 1).unwrap();
        unsafe { OcAlloc.deallocate(NonNull::new_unchecked(self.as_mut_ptr()), layout) }
    }
}

impl AsRef<[u8]> for OcBuffer {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl AsMut<[u8]> for OcBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        self
    }
}

impl Deref for OcBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.buf.as_ref() }
    }
}

impl DerefMut for OcBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.buf.as_mut() }
    }
}

#[cfg(not(feature = "hyper"))]
mod stack {
    use crate::arch;
    use crate::enclave;
    use crate::tcs;
    use core::alloc::{AllocError, Allocator, Layout};
    use core::cmp;
    use core::mem;
    use core::ptr::NonNull;
    use sgx_types::error::{SgxResult, SgxStatus};

    pub struct OcAlloc;

    unsafe impl Allocator for OcAlloc {
        fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            self.ocalloc(layout)
                .map(|addr| NonNull::slice_from_raw_parts(addr, layout.size()))
                .map_err(|_| AllocError)
        }

        #[inline]
        unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
            if layout.size() != 0 && enclave::is_within_host(ptr.as_ptr(), layout.size()) {
                let _ = self.oc_free();
            }
        }
    }

    impl OcAlloc {
        const OC_MIN_ALIGN: usize = 0x10;
        const OC_MAX_ALIGN: usize = 0x1000;

        fn ocalloc(&self, layout: Layout) -> SgxResult<NonNull<u8>> {
            ensure!(layout.size() != 0, SgxStatus::Unexpected);

            let layout = layout.align_to(OcAlloc::OC_MIN_ALIGN).unwrap();
            let align = cmp::min(layout.align(), OcAlloc::OC_MAX_ALIGN);
            let size = layout.size();

            let mut tc = tcs::current();
            let tds = tc.tds_mut();

            let ssa_gpr = tds.ssa_gpr_mut();
            let mut addr = ssa_gpr.rsp_u as usize;

            // check u_rsp points to the untrusted address.
            // if the check fails, it should be hacked. call abort directly
            ensure!(
                enclave::is_within_host(addr as *const u8, mem::size_of::<usize>()),
                SgxStatus::Unexpected
            );

            // size is too large to allocate. call abort() directly.
            if addr < size {
                bail!(SgxStatus::Unexpected);
            }

            // calculate the start address for the allocated memory
            addr -= size;
            addr &= !(align - 1);

            ensure!(
                enclave::is_within_host(addr as *const u8, size),
                SgxStatus::Unexpected
            );

            // probe the outside stack to ensure that we do not skip over the stack3 guard page
            // we need to probe all the pages including the first page and the last page
            // the first page need to be probed in case uRTS didnot touch that page before EENTER enclave
            // the last page need to be probed in case the enclave didnot touch that page before another OCALLOC
            let first_page = trim_to_page!(ssa_gpr.rsp_u as usize - 1);
            let last_page = trim_to_page!(addr);

            // To avoid the dead-loop in the following for(...) loop.
            // Attacker might fake a stack address that is within address 0x4095.
            if last_page == 0 {
                bail!(SgxStatus::Unexpected);
            }

            // the compiler may optimize the following code to probe the pages in any order
            // while we only expect the probe order should be from higher addr to lower addr
            // so use volatile to avoid optimization by the compiler
            let mut page = first_page;
            while page >= last_page {
                // OS may refuse to commit a physical page if the page fault address is smaller than RSP
                // So update the outside stack address before probe the page
                ssa_gpr.rsp_u = page as u64;
                unsafe {
                    *(page as *mut u8) = 0;
                }
                page -= arch::SE_PAGE_SIZE;
            }

            // update the outside stack address in the SSA to the allocated address
            ssa_gpr.rsp_u = addr as u64;

            NonNull::new(addr as *mut u8).ok_or(SgxStatus::Unexpected)
        }

        // ECALL stack frame
        //   last_sp -> |             |
        //               -------------
        //              | ret_addr    |
        //              | xbp_u       |
        //              | xsp_u       |
        pub(super) unsafe fn oc_free(&self) -> SgxResult {
            let mut tc = tcs::current();
            let tds = tc.tds_mut();
            let last_sp = tds.last_sp;

            let ssa_gpr = tds.ssa_gpr_mut();
            let addr = last_sp - 3 * mem::size_of::<usize>();
            let usp = *(addr as *const usize);

            if enclave::is_within_host(usp as *const u8, mem::size_of::<usize>()) {
                ssa_gpr.rsp_u = usp as u64;
                Ok(())
            } else {
                Err(SgxStatus::Unexpected)
            }
        }

        #[inline]
        pub fn oc_remain_size(&self) -> usize {
            const MAX_OC_REMAIN_SIZE: usize = 0x4000; //16K
            MAX_OC_REMAIN_SIZE
        }
    }
}

#[cfg(feature = "hyper")]
mod msbuf {
    use crate::call::MsbufInfo;
    use crate::enclave;
    use crate::tcs;
    use core::alloc::{AllocError, Allocator, Layout};
    use core::cmp;
    use core::ptr::NonNull;
    use sgx_types::error::{SgxResult, SgxStatus};

    pub struct OcAlloc;

    unsafe impl Allocator for OcAlloc {
        fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            self.ocalloc(layout)
                .map(|addr| NonNull::slice_from_raw_parts(addr, layout.size()))
                .map_err(|_| AllocError)
        }

        #[inline]
        unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
            if layout.size() != 0 && enclave::is_within_host(ptr.as_ptr(), layout.size()) {
                let _ = self.oc_free();
            }
        }
    }

    impl OcAlloc {
        const OC_MIN_ALIGN: usize = 0x10;
        const OC_MAX_ALIGN: usize = 0x1000;

        fn ocalloc(&self, layout: Layout) -> SgxResult<NonNull<u8>> {
            ensure!(layout.size() != 0, SgxStatus::Unexpected);

            let layout = layout.align_to(OcAlloc::OC_MIN_ALIGN).unwrap();
            let align = cmp::min(layout.align(), OcAlloc::OC_MAX_ALIGN);
            let layout =
                Layout::from_size_align(layout.size(), align).map_err(|_| SgxStatus::Unexpected)?;

            let tc = tcs::current();
            let tds = tc.tds();

            let msbuf_info = MsbufInfo::get();
            let addr = msbuf_info.alloc(tds.index, layout)?;

            ensure!(
                enclave::is_within_host(addr.as_ptr() as *const u8, layout.size()),
                SgxStatus::Unexpected
            );

            Ok(addr)
        }

        pub(super) unsafe fn oc_free(&self) -> SgxResult {
            let tc = tcs::current();
            let tds = tc.tds();

            let msbuf_info = MsbufInfo::get();
            msbuf_info.free(tds.index)
        }

        pub fn oc_remain_size(&self) -> usize {
            let tc = tcs::current();
            let tds = tc.tds();

            let msbuf_info = MsbufInfo::get();
            msbuf_info.remain_size(tds.index).unwrap_or(0)
        }
    }
}
