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

use super::*;

use core::mem;
use core::slice;

/* intel DCAP 1.14 */
//
// sgx_quote_4.h
//
pub const TEE_TCB_SVN_SIZE: usize = 16;

impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct TeeTcbSvn {
        pub tcb_svn: [u8; TEE_TCB_SVN_SIZE],
    }
}

impl_asref_array! {
    TeeTcbSvn;
}
impl_asmut_array! {
    TeeTcbSvn;
}
impl_from_array! {
    TeeTcbSvn;
}
impl_unsafe_marker_for! {
    BytewiseEquality,
    TeeTcbSvn
}

pub const TD_INFO_RESERVED_BYTES: usize = 112;
pub const TD_TEE_TCB_INFO_RESERVED_BYTES: usize = 111;

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TeeInfo {
    pub attributes: TeeAttributes,
    pub xfam: TeeAttributes,
    pub mr_td: TeeMeasurement,
    pub mr_config_id: TeeMeasurement,
    pub mr_owner: TeeMeasurement,
    pub mr_owner_config: TeeMeasurement,
    pub rt_mr: [TeeMeasurement; 4],
    pub reserved: [u8; TD_INFO_RESERVED_BYTES],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TeeTcbInfo {
    pub valid: [u8; 8],
    pub tee_tcb_svn: TeeTcbSvn,
    pub mr_seam: TeeMeasurement,
    pub mr_seam_signer: TeeMeasurement,
    pub attributes: TeeAttributes,
    pub reserved: [u8; TD_TEE_TCB_INFO_RESERVED_BYTES],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct QeReportCertificationData {
    pub qe_report: ReportBody,
    pub qe_report_sig: [u8; 64],
    pub auth_certification_data: [u8; 0],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct EcdsaSigDataV4 {
    pub sig: [u8; 64],
    pub attest_pub_key: [u8; 64],
    pub certification_data: [uint8_t; 0],
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct Report2Body {
    pub tee_tcb_svn: TeeTcbSvn,
    pub mr_seam: TeeMeasurement,
    pub mrsigner_seam: TeeMeasurement,
    pub seam_attributes: TeeAttributes,
    pub td_attributes: TeeAttributes,
    pub xfam: TeeAttributes,
    pub mr_td: TeeMeasurement,
    pub mr_config_id: TeeMeasurement,
    pub mr_owner: TeeMeasurement,
    pub mr_owner_config: TeeMeasurement,
    pub rt_mr: [TeeMeasurement; 4],
    pub report_data: TeeReportData,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub struct Quote4Header {
    pub version: u16,
    pub att_key_type: u16,
    pub tee_type: u32,
    pub reserved: u32,
    pub vendor_id: [u8; 16],
    pub user_data: [u8; 20],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Quote4 {
    pub header: Quote4Header,
    pub report_body: Report2Body,
    pub signature_data_len: u32,
    pub signature_data: [u8; 0],
}

impl_struct_default! {
    TeeInfo; //512
    TeeTcbInfo; //239
    QeReportCertificationData; //448
    EcdsaSigDataV4; //128
    Quote4; //636
}

impl_struct_ContiguousMemory! {
    TeeInfo;
    TeeTcbInfo;
    QeReportCertificationData;
    EcdsaSigDataV4;
    Quote4;
}

impl_asref_array! {
    TeeInfo;
    TeeTcbInfo;
    Report2Body;
    Quote4Header;
}

impl AsRef<[u8]> for QeReportCertificationData {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            let p_report_cert_data = self as *const QeReportCertificationData;
            let p_auth_data = p_report_cert_data.add(1) as *const QlAuthData;
            let p_cert_data = (p_auth_data.add(1) as *const u8).add((*p_auth_data).size as usize)
                as *const QlCertificationData;

            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<QeReportCertificationData>()
                    + mem::size_of::<QlAuthData>()
                    + mem::size_of::<QlCertificationData>()
                    + (*p_auth_data).size as usize
                    + (*p_cert_data).size as usize,
            )
        }
    }
}

impl AsRef<[u8]> for EcdsaSigDataV4 {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            let p_sig_data = self as *const EcdsaSigDataV4;
            let p_cert_data = p_sig_data.add(1) as *const QlCertificationData;

            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<EcdsaSigDataV4>()
                    + mem::size_of::<QlCertificationData>()
                    + (*p_cert_data).size as usize,
            )
        }
    }
}

//
// Quote4
// EcdsaSigDataV4
// QlCertificationData  size = QeReportCertificationData + QlAuthData + AuthDataSize + QlCertificationData + CertChainSize
// QeReportCertificationData
// QlAuthData
// QlCertificationData
// CertChain
//
impl AsRef<[u8]> for Quote4 {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<Quote4>() + self.signature_data_len as usize,
            )
        }
    }
}
