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

//! # liballoc crate for Rust SGX SDK
//!
//! This crate equals to the `liballoc_system` crate in Rust.
//! It connects Rust memory allocation to Intel SGX's sgx_tstd library.
//! It is essential, because we depends on Intel SGX's SDK.
//! 2018-06-22 Add liballoc components here

use core::alloc::{
    AllocErr, AllocInit, AllocRef, GlobalAlloc, Layout, MemoryBlock, ReallocPlacement,
};
use core::intrinsics;
use core::ptr::NonNull;

// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values. In practice, the alignment is a
// constant at the call site and the branch will be optimized out.
#[cfg(target_arch = "x86")]
const MIN_ALIGN: usize = 8;

// The alignment of sgx tlibc is 16
// https://github.com/intel/linux-sgx/blob/master/sdk/tlibc/stdlib/malloc.c#L541
#[cfg(target_arch = "x86_64")]
const MIN_ALIGN: usize = 16;

pub struct System;

unsafe impl AllocRef for System {
    #[inline]
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        unsafe {
            let size = layout.size();
            if size == 0 {
                Ok(MemoryBlock {
                    ptr: layout.dangling(),
                    size: 0,
                })
            } else {
                let raw_ptr = match init {
                    AllocInit::Uninitialized => GlobalAlloc::alloc(self, layout),
                    AllocInit::Zeroed => GlobalAlloc::alloc_zeroed(self, layout),
                };
                let ptr = NonNull::new(raw_ptr).ok_or(AllocErr)?;
                Ok(MemoryBlock { ptr, size })
            }
        }
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 {
            GlobalAlloc::dealloc(self, ptr.as_ptr(), layout)
        }
    }

    #[inline]
    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<MemoryBlock, AllocErr> {
        let size = layout.size();
        debug_assert!(
            new_size >= size,
            "`new_size` must be greater than or equal to `memory.size()`"
        );

        if size == new_size {
            return Ok(MemoryBlock { ptr, size });
        }

        match placement {
            ReallocPlacement::InPlace => Err(AllocErr),
            ReallocPlacement::MayMove if layout.size() == 0 => {
                let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
                self.alloc(new_layout, init)
            }
            ReallocPlacement::MayMove => {
                // `realloc` probably checks for `new_size > size` or something similar.
                intrinsics::assume(new_size > size);
                let ptr = GlobalAlloc::realloc(self, ptr.as_ptr(), layout, new_size);
                let memory = MemoryBlock {
                    ptr: NonNull::new(ptr).ok_or(AllocErr)?,
                    size: new_size,
                };
                init.init_offset(memory, size);
                Ok(memory)
            }
        }
    }

    #[inline]
    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        let size = layout.size();
        debug_assert!(
            new_size <= size,
            "`new_size` must be smaller than or equal to `memory.size()`"
        );

        if size == new_size {
            return Ok(MemoryBlock { ptr, size });
        }

        match placement {
            ReallocPlacement::InPlace => Err(AllocErr),
            ReallocPlacement::MayMove if new_size == 0 => {
                self.dealloc(ptr, layout);
                Ok(MemoryBlock {
                    ptr: layout.dangling(),
                    size: 0,
                })
            }
            ReallocPlacement::MayMove => {
                // `realloc` probably checks for `new_size < size` or something similar.
                intrinsics::assume(new_size < size);
                let ptr = GlobalAlloc::realloc(self, ptr.as_ptr(), layout, new_size);
                Ok(MemoryBlock {
                    ptr: NonNull::new(ptr).ok_or(AllocErr)?,
                    size: new_size,
                })
            }
        }
    }
}

mod realloc_fallback {
    use core::alloc::{GlobalAlloc, Layout};
    use core::cmp;
    use core::ptr;

    impl super::System {
        pub(crate) unsafe fn realloc_fallback(
            &self,
            ptr: *mut u8,
            old_layout: Layout,
            new_size: usize,
        ) -> *mut u8 {
            // Docs for GlobalAlloc::realloc require this to be valid:
            let new_layout = Layout::from_size_align_unchecked(new_size, old_layout.align());

            let new_ptr = GlobalAlloc::alloc(self, new_layout);
            if !new_ptr.is_null() {
                let size = cmp::min(old_layout.size(), new_size);
                ptr::copy_nonoverlapping(ptr, new_ptr, size);
                GlobalAlloc::dealloc(self, ptr, old_layout);
            }
            new_ptr
        }
    }
}

mod platform {
    use super::*;
    use core::alloc::{GlobalAlloc, Layout};
    use core::ffi::c_void;
    use core::ptr;
    use libc;

    unsafe impl GlobalAlloc for System {
        #[inline]
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
                libc::malloc(layout.size()) as *mut u8
            } else {
                aligned_malloc(&layout)
            }
        }

        #[inline]
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
                libc::calloc(layout.size(), 1) as *mut u8
            } else {
                let ptr = self.alloc(layout);
                if !ptr.is_null() {
                    ptr::write_bytes(ptr, 0, layout.size());
                }
                ptr
            }
        }

        #[inline]
        unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
            libc::free(ptr as *mut c_void)
        }

        #[inline]
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            if layout.align() <= MIN_ALIGN && layout.align() <= new_size {
                libc::realloc(ptr as *mut c_void, new_size) as *mut u8
            } else {
                self.realloc_fallback(ptr, layout, new_size)
            }
        }
    }

    #[inline]
    unsafe fn aligned_malloc(layout: &Layout) -> *mut u8 {
        libc::memalign(layout.align(), layout.size()) as *mut u8
    }
}

mod libc {
    use core::ffi::c_void;
    type size_t = usize;
    extern "C" {
        pub fn calloc(nobj: size_t, size: size_t) -> *mut c_void;
        pub fn malloc(size: size_t) -> *mut c_void;
        pub fn realloc(p: *mut c_void, size: size_t) -> *mut c_void;
        pub fn free(p: *mut c_void);
        pub fn memalign(align: size_t, size: size_t) -> *mut c_void;
    }
}
