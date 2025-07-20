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

use crate::mac::AesCMac;
use core::convert::From;
use core::convert::TryInto;
use core::mem;
use core::ptr;
use sgx_crypto_sys::*;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::{BytewiseEquality, ContiguousMemory};
use sgx_types::memeq::ConstTimeEq;
use sgx_types::types::ECP256_KEY_SIZE;
use sgx_types::types::{
    AlignEc256PrivateKey, AlignEc256SharedKey, AlignKey128bit, Ec256PrivateKey, Ec256PublicKey,
    Ec256SharedKey, Ec256Signature, EcResult, EccHandle, Key128bit, Sha256Hash,
};

#[cfg(any(feature = "tserialize", feature = "userialize"))]
use sgx_serialize::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct EcKeyPair {
    private: EcPrivateKey,
    public: EcPublicKey,
}

impl EcKeyPair {
    pub fn create() -> SgxResult<EcKeyPair> {
        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_ecc256_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut private = EcPrivateKey::default();
        let mut public = EcPublicKey::default();
        let status = unsafe {
            sgx_ecc256_create_key_pair(
                &mut private.0.key as *mut Ec256PrivateKey,
                &mut public.0 as *mut Ec256PublicKey,
                handle,
            )
        };
        let _ = unsafe { sgx_ecc256_close_context(handle) };

        ensure!(status.is_success(), status);
        Ok(EcKeyPair { private, public })
    }

    pub fn create_with_seed<H: AsRef<[u8]>>(hash_drg: H) -> SgxResult<EcKeyPair> {
        let private = EcPrivateKey::create_with_seed(hash_drg)?;
        let public = private.export_public_key()?;

        Ok(EcKeyPair { private, public })
    }

    #[inline]
    pub fn shared_key(&self, peer_public_key: &EcPublicKey) -> SgxResult<EcShareKey> {
        self.private.shared_key(peer_public_key)
    }

    #[inline]
    pub fn private_key(&self) -> EcPrivateKey {
        self.private
    }

    #[inline]
    pub fn public_key(&self) -> EcPublicKey {
        self.public
    }

    #[inline]
    pub fn clear(&mut self) {
        self.private.clear();
        self.public.clear();
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct EcPrivateKey(AlignEc256PrivateKey);

impl EcPrivateKey {
    const NISTP256: [u8; 32] = [
        0x51, 0x25, 0x63, 0xFC, 0xC2, 0xCA, 0xB9, 0xF3, 0x84, 0x9E, 0x17, 0xA7, 0xAD, 0xFA, 0xE6,
        0xBC, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0xFF, 0xFF, 0xFF,
        0xFF,
    ];

    pub fn sign<T: ?Sized>(&self, data: &T) -> SgxResult<EcSignature>
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_ecc256_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut signature = Ec256Signature::default();
        let status = unsafe {
            sgx_ecdsa_sign(
                (data as *const T).cast(),
                size as u32,
                &self.0.key as *const Ec256PrivateKey,
                &mut signature as *mut Ec256Signature,
                handle,
            )
        };
        let _ = unsafe { sgx_ecc256_close_context(handle) };

        ensure!(status.is_success(), status);
        Ok(signature.into())
    }

    pub fn create_with_seed<H: AsRef<[u8]>>(hash_drg: H) -> SgxResult<EcPrivateKey> {
        let hash_drg = hash_drg.as_ref();
        if (hash_drg.is_empty()) || (hash_drg.len() > i32::MAX as usize) {
            bail!(SgxStatus::InvalidParameter);
        }

        let mut key = EcPrivateKey::default();
        let status = unsafe {
            sgx_calculate_ecdsa_priv_key(
                hash_drg.as_ptr(),
                hash_drg.len() as i32,
                EcPrivateKey::NISTP256.as_ptr(),
                32,
                &mut key.0.key as *mut _ as *mut u8,
                key.as_ref().len() as i32,
            )
        };

        ensure!(status.is_success(), status);
        Ok(key)
    }

    pub fn shared_key(&self, peer_public_key: &EcPublicKey) -> SgxResult<EcShareKey> {
        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_ecc256_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut shared_key = AlignEc256SharedKey::default();
        let status = unsafe {
            sgx_ecc256_compute_shared_dhkey(
                &self.0.key as *const Ec256PrivateKey,
                &peer_public_key.0 as *const Ec256PublicKey,
                &mut shared_key.key as *mut Ec256SharedKey,
                handle,
            )
        };
        let _ = unsafe { sgx_ecc256_close_context(handle) };

        ensure!(status.is_success(), status);
        Ok(From::from(shared_key))
    }

    pub fn export_public_key(&self) -> SgxResult<EcPublicKey> {
        let mut public = EcPublicKey::default();
        let status = unsafe {
            sgx_ecc256_calculate_pub_from_priv(
                &self.0.key as *const Ec256PrivateKey,
                &mut public.0 as *mut Ec256PublicKey,
            )
        };

        ensure!(status.is_success(), status);
        Ok(public)
    }

    #[inline]
    pub fn private_key(&self) -> Ec256PrivateKey {
        self.0.key
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.as_mut().fill(0);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct EcPublicKey(Ec256PublicKey);

impl EcPublicKey {
    pub fn verify<T: ?Sized>(&self, data: &T, signature: &EcSignature) -> SgxResult<bool>
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_ecc256_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut ec_result = EcResult::InvalidSignature;
        let status = unsafe {
            sgx_ecdsa_verify(
                (data as *const T).cast(),
                size as u32,
                &self.0 as *const Ec256PublicKey,
                &signature.0 as *const Ec256Signature,
                &mut ec_result as *mut EcResult as *mut u8,
                handle,
            )
        };
        let _ = unsafe { sgx_ecc256_close_context(handle) };

        ensure!(status.is_success(), status);
        match ec_result {
            EcResult::Valid => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn verify_hash(&self, hash: &Sha256Hash, signature: &EcSignature) -> SgxResult<bool> {
        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_ecc256_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut ec_result = EcResult::InvalidSignature;
        let status = unsafe {
            sgx_ecdsa_verify_hash(
                hash.as_ptr(),
                &self.0 as *const Ec256PublicKey,
                &signature.0 as *const Ec256Signature,
                &mut ec_result as *mut EcResult as *mut u8,
                handle,
            )
        };
        let _ = unsafe { sgx_ecc256_close_context(handle) };

        ensure!(status.is_success(), status);
        match ec_result {
            EcResult::Valid => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn check_point(&self) -> SgxResult<bool> {
        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_ecc256_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut valid: i32 = 0;
        let status = unsafe {
            sgx_ecc256_check_point(
                &self.0 as *const Ec256PublicKey,
                handle,
                &mut valid as *mut i32,
            )
        };
        let _ = unsafe { sgx_ecc256_close_context(handle) };

        ensure!(status.is_success(), status);
        if valid > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[inline]
    pub fn from_private_key(key: &EcPrivateKey) -> SgxResult<EcPublicKey> {
        key.export_public_key()
    }

    #[inline]
    pub fn public_key(&self) -> Ec256PublicKey {
        self.0
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.as_mut().fill(0);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct EcShareKey(AlignEc256SharedKey);

impl EcShareKey {
    pub fn derive_key(&self, label: &[u8]) -> SgxResult<AlignKey128bit> {
        ensure!(!label.is_empty(), SgxStatus::InvalidParameter);

        let key = Key128bit::default();
        let mut derive_key = AesCMac::cmac(&key, &self.0.key)?;

        let derivation_len = label
            .len()
            .checked_add(4)
            .ok_or(SgxStatus::InvalidParameter)?;
        let mut derivation = vec![0_u8; label.len() + 4];
        derivation[0] = 0x01;
        derivation[1..derivation_len - 3].copy_from_slice(label);
        derivation[derivation_len - 3..].copy_from_slice(&[0x00, 0x80, 0x00]);

        let key = AesCMac::cmac_align(&derive_key, derivation.as_slice())
            .map(|cmac| AlignKey128bit::from(cmac.mac))?;

        derive_key.as_mut().fill(0);
        Ok(key)
    }

    #[inline]
    pub fn shared_key(&self) -> Ec256SharedKey {
        self.0.key
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.as_mut().fill(0);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct EcSignature(Ec256Signature);

impl EcSignature {
    #[inline]
    pub fn signature(&self) -> Ec256Signature {
        self.0
    }
}

impl From<(EcPrivateKey, EcPublicKey)> for EcKeyPair {
    fn from(key_pair: (EcPrivateKey, EcPublicKey)) -> EcKeyPair {
        EcKeyPair {
            private: key_pair.0,
            public: key_pair.1,
        }
    }
}

impl From<EcKeyPair> for (EcPrivateKey, EcPublicKey) {
    fn from(key_pair: EcKeyPair) -> (EcPrivateKey, EcPublicKey) {
        (key_pair.private, key_pair.public)
    }
}

impl From<&EcKeyPair> for (EcPrivateKey, EcPublicKey) {
    fn from(key_pair: &EcKeyPair) -> (EcPrivateKey, EcPublicKey) {
        (key_pair.private, key_pair.public)
    }
}

impl From<EcKeyPair> for [u8; ECP256_KEY_SIZE * 3] {
    #[inline]
    fn from(key: EcKeyPair) -> [u8; ECP256_KEY_SIZE * 3] {
        From::<&EcKeyPair>::from(&key)
    }
}

impl From<&EcKeyPair> for [u8; ECP256_KEY_SIZE * 3] {
    #[inline]
    fn from(key: &EcKeyPair) -> [u8; ECP256_KEY_SIZE * 3] {
        let mut array = [0_u8; ECP256_KEY_SIZE * 3];
        array[..ECP256_KEY_SIZE].copy_from_slice(&key.private.as_ref()[..]);
        array[ECP256_KEY_SIZE..].copy_from_slice(&key.public.as_ref()[..]);
        array
    }
}

impl From<[u8; ECP256_KEY_SIZE * 3]> for EcKeyPair {
    #[inline]
    fn from(key: [u8; ECP256_KEY_SIZE * 3]) -> EcKeyPair {
        From::<&[u8; ECP256_KEY_SIZE * 3]>::from(&key)
    }
}

impl From<&[u8; ECP256_KEY_SIZE * 3]> for EcKeyPair {
    #[inline]
    fn from(key: &[u8; ECP256_KEY_SIZE * 3]) -> EcKeyPair {
        let array: &[u8; ECP256_KEY_SIZE] = key[..ECP256_KEY_SIZE].try_into().unwrap();
        let private_key = EcPrivateKey::from(array);

        let array: &[u8; ECP256_KEY_SIZE * 2] = key[ECP256_KEY_SIZE..].try_into().unwrap();
        let public_key = EcPublicKey::from(array);

        EcKeyPair::from((private_key, public_key))
    }
}

impl From<AlignEc256PrivateKey> for EcPrivateKey {
    #[inline]
    fn from(key: AlignEc256PrivateKey) -> EcPrivateKey {
        EcPrivateKey(key)
    }
}

impl From<&AlignEc256PrivateKey> for EcPrivateKey {
    #[inline]
    fn from(key: &AlignEc256PrivateKey) -> EcPrivateKey {
        EcPrivateKey(*key)
    }
}

impl From<Ec256PrivateKey> for EcPrivateKey {
    #[inline]
    fn from(key: Ec256PrivateKey) -> EcPrivateKey {
        EcPrivateKey(From::from(key))
    }
}

impl From<&Ec256PrivateKey> for EcPrivateKey {
    #[inline]
    fn from(key: &Ec256PrivateKey) -> EcPrivateKey {
        EcPrivateKey(From::from(key))
    }
}

impl AsRef<[u8; ECP256_KEY_SIZE]> for EcPrivateKey {
    #[inline]
    fn as_ref(&self) -> &[u8; ECP256_KEY_SIZE] {
        self.0.as_ref()
    }
}

impl From<EcPrivateKey> for [u8; ECP256_KEY_SIZE] {
    #[inline]
    fn from(key: EcPrivateKey) -> [u8; ECP256_KEY_SIZE] {
        *key.as_ref()
    }
}

impl From<&EcPrivateKey> for [u8; ECP256_KEY_SIZE] {
    #[inline]
    fn from(key: &EcPrivateKey) -> [u8; ECP256_KEY_SIZE] {
        *key.as_ref()
    }
}

impl From<[u8; ECP256_KEY_SIZE]> for EcPrivateKey {
    #[inline]
    fn from(key: [u8; ECP256_KEY_SIZE]) -> EcPrivateKey {
        EcPrivateKey(AlignEc256PrivateKey::from(key))
    }
}

impl From<&[u8; ECP256_KEY_SIZE]> for EcPrivateKey {
    #[inline]
    fn from(key: &[u8; ECP256_KEY_SIZE]) -> EcPrivateKey {
        EcPrivateKey(AlignEc256PrivateKey::from(key))
    }
}

impl From<EcPrivateKey> for Ec256PrivateKey {
    #[inline]
    fn from(key: EcPrivateKey) -> Ec256PrivateKey {
        key.0.key
    }
}

impl From<&EcPrivateKey> for Ec256PrivateKey {
    #[inline]
    fn from(key: &EcPrivateKey) -> Ec256PrivateKey {
        key.0.key
    }
}

impl From<EcPrivateKey> for AlignEc256PrivateKey {
    #[inline]
    fn from(key: EcPrivateKey) -> AlignEc256PrivateKey {
        key.0
    }
}

impl From<&EcPrivateKey> for AlignEc256PrivateKey {
    #[inline]
    fn from(key: &EcPrivateKey) -> AlignEc256PrivateKey {
        key.0
    }
}

impl From<Ec256PublicKey> for EcPublicKey {
    #[inline]
    fn from(key: Ec256PublicKey) -> EcPublicKey {
        EcPublicKey(key)
    }
}

impl From<&Ec256PublicKey> for EcPublicKey {
    #[inline]
    fn from(key: &Ec256PublicKey) -> EcPublicKey {
        EcPublicKey(*key)
    }
}

impl AsRef<[u8; ECP256_KEY_SIZE * 2]> for EcPublicKey {
    #[inline]
    fn as_ref(&self) -> &[u8; ECP256_KEY_SIZE * 2] {
        self.0.as_ref()
    }
}

impl From<EcPublicKey> for [u8; ECP256_KEY_SIZE * 2] {
    #[inline]
    fn from(key: EcPublicKey) -> [u8; ECP256_KEY_SIZE * 2] {
        *key.as_ref()
    }
}

impl From<&EcPublicKey> for [u8; ECP256_KEY_SIZE * 2] {
    #[inline]
    fn from(key: &EcPublicKey) -> [u8; ECP256_KEY_SIZE * 2] {
        *key.as_ref()
    }
}

impl From<[u8; ECP256_KEY_SIZE * 2]> for EcPublicKey {
    #[inline]
    fn from(key: [u8; ECP256_KEY_SIZE * 2]) -> EcPublicKey {
        EcPublicKey(Ec256PublicKey::from(key))
    }
}

impl From<&[u8; ECP256_KEY_SIZE * 2]> for EcPublicKey {
    #[inline]
    fn from(key: &[u8; ECP256_KEY_SIZE * 2]) -> EcPublicKey {
        EcPublicKey(Ec256PublicKey::from(key))
    }
}

impl From<EcPublicKey> for Ec256PublicKey {
    #[inline]
    fn from(key: EcPublicKey) -> Ec256PublicKey {
        key.0
    }
}

impl From<&EcPublicKey> for Ec256PublicKey {
    #[inline]
    fn from(key: &EcPublicKey) -> Ec256PublicKey {
        key.0
    }
}

impl From<AlignEc256SharedKey> for EcShareKey {
    #[inline]
    fn from(key: AlignEc256SharedKey) -> EcShareKey {
        EcShareKey(key)
    }
}

impl From<&AlignEc256SharedKey> for EcShareKey {
    #[inline]
    fn from(key: &AlignEc256SharedKey) -> EcShareKey {
        EcShareKey(*key)
    }
}

impl From<Ec256SharedKey> for EcShareKey {
    #[inline]
    fn from(key: Ec256SharedKey) -> EcShareKey {
        EcShareKey(From::from(key))
    }
}

impl From<&Ec256SharedKey> for EcShareKey {
    #[inline]
    fn from(key: &Ec256SharedKey) -> EcShareKey {
        EcShareKey(From::from(key))
    }
}

impl AsRef<[u8; ECP256_KEY_SIZE]> for EcShareKey {
    #[inline]
    fn as_ref(&self) -> &[u8; ECP256_KEY_SIZE] {
        self.0.as_ref()
    }
}

impl From<EcShareKey> for [u8; ECP256_KEY_SIZE] {
    #[inline]
    fn from(key: EcShareKey) -> [u8; ECP256_KEY_SIZE] {
        *key.as_ref()
    }
}

impl From<&EcShareKey> for [u8; ECP256_KEY_SIZE] {
    #[inline]
    fn from(key: &EcShareKey) -> [u8; ECP256_KEY_SIZE] {
        *key.as_ref()
    }
}

impl From<[u8; ECP256_KEY_SIZE]> for EcShareKey {
    #[inline]
    fn from(key: [u8; ECP256_KEY_SIZE]) -> EcShareKey {
        EcShareKey(AlignEc256SharedKey::from(key))
    }
}

impl From<&[u8; ECP256_KEY_SIZE]> for EcShareKey {
    #[inline]
    fn from(key: &[u8; ECP256_KEY_SIZE]) -> EcShareKey {
        EcShareKey(AlignEc256SharedKey::from(key))
    }
}

impl From<EcShareKey> for Ec256SharedKey {
    #[inline]
    fn from(key: EcShareKey) -> Ec256SharedKey {
        key.0.key
    }
}

impl From<&EcShareKey> for Ec256SharedKey {
    #[inline]
    fn from(key: &EcShareKey) -> Ec256SharedKey {
        key.0.key
    }
}

impl From<EcShareKey> for AlignEc256SharedKey {
    #[inline]
    fn from(key: EcShareKey) -> AlignEc256SharedKey {
        key.0
    }
}

impl From<Ec256Signature> for EcSignature {
    #[inline]
    fn from(signature: Ec256Signature) -> EcSignature {
        EcSignature(signature)
    }
}

impl From<&Ec256Signature> for EcSignature {
    #[inline]
    fn from(signature: &Ec256Signature) -> EcSignature {
        EcSignature(*signature)
    }
}

impl AsRef<[u8; ECP256_KEY_SIZE * 2]> for EcSignature {
    #[inline]
    fn as_ref(&self) -> &[u8; ECP256_KEY_SIZE * 2] {
        self.0.as_ref()
    }
}

impl From<EcSignature> for [u8; ECP256_KEY_SIZE * 2] {
    #[inline]
    fn from(signature: EcSignature) -> [u8; ECP256_KEY_SIZE * 2] {
        *signature.as_ref()
    }
}

impl From<&EcSignature> for [u8; ECP256_KEY_SIZE * 2] {
    #[inline]
    fn from(signature: &EcSignature) -> [u8; ECP256_KEY_SIZE * 2] {
        *signature.as_ref()
    }
}

impl From<[u8; ECP256_KEY_SIZE * 2]> for EcSignature {
    #[inline]
    fn from(signature: [u8; ECP256_KEY_SIZE * 2]) -> EcSignature {
        EcSignature(Ec256Signature::from(signature))
    }
}

impl From<&[u8; ECP256_KEY_SIZE * 2]> for EcSignature {
    #[inline]
    fn from(signature: &[u8; ECP256_KEY_SIZE * 2]) -> EcSignature {
        EcSignature(Ec256Signature::from(signature))
    }
}

impl From<EcSignature> for Ec256Signature {
    #[inline]
    fn from(signature: EcSignature) -> Ec256Signature {
        signature.0
    }
}

impl From<&EcSignature> for Ec256Signature {
    #[inline]
    fn from(signature: &EcSignature) -> Ec256Signature {
        signature.0
    }
}

impl From<&EcShareKey> for AlignEc256SharedKey {
    #[inline]
    fn from(key: &EcShareKey) -> AlignEc256SharedKey {
        key.0
    }
}

impl ConstTimeEq<EcPrivateKey> for EcPrivateKey {
    #[inline]
    fn ct_eq(&self, other: &EcPrivateKey) -> bool {
        self.0.ct_eq(&other.0)
    }
}

impl ConstTimeEq<EcPublicKey> for EcPublicKey {
    #[inline]
    fn ct_eq(&self, other: &EcPublicKey) -> bool {
        self.0.ct_eq(&other.0)
    }
}

impl ConstTimeEq<EcShareKey> for EcShareKey {
    #[inline]
    fn ct_eq(&self, other: &EcShareKey) -> bool {
        self.0.ct_eq(&other.0)
    }
}

impl ConstTimeEq<EcSignature> for EcSignature {
    #[inline]
    fn ct_eq(&self, other: &EcSignature) -> bool {
        self.0.ct_eq(&other.0)
    }
}

unsafe impl ContiguousMemory for EcKeyPair {}
unsafe impl ContiguousMemory for EcPrivateKey {}
unsafe impl ContiguousMemory for EcPublicKey {}
unsafe impl ContiguousMemory for EcShareKey {}
unsafe impl ContiguousMemory for EcSignature {}

unsafe impl BytewiseEquality for EcKeyPair {}
unsafe impl BytewiseEquality for EcPrivateKey {}
unsafe impl BytewiseEquality for EcPublicKey {}
unsafe impl BytewiseEquality for EcShareKey {}
unsafe impl BytewiseEquality for EcSignature {}
