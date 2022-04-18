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

use crate::arch::{Secinfo, Tcs};
use crate::enclave::EnclaveRange;
use crate::error::abort as gp;
use crate::inst::sim::derive::{
    self, DerivLicenseKey, DerivProvisionKey, DerivReportKey, DerivSealKey, DeriveKey, SeOwnerEpoch,
};
use crate::inst::sim::{TcsSim, TcsState};
use crate::inst::{INVALID_ATTRIBUTE, INVALID_CPUSVN, INVALID_ISVSVN, INVALID_LEAF};
use crate::se::{
    AlignKey, AlignKeyRequest, AlignReport, AlignReport2Mac, AlignReportData, AlignTargetInfo,
};
use core::mem;
use core::sync::atomic::Ordering;
use sgx_types::types::KEY_REQUEST_RESERVED2_BYTES;
use sgx_types::types::{
    Attributes, AttributesFlags, CpuSvn, KeyName, KeyPolicy, KeyRequest, Report, ReportBody,
};

macro_rules! gp_on {
    ($cond:expr) => {
        if ($cond) {
            gp();
        }
    };
}

macro_rules! is_unaligned {
    ($num:expr, $align:expr) => {
        $num & ($align - 1) != 0
    };
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
struct EncluRegs {
    pub rax: usize,
    pub rbx: usize,
    pub rcx: usize,
    pub rdx: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rbp: usize,
    pub rsp: usize,
    pub rip: usize,
}

const ARCH_SET_GS: i32 = 0x1001;
const ARCH_SET_FS: i32 = 0x1002;

const SIMU_OWNER_EPOCH_MSR: SeOwnerEpoch = [
    0x54, 0x48, 0x49, 0x53, 0x49, 0x53, 0x4f, 0x57, 0x4e, 0x45, 0x52, 0x45, 0x50, 0x4f, 0x43, 0x48,
];

const DEFAULT_CPUSVN: CpuSvn = CpuSvn {
    svn: [
        0x48, 0x20, 0xf3, 0x37, 0x6a, 0xe6, 0xb2, 0xf2, 0x03, 0x4d, 0x3b, 0x7a, 0x4b, 0x48, 0xa7,
        0x78,
    ],
};
const UPGRADED_CPUSVN: CpuSvn = CpuSvn {
    svn: [
        0x53, 0x39, 0xae, 0x8c, 0x93, 0xae, 0x8f, 0x3c, 0xe4, 0x32, 0xdb, 0x92, 0x4d, 0x0f, 0x07,
        0x33,
    ],
};

const DOWNGRADED_CPUSVN: CpuSvn = CpuSvn {
    svn: [
        0x64, 0xea, 0x4f, 0x3f, 0xa0, 0x03, 0x0c, 0x36, 0x38, 0x3c, 0x32, 0x2d, 0x4f, 0x3a, 0x8d,
        0x4f,
    ],
};

pub struct EncluInst;

impl EncluInst {
    #[link_section = ".nipx"]
    pub fn eexit(dest: usize, rcx: usize, rdx: usize, rsi: usize, rdi: usize) {
        let mut regs = EncluRegs {
            rsp: rcx,  // xcx = xsp = ssa.rsp_u
            rbp: rdx,  // xdx = xbp = ssa.rbp_u
            rip: dest, // dest = xbx = xcx on EENTER = return address
            ..Default::default()
        };

        let tcs = unsafe { (*((rdx - 10 * mem::size_of::<usize>()) as *mut usize)) as *mut Tcs };
        gp_on!(tcs.is_null());

        let tcs = unsafe { &mut *tcs };
        let tcs_sim = TcsSim::get(tcs);

        // restore the used _tls_array
        tcs_sim.restore_td();

        gp_on!(tcs_sim
            .tcs_state
            .compare_exchange(
                TcsState::Active.into(),
                TcsState::Inactive.into(),
                Ordering::Relaxed,
                Ordering::Relaxed
            )
            .is_err());

        regs.rax = 0;
        regs.rbx = dest;
        regs.rcx = tcs_sim.saved_aep;
        regs.rsi = rsi;
        regs.rdi = rdi;

        extern "C" {
            fn load_regs(regs: *mut EncluRegs);
        }
        unsafe { load_regs(&mut regs as *mut EncluRegs) };
        // jump back to the instruction after the call to _SE3
        // Never returns.....
    }

    pub fn egetkey(kr: &AlignKeyRequest) -> Result<AlignKey, u32> {
        gp_on!(is_unaligned!(
            kr as *const _ as usize,
            AlignKeyRequest::ALIGN_SIZE
        ));
        gp_on!(!kr.is_enclave_range());
        gp_on!(!kr.0.key_policy.is_valid());
        gp_on!(kr
            .0
            .key_policy
            .intersects(!(KeyPolicy::MRENCLAVE | KeyPolicy::MRSIGNER)));
        gp_on!(kr.0.reserved1 != 0);
        gp_on!(kr.0.reserved2 != [0; KEY_REQUEST_RESERVED2_BYTES]);

        let secs = unsafe { &*(super::GlobalSim::get().secs) };
        let cpu_svn_sim = super::GlobalSim::get().cpu_svn;
        // Determine which enclave attributes that must be included in the key.
        // Attributes that must always be included INIT & DEBUG.
        let attributes = Attributes {
            flags: (kr.0.attribute_mask.flags | AttributesFlags::INITTED | AttributesFlags::DEBUG)
                & secs.attributes.flags,
            xfrm: kr.0.attribute_mask.xfrm & secs.attributes.xfrm,
        };

        // HW supports CPUSVN to be set as 0.
        // To be consistent with HW behaviour, we replace the cpusvn as DEFAULT_CPUSVN if the input cpusvn is 0.
        let cpu_svn = if kr.0.cpu_svn.svn == [0; 16] {
            DEFAULT_CPUSVN
        } else {
            kr.0.cpu_svn
        };

        let key = match kr.0.key_name {
            KeyName::Seal => {
                if kr.0.isv_svn > secs.isv_svn {
                    return Err(INVALID_ISVSVN);
                }
                if !Self::check_cpusvn(&kr.0, cpu_svn_sim) {
                    return Err(INVALID_CPUSVN);
                }

                let derive = DerivSealKey {
                    key_name: kr.0.key_name,
                    attributes,
                    attribute_mask: kr.0.attribute_mask,
                    csr_owner_epoch: SIMU_OWNER_EPOCH_MSR,
                    cpu_svn,
                    isv_svn: kr.0.isv_svn,
                    isv_prod_id: secs.isv_prod_id,
                    mr_enclave: if kr.0.key_policy.contains(KeyPolicy::MRENCLAVE) {
                        secs.mr_enclave
                    } else {
                        Default::default()
                    },
                    mr_signer: if kr.0.key_policy.contains(KeyPolicy::MRSIGNER) {
                        secs.mr_signer
                    } else {
                        Default::default()
                    },
                    key_id: kr.0.key_id,
                    ..Default::default()
                };
                derive.derive_key()
            }
            KeyName::Report => {
                let derive = DerivReportKey {
                    key_name: kr.0.key_name,
                    attributes: secs.attributes,
                    csr_owner_epoch: SIMU_OWNER_EPOCH_MSR,
                    mr_enclave: secs.mr_enclave,
                    cpu_svn: cpu_svn_sim,
                    key_id: kr.0.key_id,
                    ..Default::default()
                };
                derive.derive_key()
            }
            KeyName::EInitToken => {
                if !secs
                    .attributes
                    .flags
                    .contains(AttributesFlags::EINITTOKENKEY)
                {
                    return Err(INVALID_ATTRIBUTE);
                }
                if kr.0.isv_svn > secs.isv_svn {
                    return Err(INVALID_ISVSVN);
                }
                if !Self::check_cpusvn(&kr.0, cpu_svn_sim) {
                    return Err(INVALID_CPUSVN);
                }

                let derive = DerivLicenseKey {
                    key_name: kr.0.key_name,
                    attributes: secs.attributes,
                    csr_owner_epoch: SIMU_OWNER_EPOCH_MSR,
                    cpu_svn,
                    isv_svn: kr.0.isv_svn,
                    isv_prod_id: secs.isv_prod_id,
                    key_id: kr.0.key_id,
                    ..Default::default()
                };
                derive.derive_key()
            }
            KeyName::Provision | KeyName::ProvisionSeal => {
                if !secs
                    .attributes
                    .flags
                    .contains(AttributesFlags::PROVISIONKEY)
                {
                    return Err(INVALID_ATTRIBUTE);
                }
                if kr.0.isv_svn > secs.isv_svn {
                    return Err(INVALID_ISVSVN);
                }
                if !Self::check_cpusvn(&kr.0, cpu_svn_sim) {
                    return Err(INVALID_CPUSVN);
                }

                let derive = DerivProvisionKey {
                    key_name: kr.0.key_name,
                    attributes,
                    attribute_mask: kr.0.attribute_mask,
                    cpu_svn,
                    isv_svn: kr.0.isv_svn,
                    isv_prod_id: secs.isv_prod_id,
                    mr_signer: secs.mr_signer,
                    ..Default::default()
                };
                derive.derive_key()
            }
        };
        Ok(key)
    }

    pub fn ereport(ti: &AlignTargetInfo, rd: &AlignReportData) -> Result<AlignReport, u32> {
        gp_on!(is_unaligned!(
            ti as *const _ as usize,
            AlignTargetInfo::ALIGN_SIZE
        ));
        gp_on!(is_unaligned!(
            rd as *const _ as usize,
            AlignReportData::ALIGN_SIZE
        ));
        gp_on!(!ti.is_enclave_range());
        gp_on!(!rd.is_enclave_range());

        let secs = unsafe { &*(super::GlobalSim::get().secs) };
        let cpu_svn_sim = super::GlobalSim::get().cpu_svn;

        let mut derive = DerivReportKey {
            key_name: KeyName::Report,
            attributes: ti.0.attributes,
            csr_owner_epoch: SIMU_OWNER_EPOCH_MSR,
            mr_enclave: secs.mr_enclave,
            cpu_svn: cpu_svn_sim,
            ..Default::default()
        };
        let base_key = derive.base_key();
        derive.key_id.id[..16].copy_from_slice(&base_key.0[..]);
        let key = derive.derive_key();

        let mut report = AlignReport(Report {
            body: ReportBody {
                cpu_svn: cpu_svn_sim,
                attributes: secs.attributes,
                mr_enclave: secs.mr_enclave,
                mr_signer: secs.mr_signer,
                isv_prod_id: secs.isv_prod_id,
                isv_svn: secs.isv_svn,
                report_data: rd.0,
                ..Default::default()
            },
            key_id: derive.key_id,
            ..Default::default()
        });

        report.0.mac = derive::cmac(&key, report.0.body.as_ref());

        Ok(report)
    }

    #[inline]
    pub fn everify_report2(_r: &AlignReport2Mac) -> Result<(), u32> {
        Err(INVALID_LEAF)
    }

    #[inline]
    pub fn eaccept(_addr: usize, _info: &Secinfo) -> Result<(), u32> {
        Ok(())
    }

    #[inline]
    pub fn emodpe(_addr: usize, _info: &Secinfo) -> Result<(), u32> {
        Ok(())
    }

    fn check_cpusvn(kr: &KeyRequest, cpu_svn: CpuSvn) -> bool {
        if kr.cpu_svn != DEFAULT_CPUSVN
            && kr.cpu_svn != UPGRADED_CPUSVN
            && kr.cpu_svn != DOWNGRADED_CPUSVN
        {
            return false;
        }

        if (cpu_svn == DEFAULT_CPUSVN && kr.cpu_svn == UPGRADED_CPUSVN)
            || (cpu_svn == DOWNGRADED_CPUSVN && kr.cpu_svn != DOWNGRADED_CPUSVN)
        {
            return false;
        }
        true
    }
}
