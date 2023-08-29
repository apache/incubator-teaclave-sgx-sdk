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

use core::alloc::{AllocError, Allocator, Layout};
use core::ptr::NonNull;

use crate::emm::interior::{RES_ALLOCATOR, STATIC};

/// Alloc layout memory from reserve memory region
#[derive(Clone, Copy)]
pub struct ResAlloc;

unsafe impl Allocator for ResAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let size = layout.size();
        RES_ALLOCATOR
            .get()
            .unwrap()
            .lock()
            .emalloc(size)
            .map(|addr| NonNull::slice_from_raw_parts(NonNull::new(addr as *mut u8).unwrap(), size))
            .map_err(|_| AllocError)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        RES_ALLOCATOR.get().unwrap().lock().efree(ptr.addr().get())
    }
}

/// Alloc layout memory from static memory region
#[derive(Clone, Copy)]
pub struct StaticAlloc;

unsafe impl Allocator for StaticAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        STATIC
            .get()
            .unwrap()
            .lock()
            .alloc(layout)
            .map(|addr| NonNull::slice_from_raw_parts(addr, layout.size()))
            .map_err(|_| AllocError)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        STATIC.get().unwrap().lock().dealloc(ptr, layout);
    }
}
