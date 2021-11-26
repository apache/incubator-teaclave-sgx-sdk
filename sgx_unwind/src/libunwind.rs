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

#![allow(nonstandard_style)]

use core::ffi::c_void;
type c_int = i32;
type uintptr_t = usize;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum _Unwind_Reason_Code {
    _URC_NO_REASON = 0,
    _URC_FOREIGN_EXCEPTION_CAUGHT = 1,
    _URC_FATAL_PHASE2_ERROR = 2,
    _URC_FATAL_PHASE1_ERROR = 3,
    _URC_NORMAL_STOP = 4,
    _URC_END_OF_STACK = 5,
    _URC_HANDLER_FOUND = 6,
    _URC_INSTALL_CONTEXT = 7,
    _URC_CONTINUE_UNWIND = 8,
    _URC_FAILURE = 9, // used only by ARM EHABI
}
pub use _Unwind_Reason_Code::*;

pub type _Unwind_Exception_Class = u64;
pub type _Unwind_Word = uintptr_t;
pub type _Unwind_Ptr = uintptr_t;
pub type _Unwind_Trace_Fn =
    extern "C" fn(ctx: *mut _Unwind_Context, arg: *mut c_void) -> _Unwind_Reason_Code;

#[cfg(target_arch = "x86")]
pub const unwinder_private_data_size: usize = 5;

#[cfg(target_arch = "x86_64")]
pub const unwinder_private_data_size: usize = 6;

#[repr(C)]
pub struct _Unwind_Exception {
    pub exception_class: _Unwind_Exception_Class,
    pub exception_cleanup: _Unwind_Exception_Cleanup_Fn,
    pub private: [_Unwind_Word; unwinder_private_data_size],
}

pub enum _Unwind_Context {}

pub type _Unwind_Exception_Cleanup_Fn =
    extern "C" fn(unwind_code: _Unwind_Reason_Code, exception: *mut _Unwind_Exception);

extern "C-unwind" {
    pub fn _Unwind_Resume(exception: *mut _Unwind_Exception) -> !;
}
extern "C" {
    pub fn _Unwind_DeleteException(exception: *mut _Unwind_Exception);
    pub fn _Unwind_GetLanguageSpecificData(ctx: *mut _Unwind_Context) -> *mut c_void;
    pub fn _Unwind_GetRegionStart(ctx: *mut _Unwind_Context) -> _Unwind_Ptr;
    pub fn _Unwind_GetTextRelBase(ctx: *mut _Unwind_Context) -> _Unwind_Ptr;
    pub fn _Unwind_GetDataRelBase(ctx: *mut _Unwind_Context) -> _Unwind_Ptr;
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum _Unwind_Action {
    _UA_SEARCH_PHASE = 1,
    _UA_CLEANUP_PHASE = 2,
    _UA_HANDLER_FRAME = 4,
    _UA_FORCE_UNWIND = 8,
    _UA_END_OF_STACK = 16,
}
pub use _Unwind_Action::*;

extern "C" {
    pub fn _Unwind_GetGR(ctx: *mut _Unwind_Context, reg_index: c_int) -> _Unwind_Word;
    pub fn _Unwind_SetGR(ctx: *mut _Unwind_Context, reg_index: c_int, value: _Unwind_Word);
    pub fn _Unwind_GetIP(ctx: *mut _Unwind_Context) -> _Unwind_Word;
    pub fn _Unwind_SetIP(ctx: *mut _Unwind_Context, value: _Unwind_Word);
    pub fn _Unwind_GetIPInfo(ctx: *mut _Unwind_Context, ip_before_insn: *mut c_int)
        -> _Unwind_Word;
    pub fn _Unwind_FindEnclosingFunction(pc: *mut c_void) -> *mut c_void;
    pub fn _Unwind_GetCFA(ctx: *mut _Unwind_Context) -> _Unwind_Ptr;
}

extern "C-unwind" {
    pub fn _Unwind_RaiseException(exception: *mut _Unwind_Exception) -> _Unwind_Reason_Code;
}
extern "C" {
    pub fn _Unwind_Backtrace(
        trace: _Unwind_Trace_Fn,
        trace_argument: *mut c_void,
    ) -> _Unwind_Reason_Code;
}
