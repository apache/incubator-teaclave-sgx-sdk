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

use crate::arch::{Enclu, Secinfo};
use crate::se::{
    AlignKey, AlignKeyRequest, AlignReport, AlignReport2Mac, AlignReportData, AlignTargetInfo,
};
use core::arch::asm;
use core::mem::MaybeUninit;

pub struct EncluInst;

impl EncluInst {
    pub fn ereport(ti: &AlignTargetInfo, rd: &AlignReportData) -> Result<AlignReport, u32> {
        unsafe {
            let mut report = MaybeUninit::uninit();
            asm!(
                "xchg rbx, {0}",
                "enclu",
                "mov rbx, {0}",
                inout(reg) ti => _,
                in("eax") Enclu::EReport as u32,
                in("rcx") rd,
                in("rdx") report.as_mut_ptr(),
                options(preserves_flags, nostack),
            );
            Ok(report.assume_init())
        }
    }

    pub fn everify_report2(r: &AlignReport2Mac) -> Result<(), u32> {
        extern "C" {
            fn everifyreport2(r: *const AlignReport2Mac) -> u32;
        }
        let error = unsafe { everifyreport2(r) };
        if error == 0 {
            Ok(())
        } else {
            Err(error)
        }
    }

    pub fn egetkey(kr: &AlignKeyRequest) -> Result<AlignKey, u32> {
        unsafe {
            let mut key = MaybeUninit::uninit();
            let error;
            asm!(
                "xchg rbx, {0}",
                "enclu",
                "mov rbx, {0}",
                inout(reg) kr => _,
                inlateout("eax") Enclu::EGetkey as u32 => error,
                in("rcx") key.as_mut_ptr(),
                options(nostack),
            );
            if error == 0 {
                Ok(key.assume_init())
            } else {
                Err(error)
            }
        }
    }

    pub fn eaccept(info: &Secinfo, addr: usize) -> Result<(), u32> {
        unsafe {
            let error;
            asm!(
                "xchg rbx, {0}",
                "enclu",
                "mov rbx, {0}",
                inout(reg) info => _,
                inlateout("eax") Enclu::EAccept as u32 => error,
                in("rcx") addr,
                options(nostack),
            );
            match error {
                0 => Ok(()),
                _ => Err(error),
            }
        }
    }

    pub fn eacceptcopy(info: &Secinfo, addr: usize, source: usize) -> Result<(), u32> {
        unsafe {
            let error;
            asm!(
                "xchg rbx, {0}",
                "enclu",
                "mov rbx, {0}",
                inout(reg) info => _,
                inlateout("eax") Enclu::EAccept as u32 => error,
                in("rcx") addr,
                in("rdx") source,
                options(nostack),
            );
            match error {
                0 => Ok(()),
                _ => Err(error),
            }
        }
    }

    pub fn emodpe(info: &Secinfo, addr: usize) -> Result<(), u32> {
        unsafe {
            asm!(
                "xchg rbx, {0}",
                "enclu",
                "mov rbx, {0}",
                inout(reg) info => _,
                in("eax") Enclu::EModpe as u32,
                in("rcx") addr,
                options(preserves_flags, nostack),
            );
            Ok(())
        }
    }
}
