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

use crate::marker::BytewiseEquality;
use core::convert::{From, TryInto};
use core::fmt;

mod crypto;
mod dcap;
mod dh;
mod key_exchange;
mod quote;
mod ra;
mod raw;
mod report2;
mod seal;
mod switchless;
mod urts;

pub use crypto::*;
pub use dcap::*;
pub use dh::*;
pub use key_exchange::*;
pub use quote::*;
pub use ra::*;
pub use raw::*;
pub use report2::*;
pub use seal::*;
pub use switchless::*;
pub use urts::*;

//
// sgx_urts.h/sgx_trts.h
//
impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum EnclaveMode {
        Hw    = 1,
        Sim   = 2,
        Hyper = 3,
    }
}

//
// sgx_attributes.h
//
// Enclave Flags Bit Masks
impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct AttributesFlags: u64 {
        const INITTED       = 0x0000_0000_0000_0001;
        const DEBUG         = 0x0000_0000_0000_0002;
        const MODE64BIT     = 0x0000_0000_0000_0004;
        const PROVISIONKEY  = 0x0000_0000_0000_0010;
        const EINITTOKENKEY = 0x0000_0000_0000_0020;
        const CET           = 0x0000_0000_0000_0040;
        const KSS           = 0x0000_0000_0000_0080;
        const NON_SECURITY  = 0x0000_0000_0000_0004 | 0x0000_0000_0000_0010 | 0x0000_0000_0000_0020;
        const DEFAULT_MASK  = !(0x00FF_FFFF_FFFF_FFC0 | 0x0000_0000_0000_0004 | 0x0000_0000_0000_0010 | 0x0000_0000_0000_0020);
    }
}

// XSAVE Feature Request Mask
pub const XFRM_LEGACY: u64 = 0x0000_0000_0000_0003; // Legacy XFRM
pub const XFRM_AVX: u64 = 0x0000_0000_0000_0006; // AVX
pub const XFRM_AVX512: u64 = 0x0000_0000_0000_00E6; // AVX-512 - not supported
pub const XFRM_MPX: u64 = 0x0000_0000_0000_0018; // MPX - not supported
pub const XFRM_PKRU: u64 = 0x0000_0000_0000_0200; // PKRU state
pub const XFRM_RESERVED: u64 = !(XFRM_LEGACY | XFRM_AVX | XFRM_AVX512 | XFRM_PKRU);

impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct MiscSelect: u32 {
        const EXINFO = 0x0000_0001;
        const CPINFO = 0x0000_0002;
    }
}

impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Attributes {
        pub flags: AttributesFlags,
        pub xfrm: u64,
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct MiscAttribute {
        pub secs_attr: Attributes,
        pub misc_select: MiscSelect,
    }
}

impl_asref_array! {
    Attributes;
}

//
// sgx_key.h
//
pub const CPUSVN_SIZE: usize = 16;
pub const CONFIGID_SIZE: usize = 64;

pub type ConfigId = [u8; CONFIGID_SIZE];

impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct CpuSvn {
        pub svn: [u8; CPUSVN_SIZE],
    }
}

impl_asref_array! {
    CpuSvn;
}
impl_asmut_array! {
    CpuSvn;
}
impl_from_array! {
    CpuSvn;
}

impl_enum! {
    #[repr(u16)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum KeyName {
        EInitToken    = 0,
        Provision     = 1,
        ProvisionSeal = 2,
        Report        = 3,
        Seal          = 4,
    }
}

pub const KEYID_SIZE: usize = 32;
impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct KeyId {
        pub id: [u8; KEYID_SIZE],
    }
}

impl_asref_array! {
    KeyId;
}
impl_asmut_array! {
    KeyId;
}
impl_from_array! {
    KeyId;
}

impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct KeyPolicy: u16 {
        const MRENCLAVE     = 0x0001;
        const MRSIGNER      = 0x0002;
        const NOISVPRODID   = 0x0004;
        const CONFIGID      = 0x0008;
        const ISVFAMILYID   = 0x0010;
        const ISVEXTPRODID  = 0x0020;
        const KSS           = 0x0008 | 0x0010 | 0x0020;
    }
}

impl KeyPolicy {
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.bits() & !Self::all().bits() == 0
    }
}

pub const KEY_REQUEST_RESERVED2_BYTES: usize = 434;
impl_copy_clone! {
    #[repr(C)]
    pub struct KeyRequest {
        pub key_name: KeyName,
        pub key_policy: KeyPolicy,
        pub isv_svn: u16,
        pub reserved1: u16,
        pub cpu_svn: CpuSvn,
        pub attribute_mask: Attributes,
        pub key_id: KeyId,
        pub misc_mask: u32,
        pub config_svn: u16,
        pub reserved2: [u8; KEY_REQUEST_RESERVED2_BYTES],
    }
}

impl_struct_default! {
    KeyRequest; //512
}
impl_struct_ContiguousMemory! {
    KeyRequest;
}

impl_asref_array! {
    KeyRequest;
}

impl fmt::Debug for KeyRequest {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("KeyRequest")
            .field("key_name", &self.key_name)
            .field("key_policy", &self.key_policy)
            .field("isv_svn", &self.isv_svn)
            .field("cpu_svn", &self.cpu_svn)
            .field("attribute_mask", &self.attribute_mask)
            .field("key_id", &self.key_id)
            .field("misc_mask", &self.misc_mask)
            .field("config_svn", &self.config_svn)
            .finish()
    }
}

//
// sgx_report.h
//
pub const HASH_SIZE: usize = 32;
pub const MAC_SIZE: usize = 16;

pub const ISVEXT_PROD_ID_SIZE: usize = 16;
pub const ISV_FAMILY_ID_SIZE: usize = 16;

pub type IsvExtProdId = [u8; ISVEXT_PROD_ID_SIZE];
pub type IsvFamilyId = [u8; ISV_FAMILY_ID_SIZE];

impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Measurement {
        pub m: [u8; HASH_SIZE],
    }
}

impl_asref_array! {
    Measurement;
}
impl_asmut_array! {
    Measurement;
}
impl_from_array! {
    Measurement;
}

pub type Mac = [u8; MAC_SIZE];

pub const REPORT_DATA_SIZE: usize = 64;

impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct ReportData {
        pub d: [u8; REPORT_DATA_SIZE],
    }
}

pub const TARGET_INFO_RESERVED1_BYTES: usize = 2;
pub const TARGET_INFO_RESERVED2_BYTES: usize = 8;
pub const TARGET_INFO_RESERVED3_BYTES: usize = 384;
impl_copy_clone! {
    #[repr(C)]
    pub struct TargetInfo {
        pub mr_enclave: Measurement,
        pub attributes: Attributes,
        pub reserved1: [u8; TARGET_INFO_RESERVED1_BYTES],
        pub config_svn: u16,
        pub misc_select: MiscSelect,
        pub reserved2: [u8; TARGET_INFO_RESERVED2_BYTES],
        pub config_id: ConfigId,
        pub reserved3: [u8; TARGET_INFO_RESERVED3_BYTES],
    }
}

pub const REPORT_BODY_RESERVED1_BYTES: usize = 12;
pub const REPORT_BODY_RESERVED2_BYTES: usize = 32;
pub const REPORT_BODY_RESERVED3_BYTES: usize = 32;
pub const REPORT_BODY_RESERVED4_BYTES: usize = 42;
impl_copy_clone! {
    #[repr(C)]
    pub struct ReportBody {
        pub cpu_svn: CpuSvn,
        pub misc_select: MiscSelect,
        pub reserved1: [u8; REPORT_BODY_RESERVED1_BYTES],
        pub isv_ext_prod_id: IsvExtProdId,
        pub attributes: Attributes,
        pub mr_enclave: Measurement,
        pub reserved2: [u8; REPORT_BODY_RESERVED2_BYTES],
        pub mr_signer: Measurement,
        pub reserved3: [u8; REPORT_BODY_RESERVED3_BYTES],
        pub config_id: ConfigId,
        pub isv_prod_id: u16,
        pub isv_svn: u16,
        pub config_svn: u16,
        pub reserved4: [u8; REPORT_BODY_RESERVED4_BYTES],
        pub isv_family_id: IsvFamilyId,
        pub report_data: ReportData,
    }
}

impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct Report {
        pub body: ReportBody,
        pub key_id: KeyId,
        pub mac: Mac,
    }
}

impl_struct_default! {
    TargetInfo; //512
    ReportData; //64
    ReportBody; //384
    Report;     //432
}

impl_struct_ContiguousMemory! {
    TargetInfo;
    ReportData;
    ReportBody;
    Report;
}

impl_asref_array! {
    TargetInfo;
    ReportBody;
    ReportData;
    Report;
}

impl_asmut_array! {
    ReportData;
}
impl_from_array! {
    ReportData;
}

impl_unsafe_marker_for! {
    BytewiseEquality,
    ReportData
}

impl fmt::Debug for TargetInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("TargetInfo")
            .field("mr_enclave", &self.mr_enclave)
            .field("attributes", &self.attributes)
            .field("config_svn", &self.config_svn)
            .field("misc_select", &self.misc_select)
            .field("config_id", &self.config_id)
            .finish()
    }
}

impl fmt::Debug for ReportBody {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("ReportBody")
            .field("cpu_svn", &self.cpu_svn)
            .field("misc_select", &self.misc_select)
            .field("isv_ext_prod_id", &self.isv_ext_prod_id)
            .field("attributes", &self.attributes)
            .field("mr_enclave", &self.mr_enclave)
            .field("mr_signer", &self.mr_signer)
            .field("config_id", &self.config_id)
            .field("isv_prod_id", &self.isv_prod_id)
            .field("isv_svn", &self.isv_svn)
            .field("config_svn", &self.config_svn)
            .field("isv_family_id", &self.isv_family_id)
            .field("report_data", &self.report_data)
            .finish()
    }
}

impl From<Report> for TargetInfo {
    fn from(r: Report) -> TargetInfo {
        TargetInfo {
            mr_enclave: r.body.mr_enclave,
            attributes: r.body.attributes,
            config_svn: r.body.config_svn,
            misc_select: r.body.misc_select,
            config_id: r.body.config_id,
            ..TargetInfo::default()
        }
    }
}

impl From<&Report> for TargetInfo {
    fn from(r: &Report) -> TargetInfo {
        TargetInfo {
            mr_enclave: r.body.mr_enclave,
            attributes: r.body.attributes,
            config_svn: r.body.config_svn,
            misc_select: r.body.misc_select,
            config_id: r.body.config_id,
            ..TargetInfo::default()
        }
    }
}

/* intel sgx sdk 2.8 */
//
// sgx_rsrv_mem_mngr.h
//
impl_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum ProtectPerm {
        None            = 0x0,
        Read            = 0x1,
        ReadWrite       = 0x3,
        ReadExec        = 0x5,
        ReadWriteExec   = 0x7,
    }
}

impl ProtectPerm {
    pub fn can_read(&self) -> bool {
        *self != Self::None
    }

    pub fn can_write(&self) -> bool {
        !matches!(*self, Self::None | Self::Read | Self::ReadExec)
    }

    pub fn can_execute(&self) -> bool {
        !matches!(*self, Self::None | Self::Read | Self::ReadWrite)
    }
}
