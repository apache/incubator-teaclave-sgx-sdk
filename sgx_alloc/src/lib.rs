// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! # liballoc crate for Rust SGX SDK
//!
//! This crate equals to the `liballoc_system` crate in Rust.
//! It connects Rust memory allocation to Intel SGX's sgx_tstd library.
//! It is essential, because we depends on Intel SGX's SDK.

#![no_std]

#![feature(global_allocator)]
#![feature(allocator_api)]

extern crate sgx_trts;

use core::alloc::{Alloc, GlobalAlloc, AllocErr, Layout, Opaque};
use core::ptr::NonNull;

// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values. In practice, the alignment is a
// constant at the call site and the branch will be optimized out.
#[cfg(target_arch = "x86")]
#[allow(dead_code)]
const MIN_ALIGN: usize = 8;
#[cfg(target_arch = "x86_64")]
#[allow(dead_code)]
const MIN_ALIGN: usize = 16;

pub struct System;

unsafe impl Alloc for System {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<Opaque>, AllocErr> {
        NonNull::new(GlobalAlloc::alloc(self, layout)).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<Opaque>, AllocErr> {
        NonNull::new(GlobalAlloc::alloc_zeroed(self, layout)).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<Opaque>, layout: Layout) {
        GlobalAlloc::dealloc(self, ptr.as_ptr(), layout)
    }

    #[inline]
    unsafe fn realloc(&mut self,
                      ptr: NonNull<Opaque>,
                      layout: Layout,
                      new_size: usize) -> Result<NonNull<Opaque>, AllocErr> {
        NonNull::new(GlobalAlloc::realloc(self, ptr.as_ptr(), layout, new_size)).ok_or(AllocErr)
    }
}

#[cfg(stage0)]
unsafe impl<'a> Alloc for &'a System {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<Opaque>, AllocErr> {
        NonNull::new(GlobalAlloc::alloc(*self, layout)).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<Opaque>, AllocErr> {
        NonNull::new(GlobalAlloc::alloc_zeroed(*self, layout)).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<Opaque>, layout: Layout) {
        GlobalAlloc::dealloc(*self, ptr.as_ptr(), layout)
    }

    #[inline]
    unsafe fn realloc(&mut self,
                      ptr: NonNull<Opaque>,
                      layout: Layout,
                      new_size: usize) -> Result<NonNull<Opaque>, AllocErr> {
        NonNull::new(GlobalAlloc::realloc(*self, ptr.as_ptr(), layout, new_size)).ok_or(AllocErr)
    }
}

mod realloc_fallback {
    use core::alloc::{GlobalAlloc, Opaque, Layout};
    use core::cmp;
    use core::ptr;

    impl super::System {
        pub(crate) unsafe fn realloc_fallback(&self, ptr: *mut Opaque, old_layout: Layout,
                                              new_size: usize) -> *mut Opaque {
            // Docs for GlobalAlloc::realloc require this to be valid:
            let new_layout = Layout::from_size_align_unchecked(new_size, old_layout.align());

            let new_ptr = GlobalAlloc::alloc(self, new_layout);
            if !new_ptr.is_null() {
                let size = cmp::min(old_layout.size(), new_size);
                ptr::copy_nonoverlapping(ptr as *mut u8, new_ptr as *mut u8, size);
                GlobalAlloc::dealloc(self, ptr, old_layout);
            }
            new_ptr
        }
    }
}

mod platform {
    use sgx_trts::libc;

    use core::ptr;

    use MIN_ALIGN;
    use System;
    use core::alloc::{GlobalAlloc, Layout, Opaque};

    unsafe impl GlobalAlloc for System {
        #[inline]
        unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
            if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
                libc::malloc(layout.size()) as *mut Opaque
            } else {
                aligned_malloc(&layout)
            }
        }

        #[inline]
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut Opaque {
            if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
                libc::calloc(layout.size(), 1) as *mut Opaque
            } else {
                let ptr = self.alloc(layout.clone());
                if !ptr.is_null() {
                    ptr::write_bytes(ptr as *mut u8, 0, layout.size());
                }
                ptr
            }
        }

        #[inline]
        unsafe fn dealloc(&self, ptr: *mut Opaque, _layout: Layout) {
            libc::free(ptr as *mut libc::c_void)
        }

        #[inline]
        unsafe fn realloc(&self, ptr: *mut Opaque, layout: Layout, new_size: usize) -> *mut Opaque {
            if layout.align() <= MIN_ALIGN && layout.align() <= new_size {
                libc::realloc(ptr as *mut libc::c_void, new_size) as *mut Opaque
            } else {
                self.realloc_fallback(ptr, layout, new_size)
            }
        }
    }

    #[inline]
    unsafe fn aligned_malloc(layout: &Layout) -> *mut Opaque {
        let mut out = ptr::null_mut();
        let ret = libc::posix_memalign(&mut out, layout.align(), layout.size());
        if ret != 0 {
            // FIXME: use Opaque::null_mut https://github.com/rust-lang/rust/issues/49659
            0 as *mut Opaque
        } else {
            out as *mut Opaque
        }
    }
}
