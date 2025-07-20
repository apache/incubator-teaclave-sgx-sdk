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
// under the License.

use crate::rsrvmm::manager::MmAllocAddr;
use crate::rsrvmm::RsrvMem;
use core::convert::TryFrom;
use core::ffi::c_void;
use core::ptr;
use sgx_trts::error::set_errno;
use sgx_types::error::errno::*;
use sgx_types::error::SgxStatus;
use sgx_types::types::ProtectPerm;

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_get_rsrv_mem_info(
    start_addr: *mut *const c_void,
    max_size: *mut usize,
) -> SgxStatus {
    if start_addr.is_null() && max_size.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let (base, size) = if let Ok(rsrvmem) = RsrvMem::get_or_init() {
        (rsrvmem.base(), rsrvmem.size())
    } else {
        (0, 0)
    };

    if !start_addr.is_null() {
        *start_addr = base as *const c_void;
    }

    if !max_size.is_null() {
        *max_size = size;
    }
    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_alloc_rsrv_mem_ex(
    desired_addr: *const c_void,
    length: usize,
) -> *mut c_void {
    let rsrvmem = match RsrvMem::get_or_init() {
        Ok(m) => m,
        Err(e) => {
            set_errno(e);
            return ptr::null_mut();
        }
    };

    let desired_addr = if desired_addr.is_null() {
        MmAllocAddr::Any
    } else {
        MmAllocAddr::Need(desired_addr as usize)
    };

    rsrvmem
        .mmap::<()>(desired_addr, length, None, None)
        .unwrap_or_else(|e| {
            set_errno(e);
            0
        }) as *mut c_void
}

/// # Safety
#[no_mangle]
#[inline]
pub unsafe extern "C" fn sgx_alloc_rsrv_mem(length: usize) -> *mut c_void {
    sgx_alloc_rsrv_mem_ex(ptr::null(), length)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_free_rsrv_mem(addr: *const c_void, length: usize) -> i32 {
    let rsrvmem = match RsrvMem::get_or_init() {
        Ok(m) => m,
        Err(e) => {
            set_errno(e);
            return e;
        }
    };

    rsrvmem
        .munmap(addr as usize, length)
        .map(|_| 0)
        .unwrap_or_else(|e| {
            set_errno(e);
            e
        })
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_tprotect_rsrv_mem(
    addr: *const c_void,
    length: usize,
    prot: i32,
) -> SgxStatus {
    let rsrvmem = match RsrvMem::get_or_init() {
        Ok(m) => m,
        Err(_) => {
            return SgxStatus::InvalidParameter;
        }
    };

    let perm = match ProtectPerm::try_from(prot as u8) {
        Ok(p) => p,
        Err(_) => return SgxStatus::InvalidParameter,
    };

    match rsrvmem.mprotect(addr as usize, length, perm) {
        Ok(_) => SgxStatus::Success,
        Err(e) => match e {
            EINVAL => SgxStatus::InvalidParameter,
            ENOMEM => SgxStatus::OutOfMemory,
            _ => SgxStatus::Unexpected,
        },
    }
}
