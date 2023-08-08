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

use crate::{
    edmm::ProtFlags,
    emm::{
        flags::AllocFlags,
        range::{RangeType, RM},
    },
    veh::HandleResult,
};

#[repr(C)]
pub struct PfInfo {
    maddr: u64, // address for #PF.
    pfec: Pfec,
    reserved: u32,
}

#[repr(C)]
union Pfec {
    errcd: u32,
    bits: PfecBits,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct PfecBits {
    p: u32,  // P flag.
    rw: u32, // RW access flag, 0 for read, 1 for write.
    reserved1: u32,
    sgx: u32, // SGX bit.
    reserved2: u32,
}

impl Default for PfecBits {
    fn default() -> Self {
        Self {
            p: 1,
            rw: 1,
            reserved1: 13,
            sgx: 1,
            reserved2: 16,
        }
    }
}

pub type PfHandler = extern "C" fn(info: &mut PfInfo) -> HandleResult;

extern "C" fn mm_enclave_pfhandler(info: &mut PfInfo) -> HandleResult {
    let addr = trim_to_page!(info.maddr as usize);
    let mut range_manage = RM.get().unwrap().lock();
    let mut ema_cursor = match range_manage.search_ema(addr, RangeType::User) {
        Err(_) => {
            let ema_cursor = range_manage.search_ema(addr, RangeType::Rts);
            if ema_cursor.is_err() {
                return HandleResult::Search;
            }
            ema_cursor.unwrap()
        }
        Ok(ema_cursor) => ema_cursor,
    };

    let ema = unsafe { ema_cursor.get_mut().unwrap() };
    let (handler, priv_data) = ema.fault_handler();
    if let Some(handler) = handler {
        drop(range_manage);
        let mut pf_info = unsafe { priv_data.unwrap().read() };
        return handler(&mut pf_info);
    }

    // No customized page fault handler
    if ema.is_page_committed(addr) {
        // check spurious #pf
        let rw_bit = unsafe { info.pfec.bits.rw };
        if (rw_bit == 0 && !ema.info().prot.contains(ProtFlags::R))
            || (rw_bit == 1 && !ema.info().prot.contains(ProtFlags::W))
        {
            return HandleResult::Search;
        } else {
            return HandleResult::Execution;
        }
    }

    if ema.flags().contains(AllocFlags::COMMIT_ON_DEMAND) {
        let rw_bit = unsafe { info.pfec.bits.rw };
        if (rw_bit == 0 && !ema.info().prot.contains(ProtFlags::R))
            || (rw_bit == 1 && !ema.info().prot.contains(ProtFlags::W))
        {
            return HandleResult::Search;
        };
        ema.commit_check()
            .expect("The EPC page fails to meet the commit condition.");
        ema.commit(addr, addr + crate::arch::SE_PAGE_SIZE)
            .expect("The EPC page fails to be committed.");
        HandleResult::Execution
    } else {
        // Some things are wrong
        panic!()
    }
}
