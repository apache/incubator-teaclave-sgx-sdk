// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

//! # align box crate for Rust SGX SDK
//!
use sgx_types::*;
use core::ptr::{Unique, NonNull};
use core::ops::{DerefMut, Deref};
use core::mem;
use core::ptr;
use core::fmt;
use core::borrow;
use alloc::alloc::{Layout, handle_alloc_error};
use super::alignalloc::AlignAlloc;

pub struct AlignBox<T> {
    ptr: Unique<T>,
    align_layout: Layout,
    origin_layout: Layout,
}

impl<T> AlignBox<T> {
    /// Gets a raw pointer to the start of the allocation. Note that this is
    /// Unique::empty() if `cap = 0` or T is zero-sized. In the former case, you must
    /// be careful.
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> Deref for AlignBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe{self.ptr.as_ref()}
    }
}

impl<T> DerefMut for AlignBox<T> {
    fn deref_mut(&mut self) -> &mut T {
       unsafe{self.ptr.as_mut()}
    }
}

impl<T> AsRef<T> for AlignBox<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T> AsMut<T> for AlignBox<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T> borrow::Borrow<T> for AlignBox<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T> borrow::BorrowMut<T> for AlignBox<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T: fmt::Display> fmt::Display for AlignBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for AlignBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlignBox")
            .field("align_layout", &self.align_layout)
            .field("data", &**self)
            .finish()
    }
}

impl<T: Clone> Clone for AlignBox<T> {
    #[rustfmt::skip]
    #[inline]
    fn clone(&self) -> AlignBox<T> {
        let ptr = match unsafe{AlignAlloc.alloc_with_pad_align_zeroed(self.origin_layout, self.align_layout)} {
            Ok(p) => p,
            Err(_) => handle_alloc_error(self.align_layout),
        };
        unsafe{ptr::copy_nonoverlapping(&(**self).clone() as *const _ as *const u8, ptr.as_ptr() , self.origin_layout.size())};
        AlignBox {ptr: ptr.cast().into(), align_layout: self.align_layout, origin_layout: self.origin_layout}
    }

    #[inline]
    fn clone_from(&mut self, source: &AlignBox<T>) {
        if source.align_layout.size() != self.align_layout.size() {
            let ptr = match unsafe{AlignAlloc.alloc_with_pad_align_zeroed(source.origin_layout, source.align_layout)} {
                Ok(p) => p,
                Err(_) => handle_alloc_error(source.align_layout),
            };
            unsafe {
                ptr::copy_nonoverlapping(&(**source).clone() as *const _ as *const u8, ptr.as_ptr(), source.origin_layout.size());
                self.dealloc_buffer();
            }
            self.ptr = ptr.cast().into();
        }  else   {
            (**self).clone_from(&(**source));
        }
        self.align_layout = source.align_layout;
        self.origin_layout = source.origin_layout;
    }
}

impl<T> AlignBox<T> {
    unsafe fn dealloc_buffer(&mut self) {
        let elem_size = mem::size_of::<T>();
        if elem_size != 0 {
            AlignAlloc.dealloc(NonNull::from(self.ptr).cast(), self.origin_layout)
        }
    }
}

unsafe impl<#[may_dangle] T> Drop for AlignBox<T> {
    fn drop(&mut self) {
        unsafe {
            self.dealloc_buffer();
        }
    }
}

impl<T> AlignBox<T> {

    fn new_with_req_in(align: usize, align_req: &[align_req_t]) -> Option<Self> {
        if align_req.len() == 0 {
            AlignBox::new_in()
        } else {
            AlignBox::allocate_in(true, align, align_req)
        }
    }

    fn new_with_align_in(align: usize) -> Option<Self> {
        let v: [align_req_t; 1] = [align_req_t{offset:0, len:mem::size_of::<T>()}];
        AlignBox::allocate_in(true, align, &v)
    }

    fn new_in() -> Option<Self> {
        let v: [align_req_t; 1] = [align_req_t{offset: 0, len:mem::size_of::<T>()}];
        AlignBox::allocate_in(true, mem::align_of::<T>(), &v)
    }

    fn allocate_in(zeroed: bool, align: usize, align_req: &[align_req_t]) -> Option<Self> {
        if mem::size_of::<T>() == 0 {
            return None;
        }

        let layout = match Layout::from_size_align(mem::size_of::<T>(), align) {
            Ok(n) => {n},
            Err(_) => {return None},
        };
    
        let align_layout = match AlignAlloc.pad_align_to(layout, align_req) {
            Ok(n) => {n},
            Err(_) => {return None},
        };
       
        // handles ZSTs and `cap = 0` alike
        let result = if zeroed {
            unsafe{AlignAlloc.alloc_with_req_zeroed(layout, align_req)}
        } else {
            unsafe{AlignAlloc.alloc_with_req(layout, align_req)}
        };
        let ptr = match result {
            Ok(r) => r.cast(),
            Err(_) => handle_alloc_error(align_layout),
        };

        Some(AlignBox{ptr: ptr.into(), align_layout: align_layout, origin_layout: layout})
    }
}

impl<T> AlignBox<T> {
    pub fn new() -> Option<Self> {
        Self::new_in()
    }
     pub fn new_with_align(align: usize) -> Option<Self> {
        Self::new_with_align_in(align)
    }
    pub fn new_with_req(align: usize, align_req: &[align_req_t]) -> Option<Self> {
        Self::new_with_req_in(align, align_req)
    }
}

impl<T> AlignBox<T> {
    pub fn heap_init<F>(initialize: F) -> Option<Self>
    where
        F: Fn(&mut T),
    {
        unsafe {
            let mut t = Self::new_in();
            match t {
                Some(ref mut b) => initialize(b.ptr.as_mut()),
                None => (),
            }
            t
        }
    }
    pub fn heap_init_with_align<F>(initialize: F, align: usize) -> Option<Self>
    where
        F: Fn(&mut T),
    {
        unsafe {
            let mut t = Self::new_with_align(align);
            match t {
                Some(ref mut b) => initialize(b.ptr.as_mut()),
                None => (),
            }
            t
        }
    }
    pub fn heap_init_with_req<F>(initialize: F, align: usize, data: &[align_req_t]) -> Option<Self>
    where
        F: Fn(&mut T),
    {
      unsafe {
            let mut t = Self::new_with_req(align, data);
            match t {
                Some(ref mut b) => initialize(b.ptr.as_mut()),
                None => (),
            }
            t
        }
    }
}
