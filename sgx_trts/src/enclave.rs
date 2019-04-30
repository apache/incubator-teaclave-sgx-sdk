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

//! Get the metadata of the current enclave.
//!
//! This mod has clear interface and is easy to understand. Currently we don't
//! have time for its documents.

use sgx_types::*;
use sgx_types::metadata::*;

pub const LAYOUT_ENTRY_NUM : usize = 38;

#[link(name = "sgx_trts")]
extern {
    static g_global_data: global_data_t;
    pub fn get_thread_data() -> * const c_void;
    pub fn get_enclave_base() -> * const c_void;
    pub fn get_heap_base() -> * const c_void;
    pub fn get_heap_size() -> size_t;
}

#[repr(C)]
struct global_data_t {
    enclave_size: usize,
    heap_offset: usize,
    heap_size: usize,
    thread_policy: usize,
    td_template : thread_data_t,
    tcs_template: [u8;TCS_TEMPLATE_SIZE], // 72
    layout_entry_num : u32,
    reserved : u32,
    layout_table : [layout_t;LAYOUT_ENTRY_NUM],
}

#[repr(C)]
struct thread_data_t {
    self_addr: usize,
    last_sp: usize,
    stack_base_addr: usize,
    stack_limit_addr: usize,
    first_ssa_gpr: usize,
    stack_guard: usize,
    flags: usize,
    xsave_size: usize,
    last_error: usize,
    m_next: usize,
    tls_addr: usize,
    tls_array: usize,
    exception_flag: usize,
    cxx_thread_info: [usize; 6],
    stack_commit_addr: usize,
}

#[derive(Copy, Clone)]
pub struct SgxGlobalData {
    enclave_base: usize,
    enclave_size: usize,
    heap_base: usize,
    heap_offset: usize,
    heap_size: usize,
    thread_policy: SgxThreadPolicy,
}

impl Default for SgxGlobalData {
    fn default() -> Self {
        Self::new()
    }
}

impl SgxGlobalData {

    ///
    /// get global_data.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn new() -> Self {
        SgxGlobalData {
           enclave_base: rsgx_get_enclave_base() as usize,
           enclave_size: rsgx_get_enclave_size(),
           heap_base: rsgx_get_heap_base() as usize,
           heap_offset: rsgx_get_heap_offset(),
           heap_size: rsgx_get_heap_size(),
           thread_policy: rsgx_get_thread_policy(),
        }
    }

    ///
    /// enclave_base is to get enclave map base address.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn enclave_base(&self) -> usize {
        self.enclave_base
    }
    ///
    /// enclave_size is to get enclave map size.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn enclave_size(&self) -> usize {
        self.enclave_size
    }
    ///
    /// heap_base is to get heap base address.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn heap_base(&self) -> usize {
        self.heap_base
    }
    ///
    /// heap_offset is to get heap offset.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn heap_offset(&self) -> usize {
        self.heap_offset
    }
    ///
    /// heap_size is to get heap size.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn heap_size(&self) -> usize {
        self.heap_size
    }
    ///
    /// thread_policy is to get TCS policy.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn thread_policy(&self) -> SgxThreadPolicy {
        self.thread_policy
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Default)]
pub struct SgxThreadData {
    td_addr: usize,
    last_sp: usize,
    stack_base_addr: usize,
    stack_limit_addr: usize,
    first_ssa_gpr: usize,
    stack_guard: usize,
    xsave_size: usize,
    last_error: usize,
    tls_addr: usize,
    tls_array: usize,
    exception_flag: usize,
    cxx_thread_info: [usize; 6],
}

impl SgxThreadData {

    ///
    /// get thread_data per thread.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    #[allow(clippy::cast_ptr_alignment)]
    pub fn new() -> Self {
        let td = unsafe {
            let p = rsgx_get_thread_data() as * const thread_data_t;
            &*p
        };
        SgxThreadData {
            td_addr: td.self_addr,
            last_sp: td.last_sp,
            stack_base_addr: td.stack_base_addr,
            stack_limit_addr: td.stack_limit_addr,
            first_ssa_gpr: td.first_ssa_gpr,
            stack_guard: td.stack_guard,
            xsave_size: td.xsave_size,
            last_error: td.last_error,
            tls_addr: td.tls_addr,
            tls_array: td.tls_array,
            exception_flag: td.exception_flag,
            cxx_thread_info: td.cxx_thread_info,
        }
    }

    pub unsafe fn from_raw(raw: usize) -> Self {
        let p = raw as * const thread_data_t;
        let td = &*p;
        SgxThreadData {
            td_addr: td.self_addr,
            last_sp: td.last_sp,
            stack_base_addr: td.stack_base_addr,
            stack_limit_addr: td.stack_limit_addr,
            first_ssa_gpr: td.first_ssa_gpr,
            stack_guard: td.stack_guard,
            xsave_size: td.xsave_size,
            last_error: td.last_error,
            tls_addr: td.tls_addr,
            tls_array: td.tls_array,
            exception_flag: td.exception_flag,
            cxx_thread_info: td.cxx_thread_info,
        }
    }

    ///
    /// td_base is to get TD base address per thread.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn td_base(&self) -> usize {
        self.td_addr
    }
    ///
    /// stack_base is to get stack base address per thread.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn stack_base(&self) -> usize {
        self.stack_base_addr
    }
    ///
    /// stack_limit is to get stack limit per thread.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn stack_limit(&self) -> usize {
        self.stack_limit_addr
    }
    ///
    /// tls_base is to get tls base address per thread.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn tls_base(&self) -> usize {
        self.tls_addr
    }
    ///
    /// last_error is to get last error per thread.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn last_error(&self) -> usize {
        self.last_error
    }
    ///
    /// exception_flag is to get exception flag per thread.
    ///
    /// **Note**
    ///
    /// This API is only an experimental funtion.
    ///
    pub fn exception_flag(&self) -> usize {
        self.exception_flag
    }

    pub fn get_tcs(&self) -> usize {
        self.stack_base() + STATIC_STACK_SIZE + SE_GUARD_PAGE_SIZE
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SgxThreadPolicy {
    Bound,
    Unbound
}

///
/// rsgx_get_thread_data is to get TD base address per thread.
///
/// **Note**
///
/// This API is only an experimental funtion.
///
#[inline]
pub fn rsgx_get_thread_data() -> * const u8 {

    unsafe { get_thread_data() as * const u8 }
}

///
/// rsgx_get_enclave_base is to get enclave image base address.
///
/// **Note**
///
/// This API is only an experimental funtion.
///
#[inline]
pub fn rsgx_get_enclave_base() -> * const u8 {

    unsafe { get_enclave_base() as * const u8 }
}

///
/// rsgx_get_enclave_size is to get enclave image size.
///
/// **Note**
///
/// This API is only an experimental funtion.
///
#[inline]
pub fn rsgx_get_enclave_size() -> usize {

    unsafe{ g_global_data.enclave_size }
}

///
/// rsgx_get_heap_base is to get enclave heap base address.
///
/// **Note**
///
/// This API is only an experimental funtion.
///
#[inline]
pub fn rsgx_get_heap_base() -> * const u8 {

    unsafe { get_heap_base() as * const u8 }
}

///
/// rsgx_get_heap_offset is to get enclave heap offset.
///
/// **Note**
///
/// This API is only an experimental funtion.
///
#[inline]
pub fn rsgx_get_heap_offset() -> usize {

    unsafe{ g_global_data.heap_offset }
}

///
/// rsgx_get_heap_size is to get enclave heap size.
///
/// **Note**
///
/// This API is only an experimental funtion.
///
#[inline]
pub fn rsgx_get_heap_size() -> usize {

    unsafe { get_heap_size() }
}

///
/// rsgx_get_thread_policy is to get TCS management policy.
///
/// **Note**
///
/// This API is only an experimental funtion.
///
#[inline]
pub fn rsgx_get_thread_policy() -> SgxThreadPolicy {

    unsafe {
        if g_global_data.thread_policy != 0 {
            SgxThreadPolicy::Unbound
        } else {
            SgxThreadPolicy::Bound
        }
    }
}
