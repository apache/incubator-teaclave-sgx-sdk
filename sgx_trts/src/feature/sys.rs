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

#[cfg(feature = "hyper")]
use crate::call::MsbufInfo;
use crate::enclave::EnclaveRange;
use crate::fence::lfence;
use crate::xsave;
use core::cmp;
use core::convert::TryFrom;
use core::mem;
use core::ptr;
use core::ptr::NonNull;
use sgx_types::cpu_features::*;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::ContiguousMemory;
use sgx_types::types;

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum Version {
        Sdk1_5 = 0,
        Sdk2_0 = 1,
        Sdk2_1 = 2,
        Sdk2_2 = 3,
        Sdk2_3 = 4,
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct SystemFeatures {
    pub cpu_features: u64,
    pub version: u32,
    pub system_feature_set: [u64; 1],
    pub cpuinfo_table: [[u32; 4]; 8],
    pub sealed_key: usize,
    pub size: usize,
    pub cpu_features_ext: u64,
    pub cpu_core_num: u32,
    #[cfg(feature = "hyper")]
    pub msbuf_info: MsbufInfo,
}

unsafe impl ContiguousMemory for SystemFeatures {}

impl SystemFeatures {
    const SYS_FEATURE_EXTEND: u64 = 62;
    const EDMM_ENABLE_BIT: u64 = 1;

    pub unsafe fn from_raw(features: NonNull<SystemFeatures>) -> SgxResult<SystemFeatures> {
        ensure!(features.as_ref().is_host_range(), SgxStatus::Unexpected);
        lfence();

        let mut features = *features.as_ptr();
        features.clear_unused_fileds();

        #[cfg(feature = "hyper")]
        {
            use crate::inst::GlobalHyper;
            GlobalHyper::get_mut().set_msbuf_info(&features.msbuf_info)?;
        }

        Ok(features)
    }

    unsafe fn clear_unused_fileds(&mut self) {
        let offset =
            if (self.system_feature_set[0] & (1_u64 << SystemFeatures::SYS_FEATURE_EXTEND)) != 0 {
                cmp::min(self.size, mem::size_of::<Self>())
            } else {
                let b = self as *const _ as usize;
                let p = &self.size as *const _ as usize;
                p - b
            };

        let remaining = mem::size_of::<Self>() - offset;
        if remaining > 0 {
            let p = self as *mut _ as *mut u8;
            ptr::write_bytes(p.add(offset), 0, remaining);
        }
    }

    pub fn is_edmm(&self) -> bool {
        match self.version {
            0 => false,
            _ => (self.system_feature_set[0] & SystemFeatures::EDMM_ENABLE_BIT) != 0,
        }
    }

    pub fn cpu_features_bit(&self, xfrm: u64) -> SgxResult<u64> {
        const XFEATURE_ENABLED_AVX: u64 = 0x06;
        const XFEATURE_ENABLED_AVX3: u64 = 0xE0;

        // Unset conflict cpu feature bits for legacy cpu features.
        let mut features_bit = self.cpu_features & (!INCOMPAT_FEATURE_BIT);
        if (self.system_feature_set[0] & (1_u64 << SystemFeatures::SYS_FEATURE_EXTEND)) != 0 {
            // The sys_features structure is collected by updated uRTS, so use the updated cpu features instead.
            features_bit = self.cpu_features_ext;
        }

        // Confirm the reserved bits and the unset bits by uRTS must be 0.
        if (features_bit & RESERVED_CPU_FEATURE_BIT) != 0 {
            // clear the reserved bits
            features_bit &= !RESERVED_CPU_FEATURE_BIT;
        }

        if (features_bit & (!(CPU_FEATURE_SSE4_1 - 1))) == 0 {
            bail!(SgxStatus::Unexpected);
        }

        // Check for inconsistencies in the CPUID feature mask.
        if (((features_bit & CPU_FEATURE_SSE) == CPU_FEATURE_SSE)
            && ((features_bit & (CPU_FEATURE_SSE - 1)) != (CPU_FEATURE_SSE - 1)))
            || (((features_bit & CPU_FEATURE_SSE2) == CPU_FEATURE_SSE2)
                && ((features_bit & (CPU_FEATURE_SSE2 - 1)) != (CPU_FEATURE_SSE2 - 1)))
            || (((features_bit & CPU_FEATURE_SSE3) == CPU_FEATURE_SSE3)
                && ((features_bit & (CPU_FEATURE_SSE3 - 1)) != (CPU_FEATURE_SSE3 - 1)))
            || (((features_bit & CPU_FEATURE_SSSE3) == CPU_FEATURE_SSSE3)
                && ((features_bit & (CPU_FEATURE_SSSE3 - 1)) != (CPU_FEATURE_SSSE3 - 1)))
            || (((features_bit & CPU_FEATURE_SSE4_1) == CPU_FEATURE_SSE4_1)
                && ((features_bit & (CPU_FEATURE_SSE4_1 - 1)) != (CPU_FEATURE_SSE4_1 - 1)))
            || (((features_bit & CPU_FEATURE_SSE4_2) == CPU_FEATURE_SSE4_2)
                && ((features_bit & (CPU_FEATURE_SSE4_2 - 1)) != (CPU_FEATURE_SSE4_2 - 1)))
        {
            bail!(SgxStatus::Unexpected);
        }

        // Determine whether the OS & ENCLAVE support SAVE/RESTORE of the AVX register set
        // IF NOT, clear the advanced feature set bits corresponding to AVX and beyond.
        if xfrm & XFEATURE_ENABLED_AVX != XFEATURE_ENABLED_AVX {
            // AVX is disabled by OS, so clear the AVX related feature bits
            features_bit &= !(CPU_FEATURE_AVX
                | CPU_FEATURE_VAES
                | CPU_FEATURE_VPCLMULQDQ
                | CPU_FEATURE_F16C
                | CPU_FEATURE_AVX2
                | CPU_FEATURE_FMA
                | CPU_FEATURE_MPX
                | CPU_FEATURE_RTM
                | CPU_FEATURE_HLE
                | CPU_FEATURE_BMI
                | CPU_FEATURE_RDSEED
                | CPU_FEATURE_ADX
                | CPU_FEATURE_AVX512F
                | CPU_FEATURE_AVX512CD
                | CPU_FEATURE_AVX512ER
                | CPU_FEATURE_AVX512PF
                | CPU_FEATURE_AVX512DQ
                | CPU_FEATURE_AVX512BW
                | CPU_FEATURE_AVX512VL
                | CPU_FEATURE_AVX512IFMA52
                | CPU_FEATURE_AVX512VBMI
                | CPU_FEATURE_AVX512_VPOPCNTDQ
                | CPU_FEATURE_AVX512_4VNNIW
                | CPU_FEATURE_AVX512_4FMAPS
                | CPU_FEATURE_AVX512_BITALG
                | CPU_FEATURE_AVX512_VBMI2
                | CPU_FEATURE_AVX512_VNNI);
        } else if xfrm & XFEATURE_ENABLED_AVX3 != XFEATURE_ENABLED_AVX3 {
            // AVX3 is disabled by OS, so clear the AVX related feature bits
            features_bit &= !(CPU_FEATURE_AVX512F
                | CPU_FEATURE_AVX512CD
                | CPU_FEATURE_AVX512ER
                | CPU_FEATURE_AVX512PF
                | CPU_FEATURE_AVX512DQ
                | CPU_FEATURE_AVX512BW
                | CPU_FEATURE_AVX512VL
                | CPU_FEATURE_AVX512IFMA52
                | CPU_FEATURE_AVX512VBMI
                | CPU_FEATURE_AVX512_VPOPCNTDQ
                | CPU_FEATURE_AVX512_4VNNIW
                | CPU_FEATURE_AVX512_4FMAPS
                | CPU_FEATURE_AVX512_BITALG
                | CPU_FEATURE_AVX512_VBMI2
                | CPU_FEATURE_AVX512_VNNI);
        }

        Ok(features_bit)
    }
}

#[derive(Debug)]
pub struct SysFeatures {
    version: Version,
    xfrm: u64,
    cpu_features: u64,
    cpu_core_num: u32,
    cpuinfo_table: [[u32; 4]; 8],
    is_edmm: bool,
}

unsafe impl ContiguousMemory for SysFeatures {}

#[link_section = ".data.rel.ro"]
static mut SYS_FEATURES: SysFeatures = SysFeatures {
    version: Version::Sdk1_5,
    xfrm: types::XFRM_LEGACY,
    cpu_features: 0,
    cpu_core_num: 0,
    cpuinfo_table: [[0; 4]; 8],
    is_edmm: false,
};

// Improve compatibility
// e.g. intel-sgx-ssl handles CPUID with this global variable.
#[link_section = ".data.rel.ro"]
#[no_mangle]
pub static mut g_cpu_feature_indicator: u64 = 0;

impl SysFeatures {
    pub fn init(raw: NonNull<SystemFeatures>) -> SgxResult<&'static SysFeatures> {
        let raw = unsafe { SystemFeatures::from_raw(raw) }?;
        let version = Version::try_from(raw.version).map_err(|_| SgxStatus::Unexpected)?;
        let feature = unsafe { SysFeatures::get_mut() };

        feature.version = version;
        feature.xfrm = xsave::get_xfrm();
        feature.cpu_core_num = raw.cpu_core_num;
        feature.cpuinfo_table = raw.cpuinfo_table;
        feature.is_edmm = raw.is_edmm();
        feature.cpu_features = raw.cpu_features_bit(feature.xfrm)?;

        unsafe {
            g_cpu_feature_indicator = feature.cpu_features;
        }
        Ok(SysFeatures::get())
    }

    #[inline]
    pub fn get() -> &'static SysFeatures {
        unsafe { &SYS_FEATURES }
    }

    #[inline]
    pub unsafe fn get_mut() -> &'static mut SysFeatures {
        &mut SYS_FEATURES
    }

    #[inline]
    pub fn is_edmm(&self) -> bool {
        self.is_edmm
    }

    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    #[inline]
    pub fn xfrm(&self) -> u64 {
        self.xfrm
    }

    #[inline]
    pub fn cpu_features(&self) -> u64 {
        self.cpu_features
    }

    #[inline]
    pub fn core_mum(&self) -> u32 {
        self.cpu_core_num
    }

    #[inline]
    pub fn cpuinfo_table(&self) -> &[[u32; 4]; 8] {
        &self.cpuinfo_table
    }
}
