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
use crate::call::MsbufInfo;
use crate::error::abort;
use crate::fence::lfence;
use crate::se::{AlignKey, AlignKeyRequest, AlignReport, AlignReportData, AlignTargetInfo};
use core::convert::TryFrom;
use inst::EncluInst;
use sgx_types::error::SgxResult;

pub mod inst;

#[link_section = ".nipx"]
#[no_mangle]
pub unsafe extern "C" fn se3(
    rax: usize,
    rbx: usize,
    rcx: usize,
    rdx: usize,
    _rsi: usize,
    _rdi: usize,
) -> usize {
    let enclu = match Enclu::try_from(rax as u32) {
        Ok(e) => e,
        Err(_) => abort(),
    };

    match enclu {
        Enclu::EExit => {
            EncluInst::eexit(rbx);
            0
        }
        Enclu::EGetkey => match EncluInst::egetkey(&*(rbx as *const AlignKeyRequest)) {
            Ok(key) => {
                *(rcx as *mut AlignKey) = key;
                0
            }
            Err(e) => e as usize,
        },
        Enclu::EReport => match EncluInst::ereport(
            &*(rbx as *const AlignTargetInfo),
            &*(rcx as *const AlignReportData),
        ) {
            Ok(r) => {
                *(rdx as *mut AlignReport) = r;
                0
            }
            Err(e) => e as usize,
        },
        Enclu::EAccept => match EncluInst::eaccept(rbx, &*(rcx as *const Secinfo)) {
            Ok(_) => 0,
            Err(e) => e as usize,
        },
        Enclu::EModpe => match EncluInst::emodpe(rbx, &*(rcx as *const Secinfo)) {
            Ok(_) => 0,
            Err(e) => e as usize,
        },
        _ => abort(),
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct GlobalHyper {
    msbuf_info: MsbufInfo,
}

#[no_mangle]
pub static mut g_global_data_hyper: GlobalHyper = GlobalHyper {
    msbuf_info: MsbufInfo {
        base: 0,
        num: 0,
        size: 0,
    },
};

impl GlobalHyper {
    #[inline]
    pub fn get() -> &'static GlobalHyper {
        unsafe {
            let ptr = &g_global_data_hyper as *const _ as *const GlobalHyper;
            &*ptr
        }
    }

    #[inline]
    pub fn get_mut() -> &'static mut GlobalHyper {
        unsafe {
            let ptr = &mut g_global_data_hyper as *mut _ as *mut GlobalHyper;
            &mut *ptr
        }
    }

    #[inline]
    pub fn set_msbuf_info(&mut self, msbuf_info: &MsbufInfo) -> SgxResult {
        msbuf_info.is_valid()?;
        lfence();

        self.msbuf_info = *msbuf_info;
        Ok(())
    }

    #[inline]
    pub fn get_msbuf_info(&self) -> &MsbufInfo {
        &self.msbuf_info
    }
}
