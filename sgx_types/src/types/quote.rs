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

//
// sgx_quote.h
//
pub type EpidGroupId = [u8; 4];
pub const SGX_PLATFORM_INFO_SIZE: usize = 101;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub struct Spid {
    pub id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub struct BaseName {
    pub name: [u8; 32],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub struct QuoteNonce {
    pub rand: [u8; 16],
}

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub struct UpdateInfoBit {
    pub ucodeUpdate: i32,
    pub csmeFwUpdate: i32,
    pub pswUpdate: i32,
}

impl_asref_array! {
    Spid;
    BaseName;
    QuoteNonce;
    UpdateInfoBit;
}

impl_asmut_array! {
    Spid;
    BaseName;
    QuoteNonce;
}

impl_from_array! {
    Spid;
    BaseName;
    QuoteNonce;
}

impl_struct_ContiguousMemory! {
    Spid;
    BaseName;
    QuoteNonce;
    UpdateInfoBit;
}

impl_unsafe_marker_for! {
    BytewiseEquality,
    Spid BaseName QuoteNonce UpdateInfoBit
}

impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum QuoteSignType {
        Unlinkable = 0,
        Linkable   = 1,
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Quote {
    pub version: u16,               /* 0   */
    pub sign_type: u16,             /* 2   */
    pub epid_group_id: EpidGroupId, /* 4   */
    pub qe_svn: u16,                /* 8   */
    pub pce_svn: u16,               /* 10  */
    pub xeid: u32,                  /* 12  */
    pub basename: BaseName,         /* 16  */
    pub report_body: ReportBody,    /* 48  */
    pub signature_len: u32,         /* 432 */
    pub signature: [u8; 0],         /* 436 */
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct PlatformInfo {
    pub platform_info: [u8; SGX_PLATFORM_INFO_SIZE],
}

/* intel sgx sdk 2.5 */
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct AttKeyId {
    pub att_key_id: [u8; 256],
}

/* intel sgx sdk 2.9.1 */
/* sgx_ql_att_key_id_t moved from sgx_quote_3.h to sgx_quote.h */
/* Describes a single attestation key. Contains both QE identity and the attestation algorithm ID. */
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct QlAttKeyId {
    pub id: u16,              //< Structure ID
    pub version: u16,         //< Structure version
    pub mrsigner_length: u16, //< Number of valid bytes in MRSIGNER.
    pub mrsigner: [u8; 48],   //< SHA256 or SHA384 hash of the Public key that signed the QE.
    //< The lower bytes contain MRSIGNER.  Bytes beyond mrsigner_length '0'
    pub prod_id: u32,               //< Legacy Product ID of the QE
    pub extended_prod_id: [u8; 16], //< Extended Product ID or the QE. All 0's for legacy format enclaves.
    pub config_id: [u8; 64],        //< Config ID of the QE.
    pub family_id: [u8; 16],        //< Family ID of the QE.
    pub algorithm_id: u32,          //< Identity of the attestation key algorithm.
}

/* intel sgx sdk 2.9.1 */
/* sgx_att_key_id_ext_t moved from sgx_quote_3.h to sgx_quote.h */
/* Describes an extended attestation key. Contains sgx_ql_att_key_id_t, spid and quote_type */
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct AttKeyIdExt {
    pub base: QlAttKeyId,
    pub spid: [u8; 16],    //< Service Provider ID, should be 0s for ECDSA quote
    pub att_key_type: u16, //< For non-EPID quote, it should be 0
    //< For EPID quote, it equals to sgx_quote_sign_type_t
    pub reserved: [u8; 80], //< It should have the same size of sgx_att_key_id_t
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct QeReportInfo {
    pub nonce: QuoteNonce,
    pub app_enclave_target_info: TargetInfo,
    pub qe_report: Report,
}

impl_struct_default! {
    Quote;          //436
    PlatformInfo;   //101
    AttKeyId;       //256
    QlAttKeyId;     //158
    AttKeyIdExt;    //256
    QeReportInfo;   //960
}

impl_asref_array! {
    PlatformInfo;
    AttKeyId;
    QlAttKeyId;
    AttKeyIdExt;
    QeReportInfo;
}

impl_struct_ContiguousMemory! {
    Quote;
    PlatformInfo;
    AttKeyId;
    QlAttKeyId;
    AttKeyIdExt;
    QeReportInfo;
}

impl AsRef<[u8]> for Quote {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<Quote>() + self.signature_len as usize,
            )
        }
    }
}
