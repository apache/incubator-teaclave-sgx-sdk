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

use crate::alignalloc::{AlignAlloc, AlignReq};
use core::alloc::Layout;
use core::ffi::c_void;
use core::ptr::{self, NonNull};
use core::slice;

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_aligned_malloc(
    size: usize,
    align: usize,
    align_req: *const AlignReq,
    count: usize,
) -> *mut c_void {
    if size == 0 || align == 0 {
        return ptr::null_mut();
    }
    let layout = match Layout::from_size_align(size, align) {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };

    match if align_req.is_null() || count == 0 {
        let req = [AlignReq::default(); 0];
        AlignAlloc.alloc_with_req(layout, &req)
    } else {
        let req = slice::from_raw_parts(align_req, count);
        AlignAlloc.alloc_with_req(layout, req)
    } {
        Ok(p) => p.as_ptr() as *mut c_void,
        Err(_) => ptr::null_mut(),
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_aligned_free(ptr: *mut c_void) {
    let ptr = match NonNull::new(ptr as *mut u8) {
        Some(p) => p,
        None => return,
    };
    AlignAlloc.dealloc(ptr, Layout::from_size_align_unchecked(0, 1));
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_get_aligned_ptr(
    raw: *mut c_void,
    raw_size: usize,
    alloc_size: usize,
    align: usize,
    align_req: *const AlignReq,
    count: usize,
) -> *mut c_void {
    if raw.is_null() || raw_size == 0 || alloc_size == 0 || align == 0 || alloc_size > raw_size {
        return ptr::null_mut();
    }

    if check_overflow(raw as *mut u8, raw_size) {
        return ptr::null_mut();
    }

    let layout = match Layout::from_size_align(alloc_size, align) {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };

    let align_layout = match if align_req.is_null() || count == 0 {
        let req: [AlignReq; 1] = [AlignReq {
            offset: 0,
            len: layout.size(),
        }];
        AlignAlloc.pad_align_to(layout, &req)
    } else {
        let req = slice::from_raw_parts(align_req, count);
        AlignAlloc.pad_align_to(layout, req)
    } {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };

    let ptr = make_aligned_ptr(
        raw as *mut u8,
        align_layout.layout.align(),
        align_layout.pad,
    );
    if check_overflow(ptr, alloc_size) {
        return ptr::null_mut();
    }

    if ptr as usize + alloc_size > raw as usize + raw_size {
        return ptr::null_mut();
    }

    ptr as *mut c_void
}

#[inline]
fn check_overflow(buf: *mut u8, len: usize) -> bool {
    (buf as usize).checked_add(len).is_none()
}

#[inline]
fn make_aligned_ptr(raw: *mut u8, align: usize, offset: usize) -> *mut u8 {
    ((((raw as usize) + align - 1) & !(align - 1)) + offset) as *mut u8
}
