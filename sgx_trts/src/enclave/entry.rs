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
// under the License.

use crate::arch::Tcs;
use crate::call;
use crate::call::ECallIndex;
use crate::edmm::tcs;
use crate::edmm::tcs::MkTcs;
use crate::enclave;
use crate::enclave::state::{self, State};
use crate::tcs::tc;
use crate::veh;
use core::convert::TryFrom;
use core::ffi::c_void;
use core::ptr::NonNull;
use sgx_types::error::SgxStatus;

#[link_section = ".nipx"]
#[no_mangle]
pub unsafe extern "C" fn enter_enclave(index: u64, ms: usize, tcs: *mut Tcs, cssa: i32) -> u32 {
    if tcs as usize == 0 {
        return SgxStatus::Unexpected as u32;
    }
    let tcs = &mut *tcs;

    let ecall = match ECallIndex::try_from(index as i32) {
        Ok(e) => e,
        Err(_) => return SgxStatus::Unexpected as u32,
    };

    let state = state::get_state();
    if state.is_crashed() {
        return SgxStatus::EnclaveCrashed as u32;
    }

    if !ecall.is_enclave_init() && !state.is_done() {
        state::set_state(State::Crashed);
        return SgxStatus::Unexpected as u32;
    }

    let result = if cssa == 0 {
        match ecall {
            ECallIndex::ECall(_) | ECallIndex::Thread => {
                call::ecall(ecall, tcs, ms as *mut c_void, (index >> 32) as usize)
            }
            ECallIndex::RtInit => enclave::rtinit(tcs, ms as *mut _, (index >> 32) as usize),
            ECallIndex::ORet => call::oret(ms),
            ECallIndex::MkTcs => {
                NonNull::new(ms as *mut MkTcs).map_or(Err(SgxStatus::Unexpected), tcs::mktcs)
            }
            ECallIndex::RtUninit => enclave::rtuninit(tc::ThreadControl::from_tcs(tcs)),
            ECallIndex::GlobalInit => {
                enclave::global_init(tcs, ms as *mut _, (index >> 32) as usize)
            }
            ECallIndex::GlobalExit => enclave::global_exit(tc::ThreadControl::from_tcs(tcs)),
            _ => Err(SgxStatus::Unexpected),
        }
    } else if cssa == 1 {
        match ecall {
            ECallIndex::Except => {
                let mut result = veh::handle(tcs);
                if !tc::check_static_stack_guard(tcs) {
                    result = Err(SgxStatus::StackOverRun);
                }
                result
            }
            _ => Err(SgxStatus::Unexpected),
        }
    } else {
        Err(SgxStatus::Unexpected)
    };

    if let Err(error) = result {
        if error as u32 == SgxStatus::Unexpected as u32 {
            state::set_state(State::Crashed)
        }
        error as u32
    } else {
        SgxStatus::Success as u32
    }
}
