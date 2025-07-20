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

use crate::enclave::{ExtFeatureBits, ExtFeatures, SgxEnclave};
use crate::enclave::{KSS_BIT_IDX, LAST_BIT_IDX, SWITCHLESS_BIT_IDX};
use sgx_types::error::SgxStatus;
use sgx_types::function::*;
use sgx_types::metadata::MetaData;
use sgx_types::types::{c_char, c_void};
use sgx_types::types::{
    EnclaveId, KssConfig, MiscAttribute, SwitchlessConfig, TargetInfo, MAX_EXT_FEATURES_COUNT,
};
use std::ffi::CStr;
use std::mem::ManuallyDrop;
use std::slice;

#[no_mangle]
pub unsafe extern "C" fn rsgx_create_enclave(
    path: *const c_char,
    debug: i32,
    eid: *mut EnclaveId,
    misc_attr: *mut MiscAttribute,
) -> SgxStatus {
    if path.is_null() || eid.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let debug = match debug {
        1 => true,
        0 => false,
        _ => return SgxStatus::InvalidParameter,
    };

    let path = match CStr::from_ptr(path).to_str() {
        Ok(p) => p,
        Err(_) => return SgxStatus::InvalidParameter,
    };

    let enclave = match SgxEnclave::create(path, debug) {
        Ok(enclave) => ManuallyDrop::new(enclave),
        Err(e) => return e,
    };

    *eid = enclave.eid();
    if !misc_attr.is_null() {
        *misc_attr = enclave.misc_attr().unwrap();
    }

    SgxStatus::Success
}

#[no_mangle]
pub unsafe extern "C" fn rsgx_create_enclave_ex(
    path: *const c_char,
    debug: i32,
    eid: *mut EnclaveId,
    misc_attr: *mut MiscAttribute,
    ext_features_bits: u32,
    ext_features: *const [*const c_void; MAX_EXT_FEATURES_COUNT],
) -> SgxStatus {
    if path.is_null() || eid.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let debug = match debug {
        1 => true,
        0 => false,
        _ => return SgxStatus::InvalidParameter,
    };

    let path = match CStr::from_ptr(path).to_str() {
        Ok(p) => p,
        Err(_) => return SgxStatus::InvalidParameter,
    };

    if !check_ext_features(ext_features_bits, ext_features) {
        return SgxStatus::InvalidParameter;
    }

    let features_bits = ExtFeatureBits::from_bits_truncate(ext_features_bits);
    let ext_features = &*ext_features;
    let mut features = ExtFeatures::new();

    if features_bits.contains(ExtFeatureBits::KSS) {
        let kss = ext_features[KSS_BIT_IDX] as *const KssConfig;
        features.set_kss(*kss);
    }
    if features_bits.contains(ExtFeatureBits::SWITCHLESS) {
        let switchless = ext_features[SWITCHLESS_BIT_IDX] as *const SwitchlessConfig;
        features.set_switchless(*switchless);
    }

    let enclave = match SgxEnclave::create_with_features(path, debug, features) {
        Ok(enclave) => ManuallyDrop::new(enclave),
        Err(e) => return e,
    };

    *eid = enclave.eid();
    if !misc_attr.is_null() {
        *misc_attr = enclave.misc_attr().unwrap();
    }

    SgxStatus::Success
}

#[no_mangle]
pub unsafe extern "C" fn rsgx_create_enclave_from_buffer_ex(
    buffer: *const u8,
    size: usize,
    debug: i32,
    eid: *mut EnclaveId,
    misc_attr: *mut MiscAttribute,
    ext_features_bits: u32,
    ext_features: *const [*const c_void; MAX_EXT_FEATURES_COUNT],
) -> SgxStatus {
    if buffer.is_null() || size == 0 || eid.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let debug = match debug {
        1 => true,
        0 => false,
        _ => return SgxStatus::InvalidParameter,
    };

    let buffer = slice::from_raw_parts(buffer, size);

    if !check_ext_features(ext_features_bits, ext_features) {
        return SgxStatus::InvalidParameter;
    }

    let features_bits = ExtFeatureBits::from_bits_truncate(ext_features_bits);
    let ext_features = &*ext_features;
    let mut features = ExtFeatures::new();

    if features_bits.contains(ExtFeatureBits::KSS) {
        let kss = ext_features[KSS_BIT_IDX] as *const KssConfig;
        features.set_kss(*kss);
    }
    if features_bits.contains(ExtFeatureBits::SWITCHLESS) {
        let switchless = ext_features[SWITCHLESS_BIT_IDX] as *const SwitchlessConfig;
        features.set_switchless(*switchless);
    }

    let enclave = match SgxEnclave::create_from_buffer(buffer, debug, features) {
        Ok(enclave) => ManuallyDrop::new(enclave),
        Err(e) => return e,
    };

    *eid = enclave.eid();
    if !misc_attr.is_null() {
        *misc_attr = enclave.misc_attr().unwrap();
    }

    SgxStatus::Success
}

#[no_mangle]
pub unsafe extern "C" fn rsgx_destroy_enclave(eid: EnclaveId) -> SgxStatus {
    let mut enclave = ManuallyDrop::new(SgxEnclave::from_eid(eid));
    let _ = enclave.exit();

    match enclave.destroy() {
        Ok(_) => SgxStatus::Success,
        Err(e) => e,
    }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn rsgx_get_target_info(
    eid: EnclaveId,
    target_info: *mut TargetInfo,
) -> SgxStatus {
    sgx_get_target_info(eid, target_info)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn rsgx_get_metadata(
    path: *const c_char,
    metadata: *mut MetaData,
) -> SgxStatus {
    sgx_get_metadata(path, metadata)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn rsgx_get_enclave_mode() -> i32 {
    SgxEnclave::mode() as i32
}

unsafe fn check_ext_features(
    ext_features_bits: u32,
    ext_features: *const [*const c_void; MAX_EXT_FEATURES_COUNT],
) -> bool {
    if ext_features_bits != 0 && ext_features.is_null() {
        return false;
    }

    if !ext_features.is_null() {
        let ext_features = &*ext_features;
        for (i, feature) in ext_features.iter().enumerate().take(LAST_BIT_IDX + 1) {
            if (ext_features_bits & (1 << i)) == 0 && !feature.is_null() {
                return false;
            }
            if (ext_features_bits & (1 << i)) != 0 && feature.is_null() {
                return false;
            }
        }
    }

    true
}
