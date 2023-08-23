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

use crate::types::{Attributes, IsvExtProdId, IsvFamilyId, Measurement, MiscSelect};

/* arch .h*/
pub const SE_PAGE_SHIFT: usize = 12;
pub const SE_PAGE_SIZE: usize = 0x1000;
pub const SE_KEY_SIZE: usize = 384;
pub const SE_EXPONENT_SIZE: usize = 4;

/* version of metadata */
pub const MAJOR_VERSION: u32 = 3;
pub const MINOR_VERSION: u32 = 0;
pub const SGX_2_ELRANGE_MAJOR_VERSION: u32 = 13; //MAJOR_VERSION should not larger than 0FFFFFFFF
pub const SGX_1_ELRANGE_MAJOR_VERSION: u32 = 11; //MINOR_VERSION should not larger than 0FFFFFFFF
pub const SGX_2_1_MAJOR_VERSION: u32 = 2; //MAJOR_VERSION should not larger than 0FFFFFFFF
pub const SGX_2_1_MINOR_VERSION: u32 = 2; //MINOR_VERSION should not larger than 0FFFFFFFF
pub const SGX_2_0_MAJOR_VERSION: u32 = 2; //MAJOR_VERSION should not larger than 0FFFFFFFF
pub const SGX_2_0_MINOR_VERSION: u32 = 1; //MINOR_VERSION should not larger than 0FFFFFFFF
pub const SGX_1_9_MAJOR_VERSION: u32 = 1; //MAJOR_VERSION should not larger than 0FFFFFFFF
pub const SGX_1_9_MINOR_VERSION: u32 = 4; //MINOR_VERSION should not larger than 0FFFFFFFF
pub const SGX_1_5_MAJOR_VERSION: u32 = 1; //MAJOR_VERSION should not larger than 0FFFFFFFF
pub const SGX_1_5_MINOR_VERSION: u32 = 3; //MINOR_VERSION should not larger than 0FFFFFFFF

pub const METADATA_MAGIC: u64 = 0x86A8_0294_635D_0E4C;
pub const METADATA_SIZE: usize = 0x5000;
pub const TCS_TEMPLATE_SIZE: usize = 72;

pub const TCS_POLICY_BIND: u32 = 0x0000_0000; /* If set, the TCS is bound to the application thread */
pub const TCS_POLICY_UNBIND: u32 = 0x0000_0001;

pub const TCS_NUM_MIN: u32 = 1;
pub const SSA_NUM_MIN: u32 = 2;
pub const SSA_FRAME_SIZE_MIN: u32 = 1;
pub const SSA_FRAME_SIZE_MAX: u32 = 4;
pub const STACK_SIZE_MIN: u64 = 0x0000_2000; //8 KB
pub const STACK_SIZE_MAX: u64 = 0x0004_0000; //256 KB
pub const HEAP_SIZE_MIN: u64 = 0x0000_1000; //4 KB
pub const HEAP_SIZE_MAX: u64 = 0x0100_0000; //16 MB
pub const RSRV_SIZE_MIN: u64 = 0x0000_0000; //0 KB
pub const RSRV_SIZE_MAX: u64 = 0x0000_0000; //0 KB
pub const USER_REGION_SIZE: u64 = 0x0000_0000; //0 KB
pub const DEFAULT_MISC_SELECT: u32 = 0;
pub const DEFAULT_MISC_MASK: u32 = 0xFFFF_FFFF;
pub const ISVFAMILYID_MAX: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub const ISVEXTPRODID_MAX: u64 = 0xFFFF_FFFF_FFFF_FFFF;

#[macro_export]
macro_rules! meta_data_make_version {
    ($major:ident, $minor:ident) => {
        ($major as u64) << 32 | $minor as u64
    };
}

#[macro_export]
macro_rules! major_version_of_metadata {
    ($version:ident) => {
        ($version as u64) >> 32
    };
}

#[macro_export]
macro_rules! minor_version_of_metadata {
    ($version:ident) => {
        ($version as u64) & 0x0000_0000_FFFF_FFFF
    };
}

/* metadata.h */
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct DataDir {
    pub offset: u32,
    pub size: u32,
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum DirIndex {
        DIR_PATCH  = 0,
        DIR_LAYOUT = 1,
        DIR_NUM    = 2,
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct ElRangeConfig {
    pub enclave_image_base: u64,
    pub elrange_start_base: u64,
    pub elrange_size: u64,
}

#[cfg(not(feature = "hyper"))]
pub const METADATA_DATA_BYTES: usize = 18592;
#[cfg(feature = "hyper")]
pub const METADATA_DATA_BYTES: usize = 18584;

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct MetaData {
    pub magic_num: u64,
    pub version: u64,
    pub size: u32,
    pub tcs_policy: u32,
    pub ssa_frame_size: u32,
    pub max_save_buffer_size: u32,
    pub desired_misc_select: MiscSelect,
    pub tcs_min_pool: u32,
    pub enclave_size: u64,
    pub attributes: Attributes,
    pub enclave_css: EnclaveCss,
    pub dirs: [DataDir; DirIndex::DIR_NUM as usize],
    #[cfg(feature = "hyper")]
    pub msbuf_size: u64,
    pub data: [u8; METADATA_DATA_BYTES],
}

/* arch.h */
pub const CSS_HEADER_RESERVED_BYTES: usize = 84;
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct CssHeader {
    pub header: [u8; 12],
    pub css_type: u32,
    pub module_vendor: u32,
    pub date: u32,
    pub header2: [u8; 16],
    pub hw_version: u32,
    pub reserved: [u8; CSS_HEADER_RESERVED_BYTES],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct CssKey {
    pub modulus: [u8; SE_KEY_SIZE],
    pub exponent: [u8; SE_EXPONENT_SIZE],
    pub signature: [u8; SE_KEY_SIZE],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct CssBody {
    pub misc_select: MiscSelect,
    pub misc_mask: u32,
    pub reserved: [u8; 4],
    pub isv_family_id: IsvFamilyId,
    pub attributes: Attributes,
    pub attribute_mask: Attributes,
    pub enclave_hash: Measurement,
    pub reserved2: [u8; 16],
    pub isvext_prod_id: IsvExtProdId,
    pub isv_prod_id: u16,
    pub isv_svn: u16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct CssBuffer {
    pub reserved: [u8; 12],
    pub q1: [u8; SE_KEY_SIZE],
    pub q2: [u8; SE_KEY_SIZE],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct EnclaveCss {
    pub header: CssHeader,
    pub key: CssKey,
    pub body: CssBody,
    pub buffer: CssBuffer,
}

impl_struct_default! {
    DataDir;
    CssHeader;
    CssKey;
    CssBody;
    CssBuffer;
    MetaData;
}

impl_struct_ContiguousMemory! {
    DataDir;
    CssHeader;
    CssKey;
    CssBody;
    CssBuffer;
    MetaData;
}
