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

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem;
use core::ptr;
use core::slice;
use sgx_crypto::ecc::EcPublicKey;
use sgx_crypto::mac::AesCMac;
use sgx_crypto::sha::Sha256;
use sgx_trts::trts::{is_within_enclave, is_within_host, EnclaveRange};
use sgx_tse::EnclaveReport;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::memeq::ConstTimeEq;
use sgx_types::types::SHA256_HASH_SIZE;
use sgx_types::types::{
    AlignKey128bit, CDhMsg1, CDhMsg2, CDhMsg3, Mac128bit, Report, ReportData, TargetInfo,
};

#[cfg(feature = "serialize")]
use sgx_serialize::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
pub struct DhMsg1 {
    pub pub_key_a: EcPublicKey,
    pub target: TargetInfo,
}

impl_struct_ContiguousMemory! {
    DhMsg1;
}
impl_asref_array! {
    DhMsg1;
}

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
pub struct DhMsg2 {
    pub pub_key_b: EcPublicKey,
    pub report: Report,
    pub cmac: Mac128bit,
}

impl_struct_ContiguousMemory! {
    DhMsg2;
}
impl_asref_array! {
    DhMsg2;
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
pub struct DhMsg3 {
    pub cmac: Mac128bit,
    pub report: Report,
    pub add_prop: Option<Box<[u8]>>,
}

impl DhMsg1 {
    pub(crate) fn new(pub_key: &EcPublicKey) -> SgxResult<DhMsg1> {
        let report = Report::for_self()?;
        let target = report.to_target()?;

        Ok(DhMsg1 {
            pub_key_a: *pub_key,
            target,
        })
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let raw_msg: CDhMsg1 = self.into();
        raw_msg.as_ref().to_vec()
    }

    #[inline]
    pub fn from_bytes(bytes: Vec<u8>) -> SgxResult<DhMsg1> {
        ensure!(
            is_within_enclave(bytes.as_ptr(), bytes.capacity()),
            SgxStatus::InvalidParameter
        );
        Self::from_slice(bytes.as_slice())
    }

    pub fn from_slice(bytes: &[u8]) -> SgxResult<DhMsg1> {
        ensure!(bytes.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(
            bytes.len() == mem::size_of::<CDhMsg1>(),
            SgxStatus::InvalidParameter
        );

        let raw_msg = unsafe { &*(bytes.as_ptr() as *const CDhMsg1) };
        Ok(raw_msg.into())
    }
}

impl DhMsg2 {
    const AES_CMAC_KDF_ID: [u8; 2] = [1, 0];

    pub(crate) fn new_with_lav1(
        msg1: &DhMsg1,
        pub_key_b: &EcPublicKey,
        smk: &AlignKey128bit,
    ) -> SgxResult<DhMsg2> {
        let mut sha = Sha256::new()?;
        sha.update(&msg1.pub_key_a)?;
        sha.update(pub_key_b)?;
        let hash = sha.finalize()?;

        let mut report_data = ReportData::default();
        report_data.d[..SHA256_HASH_SIZE].copy_from_slice(&hash);
        report_data.d[SHA256_HASH_SIZE..SHA256_HASH_SIZE + 2]
            .copy_from_slice(&Self::AES_CMAC_KDF_ID);

        let report = Report::for_target(&msg1.target, &report_data)?;
        let cmac = AesCMac::cmac(&smk.key, &report)?;

        Ok(DhMsg2 {
            pub_key_b: *pub_key_b,
            report,
            cmac,
        })
    }

    pub(crate) fn new_with_lav2(
        msg1: &DhMsg1,
        pub_key_b: &EcPublicKey,
        smk: &AlignKey128bit,
    ) -> SgxResult<DhMsg2> {
        let mut sha = Sha256::new()?;
        sha.update(&LAV2_PROTO_SPEC)?;
        sha.update(pub_key_b)?;
        let hash = sha.finalize()?;

        let mut report_data = ReportData::default();
        report_data.d[..SHA256_HASH_SIZE].copy_from_slice(&hash);

        let mut report = Report::for_target(&msg1.target, &report_data)?;
        report.body.report_data = LAV2_PROTO_SPEC.to_report_data();

        let cmac = AesCMac::cmac(&smk.key, pub_key_b)?;

        Ok(DhMsg2 {
            pub_key_b: *pub_key_b,
            report,
            cmac,
        })
    }

    pub(crate) fn verify_with_lav1(
        &self,
        pub_key_a: &EcPublicKey,
        smk: &AlignKey128bit,
    ) -> SgxResult {
        let hash = &self.report.body.report_data.d[..SHA256_HASH_SIZE];
        let kdf_id = &self.report.body.report_data.d[SHA256_HASH_SIZE..SHA256_HASH_SIZE + 2];
        ensure!(kdf_id.eq(&Self::AES_CMAC_KDF_ID), SgxStatus::KdfMismatch);

        let cmac = AesCMac::cmac(&smk.key, &self.report)?;
        ensure!(cmac.ct_eq(&self.cmac), SgxStatus::MacMismatch);

        self.report.verify()?;

        let mut sha = Sha256::new()?;
        sha.update(pub_key_a)?;
        sha.update(&self.pub_key_b)?;
        let msg_hash = sha.finalize()?;
        ensure!(msg_hash.eq(&hash), SgxStatus::MacMismatch);

        Ok(())
    }

    pub(crate) fn verify_with_lav2(&self, smk: &AlignKey128bit) -> SgxResult {
        let mut sha = Sha256::new()?;
        sha.update(&self.report.body.report_data)?;
        sha.update(&self.pub_key_b)?;
        let hash = sha.finalize()?;

        let mut report_data = ReportData::default();
        report_data.d[..SHA256_HASH_SIZE].copy_from_slice(&hash);

        let mut report = self.report;
        report.body.report_data = report_data;
        report.verify()?;

        let cmac = AesCMac::cmac(&smk.key, &self.pub_key_b)?;
        ensure!(cmac.ct_eq(&self.cmac), SgxStatus::MacMismatch);

        let proto_spec = LAv2ProtoSpec::from_report_data(&self.report.body.report_data);
        ensure!(
            proto_spec.signature.eq(&LAV2_PROTO_SPEC.signature)
                && proto_spec.rev == LAV2_PROTO_SPEC.rev,
            SgxStatus::Unexpected
        );

        Ok(())
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let raw_msg: CDhMsg2 = self.into();
        raw_msg.as_ref().to_vec()
    }

    #[inline]
    pub fn from_bytes(bytes: Vec<u8>) -> SgxResult<DhMsg2> {
        ensure!(
            is_within_enclave(bytes.as_ptr(), bytes.capacity()),
            SgxStatus::InvalidParameter
        );
        Self::from_slice(bytes.as_slice())
    }

    pub fn from_slice(bytes: &[u8]) -> SgxResult<DhMsg2> {
        ensure!(bytes.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(
            bytes.len() == mem::size_of::<DhMsg2>(),
            SgxStatus::InvalidParameter
        );

        let raw_msg = unsafe { &*(bytes.as_ptr() as *const CDhMsg2) };
        Ok(raw_msg.into())
    }
}

impl DhMsg3 {
    pub(crate) fn new_with_lav1(
        msg2: &DhMsg2,
        pub_key_a: &EcPublicKey,
        smk: &AlignKey128bit,
        add_prop: Option<&[u8]>,
    ) -> SgxResult<DhMsg3> {
        let add_prop_len = add_prop.map_or(0, |add| add.len());
        ensure!(
            add_prop_len <= u32::MAX as usize - mem::size_of::<CDhMsg3>(),
            SgxStatus::InvalidParameter
        );

        let mut sha = Sha256::new()?;
        sha.update(&msg2.pub_key_b)?;
        sha.update(pub_key_a)?;
        let hash = sha.finalize()?;

        let mut report_data = ReportData::default();
        report_data.d[..SHA256_HASH_SIZE].copy_from_slice(&hash);

        let target = msg2.report.to_target()?;
        let report = Report::for_target(&target, &report_data)?;

        let mut mac = AesCMac::new(&smk.key)?;
        mac.update(&report)?;
        mac.update(&add_prop_len)?;
        if add_prop_len > 0 {
            mac.update(add_prop.unwrap())?;
        }
        let cmac = mac.finalize()?;

        Ok(DhMsg3 {
            cmac,
            report,
            add_prop: add_prop.map(Box::from),
        })
    }

    pub(crate) fn new_with_lav2(
        msg2: &DhMsg2,
        pub_key_a: &EcPublicKey,
        smk: &AlignKey128bit,
        add_prop: Option<&[u8]>,
    ) -> SgxResult<DhMsg3> {
        let add_prop_len = add_prop.map_or(0, |add| add.len());
        ensure!(
            add_prop_len <= u32::MAX as usize - mem::size_of::<CDhMsg3>(),
            SgxStatus::InvalidParameter
        );

        let proto_spec = LAv2ProtoSpec::from_report_data(&msg2.report.body.report_data);
        let mut sha = Sha256::new()?;
        sha.update(pub_key_a)?;
        sha.update(&proto_spec)?;
        let hash = sha.finalize()?;

        let mut report_data = ReportData::default();
        report_data.d[..SHA256_HASH_SIZE].copy_from_slice(&hash);

        let target = msg2.report.to_target()?;
        let report = Report::for_target(&target, &report_data)?;

        let mut mac = AesCMac::new(&smk.key)?;
        if add_prop_len > 0 {
            mac.update(add_prop.unwrap())?;
        }
        mac.update(pub_key_a)?;
        let cmac = mac.finalize()?;

        Ok(DhMsg3 {
            cmac,
            report,
            add_prop: add_prop.map(Box::from),
        })
    }

    pub(crate) fn verify_with_lav1(
        &self,
        pub_key_a: &EcPublicKey,
        pub_key_b: &EcPublicKey,
        smk: &AlignKey128bit,
    ) -> SgxResult {
        let add_prop_len = self.add_prop.as_ref().map_or(0, |add| add.len());
        ensure!(
            add_prop_len <= u32::MAX as usize - mem::size_of::<CDhMsg3>(),
            SgxStatus::Unexpected
        );

        let mut mac = AesCMac::new(&smk.key)?;
        mac.update(&self.report)?;
        mac.update(&add_prop_len)?;
        if add_prop_len > 0 {
            mac.update(&**(self.add_prop.as_ref().unwrap()))?;
        }
        let cmac = mac.finalize()?;
        ensure!(cmac.ct_eq(&self.cmac), SgxStatus::MacMismatch);

        self.report.verify()?;

        let hash = &self.report.body.report_data.d[..SHA256_HASH_SIZE];
        let mut sha = Sha256::new()?;
        sha.update(pub_key_b)?;
        sha.update(pub_key_a)?;
        let msg_hash = sha.finalize()?;
        ensure!(msg_hash.eq(&hash), SgxStatus::MacMismatch);

        Ok(())
    }

    pub(crate) fn verify_with_lav2(
        &self,
        pub_key_a: &EcPublicKey,
        _pub_key_b: &EcPublicKey,
        smk: &AlignKey128bit,
    ) -> SgxResult {
        let add_prop_len = self.add_prop.as_ref().map_or(0, |add| add.len());
        ensure!(
            add_prop_len <= u32::MAX as usize - mem::size_of::<CDhMsg3>(),
            SgxStatus::Unexpected
        );

        let mut sha = Sha256::new()?;
        sha.update(pub_key_a)?;
        sha.update(&LAV2_PROTO_SPEC)?;
        let msg_hash = sha.finalize()?;

        let mut report = self.report;
        report.body.report_data = ReportData::default();
        report.body.report_data.d[..SHA256_HASH_SIZE].copy_from_slice(&msg_hash);
        ensure!(
            report
                .body
                .report_data
                .d
                .eq(&self.report.body.report_data.d),
            SgxStatus::Unexpected
        );

        report.verify()?;

        let mut mac = AesCMac::new(&smk.key)?;
        if add_prop_len > 0 {
            mac.update(&**(self.add_prop.as_ref().unwrap()))?;
        }
        mac.update(pub_key_a)?;
        let cmac = mac.finalize()?;
        ensure!(cmac.ct_eq(&self.cmac), SgxStatus::MacMismatch);

        Ok(())
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let raw_len = self.get_raw_ize();

        let mut bytes = vec![0_u8; raw_len as usize];
        let header_len = mem::size_of::<CDhMsg3>() as u32;

        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CDhMsg3) };
        raw_msg.cmac = self.cmac;
        raw_msg.msg_body.report = self.report;
        raw_msg.msg_body.add_prop_len = raw_len - header_len;
        if let Some(add) = self.add_prop {
            bytes[header_len as usize..].copy_from_slice(&add);
        }
        bytes
    }

    #[inline]
    pub fn from_bytes(bytes: Vec<u8>) -> SgxResult<DhMsg3> {
        ensure!(
            is_within_enclave(bytes.as_ptr(), bytes.capacity()),
            SgxStatus::InvalidParameter
        );
        Self::from_slice(bytes.as_slice())
    }

    pub fn from_slice(bytes: &[u8]) -> SgxResult<DhMsg3> {
        ensure!(bytes.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(
            bytes.len() >= mem::size_of::<CDhMsg3>(),
            SgxStatus::InvalidParameter
        );

        let header_len = mem::size_of::<CDhMsg3>() as u32;
        let raw_msg = unsafe { &*(bytes.as_ptr() as *const CDhMsg3) };

        ensure!(
            (raw_msg.msg_body.add_prop_len <= u32::MAX - header_len)
                && (bytes.len() == (header_len + raw_msg.msg_body.add_prop_len) as usize),
            SgxStatus::InvalidParameter
        );

        let add_prop_len = raw_msg.msg_body.add_prop_len as usize;
        let add_prop = if add_prop_len > 0 {
            let mut add_prop = vec![0_u8; add_prop_len];
            add_prop.as_mut_slice().copy_from_slice(unsafe {
                slice::from_raw_parts(
                    &raw_msg.msg_body.add_prop as *const _ as *const u8,
                    add_prop_len,
                )
            });
            Some(add_prop.into_boxed_slice())
        } else {
            None
        };

        Ok(DhMsg3 {
            cmac: raw_msg.cmac,
            report: raw_msg.msg_body.report,
            add_prop,
        })
    }

    pub fn get_raw_ize(&self) -> u32 {
        let dh_msg3_len = mem::size_of::<CDhMsg3>();
        let add_prop_len = self.add_prop.as_ref().map_or(0, |add| add.len());

        if add_prop_len <= (u32::MAX as usize) - dh_msg3_len {
            (dh_msg3_len + add_prop_len) as u32
        } else {
            u32::MAX
        }
    }
}

impl EnclaveRange for DhMsg3 {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(self as *const _ as *const u8, mem::size_of::<DhMsg3>()) {
            return false;
        }

        let add_prop = self.add_prop.as_ref();
        let (ptr, len) = add_prop.map_or((ptr::null(), 0), |add| {
            if !add.is_empty() {
                (add.as_ptr(), add.len())
            } else {
                (ptr::null(), 0)
            }
        });
        if len > 0 && !is_within_enclave(ptr, len) {
            return false;
        }

        true
    }

    fn is_host_range(&self) -> bool {
        if !is_within_host(self as *const _ as *const u8, mem::size_of::<DhMsg3>()) {
            return false;
        }

        let add_prop = self.add_prop.as_ref();
        let (ptr, len) = add_prop.map_or((ptr::null(), 0), |add| {
            if !add.is_empty() {
                (add.as_ptr(), add.len())
            } else {
                (ptr::null(), 0)
            }
        });
        if len > 0 && !is_within_host(ptr, len) {
            return false;
        }

        true
    }
}

impl From<DhMsg1> for CDhMsg1 {
    fn from(msg: DhMsg1) -> CDhMsg1 {
        CDhMsg1 {
            g_a: msg.pub_key_a.into(),
            target: msg.target,
        }
    }
}

impl From<&DhMsg1> for CDhMsg1 {
    fn from(msg: &DhMsg1) -> CDhMsg1 {
        CDhMsg1 {
            g_a: msg.pub_key_a.into(),
            target: msg.target,
        }
    }
}

impl From<CDhMsg1> for DhMsg1 {
    fn from(msg: CDhMsg1) -> DhMsg1 {
        DhMsg1 {
            pub_key_a: msg.g_a.into(),
            target: msg.target,
        }
    }
}

impl From<&CDhMsg1> for DhMsg1 {
    fn from(msg: &CDhMsg1) -> DhMsg1 {
        DhMsg1 {
            pub_key_a: msg.g_a.into(),
            target: msg.target,
        }
    }
}

impl From<DhMsg2> for CDhMsg2 {
    fn from(msg: DhMsg2) -> CDhMsg2 {
        CDhMsg2 {
            g_b: msg.pub_key_b.into(),
            report: msg.report,
            cmac: msg.cmac,
        }
    }
}

impl From<&DhMsg2> for CDhMsg2 {
    fn from(msg: &DhMsg2) -> CDhMsg2 {
        CDhMsg2 {
            g_b: msg.pub_key_b.into(),
            report: msg.report,
            cmac: msg.cmac,
        }
    }
}

impl From<CDhMsg2> for DhMsg2 {
    fn from(msg: CDhMsg2) -> DhMsg2 {
        DhMsg2 {
            pub_key_b: msg.g_b.into(),
            report: msg.report,
            cmac: msg.cmac,
        }
    }
}

impl From<&CDhMsg2> for DhMsg2 {
    fn from(msg: &CDhMsg2) -> DhMsg2 {
        DhMsg2 {
            pub_key_b: msg.g_b.into(),
            report: msg.report,
            cmac: msg.cmac,
        }
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
    #[inline]
    fn to_report_data(self) -> ReportData {
        unsafe { mem::transmute::<LAv2ProtoSpec, ReportData>(self) }
    }

    #[inline]
    fn from_report_data(report_data: &ReportData) -> LAv2ProtoSpec {
        unsafe { mem::transmute::<ReportData, LAv2ProtoSpec>(*report_data) }
    }
}

impl_struct_ContiguousMemory! {
    LAv2ProtoSpec;
}
