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

use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::vec::{self, Vec};
use core::alloc::Allocator;
use core::mem;
#[cfg(feature = "tmsg")]
use core::ptr;
use core::slice;
use sgx_crypto::ecc::{EcPrivateKey, EcPublicKey, EcSignature};
use sgx_crypto::mac::AesCMac;
#[cfg(feature = "tmsg")]
use sgx_trts::trts::{is_within_enclave, is_within_host, EnclaveRange};
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::memeq::ConstTimeEq;
use sgx_types::types::{
    AlignKey128bit, CRaMsg1, CRaMsg2, CRaMsg3, Ec256PublicKey, EpidGroupId, Mac, PsSecPropDesc,
    Quote, QuoteSignType, Spid,
};

#[cfg(any(feature = "tserialize", feature = "userialize"))]
use sgx_serialize::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct RaMsg1 {
    pub pub_key_a: EcPublicKey,
    pub gid: EpidGroupId,
}

impl_struct_ContiguousMemory! {
    RaMsg1;
}
impl_asref_array! {
    RaMsg1;
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct RaMsg2 {
    pub pub_key_b: EcPublicKey,
    pub spid: Spid,
    pub quote_type: QuoteSignType,
    pub kdf_id: u16,
    pub sign_gb_ga: EcSignature,
    pub mac: Mac,
    pub sig_rl: Option<Box<[u8]>>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct RaMsg3 {
    pub mac: Mac,
    pub pub_key_a: EcPublicKey,
    pub ps_sec_prop: PsSecPropDesc,
    pub quote: Box<[u8]>,
}

impl RaMsg1 {
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
        let raw_msg: CRaMsg1 = self.into();
        Ok(raw_msg.as_ref().to_vec_in(alloc))
    }

    pub fn copy_to_slice(&self, bytes: &mut [u8]) -> SgxResult {
        ensure!(
            bytes.len() == mem::size_of::<CRaMsg1>(),
            SgxStatus::InvalidParameter
        );

        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CRaMsg1) };
        raw_msg.g_a = self.pub_key_a.into();
        raw_msg.gid = self.gid;
        Ok(())
    }

    #[inline]
    pub fn from_bytes<A: Allocator>(bytes: Vec<u8, A>) -> SgxResult<RaMsg1> {
        Self::from_slice(bytes.as_slice())
    }

    pub fn from_slice(bytes: &[u8]) -> SgxResult<RaMsg1> {
        let raw_msg = unsafe { &*(bytes.as_ptr() as *const CRaMsg1) };
        Ok(raw_msg.into())
    }
}

impl RaMsg2 {
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
        cmac.update(&self.spid)?;
        cmac.update(&(self.quote_type as u16))?;
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
        cmac.update(&self.spid)?;
        cmac.update(&(self.quote_type as u16))?;
        cmac.update(&self.kdf_id)?;
        cmac.update(&self.sign_gb_ga)?;
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
        let raw_len = self.get_raw_ize().ok_or(SgxStatus::InvalidParameter)?;

        let mut bytes = vec::from_elem_in(0_u8, raw_len as usize, alloc);
        let header_len = mem::size_of::<CRaMsg2>() as u32;

        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CRaMsg2) };
        raw_msg.g_b = self.pub_key_b.into();
        raw_msg.spid = self.spid;
        raw_msg.quote_type = self.quote_type as u16;
        raw_msg.kdf_id = self.kdf_id;
        raw_msg.sign_gb_ga = self.sign_gb_ga.into();
        raw_msg.mac = self.mac;
        raw_msg.sig_rl_size = raw_len - header_len as u32;
        if let Some(sig_rl) = self.sig_rl.as_ref() {
            bytes[header_len as usize..].copy_from_slice(sig_rl);
        }
        Ok(bytes)
    }

    pub fn copy_to_slice(&self, bytes: &mut [u8]) -> SgxResult {
        let raw_len = self.get_raw_ize().ok_or(SgxStatus::InvalidParameter)?;
        ensure!(bytes.len() == raw_len as usize, SgxStatus::InvalidParameter);

        let header_len = mem::size_of::<CRaMsg2>() as u32;
        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CRaMsg2) };

        raw_msg.g_b = self.pub_key_b.into();
        raw_msg.spid = self.spid;
        raw_msg.quote_type = self.quote_type as u16;
        raw_msg.kdf_id = self.kdf_id;
        raw_msg.sign_gb_ga = self.sign_gb_ga.into();
        raw_msg.mac = self.mac;
        raw_msg.sig_rl_size = raw_len - header_len as u32;
        if let Some(sig_rl) = self.sig_rl.as_ref() {
            bytes[header_len as usize..].copy_from_slice(sig_rl);
        }
        Ok(())
    }

    #[inline]
    pub fn from_bytes<A: Allocator>(bytes: Vec<u8, A>) -> SgxResult<RaMsg2> {
        Self::from_slice(bytes.as_slice())
    }

    pub fn from_slice(bytes: &[u8]) -> SgxResult<RaMsg2> {
        ensure!(
            bytes.len() >= mem::size_of::<CRaMsg2>(),
            SgxStatus::InvalidParameter
        );

        let header_len = mem::size_of::<CRaMsg2>() as u32;
        let raw_msg = unsafe { &*(bytes.as_ptr() as *const CRaMsg2) };

        ensure!(
            (raw_msg.sig_rl_size <= u32::MAX - header_len)
                && (bytes.len() == (header_len + raw_msg.sig_rl_size) as usize),
            SgxStatus::InvalidParameter
        );

        let sig_rl_len = raw_msg.sig_rl_size as usize;
        let sig_rl = if sig_rl_len > 0 {
            let mut sig_rl = vec![0_u8; sig_rl_len];
            sig_rl.as_mut_slice().copy_from_slice(unsafe {
                slice::from_raw_parts(&raw_msg.sig_rl as *const _ as *const u8, sig_rl_len)
            });
            Some(sig_rl.into_boxed_slice())
        } else {
            None
        };

        let quote_type = if raw_msg.quote_type == 0 {
            QuoteSignType::Unlinkable
        } else {
            QuoteSignType::Linkable
        };

        Ok(RaMsg2 {
            pub_key_b: raw_msg.g_b.into(),
            spid: raw_msg.spid,
            quote_type,
            kdf_id: raw_msg.kdf_id,
            sign_gb_ga: raw_msg.sign_gb_ga.into(),
            mac: raw_msg.mac,
            sig_rl,
        })
    }

    pub fn get_raw_ize(&self) -> Option<u32> {
        let sig_rl_len = self.sig_rl.as_ref().map_or(0, |sig_rl| sig_rl.len());

        if Self::check_sig_rl_len(sig_rl_len) {
            Some((mem::size_of::<CRaMsg2>() + sig_rl_len) as u32)
        } else {
            None
        }
    }

    #[inline]
    pub fn check_sig_rl_len(sig_rl_len: usize) -> bool {
        sig_rl_len <= (u32::MAX as usize) - mem::size_of::<CRaMsg2>()
    }
}

impl RaMsg3 {
    pub fn gen_cmac(&mut self, cmac_key: &AlignKey128bit) -> SgxResult {
        let mut cmac = AesCMac::new(&cmac_key.key)?;
        cmac.update(&self.pub_key_a)?;
        cmac.update(&self.ps_sec_prop)?;
        cmac.update(&self.quote[..])?;
        self.mac = cmac.finalize()?;

        Ok(())
    }

    pub fn verify_cmac(&self, cmac_key: &AlignKey128bit) -> SgxResult {
        let mut cmac = AesCMac::new(&cmac_key.key)?;
        cmac.update(&self.pub_key_a)?;
        cmac.update(&self.ps_sec_prop)?;
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
        let header_len = mem::size_of::<CRaMsg3>();

        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CRaMsg3) };
        raw_msg.mac = self.mac;
        raw_msg.g_a = self.pub_key_a.into();
        raw_msg.ps_sec_prop = self.ps_sec_prop;
        bytes[header_len..].copy_from_slice(&self.quote);

        Ok(bytes)
    }

    pub fn copy_to_slice(&self, bytes: &mut [u8]) -> SgxResult {
        ensure!(!self.quote.is_empty(), SgxStatus::InvalidParameter);
        let raw_len = self.get_raw_ize().ok_or(SgxStatus::InvalidParameter)?;
        ensure!(bytes.len() == raw_len as usize, SgxStatus::InvalidParameter);

        let header_len = mem::size_of::<CRaMsg3>();
        let raw_msg = unsafe { &mut *(bytes.as_mut_ptr() as *mut CRaMsg3) };

        raw_msg.mac = self.mac;
        raw_msg.g_a = self.pub_key_a.into();
        raw_msg.ps_sec_prop = self.ps_sec_prop;
        bytes[header_len..].copy_from_slice(&self.quote);

        Ok(())
    }

    #[inline]
    pub fn from_bytes<A: Allocator>(bytes: Vec<u8, A>) -> SgxResult<RaMsg3> {
        Self::from_slice(bytes.as_slice())
    }

    pub fn from_slice(bytes: &[u8]) -> SgxResult<RaMsg3> {
        let raw_msg_len = bytes.len();
        ensure!(
            raw_msg_len > mem::size_of::<CRaMsg3>(),
            SgxStatus::InvalidParameter
        );

        let header_len = mem::size_of::<CRaMsg3>();
        let raw_msg = unsafe { &*(bytes.as_ptr() as *const CRaMsg3) };

        let quote_len = raw_msg_len - header_len;
        ensure!(
            Self::check_quote_len(quote_len),
            SgxStatus::InvalidParameter
        );

        let mut quote = vec![0_u8; quote_len];
        quote.as_mut_slice().copy_from_slice(unsafe {
            slice::from_raw_parts(&raw_msg.quote as *const _ as *const u8, quote_len)
        });

        Ok(RaMsg3 {
            mac: raw_msg.mac,
            pub_key_a: raw_msg.g_a.into(),
            ps_sec_prop: raw_msg.ps_sec_prop,
            quote: quote.into_boxed_slice(),
        })
    }

    pub fn get_raw_ize(&self) -> Option<u32> {
        let quote_len = self.quote.len();

        if Self::check_quote_len(quote_len) {
            Some((mem::size_of::<CRaMsg3>() + quote_len) as u32)
        } else {
            None
        }
    }

    #[inline]
    pub fn check_quote_len(quote_len: usize) -> bool {
        quote_len <= (u32::MAX as usize) - mem::size_of::<CRaMsg3>()
            && quote_len > mem::size_of::<Quote>()
    }
}

impl From<RaMsg1> for CRaMsg1 {
    fn from(msg: RaMsg1) -> CRaMsg1 {
        CRaMsg1 {
            g_a: msg.pub_key_a.into(),
            gid: msg.gid,
        }
    }
}

impl From<&RaMsg1> for CRaMsg1 {
    fn from(msg: &RaMsg1) -> CRaMsg1 {
        CRaMsg1 {
            g_a: msg.pub_key_a.into(),
            gid: msg.gid,
        }
    }
}

impl From<CRaMsg1> for RaMsg1 {
    fn from(msg: CRaMsg1) -> RaMsg1 {
        RaMsg1 {
            pub_key_a: msg.g_a.into(),
            gid: msg.gid,
        }
    }
}

impl From<&CRaMsg1> for RaMsg1 {
    fn from(msg: &CRaMsg1) -> RaMsg1 {
        RaMsg1 {
            pub_key_a: msg.g_a.into(),
            gid: msg.gid,
        }
    }
}

#[cfg(feature = "tmsg")]
impl EnclaveRange for RaMsg2 {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(self as *const _ as *const u8, mem::size_of::<RaMsg2>()) {
            return false;
        }

        let sig_rl = self.sig_rl.as_ref();
        let (ptr, len) = sig_rl.map_or((ptr::null(), 0), |sig_rl| {
            if !sig_rl.is_empty() {
                (sig_rl.as_ptr(), sig_rl.len())
            } else {
                (ptr::null(), 0)
            }
        });
        if len > 0 && !is_within_enclave(ptr, len) {
            return false;
        }

        true
    }

    fn is_host_range(&self) -> bool {
        if !is_within_host(self as *const _ as *const u8, mem::size_of::<RaMsg2>()) {
            return false;
        }

        let sig_rl = self.sig_rl.as_ref();
        let (ptr, len) = sig_rl.map_or((ptr::null(), 0), |sig_rl| {
            if !sig_rl.is_empty() {
                (sig_rl.as_ptr(), sig_rl.len())
            } else {
                (ptr::null(), 0)
            }
        });
        if len > 0 && !is_within_host(ptr, len) {
            return false;
        }

        true
    }
}

#[cfg(feature = "tmsg")]
impl EnclaveRange for RaMsg3 {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(self as *const _ as *const u8, mem::size_of::<RaMsg3>()) {
            return false;
        }
        if self.quote.len() > 0 && !is_within_enclave(self.quote.as_ptr(), self.quote.len()) {
            return false;
        }
        true
    }

    fn is_host_range(&self) -> bool {
        if !is_within_host(self as *const _ as *const u8, mem::size_of::<RaMsg3>()) {
            return false;
        }
        if self.quote.len() > 0 && !is_within_host(self.quote.as_ptr(), self.quote.len()) {
            return false;
        }
        true
    }
}
