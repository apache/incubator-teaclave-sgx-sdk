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

use crate::enclave::is_within_enclave;
use crate::error;
use crate::sync::SpinMutex;
use crate::veh::list;
use core::num::NonZeroU64;
use core::slice;
use sgx_types::error::{SgxResult, SgxStatus};

pub const MAX_REGISTER_COUNT: usize = 64;

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ExceptionVector {
        DE = 0,  /* DIV and DIV instructions */
        DB = 1,  /* For Intel use only */
        BP = 3,  /* INT 3 instruction */
        BR = 5,  /* BOUND instruction */
        UD = 6,  /* UD2 instruction or reserved opcode */
        GP = 13, /* General protection exception */
        PF = 14, /* Page fault exception */
        MF = 16, /* x87 FPU floating-point or WAIT/FWAIT instruction */
        AC = 17, /* Any data reference in memory */
        XM = 19, /* SSE/SSE2/SSE3 floating-point instruction */
        CP = 21, /* Control protection exception */
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ExceptionType {
        Hardware = 3,
        Software = 6,
    }
}

impl_struct! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct CpuContext {
        pub rax: u64,
        pub rcx: u64,
        pub rdx: u64,
        pub rbx: u64,
        pub rsp: u64,
        pub rbp: u64,
        pub rsi: u64,
        pub rdi: u64,
        pub r8: u64,
        pub r9: u64,
        pub r10: u64,
        pub r11: u64,
        pub r12: u64,
        pub r13: u64,
        pub r14: u64,
        pub r15: u64,
        pub rflags: u64,
        pub rip: u64,
    }
}

impl_struct! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct MiscExInfo {
        pub faulting_address: u64,
        pub error_code: u32,
        pub reserved: u32,
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ExceptionInfo {
    pub context: CpuContext,
    pub vector: ExceptionVector,
    pub exception_type: ExceptionType,
    pub exinfo: MiscExInfo,
    pub exception_valid: u32,
    pub do_aex_mitigation: u32,
    pub xsave_size: u64,
    pub reserved: [u64; 1],
    pub xsave_area: [u8; 0],
}

impl ExceptionInfo {
    #[inline]
    pub fn xsave_area(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(&self.xsave_area as *const u8, self.xsave_size as usize) }
    }

    #[inline]
    pub fn xsave_area_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(&mut self.xsave_area as *mut u8, self.xsave_size as usize)
        }
    }
}

impl_struct_ContiguousMemory! {
    ExceptionInfo;
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum HandleResult {
        Search = 0,
        Execution = 0xFFFFFFFF,
    }
}

pub type ExceptionHandler = extern "C" fn(info: &mut ExceptionInfo) -> HandleResult;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Handle(NonZeroU64);

impl Handle {
    pub(crate) fn new() -> Handle {
        static GUARD: SpinMutex<u64> = SpinMutex::new(1);

        let mut counter = GUARD.lock();
        if *counter == u64::MAX {
            error::abort();
        }
        let id = *counter;
        *counter += 1;
        Handle(NonZeroU64::new(id).unwrap())
    }

    #[inline]
    pub(crate) fn into_raw(self) -> u64 {
        self.0.get()
    }

    #[inline]
    pub(crate) unsafe fn from_raw(id: u64) -> Handle {
        Handle(NonZeroU64::new_unchecked(id))
    }
}

pub fn register_exception(first: bool, handler: ExceptionHandler) -> SgxResult<Handle> {
    ensure!(
        is_within_enclave(handler as *const u8, 0),
        SgxStatus::InvalidParameter
    );

    let mut list_guard = list::EXCEPTION_LIST.lock();
    ensure!(
        list_guard.len() < MAX_REGISTER_COUNT,
        SgxStatus::OutOfMemory
    );

    if first {
        Ok(list_guard.push_front(handler))
    } else {
        Ok(list_guard.push_back(handler))
    }
}

#[inline]
pub fn register(handler: ExceptionHandler) -> SgxResult<Handle> {
    register_exception(false, handler)
}

pub fn unregister(id: Handle) -> bool {
    list::EXCEPTION_LIST.lock().remove(id).is_some()
}
