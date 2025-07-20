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
use core::mem;
use core::slice;
use sgx_crypto::ecc::{EcPrivateKey, EcPublicKey, EcSignature};
use sgx_crypto::mac::AesCMac;
#[cfg(feature = "tmsg")]
use sgx_trts::trts::{is_within_enclave, is_within_host, EnclaveRange};
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::memeq::ConstTimeEq;
use sgx_types::types::{
    AlignKey128bit, CDcapMRaMsg2, CDcapRaMsg1, CDcapRaMsg3, CDcapURaMsg2, Ec256PublicKey, Mac,
    QlAuthData, QlCertificationData, QlEcdsaSigData, Quote3,
};

#[cfg(any(feature = "tserialize", feature = "userialize"))]
use sgx_serialize::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct DcapRaMsg1 {
    pub pub_key_a: EcPublicKey,
}

impl_struct_ContiguousMemory! {
    DcapRaMsg1;
}
impl_asref_array! {
    DcapRaMsg1;
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct DcapURaMsg2 {
    pub pub_key_b: EcPublicKey,
    pub kdf_id: u32,
    pub sign_gb_ga: EcSignature,
    pub mac: Mac,
}

impl_struct_ContiguousMemory! {
    DcapURaMsg2;
}
impl_asref_array! {
    DcapURaMsg2;
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct DcapMRaMsg2 {
    pub mac: Mac,
    pub pub_key_b: EcPublicKey,
    pub kdf_id: u32,
    pub quote: Box<[u8]>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct DcapRaMsg3 {
    pub mac: Mac,
    pub pub_key_a: EcPublicKey,
    pub quote: Box<[u8]>,
}

impl DcapRaMsg1 {
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
        let raw_msg: CDcapRaMsg1 = self.into();
        Ok(raw_msg.as_ref().to_vec_in(alloc))
    }

    pub fn copy_to_slice(&self, bytes: &mut [u8]) -> SgxResult {
        ensure!(
            bytes.len() == mem::size_of::<CDcapRaMsg1>(),
            SgxStatus::InvalidParameter
        );

        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CDcapRaMsg1) };
        raw_msg.g_a = self.pub_key_a.into();
        Ok(())
    }

    #[inline]
    pub fn from_bytes<A: Allocator>(bytes: Vec<u8, A>) -> SgxResult<DcapRaMsg1> {
        Self::from_slice(bytes.as_slice())
    }

    pub fn from_slice(bytes: &[u8]) -> SgxResult<DcapRaMsg1> {
        let raw_msg = unsafe { &*(bytes.as_ptr() as *const CDcapRaMsg1) };
        Ok(raw_msg.into())
    }
}

impl DcapMRaMsg2 {
    pub fn gen_cmac(&mut self, cmac_key: &AlignKey128bit) -> SgxResult {
        let mut cmac = AesCMac::new(&cmac_key.key)?;
        cmac.update(&self.pub_key_b)?;
        cmac.update(&self.kdf_id)?;
        cmac.update(&self.quote.len())?;
        cmac.update(&self.quote[..])?;
        self.mac = cmac.finalize()?;

        Ok(())
    }

    pub fn verify_cmac(&self, cmac_key: &AlignKey128bit) -> SgxResult {
        let mut cmac = AesCMac::new(&cmac_key.key)?;
        cmac.update(&self.pub_key_b)?;
        cmac.update(&self.kdf_id)?;
        cmac.update(&self.quote.len())?;
        cmac.update(&self.quote[..])?;
        let mac = cmac.finalize()?;

        ensure!(mac.ct_eq(&self.mac), SgxStatus::MacMismatch);
        Ok(())
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
        ensure!(!self.quote.is_empty(), SgxStatus::InvalidParameter);
        let raw_len = self.get_raw_ize().ok_or(SgxStatus::InvalidParameter)?;

        let mut bytes = vec::from_elem_in(0_u8, raw_len as usize, alloc);
        let header_len = mem::size_of::<CDcapMRaMsg2>() as u32;

        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CDcapMRaMsg2) };
        raw_msg.mac = self.mac;
        raw_msg.g_b = self.pub_key_b.into();
        raw_msg.kdf_id = self.kdf_id;
        raw_msg.quote_size = self.quote.len() as u32;
        bytes[header_len as usize..].copy_from_slice(&self.quote);

        Ok(bytes)
    }

    pub fn copy_to_slice(&self, bytes: &mut [u8]) -> SgxResult {
        ensure!(!self.quote.is_empty(), SgxStatus::InvalidParameter);
        let raw_len = self.get_raw_ize().ok_or(SgxStatus::InvalidParameter)?;
        ensure!(bytes.len() == raw_len as usize, SgxStatus::InvalidParameter);

        let header_len = mem::size_of::<CDcapMRaMsg2>() as u32;
        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CDcapMRaMsg2) };

        raw_msg.mac = self.mac;
        raw_msg.g_b = self.pub_key_b.into();
        raw_msg.kdf_id = self.kdf_id;
        raw_msg.quote_size = self.quote.len() as u32;
        bytes[header_len as usize..].copy_from_slice(&self.quote);

        Ok(())
    }

    #[inline]
    pub fn from_bytes<A: Allocator>(bytes: Vec<u8, A>) -> SgxResult<DcapMRaMsg2> {
        Self::from_slice(bytes.as_slice())
    }

    pub fn from_slice(bytes: &[u8]) -> SgxResult<DcapMRaMsg2> {
        let raw_msg_len = bytes.len();
        ensure!(
            raw_msg_len > mem::size_of::<CDcapMRaMsg2>(),
            SgxStatus::InvalidParameter
        );

        let header_len = mem::size_of::<CDcapMRaMsg2>();
        let raw_msg = unsafe { &*(bytes.as_ptr() as *const CDcapMRaMsg2) };

        let quote_len = raw_msg_len - header_len;
        ensure!(
            Self::check_quote_len(quote_len),
            SgxStatus::InvalidParameter
        );
        ensure!(
            quote_len == raw_msg.quote_size as usize,
            SgxStatus::InvalidParameter
        );

        let mut quote = vec![0_u8; quote_len];
        quote.as_mut_slice().copy_from_slice(unsafe {
            slice::from_raw_parts(&raw_msg.quote as *const _ as *const u8, quote_len)
        });

        Ok(DcapMRaMsg2 {
            mac: raw_msg.mac,
            pub_key_b: raw_msg.g_b.into(),
            kdf_id: raw_msg.kdf_id,
            quote: quote.into_boxed_slice(),
        })
    }

    pub fn get_raw_ize(&self) -> Option<u32> {
        let quote_len = self.quote.len();

        if Self::check_quote_len(quote_len) {
            Some((mem::size_of::<CDcapMRaMsg2>() + quote_len) as u32)
        } else {
            None
        }
    }

    #[inline]
    pub fn check_quote_len(quote_len: usize) -> bool {
        quote_len <= (u32::MAX as usize) - mem::size_of::<CDcapMRaMsg2>()
            && quote_len
                > (mem::size_of::<Quote3>()
                    + mem::size_of::<QlEcdsaSigData>()
                    + mem::size_of::<QlAuthData>()
                    + mem::size_of::<QlCertificationData>())
    }
}

impl DcapURaMsg2 {
    pub fn gen_sign_and_cmac(
        &mut self,
        pub_key_a: &EcPublicKey,
        sign_priv_key: &EcPrivateKey,
        cmac_key: &AlignKey128bit,
    ) -> SgxResult {
        let keys: [Ec256PublicKey; 2] = [self.pub_key_b.into(), pub_key_a.into()];
        let sign_gb_ga = sign_priv_key.sign(&keys)?;

        let mut cmac = AesCMac::new(&cmac_key.key)?;
        cmac.update(&self.pub_key_b)?;
        cmac.update(&self.kdf_id)?;
        cmac.update(&sign_gb_ga)?;
        let mac = cmac.finalize()?;

        self.sign_gb_ga = sign_gb_ga;
        self.mac = mac;

        Ok(())
    }

    pub fn verify_sign_and_cmac(
        &self,
        pub_key_a: &EcPublicKey,
        verify_pub_key: &EcPublicKey,
        cmac_key: &AlignKey128bit,
    ) -> SgxResult {
        let keys: [Ec256PublicKey; 2] = [self.pub_key_b.into(), pub_key_a.into()];
        let is_valid = verify_pub_key.verify(&keys, &self.sign_gb_ga)?;
        ensure!(is_valid, SgxStatus::InvalidSignature);

        let mut cmac = AesCMac::new(&cmac_key.key)?;
        cmac.update(&self.pub_key_b)?;
        cmac.update(&self.kdf_id)?;
        cmac.update(&self.sign_gb_ga)?;
        let mac = cmac.finalize()?;

        ensure!(mac.ct_eq(&self.mac), SgxStatus::MacMismatch);
        Ok(())
    }

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
        let raw_msg: CDcapURaMsg2 = self.into();
        Ok(raw_msg.as_ref().to_vec_in(alloc))
    }

    pub fn copy_to_slice(&self, bytes: &mut [u8]) -> SgxResult {
        ensure!(
            bytes.len() == mem::size_of::<CDcapURaMsg2>(),
            SgxStatus::InvalidParameter
        );

        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CDcapURaMsg2) };
        raw_msg.g_b = self.pub_key_b.into();
        raw_msg.kdf_id = self.kdf_id;
        raw_msg.sign_gb_ga = self.sign_gb_ga.into();
        raw_msg.mac = self.mac;
        Ok(())
    }

    #[inline]
    pub fn from_bytes<A: Allocator>(bytes: Vec<u8, A>) -> SgxResult<DcapURaMsg2> {
        Self::from_slice(bytes.as_slice())
    }

    pub fn from_slice(bytes: &[u8]) -> SgxResult<DcapURaMsg2> {
        let raw_msg = unsafe { &*(bytes.as_ptr() as *const CDcapURaMsg2) };
        Ok(raw_msg.into())
    }
}

impl DcapRaMsg3 {
    pub fn gen_cmac(&mut self, cmac_key: &AlignKey128bit) -> SgxResult {
        let mut cmac = AesCMac::new(&cmac_key.key)?;
        cmac.update(&self.pub_key_a)?;
        cmac.update(&self.quote.len())?;
        cmac.update(&self.quote[..])?;
        self.mac = cmac.finalize()?;

        Ok(())
    }

    pub fn verify_cmac(&self, cmac_key: &AlignKey128bit) -> SgxResult {
        let mut cmac = AesCMac::new(&cmac_key.key)?;
        cmac.update(&self.pub_key_a)?;
        cmac.update(&self.quote.len())?;
        cmac.update(&self.quote[..])?;
        let mac = cmac.finalize()?;

        ensure!(mac.ct_eq(&self.mac), SgxStatus::MacMismatch);
        Ok(())
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
        ensure!(!self.quote.is_empty(), SgxStatus::InvalidParameter);
        let raw_len = self.get_raw_ize().ok_or(SgxStatus::InvalidParameter)?;

        let mut bytes = vec::from_elem_in(0_u8, raw_len as usize, alloc);
        let header_len = mem::size_of::<CDcapRaMsg3>() as u32;

        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CDcapRaMsg3) };
        raw_msg.mac = self.mac;
        raw_msg.g_a = self.pub_key_a.into();
        raw_msg.quote_size = self.quote.len() as u32;
        bytes[header_len as usize..].copy_from_slice(&self.quote);

        Ok(bytes)
    }

    pub fn copy_to_slice(&self, bytes: &mut [u8]) -> SgxResult {
        ensure!(!self.quote.is_empty(), SgxStatus::InvalidParameter);
        let raw_len = self.get_raw_ize().ok_or(SgxStatus::InvalidParameter)?;
        ensure!(bytes.len() == raw_len as usize, SgxStatus::InvalidParameter);

        let header_len = mem::size_of::<CDcapRaMsg3>() as u32;
        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CDcapRaMsg3) };

        raw_msg.mac = self.mac;
        raw_msg.g_a = self.pub_key_a.into();
        raw_msg.quote_size = self.quote.len() as u32;
        bytes[header_len as usize..].copy_from_slice(&self.quote);

        Ok(())
    }

    #[inline]
    pub fn from_bytes<A: Allocator>(bytes: Vec<u8, A>) -> SgxResult<DcapRaMsg3> {
        Self::from_slice(bytes.as_slice())
    }

    pub fn from_slice(bytes: &[u8]) -> SgxResult<DcapRaMsg3> {
        let raw_msg_len = bytes.len();
        ensure!(
            raw_msg_len > mem::size_of::<CDcapRaMsg3>(),
            SgxStatus::InvalidParameter
        );

        let header_len = mem::size_of::<CDcapRaMsg3>();
        let raw_msg = unsafe { &*(bytes.as_ptr() as *const CDcapRaMsg3) };

        let quote_len = raw_msg_len - header_len;
        ensure!(
            Self::check_quote_len(quote_len),
            SgxStatus::InvalidParameter
        );
        ensure!(
            quote_len == raw_msg.quote_size as usize,
            SgxStatus::InvalidParameter
        );

        let mut quote = vec![0_u8; quote_len];
        quote.as_mut_slice().copy_from_slice(unsafe {
            slice::from_raw_parts(&raw_msg.quote as *const _ as *const u8, quote_len)
        });

        Ok(DcapRaMsg3 {
            mac: raw_msg.mac,
            pub_key_a: raw_msg.g_a.into(),
            quote: quote.into_boxed_slice(),
        })
    }

    pub fn get_raw_ize(&self) -> Option<u32> {
        let quote_len = self.quote.len();

        if Self::check_quote_len(quote_len) {
            Some((mem::size_of::<CDcapRaMsg3>() + quote_len) as u32)
        } else {
            None
        }
    }

    #[inline]
    pub fn check_quote_len(quote_len: usize) -> bool {
        quote_len <= (u32::MAX as usize) - mem::size_of::<CDcapRaMsg3>()
            && quote_len
                > (mem::size_of::<Quote3>()
                    + mem::size_of::<QlEcdsaSigData>()
                    + mem::size_of::<QlAuthData>()
                    + mem::size_of::<QlCertificationData>())
    }
}

impl From<DcapRaMsg1> for CDcapRaMsg1 {
    fn from(msg: DcapRaMsg1) -> CDcapRaMsg1 {
        CDcapRaMsg1 {
            g_a: msg.pub_key_a.into(),
        }
    }
}

impl From<&DcapRaMsg1> for CDcapRaMsg1 {
    fn from(msg: &DcapRaMsg1) -> CDcapRaMsg1 {
        CDcapRaMsg1 {
            g_a: msg.pub_key_a.into(),
        }
    }
}

impl From<CDcapRaMsg1> for DcapRaMsg1 {
    fn from(msg: CDcapRaMsg1) -> DcapRaMsg1 {
        DcapRaMsg1 {
            pub_key_a: msg.g_a.into(),
        }
    }
}

impl From<&CDcapRaMsg1> for DcapRaMsg1 {
    fn from(msg: &CDcapRaMsg1) -> DcapRaMsg1 {
        DcapRaMsg1 {
            pub_key_a: msg.g_a.into(),
        }
    }
}

impl From<DcapURaMsg2> for CDcapURaMsg2 {
    fn from(msg: DcapURaMsg2) -> CDcapURaMsg2 {
        CDcapURaMsg2 {
            g_b: msg.pub_key_b.into(),
            kdf_id: msg.kdf_id,
            sign_gb_ga: msg.sign_gb_ga.into(),
            mac: msg.mac,
        }
    }
}

impl From<&DcapURaMsg2> for CDcapURaMsg2 {
    fn from(msg: &DcapURaMsg2) -> CDcapURaMsg2 {
        CDcapURaMsg2 {
            g_b: msg.pub_key_b.into(),
            kdf_id: msg.kdf_id,
            sign_gb_ga: msg.sign_gb_ga.into(),
            mac: msg.mac,
        }
    }
}

impl From<CDcapURaMsg2> for DcapURaMsg2 {
    fn from(msg: CDcapURaMsg2) -> DcapURaMsg2 {
        DcapURaMsg2 {
            pub_key_b: msg.g_b.into(),
            kdf_id: msg.kdf_id,
            sign_gb_ga: msg.sign_gb_ga.into(),
            mac: msg.mac,
        }
    }
}

impl From<&CDcapURaMsg2> for DcapURaMsg2 {
    fn from(msg: &CDcapURaMsg2) -> DcapURaMsg2 {
        DcapURaMsg2 {
            pub_key_b: msg.g_b.into(),
            kdf_id: msg.kdf_id,
            sign_gb_ga: msg.sign_gb_ga.into(),
            mac: msg.mac,
        }
    }
}

#[cfg(feature = "tmsg")]
impl EnclaveRange for DcapMRaMsg2 {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(self as *const _ as *const u8, mem::size_of::<DcapMRaMsg2>()) {
            return false;
        }
        if self.quote.len() > 0 && !is_within_enclave(self.quote.as_ptr(), self.quote.len()) {
            return false;
        }
        true
    }

    fn is_host_range(&self) -> bool {
        if !is_within_host(self as *const _ as *const u8, mem::size_of::<DcapMRaMsg2>()) {
            return false;
        }
        if self.quote.len() > 0 && !is_within_host(self.quote.as_ptr(), self.quote.len()) {
            return false;
        }
        true
    }
}

#[cfg(feature = "tmsg")]
impl EnclaveRange for DcapRaMsg3 {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(self as *const _ as *const u8, mem::size_of::<DcapRaMsg3>()) {
            return false;
        }
        if self.quote.len() > 0 && !is_within_enclave(self.quote.as_ptr(), self.quote.len()) {
            return false;
        }
        true
    }

    fn is_host_range(&self) -> bool {
        if !is_within_host(self as *const _ as *const u8, mem::size_of::<DcapRaMsg3>()) {
            return false;
        }
        if self.quote.len() > 0 && !is_within_host(self.quote.as_ptr(), self.quote.len()) {
            return false;
        }
        true
    }
}
