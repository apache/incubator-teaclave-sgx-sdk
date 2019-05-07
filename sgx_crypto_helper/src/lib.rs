// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! # Cryptography Library Helper
//!
//! This crate provides helper functions to simplify key distribution and
//! encryption/decryption. It utilizes sgx_tcrypto and sgx_ucrypto to provide
//! a uniform interface to both enclave and untrusted app. It provides key
//! serialization/deserialization by serde.
//!
//! The Intel(R) Software Guard Extensions SDK includes a trusted cryptography
//! library named sgx_tcrypto. It includes the cryptographic functions used by
//! other trusted libraries included in the SDK.
//!

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]

#![cfg_attr(all(feature = "mesalock_sgx", not(target_env = "sgx")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#![cfg_attr(test, feature(test))]

#[cfg(all(feature = "mesalock_sgx", not(target_env = "sgx")))]
#[macro_use]
extern crate sgx_tstd as std;

extern crate sgx_types;
#[cfg(any(feature = "mesalock_sgx", target_env = "sgx"))]
extern crate sgx_tcrypto as crypto;
#[cfg(not(any(feature = "mesalock_sgx", target_env = "sgx")))]
extern crate sgx_ucrypto as crypto;

use std::prelude::v1::*;
use sgx_types::SgxResult;
use crypto::SgxRsaPrivKey;
use crypto::SgxRsaPubKey;

/// A trait to express the ability to create a RSA keypair with default e
/// (65537) or customized e, and to_privkey/to_pubkey, encryption/decryption API.
pub trait RsaKeyPair {
    /// Create a new RSA keypair with default e = 65537.
    fn new() -> SgxResult<Self> where Self: std::marker::Sized;
    /// Create a new RSA keypair with customized e
    fn new_with_e(e: u32) -> SgxResult<Self> where Self: std::marker::Sized;
    /// Get a private key instance typed `SgxRsaPrivKey` which is defined in sgx_tcrypto/sgx_ucrypto.
    fn to_privkey(self) -> SgxResult<SgxRsaPrivKey>;
    /// get a public key instance typed `SgxPubPrivKey` which is defined in sgx_tcrypto/sgx_ucrypto.
    fn to_pubkey(self) -> SgxResult<SgxRsaPubKey>;
    /// Encrypt a u8 slice to a Vec<u8>. Returns the length of ciphertext if OK.
    fn encrypt_buffer(self, plaintext: &[u8], ciphertext: &mut Vec<u8>) -> SgxResult<usize>;
    /// Decrypt a u8 slice to a Vec<u8>. Returns the length of plaintext if OK.
    fn decrypt_buffer(self, ciphertext: &[u8], plaintext: &mut Vec<u8>) -> SgxResult<usize>;
}

extern crate serde;
extern crate serde_derive;
extern crate itertools;
#[macro_use]
extern crate serde_big_array;

pub mod rsa2048;
pub mod rsa3072;
