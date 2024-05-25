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

use super::ecall::EntryTable;
use crate::arch::OCallContext;
use crate::tcs;
use core::convert::{From, Into, TryFrom};
use core::ffi::c_void;
use core::mem;
use core::num;
use core::ptr;
use sgx_types::error::{SgxResult, SgxStatus};

pub const OCALL_FLAG: usize = 0x4F434944;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OCallIndex {
    OCall(i32),
    Trim,
    TrimCommit,
    Modpr,
    Mprotect,
    Alloc,
    Modify,
}

impl OCallIndex {
    pub fn is_builtin_index(index: i32) -> bool {
        (-7..=-2).contains(&index)
    }

    pub fn is_builtin(&self) -> bool {
        !matches!(*self, OCallIndex::OCall(_))
    }

    pub fn is_ocall(&self) -> bool {
        match self {
            OCallIndex::OCall(n) => *n >= 0,
            _ => false,
        }
    }
}

impl TryFrom<i32> for OCallIndex {
    type Error = num::TryFromIntError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            v if v >= 0 => Ok(OCallIndex::OCall(v)),
            -2 => Ok(OCallIndex::Trim),
            -3 => Ok(OCallIndex::TrimCommit),
            -4 => Ok(OCallIndex::Modpr),
            -5 => Ok(OCallIndex::Mprotect),
            -6 => Ok(OCallIndex::Alloc),
            -7 => Ok(OCallIndex::Modify),
            _ => Err(u8::try_from(256_u16).unwrap_err()),
        }
    }
}

impl From<OCallIndex> for i32 {
    #[link_section = ".nipx"]
    fn from(idx: OCallIndex) -> i32 {
        match idx {
            OCallIndex::OCall(n) => n,
            OCallIndex::Trim => -2,
            OCallIndex::TrimCommit => -3,
            OCallIndex::Modpr => -4,
            OCallIndex::Mprotect => -5,
            OCallIndex::Alloc => -6,
            OCallIndex::Modify => -7,
        }
    }
}

pub fn ocall<T>(idx: OCallIndex, ms: Option<&mut T>) -> SgxResult {
    extern "C" {
        fn do_ocall(index: i32, ms: *mut c_void) -> u32;
    }

    let index = Into::<i32>::into(idx);
    if index > 0 && index as usize >= EntryTable::get().nr_ocall() {
        bail!(SgxStatus::InvalidFunction);
    }

    let ms = ms.map(|t| t as *mut _ as *mut _).unwrap_or(ptr::null_mut());
    let ret = unsafe { do_ocall(index, ms) };
    let status = SgxStatus::try_from(ret).unwrap_or(SgxStatus::Unexpected);
    if status.is_success() {
        Ok(())
    } else {
        Err(status)
    }
}

#[no_mangle]
pub unsafe extern "C" fn update_ocall_lastsp(context: &mut OCallContext) -> usize {
    let mut tc = tcs::current();
    let tds = tc.tds_mut();
    let last_sp = tds.last_sp;
    context.pre_last_sp = last_sp;

    if context.pre_last_sp == tds.stack_base {
        context.ocall_depth = 1;
    } else {
        // thread_data->last_sp is only set when ocall or exception handling occurs
        // ocall is block during exception handling, so last_sp is always ocall frame here
        let context_pre = &*(context.pre_last_sp as *const OCallContext);
        context.ocall_depth = context_pre.ocall_depth + 1;
    }
    tds.last_sp = context as *const _ as usize;

    last_sp
}

pub fn oret(ret: usize) -> SgxResult {
    extern "C" {
        fn asm_oret(sp: usize, ret: usize) -> u32;
    }

    let mut tc = tcs::current();
    let tds = tc.tds_mut();

    #[cfg(not(any(feature = "sim", feature = "hyper")))]
    if tds.aex_notify_flag == 1 {
        tds.aex_notify_flag = 0;
        let _ = crate::aexnotify::AEXNotify::set(true);
    }

    let last_sp = tds.last_sp;
    let context = unsafe { &*(tds.last_sp as *const OCallContext) };
    if last_sp == 0 || last_sp <= &context as *const _ as usize {
        bail!(SgxStatus::Unexpected);
    }

    // At least 1 ecall frame and 1 ocall frame are expected on stack.
    // 30 is an estimated value: 8 for enclave_entry and 22 for do_ocall.
    if last_sp > tds.stack_base - 30 * mem::size_of::<usize>() {
        bail!(SgxStatus::Unexpected);
    }
    if context.ocall_flag != OCALL_FLAG {
        bail!(SgxStatus::Unexpected);
    }

    if context.pre_last_sp > tds.stack_base || context.pre_last_sp <= context as *const _ as usize {
        bail!(SgxStatus::Unexpected);
    }

    tds.last_sp = context.pre_last_sp;

    unsafe { asm_oret(last_sp, ret) };

    // Should not come here
    Err(SgxStatus::Unexpected)
}
