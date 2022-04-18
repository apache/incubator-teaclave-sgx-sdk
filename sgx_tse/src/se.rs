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
use core::ptr;
use sgx_crypto::mac::AesCMac;
use sgx_trts::fence::lfence;
use sgx_trts::se::{
    AlignKeyRequest, AlignReport, AlignReport2Mac, AlignReportData, AlignTargetInfo,
};
use sgx_trts::trts::EnclaveRange;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::memeq::ConstTimeEq;
use sgx_types::types::{
    AlignKey128bit, Key128bit, KeyRequest, Report, Report2Mac, ReportData, TargetInfo,
};

pub trait EnclaveReport: Sized {
    type Error;

    fn for_target(target_info: &TargetInfo, report_data: &ReportData) -> Result<Self, Self::Error>;

    fn for_self() -> Result<Self, Self::Error>;

    fn get_self() -> &'static Self;

    fn verify(&self) -> Result<(), Self::Error>;

    fn to_target(&self) -> Result<TargetInfo, Self::Error>;
}

pub trait EnclaveKey {
    type Error;

    fn get_key(&self) -> Result<Key128bit, Self::Error> {
        self.get_align_key().map(|key| key.key)
    }

    fn get_align_key(&self) -> Result<AlignKey128bit, Self::Error>;
}

pub trait EnclaveTarget: Sized {
    type Error;

    fn for_self() -> Result<Self, Self::Error>;
}

pub trait TeeReport: Sized {
    type Error;

    fn verify(&self) -> Result<(), Self::Error>;
}

impl EnclaveReport for Report {
    type Error = SgxStatus;

    #[inline]
    fn get_self() -> &'static Report {
        &AlignReport::get_self().0
    }

    #[inline]
    fn for_self() -> SgxResult<Report> {
        AlignReport::for_self().map(|report| report.into())
    }

    #[inline]
    fn for_target(target_info: &TargetInfo, report_data: &ReportData) -> SgxResult<Report> {
        ensure!(
            target_info.is_enclave_range() && report_data.is_enclave_range(),
            SgxStatus::InvalidParameter
        );

        AlignReport::for_target(
            &AlignTargetInfo::from(target_info),
            &AlignReportData::from(report_data),
        )
        .map(|report| report.into())
    }

    #[inline]
    fn verify(&self) -> SgxResult {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);

        let report = AlignReport::from(self);
        report.verify(|key, body, mac| {
            let report_mac = AesCMac::cmac(key, body)?;
            if report_mac.ct_eq(mac) {
                Ok(())
            } else {
                Err(SgxStatus::MacMismatch)
            }
        })
    }

    #[inline]
    fn to_target(&self) -> SgxResult<TargetInfo> {
        LAV2_PROTO_SPEC.make_target_info(self)
    }
}

impl EnclaveKey for KeyRequest {
    type Error = SgxStatus;

    #[inline]
    fn get_align_key(&self) -> SgxResult<AlignKey128bit> {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);

        let req = AlignKeyRequest::from(self);
        req.egetkey()
    }

    #[inline]
    fn get_key(&self) -> SgxResult<Key128bit> {
        self.get_align_key().map(|key| key.key)
    }
}

impl EnclaveTarget for TargetInfo {
    type Error = SgxStatus;

    #[inline]
    fn for_self() -> SgxResult<TargetInfo> {
        Report::get_self().to_target()
    }
}

impl TeeReport for Report2Mac {
    type Error = SgxStatus;

    #[inline]
    fn verify(&self) -> Result<(), Self::Error> {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);

        let report = AlignReport2Mac::from(self);
        report.verify()
    }
}

const LAV2_PROTO_SPEC: LAv2ProtoSpec = LAv2ProtoSpec {
    signature: [0x53, 0x47, 0x58, 0x20, 0x4C, 0x41], // "SGX LA"
    ver: 2,
    rev: 0,
    target_spec: [
        0x0600, // target_spec count & revision
        0x0405, // MRENCLAVE
        0x0304, // ATTRIBUTES
        0x0140, // CET_ATTRIBUTES
        0x1041, // CONFIGSVN
        0x0102, // MISCSELECT
        0x0C06, // CONFIGID
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
};

#[allow(dead_code)]
#[derive(Clone, Copy, Default, Debug)]
struct LAv2ProtoSpec {
    signature: [u8; 6],
    ver: u8,
    rev: u8,
    target_spec: [u16; 28],
}

impl LAv2ProtoSpec {
    fn make_target_info(&self, report: &Report) -> SgxResult<TargetInfo> {
        ensure!(self.is_valid(), SgxStatus::InvalidParameter);

        let mut ti = TargetInfo::default();
        let d = &mut ti as *mut TargetInfo as *mut u8;
        let f = report as *const Report as *const u8;

        lfence();

        let mut to = 0_i32;
        for i in 1..(self.ts_count() + 1) as usize {
            let size = (1 << (self.target_spec[i] & 0xF)) as i32;
            to += size - 1;
            to &= -size;

            ensure!(
                (to + size) as usize <= mem::size_of::<TargetInfo>(),
                SgxStatus::Unexpected
            );

            let from = (self.target_spec[i] >> 4) as i32;
            if from >= 0 {
                ensure!(
                    (from + size) as usize <= mem::size_of::<Report>(),
                    SgxStatus::Unexpected
                );

                unsafe {
                    ptr::copy_nonoverlapping(
                        f.offset(from as isize),
                        d.offset(to as isize),
                        size as usize,
                    );
                }
            } else if from == -1 {
                break;
            } else {
                bail!(SgxStatus::Unexpected);
            }
            to += size;
        }
        Ok(ti)
    }

    fn ts_count(&self) -> u16 {
        self.target_spec[0] >> 8
    }

    fn is_valid(&self) -> bool {
        self.ver == 2 && self.rev == 0 && self.target_spec[0] as u8 == 0 && self.ts_count() < 28
    }
}
