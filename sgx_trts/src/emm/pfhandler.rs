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

use sgx_tlibc_sys::c_void;

use crate::{
    emm::ProtFlags,
    emm::{
        page::AllocFlags,
        range::{RangeType, RM},
    },
    veh::HandleResult,
};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PfInfo {
    pub maddr: u64, // address for #PF.
    pub pfec: Pfec,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union Pfec {
    pub errcd: u32,
    pub bits: PfecBits,
}

impl Default for Pfec {
    fn default() -> Self {
        Pfec { errcd: 0 }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
pub struct PfecBits(u32);

impl PfecBits {
    const P_OFFSET: u32 = 0;
    const P_MASK: u32 = 0x00000001;
    const RW_OFFSET: u32 = 1;
    const RW_MASK: u32 = 0x00000002;
    const SGX_OFFSET: u32 = 15;
    const SGX_MASK: u32 = 0x00008000;

    #[inline]
    pub fn p(&self) -> u32 {
        (self.0 & Self::P_MASK) >> Self::P_OFFSET
    }

    #[inline]
    pub fn rw(&self) -> u32 {
        (self.0 & Self::RW_MASK) >> Self::RW_OFFSET
    }

    #[inline]
    pub fn sgx(&self) -> u32 {
        (self.0 & Self::SGX_MASK) >> Self::SGX_OFFSET
    }

    #[inline]
    pub fn set_p(&mut self, p: u32) {
        let p = (p << Self::P_OFFSET) & Self::P_MASK;
        self.0 = (self.0 & (!Self::P_MASK)) | p;
    }

    #[inline]
    pub fn set_rw(&mut self, rw: u32) {
        let rw = (rw << Self::RW_OFFSET) & Self::RW_MASK;
        self.0 = (self.0 & (!Self::RW_MASK)) | rw;
    }

    #[inline]
    pub fn set_sgx(&mut self, sgx: u32) {
        let sgx = (sgx << Self::SGX_OFFSET) & Self::SGX_MASK;
        self.0 = (self.0 & (!Self::SGX_MASK)) | sgx;
    }
}

pub type PfHandler = extern "C" fn(info: &mut PfInfo, priv_data: *mut c_void) -> HandleResult;

pub extern "C" fn mm_enclave_pfhandler(info: &mut PfInfo) -> HandleResult {
    let addr = trim_to_page!(info.maddr as usize);
    let mut range_manage = RM.get().unwrap().lock();
    let mut ema_cursor = match range_manage.search_ema(addr, RangeType::User) {
        None => {
            let ema_cursor = range_manage.search_ema(addr, RangeType::Rts);
            if ema_cursor.is_none() {
                return HandleResult::Search;
            }
            ema_cursor.unwrap()
        }
        Some(ema_cursor) => ema_cursor,
    };

    let ema = unsafe { ema_cursor.get_mut().unwrap() };
    let (handler, priv_data) = ema.fault_handler();
    if let Some(handler) = handler {
        drop(range_manage);
        return handler(info, priv_data.unwrap());
    }

    // No customized page fault handler
    if ema.is_page_committed(addr) {
        // check spurious #pf
        let rw_bit = unsafe { info.pfec.bits.rw() };
        if (rw_bit == 0 && !ema.info().prot.contains(ProtFlags::R))
            || (rw_bit == 1 && !ema.info().prot.contains(ProtFlags::W))
        {
            return HandleResult::Search;
        } else {
            return HandleResult::Execution;
        }
    }

    if ema.flags().contains(AllocFlags::COMMIT_ON_DEMAND) {
        let rw_bit = unsafe { info.pfec.bits.rw() };
        if (rw_bit == 0 && !ema.info().prot.contains(ProtFlags::R))
            || (rw_bit == 1 && !ema.info().prot.contains(ProtFlags::W))
        {
            return HandleResult::Search;
        };
        ema.commit_check()
            .expect("The EPC page fails to meet the commit condition.");
        ema.commit(addr, crate::arch::SE_PAGE_SIZE)
            .expect("The EPC page fails to be committed.");
        HandleResult::Execution
    } else {
        // Some things are wrong
        panic!()
    }
}
