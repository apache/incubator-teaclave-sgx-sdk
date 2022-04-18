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

use super::manager::RESPONDER_SESSION_MAGAGER;
use super::manager::{DropKey, DropPrivateKey, DropShareKey};
use super::manager::{ResponderState, Role, Session};
use super::QVE_ISVSVN_THRESHOLD;
use crate::QveReportInfo;
use core::mem;
use sgx_crypto::ecc::{EcKeyPair, EcPublicKey};
use sgx_crypto::sha::Sha256;
use sgx_dcap_ra_msg::{DcapMRaMsg2, DcapRaMsg1, DcapRaMsg3};
use sgx_trts::fence;
use sgx_trts::rand::Rng;
use sgx_trts::trts::EnclaveRange;
use sgx_tse::EnclaveReport;
use sgx_types::error::{Quote3Error, SgxResult, SgxStatus};
use sgx_types::types::SHA256_HASH_SIZE;
use sgx_types::types::{
    AlignKey128bit, EnclaveIdentity, QlQvResult, Quote3, QuoteNonce, RaContext, RaKeyType, Report,
    ReportData, TargetInfo,
};

#[derive(Debug)]
pub struct Responder {
    rctx: RaContext,
}

impl Responder {
    pub fn new() -> SgxResult<Responder> {
        let session = Session::new(Role::Responder);

        let rctx = RESPONDER_SESSION_MAGAGER.write().push(session);
        Ok(Self { rctx })
    }

    pub fn process_msg1(
        &self,
        msg1: &DcapRaMsg1,
        qe_target: &TargetInfo,
    ) -> SgxResult<(EcPublicKey, Report, QuoteNonce)> {
        ensure!(msg1.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(qe_target.is_enclave_range(), SgxStatus::InvalidParameter);

        let session = RESPONDER_SESSION_MAGAGER
            .read()
            .find(self.rctx)
            .ok_or(SgxStatus::InvalidParameter)?;

        fence::lfence();

        let context = session.context.lock();
        ensure!(
            context.state.check_responder_state(ResponderState::Inited),
            SgxStatus::InvalidState
        );
        drop(context);

        let mut key_pair = EcKeyPair::create()?;
        let (mut priv_key, pub_key) = key_pair.into();

        key_pair.clear();
        let priv_key = DropPrivateKey::new(&mut priv_key);

        let mut dh_key = priv_key.shared_key(&msg1.pub_key_a)?;
        let dh_key = DropShareKey::new(&mut dh_key);

        let (ref mut smk_key, ref mut sk_key, ref mut mk_key, ref mut vk_key) = {
            let smk_key = dh_key.derive_key("SMK".as_bytes())?;
            let sk_key = dh_key.derive_key("SK".as_bytes())?;
            let mk_key = dh_key.derive_key("MK".as_bytes())?;
            let vk_key = dh_key.derive_key("VK".as_bytes())?;
            (smk_key, sk_key, mk_key, vk_key)
        };

        let smk_key = DropKey::new(smk_key);
        let sk_key = DropKey::new(sk_key);
        let mk_key = DropKey::new(mk_key);
        let vk_key = DropKey::new(vk_key);

        let mut nonce = QuoteNonce::default();
        Rng::new().fill_bytes(&mut nonce.rand);

        let mut report_data = ReportData::default();
        let mut sha = Sha256::new()?;
        sha.update(&msg1.pub_key_a)?;
        sha.update(&pub_key)?;
        sha.update(&*vk_key)?;
        let hash = sha.finalize()?;
        report_data.d[..SHA256_HASH_SIZE].copy_from_slice(&hash);
        let report = Report::for_target(qe_target, &report_data)?;

        let mut context = session.context.lock();
        ensure!(
            context.state.check_responder_state(ResponderState::Inited),
            SgxStatus::InvalidState
        );
        context.pub_key_a = msg1.pub_key_a;
        context.pub_key_b = pub_key;
        context.priv_key = *priv_key;
        context.smk_key = *smk_key;
        context.sk_key = *sk_key;
        context.mk_key = *mk_key;
        context.vk_key = *vk_key;
        context.qe_target = *qe_target;
        context.quote_nonce = nonce;
        context.state = From::from(ResponderState::Msg1Proced);
        drop(context);

        Ok((pub_key, report, nonce))
    }

    pub fn generate_msg2(&self, qe_report: &Report, quote: &[u8]) -> SgxResult<DcapMRaMsg2> {
        ensure!(qe_report.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(!quote.is_empty(), SgxStatus::InvalidParameter);
        ensure!(
            quote.is_enclave_range() || quote.is_host_range(),
            SgxStatus::InvalidParameter
        );
        ensure!(
            DcapMRaMsg2::check_quote_len(quote.len()),
            SgxStatus::InvalidParameter
        );

        let session = RESPONDER_SESSION_MAGAGER
            .read()
            .find(self.rctx)
            .ok_or(SgxStatus::InvalidParameter)?;

        fence::lfence();

        qe_report.verify()?;

        let context = session.context.lock();
        ensure!(
            context
                .state
                .check_responder_state(ResponderState::Msg1Proced),
            SgxStatus::InvalidState
        );
        let attributes = context.qe_target.attributes;
        let mr_enclave = context.qe_target.mr_enclave;
        let pub_key_b = context.pub_key_b;
        let mut smk_key = context.smk_key;
        let nonce = context.quote_nonce;
        drop(context);

        let smk_key = DropKey::new(&mut smk_key);

        ensure!(
            attributes.eq(&qe_report.body.attributes),
            SgxStatus::InvalidParameter
        );
        ensure!(
            mr_enclave.eq(&qe_report.body.mr_enclave),
            SgxStatus::InvalidParameter
        );

        let mut sha = Sha256::new()?;
        sha.update(&nonce)?;
        sha.update(quote)?;
        let hash = sha.finalize()?;
        ensure!(
            hash.eq(&qe_report.body.report_data.d[..SHA256_HASH_SIZE]),
            SgxStatus::Unexpected
        );

        let mut msg2 = DcapMRaMsg2 {
            mac: Default::default(),
            pub_key_b,
            kdf_id: 0x0001,
            quote: quote.into(),
        };
        msg2.gen_cmac(&smk_key)?;

        let mut context = session.context.lock();
        ensure!(
            context
                .state
                .check_responder_state(ResponderState::Msg1Proced),
            SgxStatus::InvalidState
        );
        context.state = From::from(ResponderState::Msg2Gened);
        drop(context);

        Ok(msg2)
    }

    pub fn process_msg3(
        &self,
        msg3: &DcapRaMsg3,
        qve_report_info: &QveReportInfo,
    ) -> SgxResult<EnclaveIdentity> {
        ensure!(msg3.is_enclave_range(), SgxStatus::InvalidParameter);

        let session = RESPONDER_SESSION_MAGAGER
            .read()
            .find(self.rctx)
            .ok_or(SgxStatus::InvalidParameter)?;

        fence::lfence();

        let context = session.context.lock();
        ensure!(
            context
                .state
                .check_responder_state(ResponderState::Msg2Gened),
            SgxStatus::InvalidState
        );
        let pub_key_a = context.pub_key_a;
        let pub_key_b = context.pub_key_b;
        let mut smk_key = context.smk_key;
        let mut vk_key = context.vk_key;
        drop(context);

        let smk_key = DropKey::new(&mut smk_key);
        let vk_key = DropKey::new(&mut vk_key);

        ensure!(msg3.pub_key_a == pub_key_a, SgxStatus::Unexpected);
        msg3.verify_cmac(&smk_key)?;
        qve_report_info
            .verify_report_and_identity(&msg3.quote, QVE_ISVSVN_THRESHOLD)
            .map_err(|e| match e {
                Quote3Error::InvalidParameter => SgxStatus::InvalidParameter,
                Quote3Error::QveIdentityMismatch | Quote3Error::QveOutOfDate => {
                    SgxStatus::UpdateNeeded
                }
                _ => SgxStatus::Unexpected,
            })?;

        let mut sha = Sha256::new()?;
        sha.update(&pub_key_a)?;
        sha.update(&pub_key_b)?;
        sha.update(&*vk_key)?;
        let hash = sha.finalize()?;

        let quote3 = unsafe { &*(msg3.quote.as_ptr() as *const Quote3) };
        ensure!(
            hash.eq(&quote3.report_body.report_data.d[..SHA256_HASH_SIZE]),
            SgxStatus::Unexpected
        );
        let enclave_identity = quote3.report_body.into();

        let mut context = session.context.lock();
        ensure!(
            context
                .state
                .check_responder_state(ResponderState::Msg2Gened),
            SgxStatus::InvalidState
        );

        context.qv_result = Some(qve_report_info.quote_verification_result);
        context.enclave_identity = Some(enclave_identity);
        context.state = From::from(ResponderState::Established);
        drop(context);

        Ok(enclave_identity)
    }

    pub fn get_keys(&self, key_type: RaKeyType) -> SgxResult<AlignKey128bit> {
        let session = RESPONDER_SESSION_MAGAGER
            .read()
            .find(self.rctx)
            .ok_or(SgxStatus::InvalidParameter)?;

        let context = session.context.lock();
        ensure!(
            context
                .state
                .check_responder_state(ResponderState::Established),
            SgxStatus::InvalidState
        );

        let key = match key_type {
            RaKeyType::SK => context.sk_key,
            RaKeyType::MK => context.mk_key,
        };

        Ok(key)
    }

    pub fn get_peer_identity(&self) -> SgxResult<(QlQvResult, EnclaveIdentity)> {
        let session = RESPONDER_SESSION_MAGAGER
            .read()
            .find(self.rctx)
            .ok_or(SgxStatus::InvalidParameter)?;

        let context = session.context.lock();
        ensure!(
            context
                .state
                .check_responder_state(ResponderState::Established),
            SgxStatus::InvalidState
        );
        let enclave_identity = context.enclave_identity.ok_or(SgxStatus::Unexpected)?;
        let qv_result = context.qv_result.ok_or(SgxStatus::Unexpected)?;

        Ok((qv_result, enclave_identity))
    }

    #[inline]
    pub fn into_raw(self) -> RaContext {
        let rctx = self.rctx;
        mem::forget(self);
        rctx
    }

    #[inline]
    pub unsafe fn from_raw(rctx: RaContext) -> Responder {
        Self { rctx }
    }
}

impl Drop for Responder {
    fn drop(&mut self) {
        if let Some(session) = RESPONDER_SESSION_MAGAGER.write().remove(self.rctx) {
            drop(session)
        }
    }
}
