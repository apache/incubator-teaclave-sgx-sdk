// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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
#![crate_name = "sgx_alloc"]
#![crate_type = "rlib"]

#![no_std]

#![feature(global_allocator)]
#![feature(allocator_api)]
#![feature(alloc)]

extern crate sgx_types;
extern crate sgx_trts;

extern crate alloc;
use self::alloc::heap::{Alloc, AllocErr, Layout, Excess, CannotReallocInPlace};

// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values. In practice, the alignment is a
// constant at the call site and the branch will be optimized out.
#[cfg(target_arch = "x86")]
const MIN_ALIGN: usize = 8;
#[cfg(target_arch = "x86_64")]
const MIN_ALIGN: usize = 16;

pub struct System;

unsafe impl Alloc for System {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        (&*self).alloc(layout)
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout)
        -> Result<*mut u8, AllocErr>
    {
        (&*self).alloc_zeroed(layout)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        (&*self).dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn realloc(&mut self,
                      ptr: *mut u8,
                      old_layout: Layout,
                      new_layout: Layout) -> Result<*mut u8, AllocErr> {
        (&*self).realloc(ptr, old_layout, new_layout)
    }

    fn oom(&mut self, err: AllocErr) -> ! {
        (&*self).oom(err)
    }

    #[inline]
    fn usable_size(&self, layout: &Layout) -> (usize, usize) {
        (&self).usable_size(layout)
    }

    #[inline]
    unsafe fn alloc_excess(&mut self, layout: Layout) -> Result<Excess, AllocErr> {
        (&*self).alloc_excess(layout)
    }

    #[inline]
    unsafe fn realloc_excess(&mut self,
                             ptr: *mut u8,
                             layout: Layout,
                             new_layout: Layout) -> Result<Excess, AllocErr> {
        (&*self).realloc_excess(ptr, layout, new_layout)
    }

    #[inline]
    unsafe fn grow_in_place(&mut self,
                            ptr: *mut u8,
                            layout: Layout,
                            new_layout: Layout) -> Result<(), CannotReallocInPlace> {
        (&*self).grow_in_place(ptr, layout, new_layout)
    }

    #[inline]
    unsafe fn shrink_in_place(&mut self,
                              ptr: *mut u8,
                              layout: Layout,
                              new_layout: Layout) -> Result<(), CannotReallocInPlace> {
        (&*self).shrink_in_place(ptr, layout, new_layout)
    }
}

mod platform {

    use sgx_types::c_void;
    use sgx_trts::libc;
    use core::cmp;
    use core::ptr;

    use MIN_ALIGN;
    use System;
    use alloc::heap::{Alloc, AllocErr, Layout};

    unsafe impl<'a> Alloc for &'a System {
        #[inline]
        unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
            let ptr = if layout.align() <= MIN_ALIGN {
                libc::malloc(layout.size()) as *mut u8
            } else {
                aligned_malloc(&layout)
            };
            if !ptr.is_null() {
                Ok(ptr)
            } else {
                Err(AllocErr::Exhausted { request: layout })
            }
        }

        #[inline]
        unsafe fn alloc_zeroed(&mut self, layout: Layout)
            -> Result<*mut u8, AllocErr>
        {
            if layout.align() <= MIN_ALIGN {
                let ptr = libc::calloc(layout.size(), 1) as *mut u8;
                if !ptr.is_null() {
                    Ok(ptr)
                } else {
                    Err(AllocErr::Exhausted { request: layout })
                }
            } else {
                let ret = self.alloc(layout.clone());
                if let Ok(ptr) = ret {
                    ptr::write_bytes(ptr, 0, layout.size());
                }
                ret
            }
        }

        #[inline]
        unsafe fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
            libc::free(ptr as *mut c_void)
        }

        #[inline]
        unsafe fn realloc(&mut self,
                          ptr: *mut u8,
                          old_layout: Layout,
                          new_layout: Layout) -> Result<*mut u8, AllocErr> {
            if old_layout.align() != new_layout.align() {
                return Err(AllocErr::Unsupported {
                    details: "cannot change alignment on `realloc`",
                })
            }

            if new_layout.align() <= MIN_ALIGN {
                let ptr = libc::realloc(ptr as *mut c_void, new_layout.size());
                if !ptr.is_null() {
                    Ok(ptr as *mut u8)
                } else {
                    Err(AllocErr::Exhausted { request: new_layout })
                }
            } else {
                let res = self.alloc(new_layout.clone());
                if let Ok(new_ptr) = res {
                    let size = cmp::min(old_layout.size(), new_layout.size());
                    ptr::copy_nonoverlapping(ptr, new_ptr, size);
                    self.dealloc(ptr, old_layout);
                }
                res
            }
        }

        fn oom(&mut self, err: AllocErr) -> ! {

            use sgx_trts::oom;
            oom::rsgx_oom(err)
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
