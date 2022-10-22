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

//! # align box crate for Rust SGX SDK
//!

use super::alignalloc::AlignAlloc;
pub use super::alignalloc::AlignReq;
use alloc::alloc::handle_alloc_error;
use core::alloc::Layout;
use core::borrow;
use core::fmt;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::ptr::{NonNull, Unique};

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
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for AlignBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> AsRef<T> for AlignBox<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for AlignBox<T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T> borrow::Borrow<T> for AlignBox<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> borrow::BorrowMut<T> for AlignBox<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self
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
        let ptr = match unsafe {
            AlignAlloc.alloc_with_pad_align_zeroed(self.origin_layout, self.align_layout)
        } {
            Ok(p) => p,
            Err(_) => handle_alloc_error(self.align_layout),
        };
        unsafe {
            ptr::copy_nonoverlapping(
                &(**self).clone() as *const _ as *const u8,
                ptr.as_ptr(),
                self.origin_layout.size(),
            );
        }
        AlignBox {
            ptr: Unique::new(ptr.cast::<T>().as_ptr()).unwrap(),
            align_layout: self.align_layout,
            origin_layout: self.origin_layout,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &AlignBox<T>) {
        if source.align_layout.size() != self.align_layout.size() {
            let ptr = match unsafe {
                AlignAlloc.alloc_with_pad_align_zeroed(source.origin_layout, source.align_layout)
            } {
                Ok(p) => p,
                Err(_) => handle_alloc_error(source.align_layout),
            };
            unsafe {
                ptr::copy_nonoverlapping(
                    &(**source).clone() as *const _ as *const u8,
                    ptr.as_ptr(),
                    source.origin_layout.size(),
                );
                self.dealloc_buffer();
            }
            self.ptr = Unique::new(ptr.cast::<T>().as_ptr()).unwrap();
        } else {
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
    fn new_with_req_in(align: usize, align_req: &[AlignReq]) -> Option<AlignBox<T>> {
        if align_req.is_empty() {
            AlignBox::new_in()
        } else {
            AlignBox::allocate_in(true, align, align_req)
        }
    }

    fn new_with_align_in(align: usize) -> Option<AlignBox<T>> {
        let v: [AlignReq; 1] = [AlignReq {
            offset: 0,
            len: mem::size_of::<T>(),
        }];
        AlignBox::allocate_in(true, align, &v)
    }

    fn new_in() -> Option<AlignBox<T>> {
        let v: [AlignReq; 1] = [AlignReq {
            offset: 0,
            len: mem::size_of::<T>(),
        }];
        AlignBox::allocate_in(true, mem::align_of::<T>(), &v)
    }

    fn allocate_in(zeroed: bool, align: usize, align_req: &[AlignReq]) -> Option<AlignBox<T>> {
        if mem::size_of::<T>() == 0 {
            return None;
        }

        let layout = match Layout::from_size_align(mem::size_of::<T>(), align) {
            Ok(n) => n,
            Err(_) => return None,
        };

        let align_layout = match AlignAlloc.pad_align_to(layout, align_req) {
            Ok(n) => n,
            Err(_) => return None,
        };

        // handles ZSTs and `cap = 0` alike
        let result = if zeroed {
            unsafe { AlignAlloc.alloc_with_req_zeroed(layout, align_req) }
        } else {
            unsafe { AlignAlloc.alloc_with_req(layout, align_req) }
        };
        let ptr = match result {
            Ok(p) => p,
            Err(_) => handle_alloc_error(align_layout),
        };

        Some(AlignBox {
            ptr: Unique::new(ptr.cast::<T>().as_ptr()).unwrap(),
            align_layout,
            origin_layout: layout,
        })
    }
}

impl<T> AlignBox<T> {
    pub fn new() -> Option<AlignBox<T>> {
        Self::new_in()
    }
    pub fn new_with_align(align: usize) -> Option<AlignBox<T>> {
        Self::new_with_align_in(align)
    }
    pub fn new_with_req(align: usize, align_req: &[AlignReq]) -> Option<AlignBox<T>> {
        Self::new_with_req_in(align, align_req)
    }
}

impl<T> AlignBox<T> {
    pub fn heap_init<F>(initialize: F) -> Option<AlignBox<T>>
    where
        F: Fn(&mut T),
    {
        unsafe {
            let mut t = Self::new_in();
            if let Some(ref mut b) = t {
                initialize(b.ptr.as_mut())
            }
            t
        }
    }
    pub fn heap_init_with_align<F>(initialize: F, align: usize) -> Option<AlignBox<T>>
    where
        F: Fn(&mut T),
    {
        unsafe {
            let mut t = Self::new_with_align(align);
            if let Some(ref mut b) = t {
                initialize(b.ptr.as_mut())
            }
            t
        }
    }
    pub fn heap_init_with_req<F>(
        initialize: F,
        align: usize,
        data: &[AlignReq],
    ) -> Option<AlignBox<T>>
    where
        F: Fn(&mut T),
    {
        unsafe {
            let mut t = Self::new_with_req(align, data);
            if let Some(ref mut b) = t {
                initialize(b.ptr.as_mut())
            }
            t
        }
    }
}
