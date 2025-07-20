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

use super::manager::SESSION_MAGAGER;
use super::manager::{Context, Session, State};
use super::manager::{DropKey, DropPrivateKey, DropShareKey};
use alloc::boxed::Box;
use core::mem;
use sgx_crypto::ecc::{EcKeyPair, EcPublicKey};
use sgx_crypto::sha::Sha256;
use sgx_ra_msg::{RaMsg2, RaMsg3};
use sgx_trts::fence;
use sgx_trts::rand::Rng;
use sgx_trts::trts::EnclaveRange;
use sgx_tse::EnclaveReport;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::types::SHA256_HASH_SIZE;
use sgx_types::types::{
    AlignKey128bit, Ec256SharedKey, QuoteNonce, RaContext, RaKeyType, Report, ReportData,
    TargetInfo,
};

#[derive(Debug)]
pub struct Initiator {
    rctx: RaContext,
}

impl Initiator {
    pub fn new(sp_pub_key: &EcPublicKey) -> SgxResult<Initiator> {
        ensure!(sp_pub_key.is_enclave_range(), SgxStatus::InvalidParameter);

        let is_valid = sp_pub_key.check_point()?;
        ensure!(is_valid, SgxStatus::InvalidParameter);

        let mut context = Context::new();
        context.sp_pub_key = *sp_pub_key;
        let session = Session::new_with_context(context).set_derive_key(None);

        let rctx = SESSION_MAGAGER.write().push(session);
        Ok(Self { rctx })
    }

    pub fn new_with_derive_key<F>(sp_pub_key: &EcPublicKey, derive_key: F) -> SgxResult<Initiator>
    where
        F: Fn(
                &Ec256SharedKey,
                u16,
            ) -> SgxResult<(
                AlignKey128bit,
                AlignKey128bit,
                AlignKey128bit,
                AlignKey128bit,
            )> + Sync
            + Send
            + 'static,
    {
        ensure!(sp_pub_key.is_enclave_range(), SgxStatus::InvalidParameter);

        let is_valid = sp_pub_key.check_point()?;
        ensure!(is_valid, SgxStatus::InvalidParameter);

        let mut context = Context::new();
        context.sp_pub_key = *sp_pub_key;
        let session = Session::new_with_context(context).set_derive_key(Some(Box::new(derive_key)));

        let rctx = SESSION_MAGAGER.write().push(session);
        Ok(Self { rctx })
    }

    pub fn get_ga(&self) -> SgxResult<EcPublicKey> {
        let session = SESSION_MAGAGER
            .read()
            .find(self.rctx)
            .ok_or(SgxStatus::InvalidParameter)?;

        let mut key_pair = EcKeyPair::create()?;
        let (mut priv_key, pub_key) = key_pair.into();

        let mut context = session.context.lock();
        context.priv_key = priv_key;
        context.pub_key_a = pub_key;
        context.state = State::GaGened;
        drop(context);

        key_pair.clear();
        priv_key.clear();

        Ok(pub_key)
    }

    pub fn process_msg2(
        &self,
        msg2: &RaMsg2,
        qe_target: &TargetInfo,
    ) -> SgxResult<(Report, QuoteNonce)> {
        ensure!(msg2.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(qe_target.is_enclave_range(), SgxStatus::InvalidParameter);

        let session = SESSION_MAGAGER
            .read()
            .find(self.rctx)
            .ok_or(SgxStatus::InvalidParameter)?;

        fence::lfence();

        let context = session.context.lock();
        ensure!(context.state == State::GaGened, SgxStatus::InvalidState);
        let mut priv_key = context.priv_key;
        let pub_key_a = context.pub_key_a;
        let sp_pub_key = context.sp_pub_key;
        drop(context);

        let priv_key = DropPrivateKey::new(&mut priv_key);

        let mut dh_key = priv_key.shared_key(&msg2.pub_key_b)?;
        let dh_key = DropShareKey::new(&mut dh_key);

        let (ref mut smk_key, ref mut sk_key, ref mut mk_key, ref mut vk_key) =
            if let Some(driver_key) = &session.derive_key {
                driver_key(&(&*dh_key).into(), msg2.kdf_id)?
            } else {
                ensure!(msg2.kdf_id == 0x0001, SgxStatus::KdfMismatch);
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

        msg2.verify_sign_and_cmac(&pub_key_a, &sp_pub_key, &smk_key)?;

        let mut nonce = QuoteNonce::default();
        Rng::new().fill_bytes(&mut nonce.rand);

        let mut report_data = ReportData::default();
        let mut sha = Sha256::new()?;
        sha.update(&pub_key_a)?;
        sha.update(&msg2.pub_key_b)?;
        sha.update(&*vk_key)?;
        let hash = sha.finalize()?;
        report_data.d[..SHA256_HASH_SIZE].copy_from_slice(&hash);
        let report = Report::for_target(qe_target, &report_data)?;

        let mut context = session.context.lock();
        ensure!(context.state == State::GaGened, SgxStatus::InvalidState);
        context.pub_key_b = msg2.pub_key_b;
        context.smk_key = *smk_key;
        context.sk_key = *sk_key;
        context.mk_key = *mk_key;
        context.vk_key = *vk_key;
        context.qe_target = *qe_target;
        context.quote_nonce = nonce;
        context.state = State::Msg2Proced;
        drop(context);

        Ok((report, nonce))
    }

    pub fn generate_msg3(&self, qe_report: &Report, quote: &[u8]) -> SgxResult<RaMsg3> {
        ensure!(qe_report.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(!quote.is_empty(), SgxStatus::InvalidParameter);
        ensure!(
            quote.is_enclave_range() || quote.is_host_range(),
            SgxStatus::InvalidParameter
        );
        ensure!(
            RaMsg3::check_quote_len(quote.len()),
            SgxStatus::InvalidParameter
        );

        let session = SESSION_MAGAGER
            .read()
            .find(self.rctx)
            .ok_or(SgxStatus::InvalidParameter)?;

        fence::lfence();

        qe_report.verify()?;

        let context = session.context.lock();
        ensure!(context.state == State::Msg2Proced, SgxStatus::InvalidState);
        let attributes = context.qe_target.attributes;
        let mr_enclave = context.qe_target.mr_enclave;
        let pub_key_a = context.pub_key_a;
        let ps_sec_prop = context.ps_sec_prop;
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

        let mut msg3 = RaMsg3 {
            mac: Default::default(),
            pub_key_a,
            ps_sec_prop,
            quote: quote.into(),
        };
        msg3.gen_cmac(&smk_key)?;

        Ok(msg3)
    }

    pub fn get_keys(&self, key_type: RaKeyType) -> SgxResult<AlignKey128bit> {
        let session = SESSION_MAGAGER
            .read()
            .find(self.rctx)
            .ok_or(SgxStatus::InvalidParameter)?;

        let context = session.context.lock();
        ensure!(context.state == State::Msg2Proced, SgxStatus::InvalidState);

        let key = match key_type {
            RaKeyType::SK => context.sk_key,
            RaKeyType::MK => context.mk_key,
        };

        Ok(key)
    }

    #[inline]
    pub fn into_raw(self) -> RaContext {
        let rctx = self.rctx;
        mem::forget(self);
        rctx
    }

    #[inline]
    pub unsafe fn from_raw(rctx: RaContext) -> Initiator {
        Self { rctx }
    }
}

impl Drop for Initiator {
    fn drop(&mut self) {
        if let Some(session) = SESSION_MAGAGER.write().remove(self.rctx) {
            drop(session)
        }
    }
}
