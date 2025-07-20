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

use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::vec::{self, Vec};
use core::alloc::Allocator;
use core::convert::From;
use core::mem;
use sgx_crypto::aes::gcm::{Aad, AesGcm, Nonce};
use sgx_trts::fence::lfence;
use sgx_trts::rand::rand;
use sgx_trts::trts::{is_within_enclave, is_within_host, EnclaveRange};
use sgx_tse::{EnclaveKey, EnclaveReport};
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::types::{
    Attributes, AttributesFlags, CSealedData, KeyId, KeyName, KeyPolicy, KeyRequest, Report,
};
use sgx_types::types::{SEAL_TAG_SIZE, TSEAL_DEFAULT_MISCMASK};

#[cfg(feature = "serialize")]
use sgx_serialize::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
pub struct InnerUnsealedData {
    pub payload_len: u32,
    pub plaintext: Box<[u8]>,
    pub aad: Box<[u8]>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
pub struct Payload {
    pub len: u32,
    pub tag: [u8; SEAL_TAG_SIZE],
    pub ciphertext: Box<[u8]>,
    pub aad: Box<[u8]>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
pub struct InnerSealedData {
    pub key_request: KeyRequest,
    pub payload: Payload,
}

impl InnerSealedData {
    pub fn raw_sealed_data_size(aad_len: u32, plaintext_len: u32) -> Option<u32> {
        let max = u32::MAX;
        let header_len = mem::size_of::<CSealedData>() as u32;

        if aad_len > max - plaintext_len {
            return None;
        }
        let payload_len = aad_len + plaintext_len;
        if payload_len > max - header_len {
            return None;
        }
        Some(header_len + payload_len)
    }

    #[inline]
    pub fn into_bytes(self) -> SgxResult<Vec<u8>> {
        self.to_bytes()
    }

    #[inline]
    pub fn to_bytes(&self) -> SgxResult<Vec<u8>> {
        self.to_bytes_in(Global)
    }

    #[inline]
    pub fn into_bytes_in<A: Allocator>(self, alloc: A) -> SgxResult<Vec<u8, A>> {
        self.to_bytes_in(alloc)
    }

    pub fn to_bytes_in<A: Allocator>(&self, alloc: A) -> SgxResult<Vec<u8, A>> {
        let raw_len = Self::raw_sealed_data_size(self.aad_len(), self.ciphertext_len())
            .ok_or(SgxStatus::Unexpected)?;
        let mut raw = vec::from_elem_in(0_u8, raw_len as usize, alloc);

        let header_len = mem::size_of::<CSealedData>();
        let ciphertext_len = self.payload.ciphertext.len();

        raw[header_len..(header_len + ciphertext_len)].copy_from_slice(&self.payload.ciphertext);
        if !self.payload.aad.is_empty() {
            raw[(header_len + ciphertext_len)..].copy_from_slice(&self.payload.aad);
        }

        let raw_data = unsafe { &mut *(raw.as_mut_ptr() as *mut CSealedData) };
        raw_data.key_request = self.key_request;
        raw_data.plaintext_offset = ciphertext_len as u32;
        raw_data.aes_data.payload_size = self.payload.len;
        raw_data.aes_data.payload_tag = self.payload.tag;

        Ok(raw)
    }

    #[inline]
    pub fn from_bytes<A: Allocator>(raw: Vec<u8, A>) -> SgxResult<InnerSealedData> {
        ensure!(
            is_within_enclave(raw.as_ptr(), raw.capacity()),
            SgxStatus::InvalidParameter
        );
        Self::from_slice(&raw)
    }

    pub fn from_slice(raw: &[u8]) -> SgxResult<InnerSealedData> {
        ensure!(raw.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(
            raw.len() >= mem::size_of::<CSealedData>(),
            SgxStatus::InvalidParameter
        );

        let raw_data = unsafe { &*(raw.as_ptr() as *const CSealedData) };

        ensure!(
            raw_data.plaintext_offset <= raw_data.aes_data.payload_size,
            SgxStatus::InvalidParameter
        );
        let aad_len = raw_data.aes_data.payload_size - raw_data.plaintext_offset;
        let ciphertext_len = raw_data.plaintext_offset;
        ensure!(
            ciphertext_len + aad_len == raw_data.aes_data.payload_size,
            SgxStatus::InvalidParameter
        );

        let raw_len = Self::raw_sealed_data_size(aad_len, ciphertext_len)
            .ok_or(SgxStatus::InvalidParameter)?;
        ensure!(raw.len() == raw_len as usize, SgxStatus::InvalidParameter);

        let header_len = mem::size_of::<CSealedData>();
        let ciphertext: Box<[u8]> =
            Box::from(&raw[header_len..header_len + ciphertext_len as usize]);
        let aad: Box<[u8]> = Box::from(&raw[header_len + ciphertext_len as usize..]);

        Ok(InnerSealedData {
            key_request: raw_data.key_request,
            payload: Payload {
                len: raw_data.aes_data.payload_size,
                tag: raw_data.aes_data.payload_tag,
                ciphertext,
                aad,
            },
        })
    }

    pub fn aad_len(&self) -> u32 {
        let data_len = self.payload.aad.len();
        if data_len > self.payload.len as usize || data_len >= u32::MAX as usize {
            u32::MAX
        } else {
            data_len as u32
        }
    }

    pub fn ciphertext_len(&self) -> u32 {
        let data_len = self.payload.ciphertext.len();
        if data_len > self.payload.len as usize || data_len >= u32::MAX as usize {
            u32::MAX
        } else {
            data_len as u32
        }
    }

    pub fn seal(plaintext: &[u8], aad: Option<&[u8]>) -> SgxResult<InnerSealedData> {
        let attribute_mask = Attributes {
            flags: AttributesFlags::DEFAULT_MASK,
            xfrm: 0,
        };

        let mut key_policy = KeyPolicy::MRSIGNER;
        let report = Report::get_self();
        if report
            .body
            .attributes
            .flags
            .intersects(AttributesFlags::KSS)
        {
            key_policy |= KeyPolicy::KSS;
        }

        Self::seal_with_key_policy(
            key_policy,
            attribute_mask,
            TSEAL_DEFAULT_MISCMASK,
            plaintext,
            aad,
        )
    }

    pub fn seal_with_key_policy(
        key_policy: KeyPolicy,
        attribute_mask: Attributes,
        misc_mask: u32,
        plaintext: &[u8],
        aad: Option<&[u8]>,
    ) -> SgxResult<InnerSealedData> {
        let aad_len = aad.map(|aad| aad.len()).unwrap_or(0);
        let plaintext_len = plaintext.len();

        ensure!(
            (0..u32::MAX as usize).contains(&aad_len),
            SgxStatus::InvalidParameter
        );
        ensure!(
            (1..u32::MAX as usize).contains(&plaintext_len),
            SgxStatus::InvalidParameter
        );
        ensure!(
            Self::raw_sealed_data_size(aad_len as u32, plaintext_len as u32).is_some(),
            SgxStatus::InvalidParameter
        );

        ensure!(key_policy.is_valid(), SgxStatus::InvalidParameter);
        ensure!(
            key_policy.intersects(KeyPolicy::MRENCLAVE | KeyPolicy::MRSIGNER),
            SgxStatus::InvalidParameter
        );
        ensure!(
            attribute_mask
                .flags
                .contains(AttributesFlags::INITTED | AttributesFlags::DEBUG),
            SgxStatus::InvalidParameter
        );
        ensure!(plaintext.is_enclave_range(), SgxStatus::InvalidParameter);

        let zero = [0_u8; 0];
        let aad = if let Some(aad) = aad { aad } else { &zero };

        if aad_len > 0 {
            ensure!(
                aad.is_enclave_range() || aad.is_host_range(),
                SgxStatus::InvalidParameter
            );
        }

        let mut key_id = KeyId::default();
        rand(key_id.as_mut())?;
        let report = Report::get_self();
        let key_request = KeyRequest {
            key_name: KeyName::Seal,
            key_policy,
            isv_svn: report.body.isv_svn,
            cpu_svn: report.body.cpu_svn,
            attribute_mask,
            key_id,
            misc_mask,
            config_svn: report.body.config_svn,
            ..Default::default()
        };

        let result = Self::seal_data_helper(plaintext, aad, &key_request);
        key_id.as_mut().fill(0);
        result
    }

    #[inline]
    pub fn unseal(&self) -> SgxResult<InnerUnsealedData> {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);

        self.unseal_data_helper()
    }

    pub fn mac(aad: &[u8]) -> SgxResult<InnerSealedData> {
        let attribute_mask = Attributes {
            flags: AttributesFlags::DEFAULT_MASK,
            xfrm: 0,
        };

        let mut key_policy = KeyPolicy::MRSIGNER;
        let report = Report::get_self();
        if report
            .body
            .attributes
            .flags
            .intersects(AttributesFlags::KSS)
        {
            key_policy = KeyPolicy::MRSIGNER | KeyPolicy::KSS;
        }

        Self::mac_with_key_policy(key_policy, attribute_mask, TSEAL_DEFAULT_MISCMASK, aad)
    }

    pub fn mac_with_key_policy(
        key_policy: KeyPolicy,
        attribute_mask: Attributes,
        misc_mask: u32,
        aad: &[u8],
    ) -> SgxResult<InnerSealedData> {
        let aad_len = aad.len();
        ensure!(
            (1..u32::MAX as usize).contains(&aad_len),
            SgxStatus::InvalidParameter
        );
        ensure!(
            Self::raw_sealed_data_size(aad_len as u32, 0).is_some(),
            SgxStatus::InvalidParameter
        );

        ensure!(key_policy.is_valid(), SgxStatus::InvalidParameter);
        ensure!(
            key_policy.intersects(KeyPolicy::MRENCLAVE | KeyPolicy::MRSIGNER),
            SgxStatus::InvalidParameter
        );
        ensure!(
            attribute_mask
                .flags
                .contains(AttributesFlags::INITTED | AttributesFlags::DEBUG),
            SgxStatus::InvalidParameter
        );
        ensure!(
            aad.is_enclave_range() || aad.is_host_range(),
            SgxStatus::InvalidParameter
        );

        let mut key_id = KeyId::default();
        rand(key_id.as_mut())?;
        let report = Report::get_self();
        let key_request = KeyRequest {
            key_name: KeyName::Seal,
            key_policy,
            isv_svn: report.body.isv_svn,
            cpu_svn: report.body.cpu_svn,
            attribute_mask,
            key_id,
            misc_mask,
            config_svn: report.body.config_svn,
            ..Default::default()
        };

        let result = Self::mac_data_helper(aad, &key_request);
        key_id.as_mut().fill(0);
        result
    }

    #[inline]
    pub fn verify(&self) -> SgxResult<InnerUnsealedData> {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);

        self.verify_data_helper()
    }

    fn seal_data_helper(
        plaintext: &[u8],
        aad: &[u8],
        key_request: &KeyRequest,
    ) -> SgxResult<InnerSealedData> {
        let mut key = key_request.get_align_key()?;
        let mut aes = AesGcm::new(&key.key, Nonce::zeroed(), Aad::from(aad))?;

        let mut ciphertext = vec![0_u8; plaintext.len()].into_boxed_slice();
        let result = aes.encrypt(plaintext, &mut ciphertext);
        key.as_mut().fill(0);
        let tag = result?;

        Ok(InnerSealedData {
            key_request: *key_request,
            payload: Payload {
                len: (plaintext.len() + aad.len()) as u32,
                tag,
                ciphertext,
                aad: Box::from(aad),
            },
        })
    }

    fn unseal_data_helper(&self) -> SgxResult<InnerUnsealedData> {
        let mut key = self.key_request.get_align_key().map_err(|e| {
            if e == SgxStatus::InvalidCpusvn
                || e == SgxStatus::InvalidIsvsvn
                || e == SgxStatus::OutOfMemory
            {
                e
            } else {
                SgxStatus::MacMismatch
            }
        })?;

        lfence();

        let mut plaintext = vec![0_u8; self.payload.ciphertext.len()].into_boxed_slice();
        let mut aes = AesGcm::new(&key.key, Nonce::zeroed(), Aad::from(&self.payload.aad))?;
        let result = aes.decrypt(&self.payload.ciphertext, &mut plaintext, &self.payload.tag);
        key.as_mut().fill(0);
        result?;

        Ok(InnerUnsealedData {
            payload_len: self.payload.len,
            plaintext,
            aad: self.payload.aad.clone(),
        })
    }

    fn mac_data_helper(aad: &[u8], key_request: &KeyRequest) -> SgxResult<InnerSealedData> {
        let mut key = key_request.get_align_key()?;
        let mut aes = AesGcm::new(&key.key, Nonce::zeroed(), Aad::from(aad))?;

        let result = aes.mac();
        key.as_mut().fill(0);
        let tag = result?;

        Ok(InnerSealedData {
            key_request: *key_request,
            payload: Payload {
                len: aad.len() as u32,
                tag,
                ciphertext: Box::default(),
                aad: Box::from(aad),
            },
        })
    }

    fn verify_data_helper(&self) -> SgxResult<InnerUnsealedData> {
        let mut key = self.key_request.get_align_key().map_err(|e| match e {
            SgxStatus::InvalidCpusvn | SgxStatus::InvalidIsvsvn | SgxStatus::OutOfMemory => e,
            _ => SgxStatus::MacMismatch,
        })?;

        lfence();

        let mut aes = AesGcm::new(&key.key, Nonce::zeroed(), Aad::from(&self.payload.aad))?;
        let result = aes.verify_mac(&self.payload.tag);
        key.as_mut().fill(0);
        result?;

        Ok(InnerUnsealedData {
            payload_len: self.payload.len,
            plaintext: Box::default(),
            aad: self.payload.aad.clone(),
        })
    }
}

impl EnclaveRange for Payload {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(self as *const _ as *const u8, mem::size_of_val(self)) {
            return false;
        }

        if !self.ciphertext.is_empty()
            && !is_within_enclave(self.ciphertext.as_ptr(), self.ciphertext.len())
        {
            return false;
        }

        if !self.aad.is_empty() && !is_within_enclave(self.aad.as_ptr(), self.aad.len()) {
            return false;
        }

        true
    }
    fn is_host_range(&self) -> bool {
        if !is_within_host(self as *const _ as *const u8, mem::size_of_val(self)) {
            return false;
        }

        if !self.ciphertext.is_empty()
            && !is_within_host(self.ciphertext.as_ptr(), self.ciphertext.len())
        {
            return false;
        }

        if !self.aad.is_empty() && !is_within_host(self.aad.as_ptr(), self.aad.len()) {
            return false;
        }

        true
    }
}

impl EnclaveRange for InnerSealedData {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(self as *const _ as *const u8, mem::size_of_val(self)) {
            return false;
        }
        self.payload.is_enclave_range()
    }
    fn is_host_range(&self) -> bool {
        if !is_within_host(self as *const _ as *const u8, mem::size_of_val(self)) {
            return false;
        }
        self.payload.is_host_range()
    }
}

impl EnclaveRange for InnerUnsealedData {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(self as *const _ as *const u8, mem::size_of_val(self)) {
            return false;
        }

        if !self.plaintext.is_empty()
            && !is_within_enclave(self.plaintext.as_ptr(), self.plaintext.len())
        {
            return false;
        }

        if !self.aad.is_empty() && !is_within_enclave(self.aad.as_ptr(), self.aad.len()) {
            return false;
        }

        true
    }
    fn is_host_range(&self) -> bool {
        if !is_within_host(self as *const _ as *const u8, mem::size_of_val(self)) {
            return false;
        }

        if !self.plaintext.is_empty()
            && !is_within_host(self.plaintext.as_ptr(), self.plaintext.len())
        {
            return false;
        }

        if !self.aad.is_empty() && !is_within_host(self.aad.as_ptr(), self.aad.len()) {
            return false;
        }

        true
    }
}
