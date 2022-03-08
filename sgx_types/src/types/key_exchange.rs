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

use crate::error::SgxStatus;
use core::mem;
use core::slice;

//
// sgx_key_exchange.h
//
pub type RaContext = u32;
pub type RaKey128Bit = Key128bit;

impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum RaKeyType {
        SK = 1,
        MK = 2,
    }
}

impl_struct! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct CRaMsg1 {
        pub g_a: Ec256PublicKey,
        pub gid: EpidGroupId,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct CRaMsg2 {
        pub g_b: Ec256PublicKey,
        pub spid: Spid,
        pub quote_type: u16,
        pub kdf_id: u16,
        pub sign_gb_ga: Ec256Signature,
        pub mac: Mac,
        pub sig_rl_size: u32,
        pub sig_rl: [u8; 0],
    }
}

impl_copy_clone! {
    /* intel sgx sdk 2.8 */
    #[repr(C)]
    #[derive(Debug)]
    pub struct PsSecPropDesc {
        pub ps_sec_prop_desc: [u8; 256],
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct CRaMsg3 {
        pub mac: Mac,
        pub g_a: Ec256PublicKey,
        pub ps_sec_prop: PsSecPropDesc,
        pub quote: [u8; 0],
    }
}

impl_struct_default! {
    PsSecPropDesc; //256
    CRaMsg3; //336
}

impl_asref_array! {
    CRaMsg1;
    PsSecPropDesc;
}

impl_struct_ContiguousMemory! {
    CRaMsg3;
    PsSecPropDesc;
}

impl AsRef<[u8]> for CRaMsg2 {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<CRaMsg2>() + self.sig_rl_size as usize,
            )
        }
    }
}

pub type RaDriveSecretKeyFn = unsafe extern "C" fn(
    shared_key: *const Ec256SharedKey,
    kdf_id: u16,
    smk_key: *mut Key128bit,
    sk_key: *mut Key128bit,
    mk_key: *mut Key128bit,
    vk_key: *mut Key128bit,
) -> SgxStatus;

pub type ECallGetGaFn = unsafe extern "C" fn(
    eid: EnclaveId,
    retval: *mut SgxStatus,
    context: RaContext,
    pub_key_a: *mut Ec256PublicKey,
) -> SgxStatus;

pub type ECallProcessMsg2Fn = unsafe extern "C" fn(
    eid: EnclaveId,
    retval: *mut SgxStatus,
    context: RaContext,
    msg2: *const CRaMsg2,
    qe_target: *const TargetInfo,
    report: *mut Report,
    nonce: *mut QuoteNonce,
) -> SgxStatus;

pub type ECallGetMsg3Fn = unsafe extern "C" fn(
    eid: EnclaveId,
    retval: *mut SgxStatus,
    context: RaContext,
    quote_size: u32,
    qe_report: *const Report,
    msg3: *mut CRaMsg3,
    msg3_size: u32,
) -> SgxStatus;
