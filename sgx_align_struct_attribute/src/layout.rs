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

use std::alloc::Layout;
use std::ffi::c_void;
use std::fmt;
use std::mem;
use std::ptr;
use std::slice;

#[derive(Clone, Copy, Default)]
pub struct AlignReq {
    pub offset: usize,
    pub len: usize,
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

#[allow(clippy::overflow_check_conditional)]
#[inline]
fn check_overflow(buf: usize, len: usize) -> bool {
    (buf + len < len) || (buf + len < buf)
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

#[allow(clippy::comparison_chain)]
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
    use std::ffi::c_void;
    extern "C" {
        pub fn malloc(size: usize) -> *mut c_void;
        pub fn free(p: *mut c_void);
    }
}
