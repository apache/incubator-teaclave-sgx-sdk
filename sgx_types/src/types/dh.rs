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

use core::mem;
use core::slice;

//
// sgx_dh.h
//
#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct CDhMsg1 {
    pub g_a: Ec256PublicKey,
    pub target: TargetInfo,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct CDhMsg2 {
    pub g_b: Ec256PublicKey,
    pub report: Report,
    pub cmac: Mac128bit,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct CDhMsg3 {
    pub cmac: Mac128bit,
    pub msg_body: CDhMsg3Body,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct CDhMsg3Body {
    pub report: Report,
    pub add_prop_len: u32,
    pub add_prop: [u8; 0],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct CEnclaveIdentity {
    pub cpu_svn: CpuSvn,
    pub misc_select: MiscSelect,
    pub reserved1: [u8; 28],
    pub attributes: Attributes,
    pub mr_enclave: Measurement,
    pub reserved2: [u8; 32],
    pub mr_signer: Measurement,
    pub reserved3: [u8; 96],
    pub isv_prod_id: u16,
    pub isv_svn: u16,
}

impl From<Report> for CEnclaveIdentity {
    fn from(report: Report) -> CEnclaveIdentity {
        CEnclaveIdentity {
            cpu_svn: report.body.cpu_svn,
            misc_select: report.body.misc_select,
            reserved1: [0_u8; 28],
            attributes: report.body.attributes,
            mr_enclave: report.body.mr_enclave,
            reserved2: [0_u8; 32],
            mr_signer: report.body.mr_signer,
            reserved3: [0_u8; 96],
            isv_prod_id: report.body.isv_prod_id,
            isv_svn: report.body.isv_svn,
        }
    }
}

impl From<&Report> for CEnclaveIdentity {
    fn from(report: &Report) -> CEnclaveIdentity {
        CEnclaveIdentity {
            cpu_svn: report.body.cpu_svn,
            misc_select: report.body.misc_select,
            reserved1: [0_u8; 28],
            attributes: report.body.attributes,
            mr_enclave: report.body.mr_enclave,
            reserved2: [0_u8; 32],
            mr_signer: report.body.mr_signer,
            reserved3: [0_u8; 96],
            isv_prod_id: report.body.isv_prod_id,
            isv_svn: report.body.isv_svn,
        }
    }
}

pub const DH_SESSION_DATA_SIZE: usize = 256;

#[derive(Clone, Copy, Debug)]
#[repr(C, align(64))]
pub struct CDhSession {
    pub dh_session: [u8; DH_SESSION_DATA_SIZE],
}

impl_struct_default! {
    CEnclaveIdentity;
    CDhSession;
}

impl_asref_array! {
    CDhMsg1;
    CDhMsg2;
    CEnclaveIdentity;
    CDhSession;
}

impl_struct_ContiguousMemory! {
    CDhMsg1;
    CDhMsg2;
    CDhMsg3;
    CDhMsg3Body;
    CEnclaveIdentity;
    CDhSession;
}

impl AsRef<[u8]> for CDhMsg3 {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<CDhMsg3>() + self.msg_body.add_prop_len as usize,
            )
        }
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum DhSessionRole {
        Initiator = 0,
        Responder = 1,
    }
}
