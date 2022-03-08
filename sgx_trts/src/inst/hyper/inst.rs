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

use crate::arch::Secinfo;
use crate::se::{AlignKey, AlignKeyRequest, AlignReport, AlignReportData, AlignTargetInfo};
use core::arch::asm;
use core::mem::MaybeUninit;

impl_enum! {
    #[repr(u64)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum HyperCall {
        EEnter          = 0x80000000,
        EExit           = 0x80000001,
        EResume         = 0x80000005,
        EVerifyReport   = 0x8000000A,
        EGetkey         = 0x8000000B,
        EReport         = 0x8000000C,
        EQuote          = 0x8000000D,
    }
}

pub struct EncluInst;

impl EncluInst {
    #[link_section = ".nipx"]
    pub fn eexit(dest: usize) {
        unsafe {
            asm!("
                mov rbx, {dest}
                vmmcall
                ud2",
                dest = in(reg) dest,
                in("rax") HyperCall::EExit as u64,
                lateout("rcx") _,
            );
        }
    }

    pub fn ereport(ti: &AlignTargetInfo, rd: &AlignReportData) -> Result<AlignReport, u32> {
        unsafe {
            let mut report = MaybeUninit::uninit();
            asm!("
                push rbx
                push rcx
                push rdx

                mov rbx, {ti}
                mov rcx, {rd}
                mov rdx, {report_ptr}
                vmmcall

                pop rdx
                pop rcx
                pop rbx",
                ti = in(reg) ti,
                rd = in(reg) rd,
                report_ptr = in(reg) report.as_mut_ptr(),
                in("rax") HyperCall::EReport as u64,
            );
            Ok(report.assume_init())
        }
    }

    pub fn egetkey(kr: &AlignKeyRequest) -> Result<AlignKey, u32> {
        unsafe {
            let mut key = MaybeUninit::uninit();
            asm!("
                push rbx
                push rcx

                mov rbx, {kr}
                mov rcx, {key_ptr}
                vmmcall

                pop rcx
                pop rbx",
                kr = in(reg) kr,
                key_ptr = in(reg) key.as_mut_ptr(),
                in("rax") HyperCall::EGetkey as u64,
            );
            Ok(key.assume_init())
        }
    }

    #[inline]
    pub fn eaccept(_addr: usize, _info: &Secinfo) -> Result<(), u32> {
        Ok(())
    }

    #[inline]
    pub fn emodpe(_addr: usize, _info: &Secinfo) -> Result<(), u32> {
        Ok(())
    }
}
