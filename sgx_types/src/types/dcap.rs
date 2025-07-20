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

use super::*;

use core::mem;
use core::slice;

//
// sgx_pce.h
//
/* PCE ID for the PCE in this library */
pub const PCE_ID: u16 = 0;
pub const PCE_ALG_RSA_OAEP_3072: u8 = 1;
pub const PCE_NIST_P256_ECDSA_SHA256: u8 = 0;

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum QlRequestPolicy {
        Persistent = 0,
        Ephemeral = 1,
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub struct PceInfo {
    pub isv_svn: u16,
    pub pce_id: u16,
}

//
// sgx_ql_lib_common.h
//
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub struct QlQe3Id {
    pub id: [u8; 16],
}

impl_asref_array! {
    PceInfo;
    QlQe3Id;
}

impl_asmut_array! {
    QlQe3Id;
}

impl_from_array! {
    QlQe3Id;
}

impl_struct_ContiguousMemory! {
    PceInfo;
    QlQe3Id;
}

impl_unsafe_marker_for! {
    BytewiseEquality,
    PceInfo QlQe3Id
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum QlConfigVersion {
        QlConfigVersion1 = 0,
    }
}

#[repr(C, packed)]
pub struct CQlPckCertId {
    pub p_qe3_id: *mut u8,
    pub qe3_id_size: u32,
    pub p_platform_cpu_svn: *mut CpuSvn,
    pub p_platform_pce_isv_svn: *mut u16,
    pub p_encrypted_ppid: *mut u8,
    pub encrypted_ppid_size: u32,
    pub crypto_suite: u8,
    pub pce_id: u16,
}

#[repr(C, packed)]
pub struct CQlConfig {
    pub version: QlConfigVersion,
    pub cert_cpu_svn: CpuSvn,
    pub cert_pce_isv_svn: u16,
    pub cert_data_size: u32,
    pub p_cert_data: *mut u8,
}

/* intel DCAP 1.13 */
pub const MAX_PARAM_STRING_SIZE: usize = 256;
impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct QlQveCollateralParam {
        key: [u8; MAX_PARAM_STRING_SIZE + 1],
        value: [u8; MAX_PARAM_STRING_SIZE + 1]
    }
}

impl_struct_default! {
    QlQveCollateralParam; //514
}

impl_asref_array! {
    QlQveCollateralParam;
}

impl_struct_ContiguousMemory! {
    QlQveCollateralParam;
}

#[repr(C)]
pub struct CQlQveCollateral {
    pub version: u32, // version = 1.  PCK Cert chain is in the Quote.
    /* intel DCAP 1.13 */
    pub tee_type: u32, // 0x00000000: SGX or 0x00000081: TDX
    pub pck_crl_issuer_chain: *mut c_char,
    pub pck_crl_issuer_chain_size: u32,
    pub root_ca_crl: *mut c_char, // Root CA CRL
    pub root_ca_crl_size: u32,
    pub pck_crl: *mut c_char, // PCK Cert CRL
    pub pck_crl_size: u32,
    pub tcb_info_issuer_chain: *mut c_char,
    pub tcb_info_issuer_chain_size: u32,
    pub tcb_info: *mut c_char, // TCB Info structure
    pub tcb_info_size: u32,
    pub qe_identity_issuer_chain: *mut c_char,
    pub qe_identity_issuer_chain_size: u32,
    pub qe_identity: *mut c_char, // QE Identity Structure
    pub qe_identity_size: u32,
}

/* intel DCAP 1.14 */
impl_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ProdType {
        SGX = 0,
        TDX = 1,
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum QlLogLevel {
        LogError = 0,
        LogInfo = 1,
    }
}

pub type QlLoggingCallback = extern "C" fn(level: QlLogLevel, message: *const c_char);

//
// sgx_quote_3.h
//
pub const REF_QUOTE_MAX_AUTHENTICATON_DATA_SIZE: u16 = 64;

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum QlAttestationAlgorithmId {
        Epid = 0,
        Reserved1 = 1,
        EcdsaP256 = 2,
        EcdsaP384 = 3,
        Max = 4,
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum QlCertKeyType {
        PPIDClearText = 1,
        PPIDRsa2048Encrypted = 2,
        PPIDRsa3072Encrypted = 3,
        PCKClearText = 4,
        EcdsaSigAuxData = 6,
        QlCertKeyTypeMax = 16,
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct QlPPIDRsa3072EncryptedCertInfo {
    pub enc_ppid: [u8; 384],
    pub cpu_svn: CpuSvn,
    pub pce_info: PceInfo,
}

impl_struct_default! {
    QlPPIDRsa3072EncryptedCertInfo; //404
}

impl_asref_array! {
    QlPPIDRsa3072EncryptedCertInfo;
}

impl_struct_ContiguousMemory! {
    QlPPIDRsa3072EncryptedCertInfo;
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct QlAuthData {
    pub size: u16,
    pub auth_data: [u8; 0],
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct QlCertificationData {
    pub cert_key_type: u16,
    pub size: u32,
    pub certification_data: [u8; 0],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct QlEcdsaSigData {
    pub sig: [u8; 64],
    pub attest_pub_key: [u8; 64],
    pub qe_report: ReportBody,
    pub qe_report_sig: [u8; 64],
    pub auth_certification_data: [u8; 0],
}

impl_struct_default! {
    QlEcdsaSigData; //576
}

impl_struct_ContiguousMemory! {
    QlAuthData;
    QlCertificationData;
    QlEcdsaSigData;
}

impl QlAuthData {
    /// # Safety
    pub unsafe fn as_slice_unchecked(&self) -> &[u8] {
        slice::from_raw_parts(
            self as *const _ as *const u8,
            mem::size_of::<QlAuthData>() + self.size as usize,
        )
    }
}

impl QlCertificationData {
    /// # Safety
    pub unsafe fn as_slice_unchecked(&self) -> &[u8] {
        slice::from_raw_parts(
            self as *const _ as *const u8,
            mem::size_of::<QlCertificationData>() + self.size as usize,
        )
    }
}

impl QlEcdsaSigData {
    /// # Safety
    pub unsafe fn as_slice_unchecked(&self) -> &[u8] {
        let p_sig_data = self as *const QlEcdsaSigData;
        let p_auth_data = p_sig_data.add(1) as *const QlAuthData;
        let p_cert_data = (p_auth_data.add(1) as *const u8).add((*p_auth_data).size as usize)
            as *const QlCertificationData;

        slice::from_raw_parts(
            self as *const _ as *const u8,
            mem::size_of::<QlEcdsaSigData>()
                + mem::size_of::<QlAuthData>()
                + mem::size_of::<QlCertificationData>()
                + (*p_auth_data).size as usize
                + (*p_cert_data).size as usize,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub struct QuoteHeader {
    pub version: u16,
    pub att_key_type: u16,
    pub att_key_data: u32,
    pub qe_svn: u16,
    pub pce_svn: u16,
    pub vendor_id: [u8; 16],
    pub user_data: [u8; 20],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Quote3 {
    pub header: QuoteHeader,
    pub report_body: ReportBody,
    pub signature_len: u32,
    pub signature: [u8; 0],
}

impl_asref_array! {
    QuoteHeader;
}

impl_struct_default! {
    Quote3; //436
}

impl_struct_ContiguousMemory! {
    QuoteHeader;
    Quote3;
}

impl Quote3 {
    /// # Safety
    pub unsafe fn as_slice_unchecked(&self) -> &[u8] {
        slice::from_raw_parts(
            self as *const _ as *const u8,
            mem::size_of::<Quote3>() + self.signature_len as usize,
        )
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct QlQeReportInfo {
    pub nonce: QuoteNonce,
    pub app_enclave_target_info: TargetInfo,
    pub qe_report: Report,
}

impl_asref_array! {
    QlQeReportInfo;
}

impl_struct_default! {
    QlQeReportInfo; //960
}

impl_struct_ContiguousMemory! {
    QlQeReportInfo;
}

/* intel DCAP 1.6 */
//
// sgx_dcap_ql_wrapper.h
//
impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum QlPathType {
        Qe3Path = 0,
        PcePath = 1,
        QplPath = 2,
        /* intel DCAP 1.13 */
        IdePath = 3,
    }
}

//
// qve_header.h
//
pub const ROOT_KEY_ID_SIZE: usize = 48;
pub const PLATFORM_INSTANCE_ID_SIZE: usize = 16;

/* intel DCAP 1.7 */
impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum PckCertFlag {
        False = 0,
        True = 1,
        Undefined = 2,
    }
}

impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct QlQvSupplemental {
        pub version: u32,
        pub earliest_issue_date: time_t,
        pub latest_issue_date: time_t,
        pub earliest_expiration_date: time_t,
        pub tcb_level_date_tag: time_t,
        pub pck_crl_num: u32,
        pub root_ca_crl_num: u32,
        pub tcb_eval_ref_num: u32,
        pub root_key_id: [u8; ROOT_KEY_ID_SIZE],
        pub pck_ppid: Key128bit,
        pub tcb_cpusvn: CpuSvn,
        pub tcb_pce_isvsvn: u16,
        pub pce_id: u16,
        /* intel DCAP 1.13 */
        pub tee_type: u32,
        /* intel DCAP 1.7 */
        pub sgx_type: u8,

        pub platform_instance_id: [u8; PLATFORM_INSTANCE_ID_SIZE],
        pub dynamic_platform: PckCertFlag,
        pub cached_keys: PckCertFlag,
        pub smt_enabled: PckCertFlag,
    }
}

impl_struct_default! {
    QlQvSupplemental; //176
}

impl_asref_array! {
    QlQvSupplemental;
}

impl_struct_ContiguousMemory! {
    QlQvSupplemental;
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum QlQvResult {
        Ok                          = 0x0000_0000,
        ConfigNeeded                = 0x0000_A001,
        OutOfDate                   = 0x0000_A002,
        OutOfDateConfigNeeded       = 0x0000_A003,
        InvalidSignature            = 0x0000_A004,
        Revoked                     = 0x0000_A005,
        Unspecified                 = 0x0000_A006,
        SWHardeningNeeded           = 0x0000_A007,
        ConfigAndSWHardeningNeeded  = 0x0000_A008,
        Max                         = 0x0000_A0FF,
    }
}

impl QlQvResult {
    pub fn __description(&self) -> &str {
        match *self {
            QlQvResult::Ok => "The Quote verification passed and is at the latest TCB level.",
            QlQvResult::ConfigNeeded => {
                "The Quote verification passed and the platform is patched to the latest TCB level but additional configuration of the SGX platform may be needed."
            }
            QlQvResult::OutOfDate => {
                "The Quote is good but TCB level of the platform is out of date, The platform needs patching to be at the latest TCB level."
            }
            QlQvResult::OutOfDateConfigNeeded => {
                "The Quote is good but the TCB level of the platform is out of date and additional configuration of the SGX Platform at its current patching level may be needed. The platform needs patching to be at the latest TCB level."
            }
            QlQvResult::InvalidSignature => "The signature over the application report is invalid.",
            QlQvResult::Revoked => "The attestation key or platform has been revoked.",
            QlQvResult::Unspecified => "The Quote verification failed due to an error in one of the input.",
            QlQvResult::SWHardeningNeeded => "The TCB level of the platform is up to date, but SGX SW Hardening is needed.",
            QlQvResult::ConfigAndSWHardeningNeeded => {
                "The TCB level of the platform is up to date, but additional configuration of the platform at its current patching level may be needed. Moreove, SGX SW Hardening is also needed."
            }
            QlQvResult::Max => "Indicate max result to allow better translation.",
        }
    }

    pub fn as_str(&self) -> &str {
        match *self {
            QlQvResult::Ok => "Ok",
            QlQvResult::ConfigNeeded => "ConfigNeeded",
            QlQvResult::OutOfDate => "OutOfDate",
            QlQvResult::OutOfDateConfigNeeded => "OutOfDateConfigNeeded",
            QlQvResult::InvalidSignature => "InvalidSignature",
            QlQvResult::Revoked => "Revoked",
            QlQvResult::Unspecified => "Unspecified",
            QlQvResult::SWHardeningNeeded => "SWHardeningNeeded",
            QlQvResult::ConfigAndSWHardeningNeeded => "ConfigAndSWHardeningNeeded",
            QlQvResult::Max => "Max",
        }
    }
}

impl fmt::Display for QlQvResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/* intel DCAP 1.6 */
//
// sgx_dcap_quoteverify.h
//
impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum QvPathType {
        QvePath = 0,
        QplPath = 1,
    }
}

/* intel DCAP 1.14 */
//
// sgx_default_qcnl_wrapper.h
//
impl_enum! {
    #[repr(u8)]
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum QeType {
        Ecdsa = 0,
        Td = 1,
    }
}
