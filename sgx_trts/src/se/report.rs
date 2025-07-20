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

use crate::arch::{Align128, Align256, Align512};
use crate::enclave::EnclaveRange;
use crate::inst::{self, EncluInst};
use crate::se::AlignKeyRequest;
use crate::sync::Once;
use core::convert::From;
use core::mem;
use core::ptr;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::ContiguousMemory;
use sgx_types::types::{
    Attributes, AttributesFlags, ConfigId, CpuSvn, Key128bit, KeyId, KeyName, KeyRequest, Mac,
    Measurement, MiscSelect, Report, Report2Mac, ReportBody, ReportData, TargetInfo,
};
use sgx_types::types::{
    CONFIGID_SIZE, CPUSVN_SIZE, HASH_SIZE, ISVEXT_PROD_ID_SIZE, ISV_FAMILY_ID_SIZE, KEYID_SIZE,
    MAC_SIZE, REPORT_BODY_RESERVED1_BYTES, REPORT_BODY_RESERVED2_BYTES,
    REPORT_BODY_RESERVED3_BYTES, REPORT_BODY_RESERVED4_BYTES, REPORT_DATA_SIZE,
};

#[repr(C, align(128))]
#[derive(Clone, Copy, Debug, Default)]
pub struct AlignReportData(pub ReportData);

#[repr(C, align(512))]
#[derive(Clone, Copy, Debug, Default)]
pub struct AlignTargetInfo(pub TargetInfo);

#[repr(C, align(512))]
#[derive(Clone, Copy, Debug, Default)]
pub struct AlignReport(pub Report);

#[repr(C, align(256))]
#[derive(Clone, Copy, Debug, Default)]
pub struct AlignReport2Mac(pub Report2Mac);

unsafe impl ContiguousMemory for AlignReportData {}
unsafe impl ContiguousMemory for AlignTargetInfo {}
unsafe impl ContiguousMemory for AlignReport {}
unsafe impl ContiguousMemory for AlignReport2Mac {}

static SELF_REPORT: Once = Once::new();
static mut REPORT: AlignReport = AlignReport(Report {
    body: ReportBody {
        cpu_svn: CpuSvn {
            svn: [0; CPUSVN_SIZE],
        },
        misc_select: MiscSelect::empty(),
        reserved1: [0; REPORT_BODY_RESERVED1_BYTES],
        isv_ext_prod_id: [0; ISVEXT_PROD_ID_SIZE],
        attributes: Attributes {
            flags: AttributesFlags::empty(),
            xfrm: 0,
        },
        mr_enclave: Measurement { m: [0; HASH_SIZE] },
        reserved2: [0; REPORT_BODY_RESERVED2_BYTES],
        mr_signer: Measurement { m: [0; HASH_SIZE] },
        reserved3: [0; REPORT_BODY_RESERVED3_BYTES],
        config_id: ConfigId {
            id: [0; CONFIGID_SIZE],
        },
        isv_prod_id: 0,
        isv_svn: 0,
        config_svn: 0,
        reserved4: [0; REPORT_BODY_RESERVED4_BYTES],
        isv_family_id: [0; ISV_FAMILY_ID_SIZE],
        report_data: ReportData {
            d: [0; REPORT_DATA_SIZE],
        },
    },
    key_id: KeyId {
        id: [0_u8; KEYID_SIZE],
    },
    mac: [0_u8; MAC_SIZE],
});

impl AlignReport {
    pub fn get_self() -> &'static AlignReport {
        unsafe {
            let _ = SELF_REPORT.call_once(|| {
                let report = AlignReport::for_self()?;
                REPORT = report;
                Ok(())
            });
            &REPORT
        }
    }

    pub fn for_self() -> SgxResult<AlignReport> {
        let report_data = AlignReportData::default();
        let target_info = AlignTargetInfo::default();
        EncluInst::ereport(&target_info, &report_data).map_err(|_| SgxStatus::Unexpected)
    }

    pub fn for_target(
        target_info: &AlignTargetInfo,
        report_data: &AlignReportData,
    ) -> SgxResult<AlignReport> {
        ensure!(
            target_info.is_enclave_range() && report_data.is_enclave_range(),
            SgxStatus::InvalidParameter
        );
        EncluInst::ereport(target_info, report_data).map_err(|_| SgxStatus::Unexpected)
    }

    // This function verifies the report's MAC using the provided
    // implementation of the verifying function.
    //
    // Care should be taken that `check_mac` prevents timing attacks,
    // in particular that the comparison happens in constant time.
    pub fn verify<F>(&self, check_mac: F) -> SgxResult
    where
        F: FnOnce(&Key128bit, &[u8; AlignReport::TRUNCATED_SIZE], &Mac) -> SgxResult,
    {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);

        let req = AlignKeyRequest(KeyRequest {
            key_name: KeyName::Report,
            key_id: self.0.key_id,
            ..Default::default()
        });
        let key = req.egetkey()?;
        check_mac(&key.key, self.mac_data(), &self.0.mac)
    }

    // Returns that part of the `Report` that is MACed.
    pub fn mac_data(&self) -> &[u8; AlignReport::TRUNCATED_SIZE] {
        unsafe { &*(self as *const Self as *const [u8; AlignReport::TRUNCATED_SIZE]) }
    }
}

impl AlignReport2Mac {
    pub fn verify(&self) -> SgxResult {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);

        EncluInst::everify_report2(self).map_err(|e| match e {
            inst::INVALID_REPORTMACSTRUCT => SgxStatus::MacMismatch,
            inst::INVALID_CPUSVN => SgxStatus::InvalidCpusvn,
            inst::INVALID_LEAF => SgxStatus::UnsupportedFeature,
            _ => SgxStatus::Unexpected,
        })
    }
}

impl AlignTargetInfo {
    pub const UNPADDED_SIZE: usize = mem::size_of::<TargetInfo>();
    pub const ALIGN_SIZE: usize = mem::size_of::<AlignTargetInfo>();

    pub fn try_copy_from(src: &[u8]) -> Option<AlignTargetInfo> {
        if src.len() == Self::UNPADDED_SIZE {
            unsafe {
                let mut ret: Self = mem::zeroed();
                ptr::copy_nonoverlapping(
                    src.as_ptr(),
                    &mut ret as *mut _ as *mut _,
                    Self::UNPADDED_SIZE,
                );
                Some(ret)
            }
        } else {
            None
        }
    }
}

impl AlignReportData {
    pub const UNPADDED_SIZE: usize = mem::size_of::<ReportData>();
    pub const ALIGN_SIZE: usize = mem::size_of::<AlignReportData>();

    pub fn try_copy_from(src: &[u8]) -> Option<AlignReportData> {
        if src.len() == Self::UNPADDED_SIZE {
            unsafe {
                let mut ret: Self = mem::zeroed();
                ptr::copy_nonoverlapping(
                    src.as_ptr(),
                    &mut ret as *mut _ as *mut _,
                    Self::UNPADDED_SIZE,
                );
                Some(ret)
            }
        } else {
            None
        }
    }
}

impl AlignReport {
    pub const UNPADDED_SIZE: usize = mem::size_of::<Report>();
    pub const ALIGN_SIZE: usize = mem::size_of::<AlignReport>();
    pub const TRUNCATED_SIZE: usize = mem::size_of::<ReportBody>();

    pub fn try_copy_from(src: &[u8]) -> Option<AlignReport> {
        if src.len() == Self::UNPADDED_SIZE {
            unsafe {
                let mut ret: Self = mem::zeroed();
                ptr::copy_nonoverlapping(
                    src.as_ptr(),
                    &mut ret as *mut _ as *mut _,
                    Self::UNPADDED_SIZE,
                );
                Some(ret)
            }
        } else {
            None
        }
    }
}

impl AlignReport2Mac {
    pub const UNPADDED_SIZE: usize = mem::size_of::<Report2Mac>();
    pub const ALIGN_SIZE: usize = mem::size_of::<AlignReport2Mac>();

    pub fn try_copy_from(src: &[u8]) -> Option<AlignReport2Mac> {
        if src.len() == Self::UNPADDED_SIZE {
            unsafe {
                let mut ret: Self = mem::zeroed();
                ptr::copy_nonoverlapping(
                    src.as_ptr(),
                    &mut ret as *mut _ as *mut _,
                    Self::UNPADDED_SIZE,
                );
                Some(ret)
            }
        } else {
            None
        }
    }
}

impl AsRef<Align512<[u8; AlignTargetInfo::UNPADDED_SIZE]>> for AlignTargetInfo {
    fn as_ref(&self) -> &Align512<[u8; AlignTargetInfo::UNPADDED_SIZE]> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl AsRef<Align128<[u8; AlignReportData::UNPADDED_SIZE]>> for AlignReportData {
    fn as_ref(&self) -> &Align128<[u8; AlignReportData::UNPADDED_SIZE]> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl AsRef<Align512<[u8; AlignReport::UNPADDED_SIZE]>> for AlignReport {
    fn as_ref(&self) -> &Align512<[u8; AlignReport::UNPADDED_SIZE]> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl AsRef<Align256<[u8; AlignReport2Mac::UNPADDED_SIZE]>> for AlignReport2Mac {
    fn as_ref(&self) -> &Align256<[u8; AlignReport2Mac::UNPADDED_SIZE]> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl From<AlignReport> for AlignTargetInfo {
    fn from(r: AlignReport) -> AlignTargetInfo {
        AlignTargetInfo(TargetInfo {
            mr_enclave: r.0.body.mr_enclave,
            attributes: r.0.body.attributes,
            config_svn: r.0.body.config_svn,
            misc_select: r.0.body.misc_select,
            config_id: r.0.body.config_id,
            ..TargetInfo::default()
        })
    }
}

impl From<Report> for AlignTargetInfo {
    fn from(r: Report) -> AlignTargetInfo {
        AlignTargetInfo(TargetInfo {
            mr_enclave: r.body.mr_enclave,
            attributes: r.body.attributes,
            config_svn: r.body.config_svn,
            misc_select: r.body.misc_select,
            config_id: r.body.config_id,
            ..TargetInfo::default()
        })
    }
}

impl From<Report> for AlignReport {
    fn from(r: Report) -> AlignReport {
        AlignReport(r)
    }
}

impl From<&Report> for AlignReport {
    fn from(r: &Report) -> AlignReport {
        AlignReport(*r)
    }
}

impl From<AlignReport> for Report {
    fn from(r: AlignReport) -> Report {
        r.0
    }
}

impl From<TargetInfo> for AlignTargetInfo {
    fn from(t: TargetInfo) -> AlignTargetInfo {
        AlignTargetInfo(t)
    }
}

impl From<&TargetInfo> for AlignTargetInfo {
    fn from(t: &TargetInfo) -> AlignTargetInfo {
        AlignTargetInfo(*t)
    }
}

impl From<AlignTargetInfo> for TargetInfo {
    fn from(t: AlignTargetInfo) -> TargetInfo {
        t.0
    }
}

impl From<ReportData> for AlignReportData {
    fn from(d: ReportData) -> AlignReportData {
        AlignReportData(d)
    }
}

impl From<&ReportData> for AlignReportData {
    fn from(d: &ReportData) -> AlignReportData {
        AlignReportData(*d)
    }
}

impl From<AlignReportData> for ReportData {
    fn from(d: AlignReportData) -> ReportData {
        d.0
    }
}

impl From<Report2Mac> for AlignReport2Mac {
    fn from(r: Report2Mac) -> AlignReport2Mac {
        AlignReport2Mac(r)
    }
}

impl From<&Report2Mac> for AlignReport2Mac {
    fn from(r: &Report2Mac) -> AlignReport2Mac {
        AlignReport2Mac(*r)
    }
}
