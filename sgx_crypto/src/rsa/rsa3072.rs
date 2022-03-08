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

use crate::rsa::RsaPrivateType;
use alloc::vec::Vec;
use core::cmp;
use core::convert::{From, TryInto};
use core::mem;
use core::ptr;
use sgx_crypto_sys::*;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::{BytewiseEquality, ContiguousMemory};
use sgx_types::types::{
    Rsa3072Key, Rsa3072Param, Rsa3072PubKey, Rsa3072Signature, RsaKey, RsaKeyType, RsaResult,
};
use sgx_types::types::{RSA3072_KEY_SIZE, RSA3072_PRI_EXP_SIZE, RSA3072_PUB_EXP_SIZE};

#[cfg(any(feature = "tserialize", feature = "userialize"))]
use sgx_serialize::{Deserialize, Serialize};

const RSA3072_DEFAULT_E: [u8; RSA3072_PUB_EXP_SIZE] = [0x01, 0x00, 0x00, 0x01]; // 16777217

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct Rsa3072KeyPair {
    n: [u8; RSA3072_KEY_SIZE],
    d: [u8; RSA3072_PRI_EXP_SIZE],
    e: [u8; RSA3072_PUB_EXP_SIZE],
    p: [u8; RSA3072_KEY_SIZE / 2],
    q: [u8; RSA3072_KEY_SIZE / 2],
    dmp1: [u8; RSA3072_KEY_SIZE / 2],
    dmq1: [u8; RSA3072_KEY_SIZE / 2],
    iqmp: [u8; RSA3072_KEY_SIZE / 2],
    privtype: RsaPrivateType,
}

impl Rsa3072KeyPair {
    pub fn create() -> SgxResult<Rsa3072KeyPair> {
        let mut key = Rsa3072KeyPair::default();
        let status = unsafe {
            sgx_create_rsa_key_pair(
                RSA3072_KEY_SIZE as i32,
                RSA3072_PUB_EXP_SIZE as i32,
                key.n.as_mut_ptr(),
                key.d.as_mut_ptr(),
                key.e.as_mut_ptr(),
                key.p.as_mut_ptr(),
                key.q.as_mut_ptr(),
                key.dmp1.as_mut_ptr(),
                key.dmq1.as_mut_ptr(),
                key.iqmp.as_mut_ptr(),
            )
        };

        ensure!(status.is_success(), status);
        Ok(key)
    }

    pub fn create_with_e(e: u32) -> SgxResult<Rsa3072KeyPair> {
        let mut key = Rsa3072KeyPair {
            e: e.to_le_bytes(),
            ..Default::default()
        };
        let status = unsafe {
            sgx_create_rsa_key_pair(
                RSA3072_KEY_SIZE as i32,
                RSA3072_PUB_EXP_SIZE as i32,
                key.n.as_mut_ptr(),
                key.d.as_mut_ptr(),
                key.e.as_mut_ptr(),
                key.p.as_mut_ptr(),
                key.q.as_mut_ptr(),
                key.dmp1.as_mut_ptr(),
                key.dmq1.as_mut_ptr(),
                key.iqmp.as_mut_ptr(),
            )
        };

        ensure!(status.is_success(), status);
        Ok(key)
    }

    #[inline]
    pub fn public_key(&self) -> Rsa3072PublicKey {
        Rsa3072PublicKey {
            n: self.n,
            e: self.e,
        }
    }

    #[inline]
    pub fn private_key(&self) -> Rsa3072PrivateKey {
        Rsa3072PrivateKey {
            n: self.n,
            d: self.d,
            e: self.e,
            p: self.p,
            q: self.q,
            dmp1: self.dmp1,
            dmq1: self.dmq1,
            iqmp: self.iqmp,
            privtype: self.privtype,
        }
    }

    #[inline]
    pub fn encrypt(&self, plaintext: &[u8]) -> SgxResult<Vec<u8>> {
        self.public_key().encrypt(plaintext)
    }

    #[inline]
    pub fn decrypt(&self, ciphertext: &[u8]) -> SgxResult<Vec<u8>> {
        self.private_key().decrypt(ciphertext)
    }

    #[inline]
    pub fn sign<T: ?Sized>(&self, data: &T) -> SgxResult<Rsa3072Signature>
    where
        T: ContiguousMemory,
    {
        self.private_key().sign(data)
    }

    #[inline]
    pub fn sign_and_verify<T: ?Sized>(&self, data: &T) -> SgxResult<Rsa3072Signature>
    where
        T: ContiguousMemory,
    {
        self.private_key().sign_and_verify(&self.public_key(), data)
    }

    #[inline]
    pub fn verify<T: ?Sized>(&self, data: &T, signature: &Rsa3072Signature) -> SgxResult<bool>
    where
        T: ContiguousMemory,
    {
        self.public_key().verify(data, signature)
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Default::default();
    }
}

impl Default for Rsa3072KeyPair {
    fn default() -> Self {
        Rsa3072KeyPair {
            n: [0; RSA3072_KEY_SIZE],
            d: [0; RSA3072_PRI_EXP_SIZE],
            e: RSA3072_DEFAULT_E,
            p: [0; RSA3072_KEY_SIZE / 2],
            q: [0; RSA3072_KEY_SIZE / 2],
            dmp1: [0; RSA3072_KEY_SIZE / 2],
            dmq1: [0; RSA3072_KEY_SIZE / 2],
            iqmp: [0; RSA3072_KEY_SIZE / 2],
            privtype: RsaPrivateType::Type2,
        }
    }
}

impl From<Rsa3072Param> for Rsa3072KeyPair {
    fn from(param: Rsa3072Param) -> Rsa3072KeyPair {
        Rsa3072KeyPair {
            n: param.n,
            d: param.d,
            e: param.e,
            p: param.p,
            q: param.q,
            dmp1: param.dmp1,
            dmq1: param.dmq1,
            iqmp: param.iqmp,
            privtype: RsaPrivateType::Type2,
        }
    }
}

impl From<&Rsa3072Param> for Rsa3072KeyPair {
    fn from(param: &Rsa3072Param) -> Rsa3072KeyPair {
        Rsa3072KeyPair {
            n: param.n,
            d: param.d,
            e: param.e,
            p: param.p,
            q: param.q,
            dmp1: param.dmp1,
            dmq1: param.dmq1,
            iqmp: param.iqmp,
            privtype: RsaPrivateType::Type2,
        }
    }
}

impl From<Rsa3072Key> for Rsa3072KeyPair {
    fn from(key_pair: Rsa3072Key) -> Rsa3072KeyPair {
        Rsa3072KeyPair {
            n: key_pair.modulus,
            d: key_pair.d,
            e: key_pair.e,
            p: [0; RSA3072_KEY_SIZE / 2],
            q: [0; RSA3072_KEY_SIZE / 2],
            dmp1: [0; RSA3072_KEY_SIZE / 2],
            dmq1: [0; RSA3072_KEY_SIZE / 2],
            iqmp: [0; RSA3072_KEY_SIZE / 2],
            privtype: RsaPrivateType::Type1,
        }
    }
}

impl From<&Rsa3072Key> for Rsa3072KeyPair {
    fn from(key_pair: &Rsa3072Key) -> Rsa3072KeyPair {
        Rsa3072KeyPair {
            n: key_pair.modulus,
            d: key_pair.d,
            e: key_pair.e,
            p: [0; RSA3072_KEY_SIZE / 2],
            q: [0; RSA3072_KEY_SIZE / 2],
            dmp1: [0; RSA3072_KEY_SIZE / 2],
            dmq1: [0; RSA3072_KEY_SIZE / 2],
            iqmp: [0; RSA3072_KEY_SIZE / 2],
            privtype: RsaPrivateType::Type1,
        }
    }
}

impl From<(Rsa3072PrivateKey, Rsa3072PublicKey)> for Rsa3072KeyPair {
    fn from(key_pair: (Rsa3072PrivateKey, Rsa3072PublicKey)) -> Rsa3072KeyPair {
        Rsa3072KeyPair {
            n: key_pair.0.n,
            d: key_pair.0.d,
            e: key_pair.0.e,
            p: key_pair.0.p,
            q: key_pair.0.q,
            dmp1: key_pair.0.dmp1,
            dmq1: key_pair.0.dmq1,
            iqmp: key_pair.0.iqmp,
            privtype: key_pair.0.privtype,
        }
    }
}

impl From<Rsa3072KeyPair> for (Rsa3072PrivateKey, Rsa3072PublicKey) {
    fn from(key_pair: Rsa3072KeyPair) -> (Rsa3072PrivateKey, Rsa3072PublicKey) {
        (
            Rsa3072PrivateKey {
                n: key_pair.n,
                d: key_pair.d,
                e: key_pair.e,
                p: key_pair.p,
                q: key_pair.q,
                dmp1: key_pair.dmp1,
                dmq1: key_pair.dmq1,
                iqmp: key_pair.iqmp,
                privtype: key_pair.privtype,
            },
            Rsa3072PublicKey {
                n: key_pair.n,
                e: key_pair.e,
            },
        )
    }
}

impl From<&Rsa3072KeyPair> for (Rsa3072PrivateKey, Rsa3072PublicKey) {
    fn from(key_pair: &Rsa3072KeyPair) -> (Rsa3072PrivateKey, Rsa3072PublicKey) {
        (
            Rsa3072PrivateKey {
                n: key_pair.n,
                d: key_pair.d,
                e: key_pair.e,
                p: key_pair.p,
                q: key_pair.q,
                dmp1: key_pair.dmp1,
                dmq1: key_pair.dmq1,
                iqmp: key_pair.iqmp,
                privtype: key_pair.privtype,
            },
            Rsa3072PublicKey {
                n: key_pair.n,
                e: key_pair.e,
            },
        )
    }
}

impl_asref_array! { Rsa3072KeyPair; }
impl_from_array! { Rsa3072KeyPair; }

unsafe impl ContiguousMemory for Rsa3072KeyPair {}

unsafe impl BytewiseEquality for Rsa3072KeyPair {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct Rsa3072PrivateKey {
    n: [u8; RSA3072_KEY_SIZE],
    d: [u8; RSA3072_PRI_EXP_SIZE],
    e: [u8; RSA3072_PUB_EXP_SIZE],
    p: [u8; RSA3072_KEY_SIZE / 2],
    q: [u8; RSA3072_KEY_SIZE / 2],
    dmp1: [u8; RSA3072_KEY_SIZE / 2],
    dmq1: [u8; RSA3072_KEY_SIZE / 2],
    iqmp: [u8; RSA3072_KEY_SIZE / 2],
    privtype: RsaPrivateType,
}

impl Rsa3072PrivateKey {
    pub fn decrypt(&self, ciphertext: &[u8]) -> SgxResult<Vec<u8>> {
        let bs = 384;
        ensure!(ciphertext.len() % bs == 0, SgxStatus::InvalidParameter);

        let privkey = self.create()?;
        let mut plaintext: Vec<u8> = Vec::new();
        let bs_plain = bs;
        let count = ciphertext.len() / bs;

        let mut plain = vec![0_u8; bs_plain];
        let plain_slice = plain.as_mut_slice();

        for i in 0..count {
            let cipher_slice = &ciphertext[i * bs..i * bs + bs];
            let mut plain_len = bs_plain;
            plain_slice.fill(0);

            let status = unsafe {
                sgx_rsa_priv_decrypt_sha256(
                    privkey,
                    plain_slice.as_mut_ptr(),
                    &mut plain_len as *mut usize,
                    cipher_slice.as_ptr(),
                    cipher_slice.len(),
                )
            };
            if !status.is_success() {
                let _ = Self::free(privkey);
                bail!(status);
            }
            plaintext.extend_from_slice(&plain_slice[..plain_len]);
        }
        let _ = Self::free(privkey);

        Ok(plaintext)
    }

    pub fn sign<T: ?Sized>(&self, data: &T) -> SgxResult<Rsa3072Signature>
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut signature = Rsa3072Signature::default();
        let status = unsafe {
            sgx_rsa3072_sign(
                (data as *const T).cast(),
                size as u32,
                &self.into() as *const Rsa3072Key,
                &mut signature as *mut Rsa3072Signature,
            )
        };

        ensure!(status.is_success(), status);
        Ok(signature)
    }

    pub fn sign_and_verify<T: ?Sized>(
        &self,
        public_key: &Rsa3072PublicKey,
        data: &T,
    ) -> SgxResult<Rsa3072Signature>
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut signature = Rsa3072Signature::default();
        let status = unsafe {
            sgx_rsa3072_sign_ex(
                (data as *const T).cast(),
                size as u32,
                &self.into() as *const Rsa3072Key,
                &public_key.into() as *const Rsa3072PubKey,
                &mut signature as *mut Rsa3072Signature,
            )
        };

        ensure!(status.is_success(), status);
        Ok(signature)
    }

    #[inline]
    pub fn export_public_key(&self) -> Rsa3072PublicKey {
        Rsa3072PublicKey {
            n: self.n,
            e: self.e,
        }
    }

    #[inline]
    pub fn private_key(&self) -> Rsa3072Key {
        Rsa3072Key {
            modulus: self.n,
            d: self.d,
            e: self.e,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Default::default();
    }

    fn create(&self) -> SgxResult<RsaKey> {
        let mut key: RsaKey = ptr::null_mut();
        let status = unsafe {
            match self.privtype {
                RsaPrivateType::Type1 => sgx_create_rsa_priv1_key(
                    RSA3072_KEY_SIZE as i32,
                    RSA3072_PUB_EXP_SIZE as i32,
                    RSA3072_PRI_EXP_SIZE as i32,
                    self.n.as_ptr(),
                    self.e.as_ptr(),
                    self.d.as_ptr(),
                    &mut key as *mut RsaKey,
                ),
                RsaPrivateType::Type2 => {
                    let mut status = sgx_create_rsa_priv2_key(
                        RSA3072_KEY_SIZE as i32,
                        RSA3072_PRI_EXP_SIZE as i32,
                        self.e.as_ptr(),
                        self.p.as_ptr(),
                        self.q.as_ptr(),
                        self.dmp1.as_ptr(),
                        self.dmq1.as_ptr(),
                        self.iqmp.as_ptr(),
                        &mut key as *mut RsaKey,
                    );
                    if !status.is_success() {
                        status = sgx_create_rsa_priv1_key(
                            RSA3072_KEY_SIZE as i32,
                            RSA3072_PUB_EXP_SIZE as i32,
                            RSA3072_PRI_EXP_SIZE as i32,
                            self.n.as_ptr(),
                            self.e.as_ptr(),
                            self.d.as_ptr(),
                            &mut key as *mut RsaKey,
                        );
                    }
                    status
                }
            }
        };

        ensure!(status.is_success(), status);
        Ok(key)
    }

    fn free(key: RsaKey) -> SgxResult {
        let status = unsafe {
            sgx_free_rsa_key(
                key,
                RsaKeyType::PrivateKey,
                RSA3072_KEY_SIZE as i32,
                RSA3072_PRI_EXP_SIZE as i32,
            )
        };

        ensure!(status.is_success(), status);
        Ok(())
    }
}

impl From<Rsa3072Key> for Rsa3072PrivateKey {
    fn from(key: Rsa3072Key) -> Rsa3072PrivateKey {
        Rsa3072PrivateKey {
            n: key.modulus,
            d: key.d,
            e: key.e,
            p: [0; RSA3072_KEY_SIZE / 2],
            q: [0; RSA3072_KEY_SIZE / 2],
            dmp1: [0; RSA3072_KEY_SIZE / 2],
            dmq1: [0; RSA3072_KEY_SIZE / 2],
            iqmp: [0; RSA3072_KEY_SIZE / 2],
            privtype: RsaPrivateType::Type1,
        }
    }
}

impl From<&Rsa3072Key> for Rsa3072PrivateKey {
    fn from(key: &Rsa3072Key) -> Rsa3072PrivateKey {
        Rsa3072PrivateKey {
            n: key.modulus,
            d: key.d,
            e: key.e,
            p: [0; RSA3072_KEY_SIZE / 2],
            q: [0; RSA3072_KEY_SIZE / 2],
            dmp1: [0; RSA3072_KEY_SIZE / 2],
            dmq1: [0; RSA3072_KEY_SIZE / 2],
            iqmp: [0; RSA3072_KEY_SIZE / 2],
            privtype: RsaPrivateType::Type1,
        }
    }
}

impl From<Rsa3072PrivateKey> for Rsa3072Key {
    fn from(key: Rsa3072PrivateKey) -> Rsa3072Key {
        Rsa3072Key {
            modulus: key.n,
            d: key.d,
            e: key.e,
        }
    }
}

impl From<&Rsa3072PrivateKey> for Rsa3072Key {
    fn from(key: &Rsa3072PrivateKey) -> Rsa3072Key {
        Rsa3072Key {
            modulus: key.n,
            d: key.d,
            e: key.e,
        }
    }
}

impl Default for Rsa3072PrivateKey {
    fn default() -> Self {
        Rsa3072PrivateKey {
            n: [0; RSA3072_KEY_SIZE],
            d: [0; RSA3072_PRI_EXP_SIZE],
            e: RSA3072_DEFAULT_E,
            p: [0; RSA3072_KEY_SIZE / 2],
            q: [0; RSA3072_KEY_SIZE / 2],
            dmp1: [0; RSA3072_KEY_SIZE / 2],
            dmq1: [0; RSA3072_KEY_SIZE / 2],
            iqmp: [0; RSA3072_KEY_SIZE / 2],
            privtype: RsaPrivateType::Type2,
        }
    }
}

impl_asref_array! { Rsa3072PrivateKey; }
impl_from_array! { Rsa3072PrivateKey; }

unsafe impl ContiguousMemory for Rsa3072PrivateKey {}

unsafe impl BytewiseEquality for Rsa3072PrivateKey {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    any(feature = "tserialize", feature = "userialize"),
    derive(Deserialize, Serialize)
)]
pub struct Rsa3072PublicKey {
    n: [u8; RSA3072_KEY_SIZE],
    e: [u8; RSA3072_PUB_EXP_SIZE],
}

impl Rsa3072PublicKey {
    pub fn encrypt(&self, plaintext: &[u8]) -> SgxResult<Vec<u8>> {
        ensure!(!plaintext.is_empty(), SgxStatus::InvalidParameter);

        let pubkey = self.create()?;
        let bs = 384;
        let bs_plain = bs - 2 * 256 / 8 - 2;
        let count = (plaintext.len() + bs_plain - 1) / bs_plain;
        let mut ciphertext = vec![0_u8; bs * count];

        for i in 0..count {
            let cipher_slice = &mut ciphertext[i * bs..i * bs + bs];
            let mut cipher_len = bs;
            let plain_slice =
                &plaintext[i * bs_plain..cmp::min(i * bs_plain + bs_plain, plaintext.len())];

            let status = unsafe {
                sgx_rsa_pub_encrypt_sha256(
                    pubkey,
                    cipher_slice.as_mut_ptr(),
                    &mut cipher_len as *mut usize,
                    plain_slice.as_ptr(),
                    plain_slice.len(),
                )
            };
            if !status.is_success() {
                let _ = Self::free(pubkey);
                bail!(status);
            }
        }
        let _ = Self::free(pubkey);

        Ok(ciphertext)
    }

    pub fn verify<T: ?Sized>(&self, data: &T, signature: &Rsa3072Signature) -> SgxResult<bool>
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut rsa_result = RsaResult::InvalidSignature;
        let status = unsafe {
            sgx_rsa3072_verify(
                (data as *const T).cast(),
                size as u32,
                &self.into() as *const Rsa3072PubKey,
                signature as *const Rsa3072Signature,
                &mut rsa_result as *mut RsaResult,
            )
        };

        ensure!(status.is_success(), status);
        match rsa_result {
            RsaResult::Valid => Ok(true),
            _ => Ok(false),
        }
    }

    #[inline]
    pub fn from_private_key(key: &Rsa3072PrivateKey) -> Rsa3072PublicKey {
        key.export_public_key()
    }

    #[inline]
    pub fn public_key(&self) -> Rsa3072PubKey {
        Rsa3072PubKey {
            modulus: self.n,
            exponent: self.e,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Default::default();
    }

    fn create(&self) -> SgxResult<RsaKey> {
        let mut key: RsaKey = ptr::null_mut();
        let status = unsafe {
            sgx_create_rsa_pub1_key(
                RSA3072_KEY_SIZE as i32,
                RSA3072_PUB_EXP_SIZE as i32,
                self.n.as_ptr(),
                self.e.as_ptr(),
                &mut key as *mut RsaKey,
            )
        };

        ensure!(status.is_success(), status);
        Ok(key)
    }

    fn free(key: RsaKey) -> SgxResult {
        let status = unsafe {
            sgx_free_rsa_key(
                key,
                RsaKeyType::PublicKey,
                RSA3072_KEY_SIZE as i32,
                RSA3072_PUB_EXP_SIZE as i32,
            )
        };

        ensure!(status.is_success(), status);
        Ok(())
    }
}

impl From<Rsa3072PubKey> for Rsa3072PublicKey {
    fn from(key: Rsa3072PubKey) -> Rsa3072PublicKey {
        Rsa3072PublicKey {
            n: key.modulus,
            e: key.exponent,
        }
    }
}

impl From<&Rsa3072PubKey> for Rsa3072PublicKey {
    fn from(key: &Rsa3072PubKey) -> Rsa3072PublicKey {
        Rsa3072PublicKey {
            n: key.modulus,
            e: key.exponent,
        }
    }
}

impl From<Rsa3072PublicKey> for Rsa3072PubKey {
    fn from(key: Rsa3072PublicKey) -> Rsa3072PubKey {
        Rsa3072PubKey {
            modulus: key.n,
            exponent: key.e,
        }
    }
}

impl From<&Rsa3072PublicKey> for Rsa3072PubKey {
    fn from(key: &Rsa3072PublicKey) -> Rsa3072PubKey {
        Rsa3072PubKey {
            modulus: key.n,
            exponent: key.e,
        }
    }
}

impl Default for Rsa3072PublicKey {
    fn default() -> Self {
        Rsa3072PublicKey {
            n: [0; RSA3072_KEY_SIZE],
            e: RSA3072_DEFAULT_E,
        }
    }
}

impl_asref_array! { Rsa3072PublicKey; }
impl_from_array! { Rsa3072PublicKey; }

unsafe impl ContiguousMemory for Rsa3072PublicKey {}

unsafe impl BytewiseEquality for Rsa3072PublicKey {}
