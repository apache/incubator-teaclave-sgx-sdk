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

use crate::arch::{Enclu, Secinfo, Secs, Tcs};
use crate::error::abort;
use crate::se::{AlignKey, AlignKeyRequest, AlignReport, AlignReportData, AlignTargetInfo};
use core::convert::TryFrom;
use core::ptr;
use core::sync::atomic::AtomicUsize;
use inst::EncluInst;
use sgx_types::marker::ContiguousMemory;
use sgx_types::types::{CpuSvn, IsvExtProdId, IsvFamilyId};

pub mod derive;
pub mod inst;
pub mod tls;

#[link_section = ".nipx"]
#[no_mangle]
pub unsafe extern "C" fn se3(
    rax: usize,
    rbx: usize,
    rcx: usize,
    rdx: usize,
    rsi: usize,
    rdi: usize,
) -> usize {
    let enclu = match Enclu::try_from(rax as u32) {
        Ok(e) => e,
        Err(_) => abort(),
    };

    match enclu {
        Enclu::EExit => {
            EncluInst::eexit(rbx, rcx, rdx, rsi, rdi);
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
        Enclu::EAccept => match EncluInst::eaccept(&*(rbx as *const Secinfo), rcx) {
            Ok(_) => 0,
            Err(e) => e as usize,
        },
        Enclu::EModpe => match EncluInst::emodpe(&*(rbx as *const Secinfo), rcx) {
            Ok(_) => 0,
            Err(e) => e as usize,
        },
        _ => abort(),
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct GlobalSim {
    pub secs: *const Secs,
    pub cpu_svn: CpuSvn,
    pub seed: u64,
}

#[no_mangle]
pub static mut g_global_data_sim: GlobalSim = GlobalSim {
    secs: ptr::null(),
    cpu_svn: CpuSvn { svn: [0; 16] },
    seed: 0,
};

impl GlobalSim {
    #[inline]
    pub fn get() -> &'static GlobalSim {
        unsafe {
            let ptr = &g_global_data_sim as *const _ as *const GlobalSim;
            &*ptr
        }
    }

    #[inline]
    pub fn get_mut() -> &'static mut GlobalSim {
        unsafe {
            let ptr = &mut g_global_data_sim as *mut _ as *mut GlobalSim;
            &mut *ptr
        }
    }

    #[inline]
    pub fn secs(&self) -> &Secs {
        unsafe { &*self.secs }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct IsvExtId {
    pub isv_family_id: IsvFamilyId,
    pub isv_ext_prod_id: IsvExtProdId,
}

impl IsvExtId {
    pub fn get(secs: &Secs) -> &IsvExtId {
        unsafe { &*(&secs.reserved4 as *const _ as *const IsvExtId) }
    }
}

unsafe impl ContiguousMemory for IsvExtId {}

#[repr(C)]
#[derive(Debug, Default)]
pub struct TcsSim {
    pub saved_aep: usize,
    pub tcs_state: AtomicUsize,
    pub saved_dtv: usize,
    pub saved_fs_gs_0: usize,
    pub tcs_offset_update_flag: u64,
}

impl TcsSim {
    #[link_section = ".nipx"]
    pub fn get_mut(tcs: &mut Tcs) -> &mut TcsSim {
        unsafe { &mut *(&mut tcs.reserved as *mut _ as *mut TcsSim) }
    }
}

unsafe impl ContiguousMemory for TcsSim {}

impl_enum! {
    #[repr(usize)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum TcsState {
        Inactive = 0,
        Active = 1,
    }
}
