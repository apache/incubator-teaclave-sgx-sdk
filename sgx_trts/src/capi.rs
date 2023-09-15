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

use crate::arch::SE_PAGE_SIZE;
use crate::call::{ocall, OCallIndex, OcBuffer};
use crate::emm::alloc::Alloc;
use crate::emm::page::AllocFlags;
use crate::emm::pfhandler::{PfHandler, PfInfo};
use crate::emm::range::{
    RangeType, ALLIGNMENT_MASK, ALLIGNMENT_SHIFT, ALLOC_FLAGS_MASK, ALLOC_FLAGS_SHIFT,
    PAGE_TYPE_MASK, PAGE_TYPE_SHIFT, RM,
};
use crate::emm::{apply_epc_pages, trim_epc_pages, PageInfo, PageType, ProtFlags};
use crate::enclave::{self, is_within_enclave, MmLayout};
use crate::error;
use crate::rand::rand;
use crate::tcs::{current, stack_size, tcs_max_num, tcs_policy};
use crate::trts::{cpu_core_num, enclave_mode, is_supported_edmm};
use crate::veh::{register_exception, unregister, ExceptionHandler, Handle};
use core::convert::TryFrom;
use core::ffi::c_void;
use core::num::NonZeroUsize;
use core::slice;
use core::{mem, ptr};
use sgx_types::error::SgxStatus;

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_get_enclave_mode() -> i32 {
    enclave_mode() as i32
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_register_exception_handler(
    first: i32,
    handler: ExceptionHandler,
) -> *const c_void {
    match register_exception(first != 0, handler) {
        Ok(handle) => handle.into_raw() as *const c_void,
        Err(_) => ptr::null(),
    }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_unregister_exception_handler(handle: *const c_void) -> i32 {
    let handle = Handle::from_raw(handle as u64);
    let result = unregister(handle);
    i32::from(result)
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_image_base() -> *const u8 {
    MmLayout::image_base() as *const u8
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_image_size() -> usize {
    MmLayout::image_size()
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_elrange_base() -> *const u8 {
    MmLayout::elrange_base() as *const u8
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_elrange_size() -> usize {
    MmLayout::elrange_size()
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_heap_base() -> *const u8 {
    MmLayout::heap_base() as *const u8
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_heap_size() -> usize {
    MmLayout::heap_size()
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_heap_min_size() -> usize {
    MmLayout::heap_min_size()
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_rsrvmem_base() -> *const u8 {
    MmLayout::rsrvmem_base() as *const u8
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_rsrvmem_size() -> usize {
    MmLayout::rsrvmem_size()
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_rsrvmem_min_size() -> usize {
    MmLayout::rsrvmem_min_size()
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_rsrvmm_default_perm() -> u32 {
    MmLayout::rsrvmm_default_perm() as u32
}

#[link_section = ".nipx"]
#[no_mangle]
pub extern "C" fn sgx_is_enclave_crashed() -> i32 {
    i32::from(enclave::state::is_crashed())
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_tcs_max_num() -> usize {
    tcs_max_num()
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_tcs_policy() -> u32 {
    tcs_policy() as u32
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_stack_size() -> usize {
    stack_size()
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_get_cpu_core_num() -> u32 {
    cpu_core_num()
}

#[inline]
#[no_mangle]
pub extern "C" fn sgx_is_supported_edmm() -> bool {
    is_supported_edmm()
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_is_within_enclave(p: *const u8, len: usize) -> i32 {
    i32::from(enclave::is_within_enclave(p, len))
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_is_outside_enclave(p: *const u8, len: usize) -> i32 {
    i32::from(enclave::is_within_host(p, len))
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_apply_epc_pages(addr: usize, count: usize) -> i32 {
    if apply_epc_pages(addr, count).is_ok() {
        0
    } else {
        -1
    }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_trim_epc_pages(addr: usize, count: usize) -> i32 {
    if trim_epc_pages(addr, count).is_ok() {
        0
    } else {
        -1
    }
}

// TODO: replace inarguments with "C" style arguments
#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_mm_alloc(
    addr: usize,
    size: usize,
    flags: usize,
    handler: *mut c_void,
    priv_data: *mut PfInfo,
    out_addr: *mut *mut u8,
) -> u32 {
    let handler = if handler.is_null() {
        None
    } else {
        Some(mem::transmute::<*mut c_void, PfHandler>(handler))
    };

    let alloc_flags =
        match AllocFlags::from_bits(((flags & ALLOC_FLAGS_MASK) >> ALLOC_FLAGS_SHIFT) as u32) {
            Some(flags) => flags,
            None => {
                return SgxStatus::InvalidParameter.into();
            }
        };

    let mut page_type =
        match PageType::try_from(((flags & PAGE_TYPE_MASK) >> PAGE_TYPE_SHIFT) as u8) {
            Ok(typ) => typ,
            Err(_) => return SgxStatus::InvalidParameter.into(),
        };

    if page_type == PageType::None {
        page_type = PageType::Reg;
    }

    if (size % SE_PAGE_SIZE) > 0 {
        return SgxStatus::InvalidParameter.into();
    }

    let mut align_flag: u8 = ((flags & ALLIGNMENT_MASK) >> ALLIGNMENT_SHIFT) as u8;
    if align_flag == 0 {
        align_flag = 12;
    }
    if align_flag < 12 {
        return SgxStatus::InvalidParameter.into();
    }
    let align_mask: usize = (1 << align_flag) - 1;

    if (addr & align_mask) > 0 {
        return SgxStatus::InvalidParameter.into();
    }

    if (addr > 0) && !is_within_enclave(addr as *const u8, size) {
        return SgxStatus::InvalidParameter.into();
    }

    let info = if alloc_flags.contains(AllocFlags::RESERVED) {
        PageInfo {
            prot: ProtFlags::NONE,
            typ: PageType::None,
        }
    } else {
        PageInfo {
            prot: ProtFlags::R | ProtFlags::W,
            typ: page_type,
        }
    };

    let priv_data = if priv_data.is_null() {
        None
    } else {
        Some(priv_data)
    };

    let mut range_manage = RM.get().unwrap().lock();
    match range_manage.alloc(
        Some(addr),
        size,
        alloc_flags,
        info,
        handler,
        priv_data,
        RangeType::User,
        Alloc::Reserve,
    ) {
        Ok(base) => {
            *out_addr = base as *mut u8;
            0
        }
        Err(err) => err.into(),
    }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_ocall(idx: i32, ms: *mut c_void) -> u32 {
    if let Ok(index) = OCallIndex::try_from(idx) {
        let ms = if !ms.is_null() { Some(&mut *ms) } else { None };
        match ocall(index, ms) {
            Ok(_) => SgxStatus::Success.into(),
            Err(e) => e.into(),
        }
    } else {
        SgxStatus::InvalidFunction.into()
    }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_ocalloc(size: usize) -> *mut u8 {
    if let Some(size) = NonZeroUsize::new(size) {
        OcBuffer::alloc(size)
            .map(|b| OcBuffer::into_raw(b).cast())
            .unwrap_or(ptr::null_mut())
    } else {
        ptr::null_mut()
    }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_ocalloc_aligned(align: usize, size: usize) -> *mut u8 {
    let size = match NonZeroUsize::new(size) {
        Some(size) => size,
        None => return ptr::null_mut(),
    };
    let align = match NonZeroUsize::new(align) {
        Some(align) => align,
        None => return ptr::null_mut(),
    };
    OcBuffer::alloc_aligned(size, align)
        .map(|b| OcBuffer::into_raw(b).cast())
        .unwrap_or(ptr::null_mut())
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_ocfree() {
    if OcBuffer::free().is_err() {
        error::abort();
    }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_ocremain_size() -> usize {
    OcBuffer::remain_size()
}

#[allow(clippy::redundant_closure)]
#[inline]
#[no_mangle]
pub unsafe extern "C" fn atexit(f: extern "C" fn()) -> i32 {
    if !enclave::is_within_enclave(f as *const u8, 0) {
        return -1;
    }

    let func = move || f();
    if enclave::at_exit(func) {
        0
    } else {
        -1
    }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_read_rand(p: *mut u8, len: usize) -> u32 {
    if p.is_null() || len == 0 {
        return SgxStatus::InvalidParameter.into();
    }

    let buf = slice::from_raw_parts_mut(p, len);
    match rand(buf) {
        Ok(_) => SgxStatus::Success.into(),
        Err(e) => e.into(),
    }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn get_thread_data() -> *const c_void {
    current().tds() as *const _ as *const c_void
}

pub type sgx_thread_t = *const c_void;
pub const SGX_THREAD_T_NULL: *const c_void = ptr::null();

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_thread_self() -> sgx_thread_t {
    get_thread_data()
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_thread_equal(a: sgx_thread_t, b: sgx_thread_t) -> i32 {
    i32::from(a == b)
}

#[cfg(not(feature = "hyper"))]
#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_rdpkru(val: *mut u32) -> i32 {
    if val.is_null() {
        return 0;
    }

    match crate::pkru::Pkru::read() {
        Ok(pkru) => {
            *val = pkru;
            1
        }
        Err(_) => 0,
    }
}

#[cfg(not(feature = "hyper"))]
#[inline]
#[no_mangle]
pub unsafe extern "C" fn sgx_wrpkru(val: u32) -> i32 {
    match crate::pkru::Pkru::write(val) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}
