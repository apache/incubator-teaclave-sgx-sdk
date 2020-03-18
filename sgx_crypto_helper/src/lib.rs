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

#[cfg(any(feature = "mesalock_sgx", target_env = "sgx"))]
extern crate serde_sgx;
#[cfg(not(any(feature = "mesalock_sgx", target_env = "sgx")))]
extern crate serde;

#[cfg(any(feature = "mesalock_sgx", target_env = "sgx"))]
extern crate serde_derive_sgx as serde_derive;
#[cfg(not(any(feature = "mesalock_sgx", target_env = "sgx")))]
extern crate serde_derive;

#[cfg(any(feature = "mesalock_sgx", target_env = "sgx"))]
#[macro_use]
extern crate serde_big_array_sgx as serde_big_array;
#[cfg(not(any(feature = "mesalock_sgx", target_env = "sgx")))]
#[macro_use]
extern crate serde_big_array;

pub mod rsa2048;
pub mod rsa3072;
