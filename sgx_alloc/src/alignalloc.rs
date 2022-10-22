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

//! # align alloc crate for Rust SGX SDK
//!

pub use self::platform::*;
use core::alloc::Layout;
use core::fmt;
use core::ptr::NonNull;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct AlignReq {
    pub offset: usize,
    pub len: usize,
}

pub struct AlignAlloc;

impl AlignAlloc {
    #[inline]
    pub unsafe fn alloc(&self, layout: Layout) -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc(layout)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_zeroed(&self, layout: Layout) -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc_zeroed(layout)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_with_req(
        &self,
        layout: Layout,
        align_req: &[AlignReq],
    ) -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc_with_req(layout, align_req)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_with_req_zeroed(
        &self,
        layout: Layout,
        align_req: &[AlignReq],
    ) -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc_with_req_zeroed(layout, align_req)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_with_pad_align(
        &self,
        layout: Layout,
        align_layout: Layout,
    ) -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc_with_pad_align(layout, align_layout)).ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn alloc_with_pad_align_zeroed(
        &self,
        layout: Layout,
        align_layout: Layout,
    ) -> Result<NonNull<u8>, AlighAllocErr> {
        NonNull::new(platform::alloc_with_pad_align_zeroed(layout, align_layout))
            .ok_or(AlighAllocErr)
    }

    #[inline]
    pub unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        platform::dealloc(ptr.as_ptr(), layout)
    }

    #[inline]
    pub fn pad_align_to(
        &self,
        layout: Layout,
        align_req: &[AlignReq],
    ) -> Result<Layout, AlignLayoutErr> {
        platform::pad_align_to(layout, align_req)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AlighAllocErr;

impl fmt::Display for AlighAllocErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("memory allocation failed")
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AlignLayoutErr;

impl fmt::Display for AlignLayoutErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid parameters to align Layout")
    }
}

mod platform {
    use super::AlignLayoutErr;
    use super::AlignReq;
    use core::alloc::Layout;
    use core::ffi::c_void;
    use core::mem;
    use core::ptr;
    use core::slice;

    #[inline]
    pub unsafe fn alloc(layout: Layout) -> *mut u8 {
        let req: [AlignReq; 1] = [AlignReq {
            offset: 0,
            len: layout.size(),
        }];
        let align_req = &req[..];
        alloc_with_req(layout, align_req)
    }

    pub unsafe fn alloc_with_req(layout: Layout, align_req: &[AlignReq]) -> *mut u8 {
        if !check_layout(&layout) {
            return ptr::null_mut();
        }
        let align_layout = match if align_req.is_empty() {
            let req: [AlignReq; 1] = [AlignReq {
                offset: 0,
                len: layout.size(),
            }];
            pad_align_to(layout, &req[..])
        } else {
            pad_align_to(layout, align_req)
        } {
            Ok(l) => l,
            Err(_) => return ptr::null_mut(),
        };
        alloc_with_pad_align(layout, align_layout)
    }

    #[inline]
    pub unsafe fn alloc_with_pad_align(layout: Layout, align_layout: Layout) -> *mut u8 {
        alloc_with_pad_align_in(false, layout, align_layout)
    }

    #[inline]
    pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
        let req: [AlignReq; 1] = [AlignReq {
            offset: 0,
            len: layout.size(),
        }];
        let align_req = &req[..];
        alloc_with_req_zeroed(layout, align_req)
    }

    pub unsafe fn alloc_with_req_zeroed(layout: Layout, align_req: &[AlignReq]) -> *mut u8 {
        if !check_layout(&layout) {
            return ptr::null_mut();
        }
        let align_layout = match if align_req.is_empty() {
            let req: [AlignReq; 1] = [AlignReq {
                offset: 0,
                len: layout.size(),
            }];
            pad_align_to(layout, &req[..])
        } else {
            pad_align_to(layout, align_req)
        } {
            Ok(l) => l,
            Err(_) => return ptr::null_mut(),
        };
        alloc_with_pad_align_zeroed(layout, align_layout)
    }

    #[inline]
    pub unsafe fn alloc_with_pad_align_zeroed(layout: Layout, align_layout: Layout) -> *mut u8 {
        alloc_with_pad_align_in(true, layout, align_layout)
    }

    unsafe fn alloc_with_pad_align_in(
        zeroed: bool,
        layout: Layout,
        align_layout: Layout,
    ) -> *mut u8 {
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
        if raw.is_null() {
            raw
        } else {
            if zeroed {
                ptr::write_bytes(raw, 0, align_layout.size());
            }
            let ptr = make_aligned_ptr(raw, align_layout.align(), pad);
            let p = ptr as *mut *mut u8;
            p.sub(1).write(raw);
            ptr
        }
    }

    #[inline]
    pub unsafe fn dealloc(ptr: *mut u8, _layout: Layout) {
        let p = ptr as *mut *mut u8;
        let raw = ptr::read(p.sub(1));
        libc::free(raw as *mut c_void)
    }

    pub fn pad_align_to(layout: Layout, align_req: &[AlignReq]) -> Result<Layout, AlignLayoutErr> {
        let pad = padding_needed_for(layout, align_req)?;
        let align = align_needed_for(layout, pad)?;
        Layout::from_size_align(pad + align + layout.size(), align).map_err(|_| AlignLayoutErr)
    }

    fn padding_needed_for(layout: Layout, align_req: &[AlignReq]) -> Result<usize, AlignLayoutErr> {
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
        buf.checked_add(len).is_none()
    }

    fn check_layout(layout: &Layout) -> bool {
        !(layout.size() == 0
            || !layout.align().is_power_of_two()
            || layout.size() > usize::MAX - (layout.align() - 1))
    }

    fn check_align_req(size: usize, align_req: &[AlignReq]) -> bool {
        if align_req.is_empty() {
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
                unsafe {
                    libc::free(bmp.as_mut_ptr() as *mut c_void);
                }
                return false;
            } else {
                for i in 0..req.len {
                    let offset = req.offset + i;
                    if (bmp[offset / 8] & 1 << (offset % 8)) != 0 {
                        // overlap in req data
                        unsafe {
                            libc::free(bmp.as_mut_ptr() as *mut c_void);
                        }
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
        if a > al {
            gen_alignmask(al, a >> 1, m | (m >> (a >> 1)))
        } else {
            m as i64
        }
    }

    #[inline]
    fn __rol(v: u64, c: usize, m: usize) -> u64 {
        (v << (c & m)) | (v >> (((0 - c as isize) as usize) & m))
    }

    #[inline]
    fn rol(v: i64, c: usize) -> i64 {
        __rol(v as u64, c, mem::size_of::<i64>() * 8 - 1) as i64
    }

    fn ror(v: i64, c: usize) -> i64 {
        rol(v, (0 - c as isize) as usize)
    }

    fn count_lzb(bmp: i64) -> i32 {
        match bmp {
            0 => -1,
            x if x < 0 => 0,
            _ => count_lzb(bmp << 1) + 1,
        }
    }

    fn calc_lspc(al: usize, bmp: i64) -> i32 {
        if !al.is_power_of_two() {
            -2
        } else {
            count_lzb(
                !(ror(bmp | ror(bmp, 1) | ror(bmp, 2) | ror(bmp, 3), 5) | ror(bmp, 1))
                    & gen_alignmask(
                        al,
                        mem::size_of::<u64>() * 8,
                        1_u64 << (mem::size_of::<u64>() * 8 - 1),
                    ),
            )
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
        if al > 64 {
            al
        } else {
            __calc_algn(size, mem::size_of::<u64>() * 8)
        }
    }

    fn make_bitmap(align_req: &[AlignReq]) -> i64 {
        let mut bmp: i64 = 0;
        for req in align_req {
            if req.len > 63 {
                return -1;
            } else {
                bmp |= rol(((1_i64) << req.len) - 1, req.offset);
            }
        }
        bmp
    }

    mod libc {
        use core::ffi::c_void;
        type size_t = usize;
        extern "C" {
            pub fn malloc(size: size_t) -> *mut c_void;
            pub fn free(p: *mut c_void);
        }
    }
}
