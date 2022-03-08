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

//
// sgx_eid.h
//
pub type EnclaveId = u64;

//
// sgx_urts.h
//
pub type LaunchToken = [u8; 1024];

pub const MAX_EXT_FEATURES_COUNT: usize = 32;

/* intel sgx sdk 2.4 */
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct KssConfig {
    pub config_id: ConfigId,
    pub config_svn: u16,
}

impl_struct_default! {
    KssConfig; //66
}
impl_struct_ContiguousMemory! {
    KssConfig;
}
impl_asref_array! {
    KssConfig;
}

/* intel sgx sdk 2.0 */
//
// sgx_capable.h
//
impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum SgxDeviceStatus {
        Enabled             = 0,
        RebootRequired      = 1,    /* A reboot is required to finish enabling SGX */
        LegacyOs            = 2,    /* SGX is disabled and a Software Control Interface is not available to enable it */
        Disabled            = 3,    /* SGX is not enabled on this platform. More details are unavailable */
        SciAvailable        = 4,    /* SGX is disabled, but a Software Control Interface is available to enable it */
        ManualEnable        = 5,    /* SGX is disabled, but can be enabled manually in the BIOS setup */
        HypervEnable        = 6,    /* Detected an unsupported version of Windows* 10 with Hyper-V enabled */
        Unsupported         = 7,    /* SGX is not supported by this CPU */
    }
}

/* intel sgx sdk 2.1.3 */
//
// sgx_pcl_guid.h
//
pub const PCL_GUID_SIZE: usize = 16;
pub const PCL_GUID: [u8; PCL_GUID_SIZE] = [
    0x95, 0x48, 0x6e, 0x8f, 0x8f, 0x4a, 0x41, 0x4f, 0xb1, 0x27, 0x46, 0x21, 0xa8, 0x59, 0xa8, 0xac,
];
