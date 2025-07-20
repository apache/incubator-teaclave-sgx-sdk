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

use super::Initiator;
use core::mem::{self, ManuallyDrop};
use core::slice;
use sgx_ra_msg::RaMsg2;
use sgx_trts::fence;
use sgx_trts::trts::is_within_host;
use sgx_types::error::SgxStatus;
use sgx_types::types::{
    CRaMsg2, CRaMsg3, Ec256PublicKey, QuoteNonce, QuoteSignType, RaContext, Report, TargetInfo,
};

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_get_ga(
    context: RaContext,
    pub_key_a: *mut Ec256PublicKey,
) -> SgxStatus {
    if pub_key_a.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let initiator = ManuallyDrop::new(Initiator::from_raw(context));
    let key = match initiator.get_ga() {
        Ok(key) => key,
        Err(e) => return e,
    };

    *pub_key_a = key.into();
    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_proc_msg2_trusted(
    context: RaContext,
    msg2: *const CRaMsg2,
    qe_target: *const TargetInfo,
    report: *mut Report,
    nonce: *mut QuoteNonce,
) -> SgxStatus {
    if msg2.is_null() || qe_target.is_null() || report.is_null() || nonce.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let qe_target = &*qe_target;
    let c_msg2 = &*msg2;
    let quote_type = if c_msg2.quote_type == 0 {
        QuoteSignType::Unlinkable
    } else {
        QuoteSignType::Linkable
    };

    let msg2 = RaMsg2 {
        pub_key_b: c_msg2.g_b.into(),
        spid: c_msg2.spid,
        quote_type,
        kdf_id: c_msg2.kdf_id,
        sign_gb_ga: c_msg2.sign_gb_ga.into(),
        mac: c_msg2.mac,
        sig_rl: None,
    };

    let initiator = ManuallyDrop::new(Initiator::from_raw(context));
    let (rpt, rand) = match initiator.process_msg2(&msg2, qe_target) {
        Ok(r) => r,
        Err(e) => return e,
    };

    *report = rpt;
    *nonce = rand;
    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_get_msg3_trusted(
    context: RaContext,
    quote_size: u32,
    qe_report: *const Report,
    msg3: *mut CRaMsg3,
    msg3_size: u32,
) -> SgxStatus {
    if qe_report.is_null() || msg3.is_null() {
        return SgxStatus::InvalidParameter;
    }

    if usize::MAX - (msg3 as usize) < msg3_size as usize
        || u32::MAX - quote_size < mem::size_of::<CRaMsg3>() as u32
        || mem::size_of::<CRaMsg3>() as u32 + quote_size != msg3_size
    {
        return SgxStatus::InvalidParameter;
    }

    if !is_within_host(msg3 as *const u8, msg3_size as usize) {
        return SgxStatus::InvalidParameter;
    }

    fence::lfence();

    let qe_report = &*qe_report;
    let c_msg3 = &mut *msg3;
    let quote = slice::from_raw_parts(&c_msg3.quote as *const _ as *const u8, quote_size as usize);

    let initiator = ManuallyDrop::new(Initiator::from_raw(context));
    let msg3 = match initiator.generate_msg3(qe_report, quote) {
        Ok(msg) => msg,
        Err(e) => return e,
    };

    c_msg3.mac = msg3.mac;
    c_msg3.g_a = msg3.pub_key_a.into();
    c_msg3.ps_sec_prop = msg3.ps_sec_prop;
    SgxStatus::Success
}
