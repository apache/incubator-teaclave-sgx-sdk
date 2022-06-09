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

use super::*;
use core::alloc::{AllocError, Allocator, Layout};
use core::mem;
use core::mem::ManuallyDrop;
use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::ptr::NonNull;
use core::ptr::Unique;
use sgx_trts::trts::is_within_host;

#[derive(Debug)]
pub struct HeapBuffer {
    buf: Unique<[u8]>,
}

impl HeapBuffer {
    #[inline]
    pub fn alloc(len: NonZeroUsize) -> OCallResult<HeapBuffer> {
        Self::alloc_aligned(len, NonZeroUsize::new(1).unwrap())
    }

    #[inline]
    pub fn alloc_aligned(len: NonZeroUsize, align: NonZeroUsize) -> OCallResult<HeapBuffer> {
        let layout = Layout::from_size_align(len.get(), align.get()).map_err(|_| eos!(EINVAL))?;
        let mut buf = HostAlloc.allocate(layout).map_err(|_| eos!(ENOMEM))?;
        Ok(HeapBuffer {
            buf: unsafe { Unique::from(buf.as_mut()) },
        })
    }

    #[inline]
    pub fn alloc_zeroed(len: NonZeroUsize) -> OCallResult<HeapBuffer> {
        Self::alloc_aligned_zeroed(len, NonZeroUsize::new(1).unwrap())
    }

    #[inline]
    pub fn alloc_aligned_zeroed(len: NonZeroUsize, align: NonZeroUsize) -> OCallResult<HeapBuffer> {
        let layout = Layout::from_size_align(len.get(), align.get()).map_err(|_| eos!(EINVAL))?;
        let mut buf = HostAlloc
            .allocate_zeroed(layout)
            .map_err(|_| eos!(ENOMEM))?;
        Ok(HeapBuffer {
            buf: unsafe { Unique::from(buf.as_mut()) },
        })
    }

    #[inline]
    pub fn into_raw(b: Self) -> *mut [u8] {
        ManuallyDrop::new(b).as_mut() as *mut [u8]
    }

    #[inline]
    pub unsafe fn from_raw(mut ptr: NonNull<[u8]>) -> HeapBuffer {
        HeapBuffer {
            buf: Unique::from(ptr.as_mut()),
        }
    }
}

impl Drop for HeapBuffer {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.len(), 1).unwrap();
        unsafe { HostAlloc.deallocate(NonNull::new_unchecked(self.as_mut_ptr()), layout) }
    }
}

impl AsRef<[u8]> for HeapBuffer {
    fn as_ref(&self) -> &[u8] {
        &**self
    }
}

impl AsMut<[u8]> for HeapBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut **self
    }
}

impl Deref for HeapBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.buf.as_ref() }
    }
}

impl DerefMut for HeapBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.buf.as_mut() }
    }
}

pub struct HostAlloc;

unsafe impl Allocator for HostAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            self.host_malloc(layout, false)
                .map(|addr| NonNull::slice_from_raw_parts(addr, layout.size()))
                .map_err(|_| AllocError)
        }
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            self.host_malloc(layout, true)
                .map(|addr| NonNull::slice_from_raw_parts(addr, layout.size()))
                .map_err(|_| AllocError)
        }
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 && is_within_host(ptr.as_ptr(), layout.size()) {
            let _ = self.host_free(ptr);
        }
    }
}

impl HostAlloc {
    pub unsafe fn host_malloc(&self, layout: Layout, zeroed: bool) -> OCallResult<NonNull<u8>> {
        ensure!(layout.size() > 0, eos!(EINVAL));
        let layout = layout.align_to(mem::size_of::<usize>()).unwrap();

        let mut result: *mut c_void = ptr::null_mut();
        let mut error: c_int = 0;

        let mode = enclave_mode();
        let size = layout.size();
        let status = if mode == EnclaveMode::Hyper {
            u_malloc_ocall(
                &mut result as *mut *mut c_void,
                &mut error as *mut c_int,
                size,
                layout.align(),
                if zeroed { 1 } else { 0 },
            )
        } else {
            u_malloc_ocall(
                &mut result as *mut *mut c_void,
                &mut error as *mut c_int,
                size,
                layout.align(),
                0,
            )
        };

        ensure!(status.is_success(), esgx!(status));
        ensure!(error == 0, eos!(error));
        ensure!(!result.is_null(), ecust!("Out of memory"));
        ensure!(
            is_within_host(result as *mut u8, size),
            ecust!("Malformed malloc address")
        );

        if mode != EnclaveMode::Hyper && zeroed {
            result.write_bytes(0_u8, size);
        }
        Ok(NonNull::new_unchecked(result as *mut u8))
    }

    pub unsafe fn host_free(&self, p: NonNull<u8>) {
        if is_within_host(p.as_ptr(), mem::size_of::<usize>()) {
            u_free_ocall(p.as_ptr() as _);
        }
    }
}
