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

#![feature(allocator_api)]

extern crate sgx_trts;

// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values.
#[cfg(target_arch = "x86")]
#[allow(dead_code)]
const MIN_ALIGN: usize = 8;
#[cfg(target_arch = "x86_64")]
#[allow(dead_code)]
const MIN_ALIGN: usize = 16;

use core::alloc::{Alloc, GlobalAlloc, AllocErr, Layout};
use core::ptr::NonNull;

/// The default memory allocator provided by the operating system.
///
/// This is based on `malloc` on Unix platforms and `HeapAlloc` on Windows,
/// plus related functions.
///
/// This type can be used in a `static` item
/// with the `#[global_allocator]` attribute
/// to force the global allocator to be the system’s one.
/// (The default is jemalloc for executables, on some platforms.)
///
/// ```rust
/// use std::alloc::System;
///
/// #[global_allocator]
/// static A: System = System;
///
/// fn main() {
///     let a = Box::new(4); // Allocates from the system allocator.
///     println!("{}", a);
/// }
/// ```
///
/// It can also be used directly to allocate memory
/// independently of the standard library’s global allocator.
pub struct System;

unsafe impl Alloc for System {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        NonNull::new(GlobalAlloc::alloc(self, layout)).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        NonNull::new(GlobalAlloc::alloc_zeroed(self, layout)).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        GlobalAlloc::dealloc(self, ptr.as_ptr(), layout)
    }

    #[inline]
    unsafe fn realloc(&mut self,
                      ptr: NonNull<u8>,
                      layout: Layout,
                      new_size: usize) -> Result<NonNull<u8>, AllocErr> {
        NonNull::new(GlobalAlloc::realloc(self, ptr.as_ptr(), layout, new_size)).ok_or(AllocErr)
    }
}

mod realloc_fallback {
    use core::alloc::{GlobalAlloc, Layout};
    use core::cmp;
    use core::ptr;

    impl super::System {
        pub(crate) unsafe fn realloc_fallback(&self, ptr: *mut u8, old_layout: Layout,
                                              new_size: usize) -> *mut u8 {
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
    use sgx_trts::libc;

    use core::ptr;

    use MIN_ALIGN;
    use System;
    use core::alloc::{GlobalAlloc, Layout};

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
                let ptr = self.alloc(layout.clone());
                if !ptr.is_null() {
                    ptr::write_bytes(ptr, 0, layout.size());
                }
                ptr
            }
        }

        #[inline]
        unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
            libc::free(ptr as *mut libc::c_void)
        }

        #[inline]
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            if layout.align() <= MIN_ALIGN && layout.align() <= new_size {
                libc::realloc(ptr as *mut libc::c_void, new_size) as *mut u8
            } else {
                self.realloc_fallback(ptr, layout, new_size)
            }
        }
    }

    #[inline]
    unsafe fn aligned_malloc(layout: &Layout) -> *mut u8 {
        let mut out = ptr::null_mut();
        let ret = libc::posix_memalign(&mut out, layout.align(), layout.size());
        if ret != 0 {
            ptr::null_mut()
        } else {
            out as *mut u8
        }
    }
}
