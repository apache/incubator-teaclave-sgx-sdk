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

impl_struct! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct CDcapRaMsg1 {
        pub g_a: Ec256PublicKey,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct CDcapURaMsg2 {
        pub g_b: Ec256PublicKey,
        pub kdf_id: u32,
        pub sign_gb_ga: Ec256Signature,
        pub mac: Mac,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct CDcapMRaMsg2 {
        pub mac: Mac,
        pub g_b: Ec256PublicKey,
        pub kdf_id: u32,
        pub quote_size: u32,
        pub quote: [u8; 0],
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct CDcapRaMsg3 {
        pub mac: Mac,
        pub g_a: Ec256PublicKey,
        pub quote_size: u32,
        pub quote: [u8; 0],
    }
}

impl_asref_array! {
    CDcapRaMsg1;
    CDcapURaMsg2;
}

impl AsRef<[u8]> for CDcapMRaMsg2 {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<CDcapMRaMsg2>() + self.quote_size as usize,
            )
        }
    }
}

impl AsRef<[u8]> for CDcapRaMsg3 {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<CDcapRaMsg3>() + self.quote_size as usize,
            )
        }
    }
}

impl_struct! {
    #[derive(Debug, Eq, PartialEq)]
    pub struct EnclaveIdentity {
        pub cpu_svn: CpuSvn,
        pub attributes: Attributes,
        pub mr_enclave: Measurement,
        pub mr_signer: Measurement,
        pub misc_select: MiscSelect,
        pub isv_prod_id: u16,
        pub isv_svn: u16,
    }
}

impl_asref_array! {
    EnclaveIdentity;
}

impl From<Report> for EnclaveIdentity {
    fn from(report: Report) -> EnclaveIdentity {
        report.body.into()
    }
}

impl From<&Report> for EnclaveIdentity {
    fn from(report: &Report) -> EnclaveIdentity {
        report.body.into()
    }
}

impl From<ReportBody> for EnclaveIdentity {
    fn from(body: ReportBody) -> EnclaveIdentity {
        EnclaveIdentity {
            cpu_svn: body.cpu_svn,
            attributes: body.attributes,
            mr_enclave: body.mr_enclave,
            mr_signer: body.mr_signer,
            misc_select: body.misc_select,
            isv_prod_id: body.isv_prod_id,
            isv_svn: body.isv_svn,
        }
    }
}

impl From<&ReportBody> for EnclaveIdentity {
    fn from(body: &ReportBody) -> EnclaveIdentity {
        EnclaveIdentity {
            cpu_svn: body.cpu_svn,
            attributes: body.attributes,
            mr_enclave: body.mr_enclave,
            mr_signer: body.mr_signer,
            misc_select: body.misc_select,
            isv_prod_id: body.isv_prod_id,
            isv_svn: body.isv_svn,
        }
    }
}

impl From<EnclaveIdentity> for CEnclaveIdentity {
    fn from(identity: EnclaveIdentity) -> CEnclaveIdentity {
        CEnclaveIdentity {
            cpu_svn: identity.cpu_svn,
            misc_select: identity.misc_select,
            reserved1: [0_u8; 28],
            attributes: identity.attributes,
            mr_enclave: identity.mr_enclave,
            reserved2: [0_u8; 32],
            mr_signer: identity.mr_signer,
            reserved3: [0_u8; 96],
            isv_prod_id: identity.isv_prod_id,
            isv_svn: identity.isv_svn,
        }
    }
}

impl From<&EnclaveIdentity> for CEnclaveIdentity {
    fn from(identity: &EnclaveIdentity) -> CEnclaveIdentity {
        CEnclaveIdentity {
            cpu_svn: identity.cpu_svn,
            misc_select: identity.misc_select,
            reserved1: [0_u8; 28],
            attributes: identity.attributes,
            mr_enclave: identity.mr_enclave,
            reserved2: [0_u8; 32],
            mr_signer: identity.mr_signer,
            reserved3: [0_u8; 96],
            isv_prod_id: identity.isv_prod_id,
            isv_svn: identity.isv_svn,
        }
    }
}

impl From<CEnclaveIdentity> for EnclaveIdentity {
    fn from(identity: CEnclaveIdentity) -> EnclaveIdentity {
        EnclaveIdentity {
            cpu_svn: identity.cpu_svn,
            attributes: identity.attributes,
            mr_enclave: identity.mr_enclave,
            mr_signer: identity.mr_signer,
            misc_select: identity.misc_select,
            isv_prod_id: identity.isv_prod_id,
            isv_svn: identity.isv_svn,
        }
    }
}

impl From<&CEnclaveIdentity> for EnclaveIdentity {
    fn from(identity: &CEnclaveIdentity) -> EnclaveIdentity {
        EnclaveIdentity {
            cpu_svn: identity.cpu_svn,
            attributes: identity.attributes,
            mr_enclave: identity.mr_enclave,
            mr_signer: identity.mr_signer,
            misc_select: identity.misc_select,
            isv_prod_id: identity.isv_prod_id,
            isv_svn: identity.isv_svn,
        }
    }
}
