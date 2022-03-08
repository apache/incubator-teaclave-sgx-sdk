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

use crate::arch::Align16;
use crate::error::abort;
use crate::se::AlignKey;
use sgx_crypto_sys::sgx_rijndael128_cmac_msg;
use sgx_types::types::{Attributes, CpuSvn, Key128bit, KeyId, KeyName, Mac, Measurement};

pub const OWNEREPOCH_SIZE: usize = 16;

pub type SeOwnerEpoch = [u8; OWNEREPOCH_SIZE];

impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct DerivSealKey {
        pub key_name: KeyName,
        pub _pad1: [u8; 6],
        pub attributes: Attributes,
        pub attribute_mask: Attributes,
        pub csr_owner_epoch: SeOwnerEpoch,
        pub cpu_svn: CpuSvn,
        pub isv_svn: u16,
        pub isv_prod_id: u16,
        pub mr_enclave: Measurement,
        pub mr_signer: Measurement,
        pub key_id: KeyId,
        pub _pad2: [u8; 4],
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct DerivReportKey {
        pub key_name: KeyName,
        pub _pad1: [u8; 6],
        pub attributes: Attributes,
        pub csr_owner_epoch: SeOwnerEpoch,
        pub mr_enclave: Measurement,
        pub cpu_svn: CpuSvn,
        pub key_id: KeyId,
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct DerivLicenseKey {
        pub key_name: KeyName,
        pub _pad1: [u8; 6],
        pub attributes: Attributes,
        pub csr_owner_epoch: SeOwnerEpoch,
        pub cpu_svn: CpuSvn,
        pub isv_svn: u16,
        pub isv_prod_id: u16,
        pub key_id: KeyId,
        pub _pad2: [u8; 4],
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct DerivProvisionKey {
        pub key_name: KeyName,
        pub _pad1: [u8; 6],
        pub attributes: Attributes,
        pub attribute_mask: Attributes,
        pub cpu_svn: CpuSvn,
        pub isv_svn: u16,
        pub isv_prod_id: u16,
        pub mr_signer: Measurement,
        pub _pad2: [u8; 4],
    }
}

impl_asref_array! {
    DerivSealKey;
    DerivReportKey;
    DerivLicenseKey;
    DerivProvisionKey;
}

// The built-in seal key in simulation mode
const BASE_SEAL_KEY: AlignKey = Align16([
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
]);

// The built-in report key in simulation mode
const BASE_REPORT_KEY: AlignKey = Align16([
    0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00,
]);

// The built-in EINIT token key in simulation mode
const BASE_EINITTOKEN_KEY: AlignKey = Align16([
    0xaa, 0x55, 0xaa, 0x55, 0xaa, 0x55, 0xaa, 0x55, 0xaa, 0x55, 0xaa, 0x55, 0xaa, 0x55, 0xaa, 0x55,
]);

// The built-in provision key in simulation mode
const BASE_PROVISION_KEY: AlignKey = Align16([
    0xbb, 0xaa, 0xbb, 0xee, 0xff, 0x00, 0x00, 0xdd, 0xbb, 0xaa, 0xbb, 0xee, 0xff, 0x00, 0x00, 0xdd,
]);

// The built-in provision-seal key in simulation mode
const BASE_PROV_SEAL_KEY: AlignKey = Align16([
    0x50, 0x52, 0x4f, 0x56, 0x49, 0x53, 0x49, 0x4f, 0x4e, 0x53, 0x45, 0x41, 0x4c, 0x4b, 0x45, 0x59,
]);

pub trait DeriveKey {
    fn derive_key(&self) -> AlignKey;
    fn base_key(&self) -> AlignKey;
}

macro_rules! impl_derive_key {
    ($($t:ty;)*) => {$(
        impl DeriveKey for $t {
            fn derive_key(&self) -> AlignKey {
                let mut key = AlignKey::default();
                let status = unsafe {
                    sgx_rijndael128_cmac_msg(
                        &self.base_key().0 as *const Key128bit,
                        self.as_ref().as_ptr(),
                        self.as_ref().len() as u32,
                        &mut key.0 as *mut _,
                    )
                };
                if !status.is_success() {
                    abort();
                }
                key
            }

            fn base_key(&self) -> AlignKey {
                match self.key_name {
                    KeyName::EInitToken => BASE_EINITTOKEN_KEY,
                    KeyName::Provision => BASE_PROVISION_KEY,
                    KeyName::ProvisionSeal => BASE_PROV_SEAL_KEY,
                    KeyName::Report => BASE_REPORT_KEY,
                    KeyName::Seal => BASE_SEAL_KEY,
                }
            }
        }
    )*}
}

impl_derive_key! {
    DerivSealKey;
    DerivReportKey;
    DerivLicenseKey;
    DerivProvisionKey;
}

pub fn cmac(key: &AlignKey, buf: &[u8]) -> Mac {
    let mut mac = Mac::default();
    let status = unsafe {
        sgx_rijndael128_cmac_msg(
            &key.0 as *const Key128bit,
            buf.as_ptr(),
            buf.len() as u32,
            &mut mac as *mut _,
        )
    };
    if !status.is_success() {
        abort();
    }
    mac
}
