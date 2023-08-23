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

use crate::arch::SsaGpr;
use crate::se::AlignReport;
use crate::tcs;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::types::AttributesFlags;

pub struct AEXNotify;

impl AEXNotify {
    const SSA_AEXNOTIFY_MASK: u8 = 1;

    pub fn set(is_enable: bool) -> SgxResult {
        let report = AlignReport::get_self();
        if !report
            .0
            .body
            .attributes
            .flags
            .intersects(AttributesFlags::AEXNOTIFY)
        {
            bail!(SgxStatus::Unexpected);
        }

        let mut tc = tcs::current();
        let tds = tc.tds_mut();
        let ssa_gpr = tds.ssa_gpr_mut();

        if is_enable {
            ssa_gpr.aex_notify |= Self::SSA_AEXNOTIFY_MASK;
        } else {
            ssa_gpr.aex_notify &= !Self::SSA_AEXNOTIFY_MASK;
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn is_enable(ssa_gpr: &SsaGpr) -> bool {
        (ssa_gpr.aex_notify & Self::SSA_AEXNOTIFY_MASK) != 0
    }
}
