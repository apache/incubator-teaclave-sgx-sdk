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

use core::fmt;

//
// sgx_report2.h
//
pub const TEE_HASH_384_SIZE: usize = 48; // SHA384
pub const TEE_MAC_SIZE: usize = 32; // Message SHA 256 HASH Code - 32 bytes

pub const REPORT2_DATA_SIZE: usize = 64;
pub const TEE_CPU_SVN_SIZE: usize = 16;

pub type TeeMac = [u8; TEE_MAC_SIZE];

impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct TeeCpuSvn {
        pub svn: [u8; TEE_CPU_SVN_SIZE],
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct TeeAttributes {
        pub a: [u32; 2],
    }
}

impl_asref_array! {
    TeeCpuSvn;
    TeeAttributes;
}
impl_asmut_array! {
    TeeCpuSvn;
    TeeAttributes;
}
impl_from_array! {
    TeeCpuSvn;
    TeeAttributes;
}

impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct TeeMeasurement {
        pub m: [u8; TEE_HASH_384_SIZE],
    }
}

impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct TeeReportData {
        pub d: [u8; REPORT2_DATA_SIZE],
    }
}

impl_struct_default! {
    TeeMeasurement; //48
    TeeReportData;  //64
}

impl_struct_ContiguousMemory! {
    TeeMeasurement;
    TeeReportData;
}

impl_unsafe_marker_for! {
    BytewiseEquality,
    TeeCpuSvn TeeMeasurement TeeReportData
}

impl_asref_array! {
    TeeMeasurement;
    TeeReportData;
}

impl_asmut_array! {
    TeeMeasurement;
    TeeReportData;
}

impl_from_array! {
    TeeMeasurement;
    TeeReportData;
}

pub const LEGACY_REPORT_TYPE: u8 = 0x0; // SGX Legacy Report Type
pub const TEE_REPORT2_TYPE: u8 = 0x81; // TEE Report Type2
pub const TEE_REPORT2_SUBTYPE: u8 = 0x0; // SUBTYPE for Report Type2 is 0
pub const TEE_REPORT2_VERSION: u8 = 0x0; // VERSION for Report Type2 is 0

impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct TeeReportType {
        pub report_type: u8,
        pub subtype: u8,
        pub version: u8,
        pub reserved: u8,
    }
}

impl_asref_array! {
    TeeReportType;
}

pub const REPORT2_MAC_RESERVED1_BYTES: usize = 12;
pub const REPORT2_MAC_RESERVED2_BYTES: usize = 32;
impl_copy_clone! {
    #[repr(C)]
    pub struct Report2Mac {
        pub report_type: TeeReportType,
        pub reserved1: [u8; REPORT2_MAC_RESERVED1_BYTES],
        pub cpu_svn: TeeCpuSvn,
        pub tee_tcb_info_hash: TeeMeasurement,
        pub tee_info_hash: TeeMeasurement,
        pub report_data: TeeReportData,
        pub reserved2: [u8; REPORT2_MAC_RESERVED2_BYTES],
        pub mac: TeeMac,
    }
}

pub const TEE_TCB_INFO_SIZE: usize = 239;
pub const TEE_REPORT_RESERVED_BYTES: usize = 17;
pub const TEE_INFO_SIZE: usize = 512;
impl_copy_clone! {
    #[repr(C)]
    pub struct Report2 {
        pub report_mac: Report2Mac,
        pub tee_tcb_info: [u8; TEE_TCB_INFO_SIZE],
        pub reserved: [u8; TEE_REPORT_RESERVED_BYTES],
        pub tee_info: [u8; TEE_INFO_SIZE],
    }
}

impl_struct_default! {
    Report2Mac; //256
    Report2;    //1024
}

impl_struct_ContiguousMemory! {
    Report2Mac;
    Report2;
}

impl_asref_array! {
    Report2Mac;
    Report2;
}

impl fmt::Debug for Report2Mac {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Report2Mac")
            .field("report_type", &self.report_type)
            .field("cpu_svn", &self.cpu_svn)
            .field("tee_tcb_info_hash", &self.tee_tcb_info_hash)
            .field("tee_info_hash", &self.tee_info_hash)
            .field("report_data", &self.report_data)
            .field("mac", &self.mac)
            .finish()
    }
}

impl fmt::Debug for Report2 {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Report2")
            .field("report_mac", &self.report_mac)
            .field("tee_tcb_info", &self.tee_tcb_info)
            .field("tee_info", &self.tee_info)
            .finish()
    }
}
