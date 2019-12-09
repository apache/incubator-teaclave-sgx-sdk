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

//! # align alloc crate for Rust SGX SDK
//!

use sgx_types::*;
use core::alloc::Layout;
use core::ptr::NonNull;
use core::fmt;
pub use self::platform::*;

pub struct AlignAlloc;

impl AlignAlloc {
    #[inline]
    pub unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc(layout)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_zeroed(&mut self, layout: Layout)
        -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc_zeroed(layout)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_with_req(&mut self, layout: Layout, align_req: &[align_req_t]) -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc_with_req(layout, align_req)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_with_req_zeroed(&mut self, layout: Layout, align_req: &[align_req_t])
        -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc_with_req_zeroed(layout, align_req)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_with_pad_align(&mut self, layout: Layout, align_layout: Layout) -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc_with_pad_align(layout, align_layout)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_with_pad_align_zeroed(&mut self, layout: Layout, align_layout: Layout)
        -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc_with_pad_align_zeroed(layout, align_layout)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn dealloc(&mut self, ptr: core::ptr::NonNull<u8>, layout: Layout) {
        platform::dealloc(ptr.as_ptr(), layout)
    }

    #[inline]
    pub fn pad_align_to(&self, layout: Layout, align_req: &[align_req_t]) -> Result<Layout, AlignLayoutErr> {
        platform::pad_align_to(layout, align_req)
    }
}

pub struct AlighAllocErr;

impl fmt::Display for AlighAllocErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("memory allocation failed")
    }
}

pub struct AlignLayoutErr;

impl fmt::Display for AlignLayoutErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid parameters to align Layout")
    }
}

mod platform {
    use sgx_types::*;
    use super::AlignLayoutErr;
    use core::ffi::c_void;
    use core::ptr;
    use core::mem;
    use core::slice;
    use core::alloc::Layout;

    pub unsafe fn alloc(layout: Layout) -> *mut u8 {
        let req: [align_req_t; 1] = [align_req_t{offset: 0, len: layout.size()}];
        let align_req = &req[..];
        alloc_with_req(layout, align_req)
    }

    pub unsafe fn alloc_with_req(layout: Layout, align_req: &[align_req_t]) -> *mut u8 {
        if !check_layout(&layout) {
            return ptr::null_mut();
        }

        let align_layout = match
            if align_req.len() == 0 {
                let req: [align_req_t; 1] = [align_req_t{offset: 0, len: layout.size()}];
                pad_align_to(layout, &req[..])
            } else {
                pad_align_to(layout, align_req)
            }
        {
            Ok(l) => l,
            Err(_) => return ptr::null_mut(),
        };
        alloc_with_pad_align(layout, align_layout)
    }

    pub unsafe fn alloc_with_pad_align(layout: Layout, align_layout: Layout) -> *mut u8 {
        if !check_layout(&layout) {
            return ptr::null_mut();
        }
        if !check_layout(&align_layout) {
            return ptr::null_mut();
        }
        if align_layout.size() < layout.size() + align_layout.align() {
            return ptr::null_mut();
        }
        let pad = align_layout.size() - align_layout.align() - layout.size();

        let raw = libc::malloc(align_layout.size() + mem::size_of::<*mut u8>()) as *mut u8;
        let aligned_ptr = if raw.is_null() {
            raw
        } else {
            let ptr = make_aligned_ptr(raw, align_layout.align(), pad);
            let p = ptr as *mut *mut u8;
            p.sub(1).write(raw);
            ptr
        };
        aligned_ptr
    }

    #[inline]
    pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 { 
        let ptr = alloc(layout);
        if !ptr.is_null() {
            ptr::write_bytes(ptr, 0, layout.size());
        }
        ptr
    }

    #[inline]
    pub unsafe fn alloc_with_req_zeroed(layout: Layout, align_req: &[align_req_t]) -> *mut u8 {
        let ptr = alloc_with_req(layout, align_req);
        if !ptr.is_null() {
            ptr::write_bytes(ptr, 0, layout.size());
        }
        ptr
    }

    #[inline]
    pub unsafe fn alloc_with_pad_align_zeroed(layout: Layout, align_layout: Layout) -> *mut u8 {
        let ptr = alloc_with_pad_align(align_layout, layout);
        if !ptr.is_null() {
            ptr::write_bytes(ptr, 0, align_layout.size());
        }
        ptr
    }

    #[inline]
    pub unsafe fn dealloc(ptr: *mut u8, _layout: Layout) {
        let p = ptr as *mut *mut u8;
        let raw = ptr::read(p.sub(1));
        libc::free(raw as *mut c_void)
    }

    pub fn pad_align_to(layout: Layout, align_req: &[align_req_t]) -> Result<Layout, AlignLayoutErr> {
        let pad = padding_needed_for(layout, align_req)?;
        let align = align_needed_for(layout, pad)?;
        Layout::from_size_align(pad + align + layout.size(), align).map_err(|_|AlignLayoutErr)
    }

    fn padding_needed_for(layout: Layout, align_req: &[align_req_t]) -> Result<usize, AlignLayoutErr> {
        if !check_layout(&layout) {
            return Err(AlignLayoutErr);
        }
        if !check_align_req(layout.size(), align_req) {
            return Err(AlignLayoutErr);
        }
        let bmp = make_bitmap(align_req);
        let offset = calc_lspc(layout.align(), bmp);
        if offset < 0 {
            Err(AlignLayoutErr)
        } else {
            Ok(offset as usize)
        }
    }

    fn align_needed_for(layout: Layout, offset: usize) -> Result<usize, AlignLayoutErr> {
        Ok(calc_algn(layout.align(), layout.size() + offset))
    }

    #[inline]
    fn make_aligned_ptr(raw: *mut u8, align: usize, offset: usize) -> *mut u8 {
        ((((raw as usize) + align - 1) & !(align - 1)) + offset) as *mut u8
    }

    #[inline]
    fn check_overflow(buf: usize, len: usize) -> bool {
        ((buf + len < len) || (buf + len < buf))
    }

    fn check_layout(layout: &Layout) -> bool {
        if layout.size() == 0 || !layout.align().is_power_of_two() ||
           layout.size() > usize::max_value() - (layout.align() - 1) {
            false
        } else {
            true
        }
    }

    fn check_align_req(size: usize, align_req: &[align_req_t]) -> bool {
        if align_req.len() == 0 {
            return false;
        }
        let len: usize = (size + 7) / 8;
        let bmp: &mut [u8] = unsafe {
            let ptr = libc::malloc(len) as *mut u8;
            if ptr.is_null() {
                return false;
            }
            ptr::write_bytes(ptr, 0, len);
            slice::from_raw_parts_mut(ptr, len)
        };

        for req in align_req {
            if check_overflow(req.offset, req.len) || (req.offset + req.len) > size {
                unsafe{ libc::free(bmp.as_mut_ptr() as *mut c_void); }
                return false;
            } else {
                for i in 0..req.len {
                    let offset = req.offset + i;
                    if (bmp[offset / 8] & 1 << (offset % 8)) != 0 {
                        // overlap in req data
                        unsafe{ libc::free(bmp.as_mut_ptr() as *mut c_void); }
                        return false;
                    }
                    let tmp: u8 = (1 << (offset % 8)) as u8;
                    bmp[offset / 8] |= tmp;
                }
            }
        }
        true
    }

    fn gen_alignmask(al: usize, a: usize, m: u64) -> i64 {
        if a  > al {
            gen_alignmask(al, (a >> 1) as usize, m | (m >> (a >> 1)))
        } else {
            m as i64
        }
    }

    #[inline]
    fn __rol(v: u64, c: usize, m: usize) -> u64  {
        (v << (c & m)) | (v >> (((0 - c as isize) as usize) & m))
    }

    #[inline]
    fn rol(v: i64, c: usize) -> i64 {
        __rol(v as u64 , c, mem::size_of::<i64>() * 8 - 1) as i64
    }

    fn ror(v: i64, c: usize) -> i64 {
        rol(v, (0 - c as isize) as usize)
    }

    fn count_lzb(bmp: i64) -> i32 {
        if bmp == 0 {
            -1
        } else if bmp < 0 {
            0
        } else {
            count_lzb(bmp << 1) + 1
        }
    }

    fn calc_lspc(al: usize, bmp: i64) -> i32 {
        if !al.is_power_of_two() {
            -2
        } else {
            count_lzb(
                !(ror(bmp | ror(bmp, 1) | ror(bmp, 2) | ror(bmp, 3), 5) | ror(bmp, 1)) &
                gen_alignmask(al, mem::size_of::<u64>() * 8, 1_u64 << (mem::size_of::<u64>() * 8 - 1)))
        }
    }

    fn __calc_algn(size: usize, a: usize) -> usize {
        if a > 8 && size <= a / 2 {
            __calc_algn(size, a / 2)
        } else {
            a
        }
    }

    fn calc_algn(al: usize, size: usize) -> usize {
        if al > 64  {
            al
        } else {
            __calc_algn(size, mem::size_of::<u64>() * 8)
        }
    }

    fn make_bitmap(align_req: &[align_req_t]) -> i64 {
        let mut bmp: i64 = 0;
        for req in align_req {
            if req.len > 63 {
                return -1;
            } else {
                bmp |= rol(((1 as i64) << req.len) - 1, req.offset);
            }
        }
        bmp
    }

    mod libc {
        use core::ffi::c_void;
        type size_t = usize;
        extern {
            pub fn malloc(size: size_t) -> * mut c_void;
            pub fn free(p: *mut c_void);
        }
    }
}