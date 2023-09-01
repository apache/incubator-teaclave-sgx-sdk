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

/* intel DCAP 1.17 */
pub type CTdxQlQvCollateral = CQlQveCollateral;

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

pub const TD_INFO_RESERVED_BYTES_V1: usize = 112;
pub const TD_TEE_TCB_INFO_RESERVED_BYTES_V1: usize = 111;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TeeInfo {
    pub attributes: TeeAttributes,
    pub xfam: TeeAttributes,
    pub mr_td: TeeMeasurement,
    pub mr_config_id: TeeMeasurement,
    pub mr_owner: TeeMeasurement,
    pub mr_owner_config: TeeMeasurement,
    pub rt_mr: [TeeMeasurement; 4],
    pub reserved: [u8; TD_INFO_RESERVED_BYTES_V1],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TeeTcbInfo {
    pub valid: [u8; 8],
    pub tee_tcb_svn: TeeTcbSvn,
    pub mr_seam: TeeMeasurement,
    pub mr_seam_signer: TeeMeasurement,
    pub attributes: TeeAttributes,
    pub reserved: [u8; TD_TEE_TCB_INFO_RESERVED_BYTES_V1],
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
    Quote4Header;
    Quote4;
}

impl_asref_array! {
    TeeInfo;
    TeeTcbInfo;
    Report2Body;
    Quote4Header;
}

impl QeReportCertificationData {
    /// # Safety
    pub unsafe fn as_slice_unchecked(&self) -> &[u8] {
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

impl EcdsaSigDataV4 {
    /// # Safety
    pub unsafe fn as_slice_unchecked(&self) -> &[u8] {
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

//
// Quote4
// EcdsaSigDataV4
// QlCertificationData  size = QeReportCertificationData + QlAuthData + AuthDataSize + QlCertificationData + CertChainSize
// QeReportCertificationData
// QlAuthData
// QlCertificationData
// CertChain
//
impl Quote4 {
    /// # Safety
    pub unsafe fn as_slice_unchecked(&self) -> &[u8] {
        slice::from_raw_parts(
            self as *const _ as *const u8,
            mem::size_of::<Quote4>() + self.signature_data_len as usize,
        )
    }
}

/* intel DCAP 1.18 */
//
// sgx_quote_5.h
//

pub const TD_INFO_RESERVED_BYTES_V15: usize = 64;
pub const TD_TEE_TCB_INFO_RESERVED_BYTES_V15: usize = 95;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TeeInfoV15 {
    pub attributes: TeeAttributes,
    pub xfam: TeeAttributes,
    pub mr_td: TeeMeasurement,
    pub mr_config_id: TeeMeasurement,
    pub mr_owner: TeeMeasurement,
    pub mr_owner_config: TeeMeasurement,
    pub rt_mr: [TeeMeasurement; 4],
    pub mr_servicetd: TeeMeasurement,
    pub reserved: [u8; TD_INFO_RESERVED_BYTES_V15],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TeeTcbInfoV15 {
    pub valid: [u8; 8],
    pub tee_tcb_svn: TeeTcbSvn,
    pub mr_seam: TeeMeasurement,
    pub mr_seam_signer: TeeMeasurement,
    pub attributes: TeeAttributes,
    pub tee_tcb_svn2: TeeTcbSvn,
    pub reserved: [u8; TD_TEE_TCB_INFO_RESERVED_BYTES_V15],
}

pub type Quote5Header = Quote4Header;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct Report2BodyV15 {
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
    pub tee_tcb_svn2: TeeTcbSvn,
    pub mr_servicetd: TeeMeasurement,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Quote5 {
    pub header: Quote5Header,
    pub quote_type: u16,
    pub size: u32,
    pub body: [u8; 0],
}

impl_struct_default! {
    TeeInfoV15; //512
    TeeTcbInfoV15; //239
}

impl_struct_ContiguousMemory! {
    TeeInfoV15;
    TeeTcbInfoV15;
    Report2BodyV15;
    Quote5;
}

impl_asref_array! {
    TeeInfoV15;
    TeeTcbInfoV15;
    Report2BodyV15;
}

impl Quote5 {
    /// # Safety
    pub unsafe fn as_slice_unchecked(&self) -> &[u8] {
        slice::from_raw_parts(
            self as *const _ as *const u8,
            mem::size_of::<Quote5>() + self.size as usize,
        )
    }
}

/* intel DCAP 1.15 */
//
// tdx_attes.h
//
pub const TDX_UUID_SIZE: usize = 16;
pub const TDX_REPORT_DATA_SIZE: usize = 64;
pub const TDX_REPORT_SIZE: usize = 1024;

impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct TdxUuid {
        pub d: [u8; TDX_UUID_SIZE],
    }
}

impl_asref_array! {
    TdxUuid;
}
impl_asmut_array! {
    TdxUuid;
}
impl_from_array! {
    TdxUuid;
}
impl_unsafe_marker_for! {
    BytewiseEquality,
    TdxUuid
}

impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct TdxReportData {
        pub d: [u8; TDX_REPORT_DATA_SIZE],
    }
}
impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct TdxReport {
        pub d: [u8; TDX_REPORT_SIZE],
    }
}

impl_struct_default! {
    TdxReportData; //64
    TdxReport; //1024
}

impl_struct_ContiguousMemory! {
    TdxReportData;
    TdxReport;
}

impl_asref_array! {
    TdxReportData;
    TdxReport;
}

impl_asmut_array! {
    TdxReportData;
}
impl_from_array! {
    TdxReportData;
}

impl_unsafe_marker_for! {
    BytewiseEquality,
    TdxReportData
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TdxRtmrEvent {
    pub version: u32,
    pub rtmr_index: u64,
    pub extend_data: [u8; 48],
    pub event_type: u32,
    pub event_data_size: u32,
    pub event_data: [u8; 0],
}

impl_struct_default! {
    TdxRtmrEvent; //68
}

impl_struct_ContiguousMemory! {
    TdxRtmrEvent;
}

impl TdxRtmrEvent {
    /// # Safety
    pub unsafe fn as_slice_unchecked(&self) -> &[u8] {
        slice::from_raw_parts(
            self as *const _ as *const u8,
            mem::size_of::<TdxRtmrEvent>() + self.event_data_size as usize,
        )
    }

    /// # Safety
    pub unsafe fn event_data_slice_unchecked(&self) -> &[u8] {
        slice::from_raw_parts(
            &self.event_data as *const _ as *const u8,
            self.event_data_size as usize,
        )
    }
}
