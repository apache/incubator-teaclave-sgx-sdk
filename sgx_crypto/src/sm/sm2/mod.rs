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

use crate::sm4::ccm::{Aad, Nonce, Sm4Ccm};
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
pub struct Sm2KeyPair {
    private: Sm2PrivateKey,
    public: Sm2PublicKey,
}

impl Sm2KeyPair {
    pub fn create() -> SgxResult<Sm2KeyPair> {
        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_sm2_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut private = Sm2PrivateKey::default();
        let mut public = Sm2PublicKey::default();
        let status = unsafe {
            sgx_sm2_create_key_pair(
                &mut private.0.key as *mut Ec256PrivateKey,
                &mut public.0 as *mut Ec256PublicKey,
                handle,
            )
        };
        let _ = unsafe { sgx_sm2_close_context(handle) };

        ensure!(status.is_success(), status);
        Ok(Sm2KeyPair { private, public })
    }

    pub fn create_with_seed<H: AsRef<[u8]>>(hash_drg: H) -> SgxResult<Sm2KeyPair> {
        let private = Sm2PrivateKey::create_with_seed(hash_drg)?;
        let public = private.export_public_key()?;

        Ok(Sm2KeyPair { private, public })
    }

    #[inline]
    pub fn shared_key(&self, peer_public_key: &Sm2PublicKey) -> SgxResult<Sm2ShareKey> {
        self.private.shared_key(peer_public_key)
    }

    #[inline]
    pub fn private_key(&self) -> Sm2PrivateKey {
        self.private
    }

    #[inline]
    pub fn public_key(&self) -> Sm2PublicKey {
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
pub struct Sm2PrivateKey(AlignEc256PrivateKey);

impl Sm2PrivateKey {
    const SM2_ORDER: [u8; 32] = [
        0x23, 0x41, 0xD5, 0x39, 0x09, 0xF4, 0xBB, 0x53, 0x2B, 0x05, 0xC6, 0x21, 0x6B, 0xDF, 0x03,
        0x72, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF,
        0xFF, 0xFF,
    ];

    pub fn sign<T: ?Sized>(&self, data: &T) -> SgxResult<Sm2Signature>
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_sm2_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut signature = Ec256Signature::default();
        let status = unsafe {
            sgx_sm2_sign(
                (data as *const T).cast(),
                size as u32,
                &self.0.key as *const Ec256PrivateKey,
                &mut signature as *mut Ec256Signature,
                handle,
            )
        };
        let _ = unsafe { sgx_sm2_close_context(handle) };

        ensure!(status.is_success(), status);
        Ok(signature.into())
    }

    pub fn create_with_seed<H: AsRef<[u8]>>(hash_drg: H) -> SgxResult<Sm2PrivateKey> {
        let hash_drg = hash_drg.as_ref();
        if (hash_drg.is_empty()) || (hash_drg.len() > i32::MAX as usize) {
            bail!(SgxStatus::InvalidParameter);
        }

        let mut key = Sm2PrivateKey::default();
        let status = unsafe {
            sgx_calculate_sm2_priv_key(
                hash_drg.as_ptr(),
                hash_drg.len() as i32,
                Sm2PrivateKey::SM2_ORDER.as_ptr() as *const u8,
                32,
                &mut key.0.key as *mut _ as *mut u8,
                key.as_ref().len() as i32,
            )
        };

        ensure!(status.is_success(), status);
        Ok(key)
    }

    pub fn shared_key(&self, peer_public_key: &Sm2PublicKey) -> SgxResult<Sm2ShareKey> {
        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_sm2_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut shared_key = AlignEc256SharedKey::default();
        let status = unsafe {
            sgx_sm2_compute_shared_dhkey(
                &self.0.key as *const Ec256PrivateKey,
                &peer_public_key.0 as *const Ec256PublicKey,
                &mut shared_key.key as *mut Ec256SharedKey,
                handle,
            )
        };
        let _ = unsafe { sgx_sm2_close_context(handle) };

        ensure!(status.is_success(), status);
        Ok(From::from(shared_key))
    }

    pub fn export_public_key(&self) -> SgxResult<Sm2PublicKey> {
        let mut public = Sm2PublicKey::default();
        let status = unsafe {
            sgx_sm2_calculate_pub_from_priv(
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
pub struct Sm2PublicKey(Ec256PublicKey);

impl Sm2PublicKey {
    pub fn verify<T: ?Sized>(&self, data: &T, signature: &Sm2Signature) -> SgxResult<bool>
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_sm2_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut ec_result = EcResult::InvalidSignature;
        let status = unsafe {
            sgx_sm2_verify(
                (data as *const T).cast(),
                size as u32,
                &self.0 as *const Ec256PublicKey,
                &signature.0 as *const Ec256Signature,
                &mut ec_result as *mut EcResult as *mut u8,
                handle,
            )
        };
        let _ = unsafe { sgx_sm2_close_context(handle) };

        ensure!(status.is_success(), status);
        match ec_result {
            EcResult::Valid => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn verify_hash(&self, hash: &Sha256Hash, signature: &Sm2Signature) -> SgxResult<bool> {
        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_sm2_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut ec_result = EcResult::InvalidSignature;
        let status = unsafe {
            sgx_sm2_verify_hash(
                hash.as_ptr(),
                &self.0 as *const Ec256PublicKey,
                &signature.0 as *const Ec256Signature,
                &mut ec_result as *mut EcResult as *mut u8,
                handle,
            )
        };
        let _ = unsafe { sgx_sm2_close_context(handle) };

        ensure!(status.is_success(), status);
        match ec_result {
            EcResult::Valid => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn check_point(&self) -> SgxResult<bool> {
        let mut handle: EccHandle = ptr::null_mut();
        let status = unsafe { sgx_sm2_open_context(&mut handle as *mut EccHandle) };
        ensure!(status.is_success(), status);

        let mut valid: i32 = 0;
        let status = unsafe {
            sgx_sm2_check_point(
                &self.0 as *const Ec256PublicKey,
                handle,
                &mut valid as *mut i32,
            )
        };
        let _ = unsafe { sgx_sm2_close_context(handle) };

        ensure!(status.is_success(), status);
        if valid > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[inline]
    pub fn from_private_key(key: &Sm2PrivateKey) -> SgxResult<Sm2PublicKey> {
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
pub struct Sm2ShareKey(AlignEc256SharedKey);

impl Sm2ShareKey {
    pub fn derive_key(&self, label: &[u8]) -> SgxResult<AlignKey128bit> {
        ensure!(!label.is_empty(), SgxStatus::InvalidParameter);

        let key = Key128bit::default();
        let mut sm4 = Sm4Ccm::new(&key, Nonce::zeroed(), Aad::from(self.0.key.as_ref()))?;
        let mut derive_key = sm4.mac()?;

        let derivation_len = label
            .len()
            .checked_add(4)
            .ok_or(SgxStatus::InvalidParameter)?;
        let mut derivation = vec![0_u8; label.len() + 4];
        derivation[0] = 0x01;
        derivation[1..derivation_len - 3].copy_from_slice(label);
        derivation[derivation_len - 3..].copy_from_slice(&[0x00, 0x80, 0x00]);

        let mut sm4 = Sm4Ccm::new(
            &derive_key,
            Nonce::zeroed(),
            Aad::from(derivation.as_slice()),
        )?;
        let key = sm4.mac().map(AlignKey128bit::from)?;

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
pub struct Sm2Signature(Ec256Signature);

impl Sm2Signature {
    #[inline]
    pub fn signature(&self) -> Ec256Signature {
        self.0
    }
}

impl From<(Sm2PrivateKey, Sm2PublicKey)> for Sm2KeyPair {
    fn from(key_pair: (Sm2PrivateKey, Sm2PublicKey)) -> Sm2KeyPair {
        Sm2KeyPair {
            private: key_pair.0,
            public: key_pair.1,
        }
    }
}

impl From<Sm2KeyPair> for (Sm2PrivateKey, Sm2PublicKey) {
    fn from(key_pair: Sm2KeyPair) -> (Sm2PrivateKey, Sm2PublicKey) {
        (key_pair.private, key_pair.public)
    }
}

impl From<&Sm2KeyPair> for (Sm2PrivateKey, Sm2PublicKey) {
    fn from(key_pair: &Sm2KeyPair) -> (Sm2PrivateKey, Sm2PublicKey) {
        (key_pair.private, key_pair.public)
    }
}

impl From<Sm2KeyPair> for [u8; ECP256_KEY_SIZE * 3] {
    #[inline]
    fn from(key: Sm2KeyPair) -> [u8; ECP256_KEY_SIZE * 3] {
        From::<&Sm2KeyPair>::from(&key)
    }
}

impl From<&Sm2KeyPair> for [u8; ECP256_KEY_SIZE * 3] {
    #[inline]
    fn from(key: &Sm2KeyPair) -> [u8; ECP256_KEY_SIZE * 3] {
        let mut array = [0_u8; ECP256_KEY_SIZE * 3];
        array[..ECP256_KEY_SIZE].copy_from_slice(&key.private.as_ref()[..]);
        array[ECP256_KEY_SIZE..].copy_from_slice(&key.public.as_ref()[..]);
        array
    }
}

impl From<[u8; ECP256_KEY_SIZE * 3]> for Sm2KeyPair {
    #[inline]
    fn from(key: [u8; ECP256_KEY_SIZE * 3]) -> Sm2KeyPair {
        From::<&[u8; ECP256_KEY_SIZE * 3]>::from(&key)
    }
}

impl From<&[u8; ECP256_KEY_SIZE * 3]> for Sm2KeyPair {
    #[inline]
    fn from(key: &[u8; ECP256_KEY_SIZE * 3]) -> Sm2KeyPair {
        let array: &[u8; ECP256_KEY_SIZE] = key[..ECP256_KEY_SIZE].try_into().unwrap();
        let private_key = Sm2PrivateKey::from(array);

        let array: &[u8; ECP256_KEY_SIZE * 2] = key[ECP256_KEY_SIZE..].try_into().unwrap();
        let public_key = Sm2PublicKey::from(array);

        Sm2KeyPair::from((private_key, public_key))
    }
}

impl From<AlignEc256PrivateKey> for Sm2PrivateKey {
    #[inline]
    fn from(key: AlignEc256PrivateKey) -> Sm2PrivateKey {
        Sm2PrivateKey(key)
    }
}

impl From<&AlignEc256PrivateKey> for Sm2PrivateKey {
    #[inline]
    fn from(key: &AlignEc256PrivateKey) -> Sm2PrivateKey {
        Sm2PrivateKey(*key)
    }
}

impl From<Ec256PrivateKey> for Sm2PrivateKey {
    #[inline]
    fn from(key: Ec256PrivateKey) -> Sm2PrivateKey {
        Sm2PrivateKey(From::from(key))
    }
}

impl From<&Ec256PrivateKey> for Sm2PrivateKey {
    #[inline]
    fn from(key: &Ec256PrivateKey) -> Sm2PrivateKey {
        Sm2PrivateKey(From::from(key))
    }
}

impl AsRef<[u8; ECP256_KEY_SIZE]> for Sm2PrivateKey {
    #[inline]
    fn as_ref(&self) -> &[u8; ECP256_KEY_SIZE] {
        self.0.as_ref()
    }
}

impl From<Sm2PrivateKey> for [u8; ECP256_KEY_SIZE] {
    #[inline]
    fn from(key: Sm2PrivateKey) -> [u8; ECP256_KEY_SIZE] {
        *key.as_ref()
    }
}

impl From<&Sm2PrivateKey> for [u8; ECP256_KEY_SIZE] {
    #[inline]
    fn from(key: &Sm2PrivateKey) -> [u8; ECP256_KEY_SIZE] {
        *key.as_ref()
    }
}

impl From<[u8; ECP256_KEY_SIZE]> for Sm2PrivateKey {
    #[inline]
    fn from(key: [u8; ECP256_KEY_SIZE]) -> Sm2PrivateKey {
        Sm2PrivateKey(AlignEc256PrivateKey::from(key))
    }
}

impl From<&[u8; ECP256_KEY_SIZE]> for Sm2PrivateKey {
    #[inline]
    fn from(key: &[u8; ECP256_KEY_SIZE]) -> Sm2PrivateKey {
        Sm2PrivateKey(AlignEc256PrivateKey::from(key))
    }
}

impl From<Sm2PrivateKey> for Ec256PrivateKey {
    #[inline]
    fn from(key: Sm2PrivateKey) -> Ec256PrivateKey {
        key.0.key
    }
}

impl From<&Sm2PrivateKey> for Ec256PrivateKey {
    #[inline]
    fn from(key: &Sm2PrivateKey) -> Ec256PrivateKey {
        key.0.key
    }
}

impl From<Sm2PrivateKey> for AlignEc256PrivateKey {
    #[inline]
    fn from(key: Sm2PrivateKey) -> AlignEc256PrivateKey {
        key.0
    }
}

impl From<&Sm2PrivateKey> for AlignEc256PrivateKey {
    #[inline]
    fn from(key: &Sm2PrivateKey) -> AlignEc256PrivateKey {
        key.0
    }
}

impl From<Ec256PublicKey> for Sm2PublicKey {
    #[inline]
    fn from(key: Ec256PublicKey) -> Sm2PublicKey {
        Sm2PublicKey(key)
    }
}

impl From<&Ec256PublicKey> for Sm2PublicKey {
    #[inline]
    fn from(key: &Ec256PublicKey) -> Sm2PublicKey {
        Sm2PublicKey(*key)
    }
}

impl AsRef<[u8; ECP256_KEY_SIZE * 2]> for Sm2PublicKey {
    #[inline]
    fn as_ref(&self) -> &[u8; ECP256_KEY_SIZE * 2] {
        self.0.as_ref()
    }
}

impl From<Sm2PublicKey> for [u8; ECP256_KEY_SIZE * 2] {
    #[inline]
    fn from(key: Sm2PublicKey) -> [u8; ECP256_KEY_SIZE * 2] {
        *key.as_ref()
    }
}

impl From<&Sm2PublicKey> for [u8; ECP256_KEY_SIZE * 2] {
    #[inline]
    fn from(key: &Sm2PublicKey) -> [u8; ECP256_KEY_SIZE * 2] {
        *key.as_ref()
    }
}

impl From<[u8; ECP256_KEY_SIZE * 2]> for Sm2PublicKey {
    #[inline]
    fn from(key: [u8; ECP256_KEY_SIZE * 2]) -> Sm2PublicKey {
        Sm2PublicKey(Ec256PublicKey::from(key))
    }
}

impl From<&[u8; ECP256_KEY_SIZE * 2]> for Sm2PublicKey {
    #[inline]
    fn from(key: &[u8; ECP256_KEY_SIZE * 2]) -> Sm2PublicKey {
        Sm2PublicKey(Ec256PublicKey::from(key))
    }
}

impl From<Sm2PublicKey> for Ec256PublicKey {
    #[inline]
    fn from(key: Sm2PublicKey) -> Ec256PublicKey {
        key.0
    }
}

impl From<&Sm2PublicKey> for Ec256PublicKey {
    #[inline]
    fn from(key: &Sm2PublicKey) -> Ec256PublicKey {
        key.0
    }
}

impl From<AlignEc256SharedKey> for Sm2ShareKey {
    #[inline]
    fn from(key: AlignEc256SharedKey) -> Sm2ShareKey {
        Sm2ShareKey(key)
    }
}

impl From<&AlignEc256SharedKey> for Sm2ShareKey {
    #[inline]
    fn from(key: &AlignEc256SharedKey) -> Sm2ShareKey {
        Sm2ShareKey(*key)
    }
}

impl From<Ec256SharedKey> for Sm2ShareKey {
    #[inline]
    fn from(key: Ec256SharedKey) -> Sm2ShareKey {
        Sm2ShareKey(From::from(key))
    }
}

impl From<&Ec256SharedKey> for Sm2ShareKey {
    #[inline]
    fn from(key: &Ec256SharedKey) -> Sm2ShareKey {
        Sm2ShareKey(From::from(key))
    }
}

impl AsRef<[u8; ECP256_KEY_SIZE]> for Sm2ShareKey {
    #[inline]
    fn as_ref(&self) -> &[u8; ECP256_KEY_SIZE] {
        self.0.as_ref()
    }
}

impl From<Sm2ShareKey> for [u8; ECP256_KEY_SIZE] {
    #[inline]
    fn from(key: Sm2ShareKey) -> [u8; ECP256_KEY_SIZE] {
        *key.as_ref()
    }
}

impl From<&Sm2ShareKey> for [u8; ECP256_KEY_SIZE] {
    #[inline]
    fn from(key: &Sm2ShareKey) -> [u8; ECP256_KEY_SIZE] {
        *key.as_ref()
    }
}

impl From<[u8; ECP256_KEY_SIZE]> for Sm2ShareKey {
    #[inline]
    fn from(key: [u8; ECP256_KEY_SIZE]) -> Sm2ShareKey {
        Sm2ShareKey(AlignEc256SharedKey::from(key))
    }
}

impl From<&[u8; ECP256_KEY_SIZE]> for Sm2ShareKey {
    #[inline]
    fn from(key: &[u8; ECP256_KEY_SIZE]) -> Sm2ShareKey {
        Sm2ShareKey(AlignEc256SharedKey::from(key))
    }
}

impl From<Sm2ShareKey> for Ec256SharedKey {
    #[inline]
    fn from(key: Sm2ShareKey) -> Ec256SharedKey {
        key.0.key
    }
}

impl From<&Sm2ShareKey> for Ec256SharedKey {
    #[inline]
    fn from(key: &Sm2ShareKey) -> Ec256SharedKey {
        key.0.key
    }
}

impl From<Sm2ShareKey> for AlignEc256SharedKey {
    #[inline]
    fn from(key: Sm2ShareKey) -> AlignEc256SharedKey {
        key.0
    }
}

impl From<Ec256Signature> for Sm2Signature {
    #[inline]
    fn from(signature: Ec256Signature) -> Sm2Signature {
        Sm2Signature(signature)
    }
}

impl From<&Ec256Signature> for Sm2Signature {
    #[inline]
    fn from(signature: &Ec256Signature) -> Sm2Signature {
        Sm2Signature(*signature)
    }
}

impl AsRef<[u8; ECP256_KEY_SIZE * 2]> for Sm2Signature {
    #[inline]
    fn as_ref(&self) -> &[u8; ECP256_KEY_SIZE * 2] {
        self.0.as_ref()
    }
}

impl From<Sm2Signature> for [u8; ECP256_KEY_SIZE * 2] {
    #[inline]
    fn from(signature: Sm2Signature) -> [u8; ECP256_KEY_SIZE * 2] {
        *signature.as_ref()
    }
}

impl From<&Sm2Signature> for [u8; ECP256_KEY_SIZE * 2] {
    #[inline]
    fn from(signature: &Sm2Signature) -> [u8; ECP256_KEY_SIZE * 2] {
        *signature.as_ref()
    }
}

impl From<[u8; ECP256_KEY_SIZE * 2]> for Sm2Signature {
    #[inline]
    fn from(signature: [u8; ECP256_KEY_SIZE * 2]) -> Sm2Signature {
        Sm2Signature(Ec256Signature::from(signature))
    }
}

impl From<&[u8; ECP256_KEY_SIZE * 2]> for Sm2Signature {
    #[inline]
    fn from(signature: &[u8; ECP256_KEY_SIZE * 2]) -> Sm2Signature {
        Sm2Signature(Ec256Signature::from(signature))
    }
}

impl From<Sm2Signature> for Ec256Signature {
    #[inline]
    fn from(signature: Sm2Signature) -> Ec256Signature {
        signature.0
    }
}

impl From<&Sm2Signature> for Ec256Signature {
    #[inline]
    fn from(signature: &Sm2Signature) -> Ec256Signature {
        signature.0
    }
}

impl From<&Sm2ShareKey> for AlignEc256SharedKey {
    #[inline]
    fn from(key: &Sm2ShareKey) -> AlignEc256SharedKey {
        key.0
    }
}

impl ConstTimeEq<Sm2PrivateKey> for Sm2PrivateKey {
    #[inline]
    fn ct_eq(&self, other: &Sm2PrivateKey) -> bool {
        self.0.ct_eq(&other.0)
    }
}

impl ConstTimeEq<Sm2PublicKey> for Sm2PublicKey {
    #[inline]
    fn ct_eq(&self, other: &Sm2PublicKey) -> bool {
        self.0.ct_eq(&other.0)
    }
}

impl ConstTimeEq<Sm2ShareKey> for Sm2ShareKey {
    #[inline]
    fn ct_eq(&self, other: &Sm2ShareKey) -> bool {
        self.0.ct_eq(&other.0)
    }
}

impl ConstTimeEq<Sm2Signature> for Sm2Signature {
    #[inline]
    fn ct_eq(&self, other: &Sm2Signature) -> bool {
        self.0.ct_eq(&other.0)
    }
}

unsafe impl ContiguousMemory for Sm2KeyPair {}
unsafe impl ContiguousMemory for Sm2PrivateKey {}
unsafe impl ContiguousMemory for Sm2PublicKey {}
unsafe impl ContiguousMemory for Sm2ShareKey {}
unsafe impl ContiguousMemory for Sm2Signature {}

unsafe impl BytewiseEquality for Sm2KeyPair {}
unsafe impl BytewiseEquality for Sm2PrivateKey {}
unsafe impl BytewiseEquality for Sm2PublicKey {}
unsafe impl BytewiseEquality for Sm2ShareKey {}
unsafe impl BytewiseEquality for Sm2Signature {}
