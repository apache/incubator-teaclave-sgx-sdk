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

use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::function::*;
use sgx_types::metadata::MetaData;
use sgx_types::types::EnclaveMode;
use sgx_types::types::MAX_EXT_FEATURES_COUNT;
use sgx_types::types::{c_char, c_void};
use sgx_types::types::{EnclaveId, KssConfig, MiscAttribute, SwitchlessConfig, TargetInfo};
use std::ffi::CString;
use std::io;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr;

mod init;
mod uninit;

pub(crate) use init::{args, env};

#[derive(Clone, Debug, Default)]
pub struct SgxEnclave {
    eid: EnclaveId,
    path: Option<PathBuf>,
    misc_attr: Option<MiscAttribute>,
}

fn cstr(path: &Path) -> io::Result<CString> {
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

impl SgxEnclave {
    pub fn create<P: AsRef<Path>>(path: P, debug: bool) -> SgxResult<SgxEnclave> {
        let pathname: CString = cstr(path.as_ref()).map_err(|_| SgxStatus::InvalidEnclave)?;

        let mut eid: EnclaveId = 0;
        let mut misc_attr: MiscAttribute = Default::default();
        let status = unsafe {
            sgx_create_enclave(
                pathname.as_c_str().as_ptr() as *const c_char,
                i32::from(debug),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut eid as *mut EnclaveId,
                &mut misc_attr as *mut MiscAttribute,
            )
        };
        ensure!(status.is_success(), status);

        let enclave = SgxEnclave {
            eid,
            path: Some(path.as_ref().to_owned()),
            misc_attr: Some(misc_attr),
        };
        let _ = enclave.init();

        Ok(enclave)
    }

    #[cfg_attr(feature = "hyper", allow(unused_variables))]
    pub fn create_with_features<P: AsRef<Path>>(
        path: P,
        debug: bool,
        features: ExtFeatures,
    ) -> SgxResult<SgxEnclave> {
        let pathname: CString = cstr(path.as_ref()).map_err(|_| SgxStatus::InvalidEnclave)?;

        let mut eid: EnclaveId = 0;
        let mut misc_attr: MiscAttribute = Default::default();
        let mut features_p: [*const c_void; MAX_EXT_FEATURES_COUNT] =
            [ptr::null(); MAX_EXT_FEATURES_COUNT];

        if let Some(ref kss) = features.kss {
            assert!(features.bits.contains(ExtFeatureBits::KSS));
            features_p[KSS_BIT_IDX] = kss as *const _ as *const c_void;
        }
        if let Some(ref switchless) = features.switchless {
            cfg_if! {
                if #[cfg(not(feature = "hyper"))] {
                    assert!(features.bits.contains(ExtFeatureBits::SWITCHLESS));
                    features_p[SWITCHLESS_BIT_IDX] = switchless as *const _ as *const c_void;
                } else {
                    bail!(SgxStatus::UnsupportedFeature);
                }
            }
        }

        let status = unsafe {
            sgx_create_enclave_ex(
                pathname.as_c_str().as_ptr() as *const c_char,
                i32::from(debug),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut eid as *mut EnclaveId,
                &mut misc_attr as *mut MiscAttribute,
                features.bits.bits(),
                &features_p as *const _,
            )
        };
        ensure!(status.is_success(), status);

        let enclave = SgxEnclave {
            eid,
            path: Some(path.as_ref().to_owned()),
            misc_attr: Some(misc_attr),
        };
        let _ = enclave.init();

        Ok(enclave)
    }

    #[cfg_attr(feature = "hyper", allow(unused_variables))]
    pub fn create_from_buffer<B: AsRef<[u8]>>(
        buffer: B,
        debug: bool,
        features: ExtFeatures,
    ) -> SgxResult<SgxEnclave> {
        let buffer = buffer.as_ref();
        ensure!(!buffer.is_empty(), SgxStatus::InvalidParameter);

        let mut eid: EnclaveId = 0;
        let mut misc_attr: MiscAttribute = Default::default();
        let mut features_p: [*const c_void; MAX_EXT_FEATURES_COUNT] =
            [ptr::null(); MAX_EXT_FEATURES_COUNT];

        if let Some(ref kss) = features.kss {
            assert!(features.bits.contains(ExtFeatureBits::KSS));
            features_p[KSS_BIT_IDX] = kss as *const _ as *const c_void;
        }
        if let Some(ref switchless) = features.switchless {
            cfg_if! {
                if #[cfg(not(feature = "hyper"))] {
                    assert!(features.bits.contains(ExtFeatureBits::SWITCHLESS));
                    features_p[SWITCHLESS_BIT_IDX] = switchless as *const _ as *const c_void;
                } else {
                    bail!(SgxStatus::UnsupportedFeature);
                }
            }
        }

        let status = unsafe {
            sgx_create_enclave_from_buffer_ex(
                buffer.as_ptr(),
                buffer.len(),
                i32::from(debug),
                &mut eid as *mut EnclaveId,
                &mut misc_attr as *mut MiscAttribute,
                features.bits.bits(),
                &features_p as *const _,
            )
        };
        ensure!(status.is_success(), status);

        let enclave = SgxEnclave {
            eid,
            path: None,
            misc_attr: Some(misc_attr),
        };
        let _ = enclave.init();

        Ok(enclave)
    }

    #[cfg(not(feature = "hyper"))]
    pub fn create_with_switchless<P: AsRef<Path>>(
        path: P,
        debug: bool,
        uworkers: u64,
        tworkers: u64,
    ) -> SgxResult<SgxEnclave> {
        let config = SwitchlessConfig {
            uworkers,
            tworkers,
            ..Default::default()
        };

        let mut features = ExtFeatures::new();
        features.set_switchless(config);

        Self::create_with_features(path, debug, features)
    }

    #[inline]
    pub fn eid(&self) -> EnclaveId {
        self.eid
    }

    #[inline]
    pub fn path(&self) -> Option<PathBuf> {
        self.path.clone()
    }

    #[inline]
    pub fn misc_attr(&self) -> Option<MiscAttribute> {
        self.misc_attr
    }

    pub fn target_info(&self) -> SgxResult<TargetInfo> {
        let mut target: TargetInfo = Default::default();
        let status = unsafe { sgx_get_target_info(self.eid, &mut target as *mut TargetInfo) };
        if status.is_success() {
            Ok(target)
        } else {
            Err(status)
        }
    }

    pub fn metadata<P: AsRef<Path>>(path: P) -> SgxResult<MetaData> {
        let pathname: CString = cstr(path.as_ref()).map_err(|_| SgxStatus::InvalidEnclave)?;

        let mut data = MaybeUninit::zeroed();
        let status = unsafe {
            sgx_get_metadata(
                pathname.as_c_str().as_ptr() as *const c_char,
                data.as_mut_ptr(),
            )
        };

        if status.is_success() {
            Ok(unsafe { data.assume_init() })
        } else {
            Err(status)
        }
    }

    pub fn mode() -> EnclaveMode {
        cfg_if! {
            if #[cfg(feature = "sim")] {
                EnclaveMode::Sim
            } else if #[cfg(feature = "hyper")] {
                EnclaveMode::Hyper
            } else {
                EnclaveMode::Hw
            }
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) unsafe fn from_eid(eid: EnclaveId) -> SgxEnclave {
        SgxEnclave {
            eid,
            path: None,
            misc_attr: None,
        }
    }

    pub(crate) fn destroy(&mut self) -> SgxResult {
        let status = unsafe { sgx_destroy_enclave(self.eid) };
        if status.is_success() {
            Ok(())
        } else {
            Err(status)
        }
    }
}

impl Drop for SgxEnclave {
    #[inline]
    fn drop(&mut self) {
        let _ = self.exit();
        let _ = self.destroy();
    }
}

impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ExtFeatureBits: u32 {
        const PCL           = 1 << PCL_BIT_IDX;
        const SWITCHLESS    = 1 << SWITCHLESS_BIT_IDX;
        const KSS           = 1 << KSS_BIT_IDX;
    }
}

pub const PCL_BIT_IDX: usize = 0;
pub const SWITCHLESS_BIT_IDX: usize = 1;
pub const KSS_BIT_IDX: usize = 2;
pub const LAST_BIT_IDX: usize = 2;

#[derive(Debug, Default)]
pub struct ExtFeatures {
    bits: ExtFeatureBits,
    kss: Option<KssConfig>,
    switchless: Option<SwitchlessConfig>,
}

impl ExtFeatures {
    pub fn new() -> ExtFeatures {
        ExtFeatures {
            bits: ExtFeatureBits::empty(),
            kss: None,
            switchless: None,
        }
    }

    pub fn set_kss(&mut self, kss: KssConfig) {
        self.bits |= ExtFeatureBits::KSS;
        self.kss = Some(kss);
    }

    #[cfg(not(feature = "hyper"))]
    pub fn set_switchless(&mut self, switchless: SwitchlessConfig) {
        self.bits |= ExtFeatureBits::SWITCHLESS;
        self.switchless = Some(switchless);
    }
}
