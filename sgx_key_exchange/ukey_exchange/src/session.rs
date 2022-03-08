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

#![allow(dead_code)]

use sgx_ra_msg::{RaMsg1, RaMsg2, RaMsg3};
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::function::{
    sgx_calc_quote_size, sgx_get_quote, sgx_get_quote_ex, sgx_get_quote_size_ex, sgx_init_quote,
    sgx_init_quote_ex,
};
use sgx_types::types::{
    AttKeyId, CRaMsg3, Ec256PublicKey, EnclaveId, EpidGroupId, QeReportInfo, QuoteNonce, RaContext,
    Report, TargetInfo,
};
use sgx_types::types::{ECallGetGaFn, ECallGetMsg3Fn, ECallProcessMsg2Fn};
use std::mem;
use std::ptr;
use std::vec;

#[derive(Debug)]
pub struct Initiator {
    rctx: RaContext,
    eid: EnclaveId,
    att_key_id: Option<AttKeyId>,
    qe_target: TargetInfo,
}

impl Initiator {
    #[inline]
    pub fn new(eid: EnclaveId, rctx: RaContext) -> Initiator {
        Initiator {
            rctx,
            eid,
            att_key_id: None,
            qe_target: TargetInfo::default(),
        }
    }

    pub fn generate_msg1(
        &mut self,
        att_key_id: Option<AttKeyId>,
        get_ga_fn: ECallGetGaFn,
    ) -> SgxResult<RaMsg1> {
        let mut gid = EpidGroupId::default();
        let mut qe_target = TargetInfo::default();

        let status = unsafe {
            if let Some(ref att_key_id) = att_key_id {
                let mut pub_key_id_size = 0_usize;
                let status = sgx_init_quote_ex(
                    att_key_id as *const _,
                    &mut qe_target as *mut _,
                    &mut pub_key_id_size as *mut _,
                    ptr::null_mut(),
                );
                ensure!(status.is_success(), status);

                let mut pub_key_id = vec![0_u8; pub_key_id_size];
                sgx_init_quote_ex(
                    att_key_id as *const _,
                    &mut qe_target as *mut _,
                    &mut pub_key_id_size as *mut _,
                    pub_key_id.as_mut_ptr(),
                )
            } else {
                sgx_init_quote(&mut qe_target as *mut _, &mut gid as *mut _)
            }
        };
        ensure!(status.is_success(), status);

        self.att_key_id = att_key_id;
        self.qe_target = qe_target;

        let mut retval = SgxStatus::Success;
        let mut pub_key_a = Ec256PublicKey::default();
        let status = unsafe {
            get_ga_fn(
                self.eid,
                &mut retval as *mut _,
                self.rctx,
                &mut pub_key_a as *mut _,
            )
        };
        ensure!(status.is_success(), status);
        ensure!(retval.is_success(), retval);

        Ok(RaMsg1 {
            pub_key_a: pub_key_a.into(),
            gid,
        })
    }

    pub fn process_msg2(
        &self,
        msg2: &RaMsg2,
        process_msg2_fn: ECallProcessMsg2Fn,
        get_msg3_fn: ECallGetMsg3Fn,
    ) -> SgxResult<RaMsg3> {
        let raw_msg2 = msg2.to_bytes()?;

        let (sig_rl, sig_rl_len) = msg2
            .sig_rl
            .as_ref()
            .map(|sig_rl| {
                if !sig_rl.is_empty() {
                    (sig_rl.as_ptr(), sig_rl.len() as u32)
                } else {
                    (ptr::null(), 0)
                }
            })
            .unwrap_or((ptr::null(), 0));

        if self.att_key_id.is_some() {
            ensure!(sig_rl_len == 0, SgxStatus::InvalidParameter);
        }

        let raw_msg3 = unsafe {
            let mut report = Report::default();
            let mut nonce = QuoteNonce::default();

            let mut retval = SgxStatus::Success;
            let status = process_msg2_fn(
                self.eid,
                &mut retval as *mut _,
                self.rctx,
                raw_msg2.as_ptr().cast(),
                &self.qe_target as *const _,
                &mut report as *mut _,
                &mut nonce as *mut _,
            );
            ensure!(status.is_success(), status);
            ensure!(retval.is_success(), retval);

            let mut quote_size = 0_u32;
            let status = if let Some(ref att_key_id) = self.att_key_id {
                sgx_get_quote_size_ex(att_key_id as *const _, &mut quote_size as *mut _)
            } else {
                sgx_calc_quote_size(sig_rl, sig_rl_len, &mut quote_size as *mut _)
            };
            ensure!(status.is_success(), status);

            ensure!(
                RaMsg3::check_quote_len(quote_size as usize),
                SgxStatus::Unexpected
            );

            let raw_msg3_len = mem::size_of::<CRaMsg3>() + quote_size as usize;

            cfg_if! {
                if #[cfg(feature = "hyper")] {
                    use sgx_urts::msbuf::MsBufAlloc;
                    let alloc = MsBufAlloc::new(self.eid);

                    let remain_size = alloc.remain_size();
                    ensure!(raw_msg3_len < remain_size, SgxStatus::OutOfMemory);
                } else {
                    use std::alloc::Global;
                    let alloc = Global;
                }
            }

            let mut raw_msg3 = vec::from_elem_in(0_u8, raw_msg3_len, alloc);
            let c_msg3 = &mut *(raw_msg3.as_mut_ptr() as *mut CRaMsg3);

            let qe_report = &(if let Some(ref att_key_id) = self.att_key_id {
                let mut qe_report_info = QeReportInfo {
                    nonce,
                    app_enclave_target_info: report.into(),
                    qe_report: Report::default(),
                };
                let status = sgx_get_quote_ex(
                    &report as *const _,
                    att_key_id as *const _,
                    &mut qe_report_info as *mut _,
                    (&mut c_msg3.quote).as_mut_ptr().cast(),
                    quote_size,
                );
                ensure!(status.is_success(), status);

                qe_report_info.qe_report
            } else {
                let mut qe_report = Report::default();
                let status = sgx_get_quote(
                    &report as *const _,
                    msg2.quote_type,
                    &msg2.spid as *const _,
                    &nonce as *const _,
                    sig_rl,
                    sig_rl_len,
                    &mut qe_report as *mut _,
                    (&mut c_msg3.quote).as_mut_ptr().cast(),
                    quote_size,
                );
                ensure!(status.is_success(), status);

                qe_report
            });

            let mut retval = SgxStatus::Success;
            let status = get_msg3_fn(
                self.eid,
                &mut retval as *mut _,
                self.rctx,
                quote_size,
                qe_report as *const _,
                c_msg3 as *mut _,
                raw_msg3_len as u32,
            );
            ensure!(status.is_success(), status);
            ensure!(retval.is_success(), retval);

            raw_msg3
        };

        RaMsg3::from_bytes(raw_msg3)
    }
}

impl Initiator {
    #[inline]
    pub(crate) fn set_attkey_id(&mut self, att_key_id: Option<AttKeyId>) {
        self.att_key_id = att_key_id;
    }

    #[inline]
    pub(crate) fn set_qe_target(&mut self, qe_target: TargetInfo) {
        self.qe_target = qe_target;
    }

    #[inline]
    pub(crate) fn get_attkey_id(&self) -> Option<AttKeyId> {
        self.att_key_id
    }

    #[inline]
    pub(crate) fn get_qe_target(&self) -> TargetInfo {
        self.qe_target
    }
}
