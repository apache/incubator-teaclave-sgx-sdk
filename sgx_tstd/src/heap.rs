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

pub use alloc::heap::{Heap, Alloc, Layout, Excess, CannotReallocInPlace, AllocErr};
pub use sgx_alloc::System;

#[doc(hidden)]
#[allow(unused_attributes)]
pub mod __default_lib_allocator {
    use super::{System, Layout, Alloc, AllocErr};
    use ptr;

    // for symbol names src/librustc/middle/allocator.rs
    // for signatures src/librustc_allocator/lib.rs

    // linkage directives are provided as part of the current compiler allocator
    // ABI

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_alloc(size: usize,
                                     align: usize,
                                     err: *mut u8) -> *mut u8 {
        let layout = Layout::from_size_align_unchecked(size, align);
        match System.alloc(layout) {
            Ok(p) => p,
            Err(e) => {
                ptr::write(err as *mut AllocErr, e);
                0 as *mut u8
            }
        }
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_oom(err: *const u8) -> ! {
        System.oom((*(err as *const AllocErr)).clone())
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_dealloc(ptr: *mut u8,
                                       size: usize,
                                       align: usize) {
        System.dealloc(ptr, Layout::from_size_align_unchecked(size, align))
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_usable_size(layout: *const u8,
                                           min: *mut usize,
                                           max: *mut usize) {
        let pair = System.usable_size(&*(layout as *const Layout));
        *min = pair.0;
        *max = pair.1;
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_realloc(ptr: *mut u8,
                                       old_size: usize,
                                       old_align: usize,
                                       new_size: usize,
                                       new_align: usize,
                                       err: *mut u8) -> *mut u8 {
        let old_layout = Layout::from_size_align_unchecked(old_size, old_align);
        let new_layout = Layout::from_size_align_unchecked(new_size, new_align);
        match System.realloc(ptr, old_layout, new_layout) {
            Ok(p) => p,
            Err(e) => {
                ptr::write(err as *mut AllocErr, e);
                0 as *mut u8
            }
        }
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_alloc_zeroed(size: usize,
                                            align: usize,
                                            err: *mut u8) -> *mut u8 {
        let layout = Layout::from_size_align_unchecked(size, align);
        match System.alloc_zeroed(layout) {
            Ok(p) => p,
            Err(e) => {
                ptr::write(err as *mut AllocErr, e);
                0 as *mut u8
            }
        }
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_alloc_excess(size: usize,
                                            align: usize,
                                            excess: *mut usize,
                                            err: *mut u8) -> *mut u8 {
        let layout = Layout::from_size_align_unchecked(size, align);
        match System.alloc_excess(layout) {
            Ok(p) => {
                *excess = p.1;
                p.0
            }
            Err(e) => {
                ptr::write(err as *mut AllocErr, e);
                0 as *mut u8
            }
        }
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_realloc_excess(ptr: *mut u8,
                                              old_size: usize,
                                              old_align: usize,
                                              new_size: usize,
                                              new_align: usize,
                                              excess: *mut usize,
                                              err: *mut u8) -> *mut u8 {
        let old_layout = Layout::from_size_align_unchecked(old_size, old_align);
        let new_layout = Layout::from_size_align_unchecked(new_size, new_align);
        match System.realloc_excess(ptr, old_layout, new_layout) {
            Ok(p) => {
                *excess = p.1;
                p.0
            }
            Err(e) => {
                ptr::write(err as *mut AllocErr, e);
                0 as *mut u8
            }
        }
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_grow_in_place(ptr: *mut u8,
                                             old_size: usize,
                                             old_align: usize,
                                             new_size: usize,
                                             new_align: usize) -> u8 {
        let old_layout = Layout::from_size_align_unchecked(old_size, old_align);
        let new_layout = Layout::from_size_align_unchecked(new_size, new_align);
        match System.grow_in_place(ptr, old_layout, new_layout) {
            Ok(()) => 1,
            Err(_) => 0,
        }
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_shrink_in_place(ptr: *mut u8,
                                               old_size: usize,
                                               old_align: usize,
                                               new_size: usize,
                                               new_align: usize) -> u8 {
        let old_layout = Layout::from_size_align_unchecked(old_size, old_align);
        let new_layout = Layout::from_size_align_unchecked(new_size, new_align);
        match System.shrink_in_place(ptr, old_layout, new_layout) {
            Ok(()) => 1,
            Err(_) => 0,
        }
    }
}