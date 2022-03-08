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

//
// sgx_tcrypto.h
//
pub const SHA1_HASH_SIZE: usize = 20;
pub const SHA256_HASH_SIZE: usize = 32;
pub const SM3_HASH_SIZE: usize = 32;

pub type ShaHandle = *mut c_void;
pub type CMacHandle = *mut c_void;
pub type HMacHandle = *mut c_void;
pub type AesHandle = *mut c_void;
pub type EccHandle = *mut c_void;
pub type Sm3Handle = *mut c_void;
pub type Sm4Handle = *mut c_void;

impl_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum EcResult {
        Valid                   = 0x00,   /* validation pass successfully */
        CompositeBase           = 0x01,   /* field based on composite */
        ComplicatedBase         = 0x02,   /* number of non-zero terms in the polynomial (> PRIME_ARR_MAX) */
        IsZeroDiscriminant      = 0x03,   /* zero discriminant */
        CompositeOrder          = 0x04,   /* composite order of base point */
        InvalidOrder            = 0x05,   /* invalid base point order */
        IsWeakMov               = 0x06,   /* weak Meneze-Okamoto-Vanstone  reduction attack */
        IsWeakSsa               = 0x07,   /* weak Semaev-Smart,Satoh-Araki reduction attack */
        IsSuperSingular         = 0x08,   /* supersingular curve */
        InvalidPrivateKey       = 0x09,   /* !(0 < Private < order) */
        InvalidPublicKey        = 0x0A,   /* (order*PublicKey != Infinity) */
        InvalidKeyPair          = 0x0B,   /* (Private*BasePoint != PublicKey) */
        PointOutOfGroup         = 0x0C,   /* out of group (order*P != Infinity) */
        PointIsAtInfinity       = 0x0D,   /* point (P=(Px,Py)) at Infinity */
        PointIsNotValid         = 0x0E,   /* point (P=(Px,Py)) out-of EC */
        PointIsEqual            = 0x0F,   /* compared points are equal */
        PointIsNotEqual         = 0x10,   /* compared points are different */
        InvalidSignature        = 0x11,   /* invalid signature */
    }
}

impl EcResult {
    #[inline]
    pub fn is_valid(&self) -> bool {
        *self == EcResult::Valid
    }
}

pub type Key128bit = [u8; KEY_128BIT_SIZE];
pub type Sha1Hash = [u8; SHA1_HASH_SIZE];
pub type Sha256Hash = [u8; SHA256_HASH_SIZE];
pub type Sm3Hash = [u8; SM3_HASH_SIZE];

pub const KEY_128BIT_SIZE: usize = 16;
pub const KEY_256BIT_SIZE: usize = 32;
pub const MAC_128BIT_SIZE: usize = 16;
pub const MAC_256BIT_SIZE: usize = 32;

pub const ECP256_KEY_SIZE: usize = 32;
pub const NISTP_ECP256_KEY_SIZE: usize = ECP256_KEY_SIZE / 4;

pub const RSA3072_KEY_SIZE: usize = 384;
pub const RSA3072_PRI_EXP_SIZE: usize = 384;
pub const RSA3072_PUB_EXP_SIZE: usize = 4;

pub const RSA2048_KEY_SIZE: usize = 256;
pub const RSA2048_PRI_EXP_SIZE: usize = 256;
pub const RSA2048_PUB_EXP_SIZE: usize = 4;

pub const AESGCM_IV_SIZE: usize = 12;
pub const AESCCM_IV_SIZE: usize = 12;
pub const AESCBC_IV_SIZE: usize = 16;
pub const AESCTR_CTR_SIZE: usize = 16;
pub const SM4CCM_IV_SIZE: usize = 12;
pub const SM4CBC_IV_SIZE: usize = 16;
pub const SM4CTR_CTR_SIZE: usize = 16;

pub type Key256bit = [u8; KEY_256BIT_SIZE];
pub type Mac128bit = [u8; MAC_128BIT_SIZE];
pub type Mac256bit = [u8; MAC_256BIT_SIZE];

impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Ec256SharedKey {
        pub s: [u8; ECP256_KEY_SIZE],
    }

    // // delete (intel sgx sdk 2.0)
    // pub struct Ec256Shared512Key {
    //     pub x: [u8; ECP256_KEY_SIZE],
    //     pub y: [u8; ECP256_KEY_SIZE],
    // }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Ec256PrivateKey {
        pub r: [u8; ECP256_KEY_SIZE],
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Ec256PublicKey {
        pub gx: [u8; ECP256_KEY_SIZE],
        pub gy: [u8; ECP256_KEY_SIZE],
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Ec256Signature {
        pub x: [u32; NISTP_ECP256_KEY_SIZE],
        pub y: [u32; NISTP_ECP256_KEY_SIZE],
    }
}

impl_asref_array! {
    Ec256SharedKey;
    Ec256PrivateKey;
    Ec256PublicKey;
    Ec256Signature;
}
impl_asmut_array! {
    Ec256SharedKey;
    Ec256PrivateKey;
    Ec256PublicKey;
    Ec256Signature;
}
impl_from_array! {
    Ec256SharedKey;
    Ec256PrivateKey;
    Ec256PublicKey;
    Ec256Signature;
}

impl_unsafe_marker_for! {
    BytewiseEquality,
    Ec256PrivateKey Ec256SharedKey Ec256PublicKey Ec256Signature
}

pub type RsaKey = *mut c_void;

/* intel sgx sdk 2.1.3 */
impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum RsaKeyType {
        PrivateKey     = 0,   /* RSA private key state */
        PublicKey      = 1,   /* RSA public key state */
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum RsaResult {
        Valid              = 0,   /* validation pass successfully */
        InvalidSignature   = 1,   /* invalid signature */
    }
}

impl RsaResult {
    #[inline]
    pub fn is_valid(&self) -> bool {
        *self == RsaResult::Valid
    }
}

impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Rsa3072Param {
        pub n: [u8; RSA3072_KEY_SIZE],
        pub d: [u8; RSA3072_PRI_EXP_SIZE],
        pub e: [u8; RSA3072_PUB_EXP_SIZE],
        pub p: [u8; RSA3072_KEY_SIZE / 2],
        pub q: [u8; RSA3072_KEY_SIZE / 2],
        pub dmp1: [u8; RSA3072_KEY_SIZE / 2],
        pub dmq1: [u8; RSA3072_KEY_SIZE / 2],
        pub iqmp: [u8; RSA3072_KEY_SIZE / 2],
    }
}

impl_struct_default! {
    Rsa3072Param; //1732
}

impl_struct_ContiguousMemory! {
    Rsa3072Param;
}

impl_unsafe_marker_for! {BytewiseEquality, Rsa3072Param}

impl_asref_array! {
    Rsa3072Param;
}
impl_asmut_array! {
    Rsa3072Param;
}
impl_from_array! {
    Rsa3072Param;
}

impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Rsa3072PubKey {
        pub modulus: [u8; RSA3072_KEY_SIZE],
        pub exponent: [u8; RSA3072_PUB_EXP_SIZE],
    }

    /* intel sgx sdk 1.9 */
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Rsa3072PrivKey {
        pub modulus: [u8; RSA3072_KEY_SIZE],
        pub exponent: [u8; RSA3072_PRI_EXP_SIZE],
    }

    /* intel sgx sdk 2.0 */
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Rsa3072Key {
        pub modulus: [u8; RSA3072_KEY_SIZE],
        pub d: [u8; RSA3072_PRI_EXP_SIZE],
        pub e: [u8; RSA3072_PUB_EXP_SIZE],
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Rsa3072Signature {
        pub signature: [u8; RSA3072_KEY_SIZE],
    }
}

impl From<(Rsa3072PrivKey, Rsa3072PubKey)> for Rsa3072Key {
    #[inline]
    fn from(key_pair: (Rsa3072PrivKey, Rsa3072PubKey)) -> Rsa3072Key {
        Rsa3072Key {
            modulus: key_pair.0.modulus,
            d: key_pair.0.exponent,
            e: key_pair.1.exponent,
        }
    }
}

impl From<Rsa3072Key> for (Rsa3072PrivKey, Rsa3072PubKey) {
    #[inline]
    fn from(key_pair: Rsa3072Key) -> (Rsa3072PrivKey, Rsa3072PubKey) {
        (
            Rsa3072PrivKey {
                modulus: key_pair.modulus,
                exponent: key_pair.d,
            },
            Rsa3072PubKey {
                modulus: key_pair.modulus,
                exponent: key_pair.e,
            },
        )
    }
}

impl_struct_default! {
    Rsa3072PubKey; //388
    Rsa3072PrivKey; //768
    Rsa3072Key; //772
    Rsa3072Signature; //384
}

impl_struct_ContiguousMemory! {
    Rsa3072PubKey;
    Rsa3072PrivKey;
    Rsa3072Key;
    Rsa3072Signature;
}

impl_unsafe_marker_for! {BytewiseEquality,
Rsa3072PubKey Rsa3072PrivKey Rsa3072Key Rsa3072Signature}

impl_asref_array! {
    Rsa3072PubKey;
    Rsa3072PrivKey;
    Rsa3072Key;
    Rsa3072Signature;
}
impl_asmut_array! {
    Rsa3072PubKey;
    Rsa3072PrivKey;
    Rsa3072Key;
    Rsa3072Signature;
}
impl_from_array! {
    Rsa3072PubKey;
    Rsa3072PrivKey;
    Rsa3072Key;
    Rsa3072Signature;
}

impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Rsa2048Param {
        pub n: [u8; RSA2048_KEY_SIZE],
        pub d: [u8; RSA2048_PRI_EXP_SIZE],
        pub e: [u8; RSA2048_PUB_EXP_SIZE],
        pub p: [u8; RSA2048_KEY_SIZE / 2],
        pub q: [u8; RSA2048_KEY_SIZE / 2],
        pub dmp1: [u8; RSA2048_KEY_SIZE / 2],
        pub dmq1: [u8; RSA2048_KEY_SIZE / 2],
        pub iqmp: [u8; RSA2048_KEY_SIZE / 2],
    }
}

impl_struct_default! {
    Rsa2048Param; //1156
}

impl_struct_ContiguousMemory! {
    Rsa2048Param;
}

impl_unsafe_marker_for! {BytewiseEquality, Rsa2048Param}

impl_asref_array! {
    Rsa2048Param;
}
impl_asmut_array! {
    Rsa2048Param;
}
impl_from_array! {
    Rsa2048Param;
}

impl_copy_clone! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Rsa2048PubKey {
        pub modulus: [u8; RSA2048_KEY_SIZE],
        pub exponent: [u8; RSA2048_PUB_EXP_SIZE],
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Rsa2048PrivKey {
        pub modulus: [u8; RSA2048_KEY_SIZE],
        pub exponent: [u8; RSA2048_PRI_EXP_SIZE],
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Rsa2048Key {
        pub modulus: [u8; RSA2048_KEY_SIZE],
        pub d: [u8; RSA2048_PRI_EXP_SIZE],
        pub e: [u8; RSA2048_PUB_EXP_SIZE],
    }

    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Rsa2048Signature {
        pub signature: [u8; RSA2048_KEY_SIZE],
    }
}

impl From<(Rsa2048PrivKey, Rsa2048PubKey)> for Rsa2048Key {
    #[inline]
    fn from(key_pair: (Rsa2048PrivKey, Rsa2048PubKey)) -> Rsa2048Key {
        Rsa2048Key {
            modulus: key_pair.0.modulus,
            d: key_pair.0.exponent,
            e: key_pair.1.exponent,
        }
    }
}

impl From<Rsa2048Key> for (Rsa2048PrivKey, Rsa2048PubKey) {
    #[inline]
    fn from(key_pair: Rsa2048Key) -> (Rsa2048PrivKey, Rsa2048PubKey) {
        (
            Rsa2048PrivKey {
                modulus: key_pair.modulus,
                exponent: key_pair.d,
            },
            Rsa2048PubKey {
                modulus: key_pair.modulus,
                exponent: key_pair.e,
            },
        )
    }
}

impl_struct_default! {
    Rsa2048PubKey; //260
    Rsa2048PrivKey; //512
    Rsa2048Key; //516
    Rsa2048Signature; //256
}

impl_struct_ContiguousMemory! {
    Rsa2048PubKey;
    Rsa2048PrivKey;
    Rsa2048Key;
    Rsa2048Signature;
}

impl_unsafe_marker_for! {BytewiseEquality,
Rsa2048PubKey Rsa2048PrivKey Rsa2048Key Rsa2048Signature}

impl_asref_array! {
    Rsa2048PubKey;
    Rsa2048PrivKey;
    Rsa2048Key;
    Rsa2048Signature;
}
impl_asmut_array! {
    Rsa2048PubKey;
    Rsa2048PrivKey;
    Rsa2048Key;
    Rsa2048Signature;
}
impl_from_array! {
    Rsa2048PubKey;
    Rsa2048PrivKey;
    Rsa2048Key;
    Rsa2048Signature;
}

#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct AlignKey128bit {
    _pad: [u8; 16],
    pub key: Key128bit,
}

impl From<Key128bit> for AlignKey128bit {
    fn from(key: Key128bit) -> AlignKey128bit {
        AlignKey128bit { _pad: [0; 16], key }
    }
}

impl From<&Key128bit> for AlignKey128bit {
    fn from(key: &Key128bit) -> AlignKey128bit {
        AlignKey128bit {
            _pad: [0; 16],
            key: *key,
        }
    }
}

impl AsRef<[u8; KEY_128BIT_SIZE]> for AlignKey128bit {
    #[inline]
    fn as_ref(&self) -> &[u8; KEY_128BIT_SIZE] {
        &self.key
    }
}

impl AsMut<[u8; KEY_128BIT_SIZE]> for AlignKey128bit {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8; KEY_128BIT_SIZE] {
        &mut self.key
    }
}

impl PartialEq for AlignKey128bit {
    fn eq(&self, other: &AlignKey128bit) -> bool {
        self.key.eq(&other.key)
    }
}

impl Eq for AlignKey128bit {}

impl fmt::Debug for AlignKey128bit {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AlignKey128bit")
            .field("key", &self.key)
            .finish()
    }
}

#[repr(C, align(64))]
#[derive(Clone, Copy, Default)]
pub struct AlignKey256bit {
    _pad1: [u8; 8],
    pub key: Key256bit,
    _pad2: [u8; 24],
}

impl From<Key256bit> for AlignKey256bit {
    #[inline]
    fn from(key: Key256bit) -> AlignKey256bit {
        AlignKey256bit {
            _pad1: [0; 8],
            key,
            _pad2: [0; 24],
        }
    }
}

impl From<&Key256bit> for AlignKey256bit {
    #[inline]
    fn from(key: &Key256bit) -> AlignKey256bit {
        AlignKey256bit {
            _pad1: [0; 8],
            key: *key,
            _pad2: [0; 24],
        }
    }
}

impl AsRef<[u8; KEY_256BIT_SIZE]> for AlignKey256bit {
    #[inline]
    fn as_ref(&self) -> &[u8; KEY_256BIT_SIZE] {
        &self.key
    }
}

impl AsMut<[u8; KEY_256BIT_SIZE]> for AlignKey256bit {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8; KEY_256BIT_SIZE] {
        &mut self.key
    }
}

impl PartialEq for AlignKey256bit {
    #[inline]
    fn eq(&self, other: &AlignKey256bit) -> bool {
        self.key.eq(&other.key)
    }
}

impl Eq for AlignKey256bit {}

impl fmt::Debug for AlignKey256bit {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AlignKey256bit")
            .field("key", &self.key)
            .finish()
    }
}

#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct AlignMac128bit {
    _pad: [u8; 16],
    pub mac: Mac128bit,
}

impl From<Mac128bit> for AlignMac128bit {
    #[inline]
    fn from(mac: Mac128bit) -> AlignMac128bit {
        AlignMac128bit { _pad: [0; 16], mac }
    }
}

impl From<&Mac128bit> for AlignMac128bit {
    #[inline]
    fn from(mac: &Mac128bit) -> AlignMac128bit {
        AlignMac128bit {
            _pad: [0; 16],
            mac: *mac,
        }
    }
}

impl AsRef<[u8; MAC_128BIT_SIZE]> for AlignMac128bit {
    #[inline]
    fn as_ref(&self) -> &[u8; MAC_128BIT_SIZE] {
        &self.mac
    }
}

impl AsMut<[u8; MAC_128BIT_SIZE]> for AlignMac128bit {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8; MAC_128BIT_SIZE] {
        &mut self.mac
    }
}

impl PartialEq for AlignMac128bit {
    #[inline]
    fn eq(&self, other: &AlignMac128bit) -> bool {
        self.mac.eq(&other.mac)
    }
}

impl Eq for AlignMac128bit {}

impl fmt::Debug for AlignMac128bit {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AlignMac128bit")
            .field("mac", &self.mac)
            .finish()
    }
}

#[repr(C, align(64))]
#[derive(Clone, Copy, Default)]
pub struct AlignMac256bit {
    _pad1: [u8; 8],
    pub mac: Mac256bit,
    _pad2: [u8; 24],
}

impl From<Mac256bit> for AlignMac256bit {
    fn from(mac: Mac256bit) -> AlignMac256bit {
        AlignMac256bit {
            _pad1: [0; 8],
            mac,
            _pad2: [0; 24],
        }
    }
}

impl From<&Mac256bit> for AlignMac256bit {
    #[inline]
    fn from(mac: &Mac256bit) -> AlignMac256bit {
        AlignMac256bit {
            _pad1: [0; 8],
            mac: *mac,
            _pad2: [0; 24],
        }
    }
}

impl AsRef<[u8; MAC_256BIT_SIZE]> for AlignMac256bit {
    #[inline]
    fn as_ref(&self) -> &[u8; MAC_256BIT_SIZE] {
        &self.mac
    }
}

impl AsMut<[u8; MAC_256BIT_SIZE]> for AlignMac256bit {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8; MAC_256BIT_SIZE] {
        &mut self.mac
    }
}

impl PartialEq for AlignMac256bit {
    #[inline]
    fn eq(&self, other: &AlignMac256bit) -> bool {
        self.mac.eq(&other.mac)
    }
}

impl Eq for AlignMac256bit {}

impl fmt::Debug for AlignMac256bit {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AlignMac256bit")
            .field("mac", &self.mac)
            .finish()
    }
}

#[repr(C, align(64))]
#[derive(Copy, Clone, Default)]
pub struct AlignEc256SharedKey {
    _pad1: [u8; 8],
    pub key: Ec256SharedKey,
    _pad2: [u8; 24],
}

impl From<Ec256SharedKey> for AlignEc256SharedKey {
    #[inline]
    fn from(key: Ec256SharedKey) -> AlignEc256SharedKey {
        AlignEc256SharedKey {
            _pad1: [0; 8],
            key,
            _pad2: [0; 24],
        }
    }
}

impl From<&Ec256SharedKey> for AlignEc256SharedKey {
    #[inline]
    fn from(key: &Ec256SharedKey) -> AlignEc256SharedKey {
        AlignEc256SharedKey {
            _pad1: [0; 8],
            key: *key,
            _pad2: [0; 24],
        }
    }
}

impl AsRef<[u8; ECP256_KEY_SIZE]> for AlignEc256SharedKey {
    #[inline]
    fn as_ref(&self) -> &[u8; ECP256_KEY_SIZE] {
        self.key.as_ref()
    }
}

impl AsMut<[u8; ECP256_KEY_SIZE]> for AlignEc256SharedKey {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8; ECP256_KEY_SIZE] {
        self.key.as_mut()
    }
}

impl From<[u8; ECP256_KEY_SIZE]> for AlignEc256SharedKey {
    #[inline]
    fn from(key: [u8; ECP256_KEY_SIZE]) -> AlignEc256SharedKey {
        AlignEc256SharedKey::from(Ec256SharedKey::from(key))
    }
}

impl From<&[u8; ECP256_KEY_SIZE]> for AlignEc256SharedKey {
    #[inline]
    fn from(key: &[u8; ECP256_KEY_SIZE]) -> AlignEc256SharedKey {
        AlignEc256SharedKey::from(Ec256SharedKey::from(key))
    }
}

impl PartialEq for AlignEc256SharedKey {
    #[inline]
    fn eq(&self, other: &AlignEc256SharedKey) -> bool {
        self.key.eq(&other.key)
    }
}

impl Eq for AlignEc256SharedKey {}

impl fmt::Debug for AlignEc256SharedKey {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AlignEc256SharedKey")
            .field("key", &self.key)
            .finish()
    }
}

#[repr(C, align(64))]
#[derive(Copy, Clone, Default)]
pub struct AlignEc256PrivateKey {
    _pad1: [u8; 8],
    pub key: Ec256PrivateKey,
    _pad2: [u8; 24],
}

impl From<Ec256PrivateKey> for AlignEc256PrivateKey {
    #[inline]
    fn from(key: Ec256PrivateKey) -> AlignEc256PrivateKey {
        AlignEc256PrivateKey {
            _pad1: [0; 8],
            key,
            _pad2: [0; 24],
        }
    }
}

impl From<&Ec256PrivateKey> for AlignEc256PrivateKey {
    #[inline]
    fn from(key: &Ec256PrivateKey) -> AlignEc256PrivateKey {
        AlignEc256PrivateKey {
            _pad1: [0; 8],
            key: *key,
            _pad2: [0; 24],
        }
    }
}

impl AsRef<[u8; ECP256_KEY_SIZE]> for AlignEc256PrivateKey {
    #[inline]
    fn as_ref(&self) -> &[u8; ECP256_KEY_SIZE] {
        self.key.as_ref()
    }
}

impl AsMut<[u8; ECP256_KEY_SIZE]> for AlignEc256PrivateKey {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8; ECP256_KEY_SIZE] {
        self.key.as_mut()
    }
}

impl From<[u8; ECP256_KEY_SIZE]> for AlignEc256PrivateKey {
    #[inline]
    fn from(key: [u8; ECP256_KEY_SIZE]) -> AlignEc256PrivateKey {
        AlignEc256PrivateKey::from(Ec256PrivateKey::from(key))
    }
}

impl From<&[u8; ECP256_KEY_SIZE]> for AlignEc256PrivateKey {
    #[inline]
    fn from(key: &[u8; ECP256_KEY_SIZE]) -> AlignEc256PrivateKey {
        AlignEc256PrivateKey::from(Ec256PrivateKey::from(key))
    }
}

impl PartialEq for AlignEc256PrivateKey {
    #[inline]
    fn eq(&self, other: &AlignEc256PrivateKey) -> bool {
        self.key.eq(&other.key)
    }
}

impl Eq for AlignEc256PrivateKey {}

impl fmt::Debug for AlignEc256PrivateKey {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AlignEc256PrivateKey")
            .field("key", &self.key)
            .finish()
    }
}

impl_struct_ContiguousMemory! {
    AlignKey128bit;
    AlignKey256bit;
    AlignMac128bit;
    AlignMac256bit;
    AlignEc256SharedKey;
    AlignEc256PrivateKey;
}

impl_unsafe_marker_for! {
    BytewiseEquality,
    AlignKey128bit AlignKey256bit AlignMac128bit AlignMac256bit AlignEc256SharedKey AlignEc256PrivateKey
}
