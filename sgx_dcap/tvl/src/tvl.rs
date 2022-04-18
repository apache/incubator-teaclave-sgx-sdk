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

use core::mem;
use sgx_crypto::sha::Sha256;
use sgx_trts::trts::{is_within_enclave, is_within_host, EnclaveRange};
use sgx_tse::EnclaveReport;
use sgx_types::error::{Quote3Error, SgxQuote3Result, SgxResult};
use sgx_types::types::SHA256_HASH_SIZE;
use sgx_types::types::{
    Attributes, AttributesFlags, Measurement, MiscSelect, QlQvResult, QuoteNonce, Report,
    Sha256Hash,
};

const QVE_MISC_SELECT: MiscSelect = MiscSelect::empty();
const QVE_MISC_SELECT_MASK: MiscSelect = unsafe { MiscSelect::from_bits_unchecked(0xFFFFFFFF) };

const QVE_ATTRIBUTE: Attributes = Attributes {
    flags: AttributesFlags::INITTED,
    xfrm: 0,
};
const QVE_ATTRIBUTE_MASK: Attributes = Attributes {
    flags: unsafe { AttributesFlags::from_bits_unchecked(0xFFFFFFFFFFFFFFFB) },
    xfrm: 0,
};

const QVE_MRSIGNER: Measurement = Measurement {
    m: [
        0x8c, 0x4f, 0x57, 0x75, 0xd7, 0x96, 0x50, 0x3e, 0x96, 0x13, 0x7f, 0x77, 0xc6, 0x8a, 0x82,
        0x9a, 0x00, 0x56, 0xac, 0x8d, 0xed, 0x70, 0x14, 0x0b, 0x08, 0x1b, 0x09, 0x44, 0x90, 0xc5,
        0x7b, 0xff,
    ],
};

const QVE_PROD_ID: u16 = 2;
const LEAST_QVE_ISVSVN: u16 = 5;

#[derive(Debug)]
pub struct QveReportInfo<'a, 'b> {
    pub qve_report: &'a Report,
    pub expiration_time: i64,
    pub collateral_expiration_status: u32,
    pub quote_verification_result: QlQvResult,
    pub qve_nonce: QuoteNonce,
    pub supplemental_data: Option<&'b [u8]>,
}

impl QveReportInfo<'_, '_> {
    pub fn verify_report_and_identity(
        &self,
        quote: &[u8],
        qve_isvsvn_threshold: u16,
    ) -> SgxQuote3Result {
        ensure!(!quote.is_empty(), Quote3Error::InvalidParameter);
        ensure!(quote.is_enclave_range(), Quote3Error::InvalidParameter);

        if let Some(supplemental) = self.supplemental_data {
            ensure!(!supplemental.is_empty(), Quote3Error::InvalidParameter);
            ensure!(
                supplemental.len() < u32::MAX as usize,
                Quote3Error::InvalidParameter
            );
        }
        ensure!(self.is_enclave_range(), Quote3Error::InvalidParameter);

        // Defense in depth, threshold must be greater or equal to 3
        if qve_isvsvn_threshold < LEAST_QVE_ISVSVN {
            bail!(Quote3Error::QveOutOfDate)
        }

        self.verify_report(quote)?;
        self.verify_identity(qve_isvsvn_threshold)
    }

    fn verify_report(&self, quote: &[u8]) -> SgxQuote3Result {
        self.qve_report
            .verify()
            .map_err(|_| Quote3Error::ErrorReport)?;

        let hash = self
            .calc_report_data(quote)
            .map_err(|_| Quote3Error::Unexpected)?;
        ensure!(
            hash.eq(&self.qve_report.body.report_data.d[..SHA256_HASH_SIZE]),
            Quote3Error::ErrorReport
        );

        Ok(())
    }

    fn verify_identity(&self, qve_isvsvn_threshold: u16) -> SgxQuote3Result {
        ensure!(
            self.qve_report.body.misc_select & QVE_MISC_SELECT_MASK == QVE_MISC_SELECT,
            Quote3Error::QveIdentityMismatch
        );
        ensure!(
            self.qve_report.body.attributes.flags & QVE_ATTRIBUTE_MASK.flags == QVE_ATTRIBUTE.flags,
            Quote3Error::QveIdentityMismatch
        );
        ensure!(
            self.qve_report.body.attributes.xfrm & QVE_ATTRIBUTE_MASK.xfrm == QVE_ATTRIBUTE.xfrm,
            Quote3Error::QveIdentityMismatch
        );
        ensure!(
            self.qve_report.body.mr_signer.eq(&QVE_MRSIGNER),
            Quote3Error::QveIdentityMismatch
        );
        ensure!(
            self.qve_report.body.isv_prod_id == QVE_PROD_ID,
            Quote3Error::QveIdentityMismatch
        );
        ensure!(
            self.qve_report.body.isv_svn >= qve_isvsvn_threshold,
            Quote3Error::QveOutOfDate
        );

        Ok(())
    }

    fn calc_report_data(&self, quote: &[u8]) -> SgxResult<Sha256Hash> {
        let mut sha = Sha256::new()?;
        sha.update(&self.qve_nonce)?;
        sha.update(quote)?;
        sha.update(&self.expiration_time)?;
        sha.update(&self.collateral_expiration_status)?;
        sha.update(&self.quote_verification_result)?;
        if let Some(supplemental) = self.supplemental_data {
            sha.update(supplemental)?;
        }
        sha.finalize()
    }
}

impl EnclaveRange for QveReportInfo<'_, '_> {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(
            self as *const _ as *const u8,
            mem::size_of::<QveReportInfo>(),
        ) {
            return false;
        }

        if let Some(supplemental) = self.supplemental_data {
            if !supplemental.is_enclave_range() {
                return false;
            }
        }
        self.qve_report.is_enclave_range()
    }

    fn is_host_range(&self) -> bool {
        if !is_within_host(
            self as *const _ as *const u8,
            mem::size_of::<QveReportInfo>(),
        ) {
            return false;
        }
        if let Some(supplemental) = self.supplemental_data {
            if !supplemental.is_host_range() {
                return false;
            }
        }
        self.qve_report.is_host_range()
    }
}
