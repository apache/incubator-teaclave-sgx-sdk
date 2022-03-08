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
use crate::se::{AlignKey, AlignKeyRequest, AlignReport, AlignReportData, AlignTargetInfo};
use core::arch::asm;
use core::mem::MaybeUninit;

pub struct EncluInst;

impl EncluInst {
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
                enclu

                pop rdx
                pop rcx
                pop rbx",
                ti = in(reg) ti,
                rd = in(reg) rd,
                report_ptr = in(reg) report.as_mut_ptr(),
                in("eax") Enclu::EReport as u32,
            );
            Ok(report.assume_init())
        }
    }

    pub fn egetkey(kr: &AlignKeyRequest) -> Result<AlignKey, u32> {
        unsafe {
            let mut key = MaybeUninit::uninit();
            let error;
            asm!("
                push rbx
                push rcx

                mov rbx, {kr}
                mov rcx, {key_ptr}
                enclu

                pop rcx
                pop rbx",
                kr = in(reg) kr,
                key_ptr = in(reg) key.as_mut_ptr(),
                inlateout("eax") Enclu::EGetkey as u32 => error,
            );
            if error == 0 {
                Ok(key.assume_init())
            } else {
                Err(error)
            }
        }
    }

    pub fn eaccept(addr: usize, info: &Secinfo) -> Result<(), u32> {
        unsafe {
            let error;
            asm!("
                push rbx
                push rcx

                mov rbx, {info}
                mov rcx, {addr}
                enclu

                pop rcx
                pop rbx",
                info = in(reg) info,
                addr = in(reg) addr,
                inlateout("eax") Enclu::EAccept as u32 => error,
            );
            match error {
                0 => Ok(()),
                _ => Err(error),
            }
        }
    }

    pub fn emodpe(addr: usize, info: &Secinfo) -> Result<(), u32> {
        unsafe {
            asm!("
                push rbx
                push rcx

                mov rbx, {info}
                mov rcx, {addr}
                enclu

                pop rcx
                pop rbx",
                info = in(reg) info,
                addr = in(reg) addr,
                in("eax") Enclu::EModpe as u32,
            );
            Ok(())
        }
    }
}
