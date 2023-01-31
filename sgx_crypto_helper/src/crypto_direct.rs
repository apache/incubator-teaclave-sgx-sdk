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

/// cf /teaclave-sgx-sdk/sgx_tcrypto/src/crypto.rs

///
/// Cryptographic Functions
///
use core::cell::{Cell, RefCell};
use core::mem;
use core::ops::{DerefMut, Drop};
use core::ptr;
use num_bigint::BigUint;
use rsa::{Oaep, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use sgx_types::marker::ContiguousMemory;
use sgx_types::*;

pub struct SgxRsaPubKey {
    key: RsaPublicKey,
}

impl SgxRsaPubKey {
    /// Normally it calls: "sgx_create_rsa_pub1_key"
    /// sgx_create_rsa_pub1_key generates a public key of the desired RSA
    /// cryptographic with the input RSA key components.
    /// Syntax
    /// sgx_status_t sgx_create_rsa_priv2_key(
    /// int mod_size,
    /// int exp_size,
    /// const unsigned char *le_n,
    /// const unsigned char *le_e,
    /// void **new_pub_key1
    /// );
    /// Parameters
    /// mod_size [in]
    /// Size in bytes of the RSA key modulus.
    /// exp_size [in]
    /// Size in bytes of the RSA public exponent.
    /// le_n [in]
    /// Pointer to the RSA modulus buffer.
    /// le_e [in]
    /// Pointer to the RSA public exponent buffer.
    /// new_pub_key1 [out]
    /// Pointer to the generated RSA public key.
    /// Return value
    /// SGX_SUCCESS
    /// The RSA public key is successfully generated.
    /// SGX_ERROR_INVALID_PARAMETER
    /// Some of the pointers is NULL, or the input size is less than 0.
    /// SGX_ERROR_UNEXPECTED
    /// Unexpected error occurs during generating the RSA public key.
    pub fn new(_mod_size: i32, _exp_size: i32, n: &[u8], e: &[u8]) -> SgxRsaPubKey {
        SgxRsaPubKey {
            key: RsaPublicKey::new(BigUint::from_bytes_be(n), BigUint::from_bytes_be(e))
                .map_err(|err| sgx_status_t::SGX_ERROR_INVALID_PARAMETER)
                .unwrap(),
        }
    }

    /// Normally it calls: "sgx_rsa_pub_encrypt_sha256"
    /// sgx_rsa_pub_encrypt_sha256 performs RSA-OAEP encryption oper-
    /// ation, with SHA-256 algorithm
    /// sgx_status_t sgx_rsa_pub_encrypt_sha256(
    ///     void* rsa_key,
    ///     unsigned char* pout_data,
    ///     size_t* pout_len,
    ///     const unsigned char* pin_data,
    ///     const size_t pin_len
    /// );
    /// rsa_key [in]
    /// Pointer to the RSA public key.
    /// pout_data [out]
    /// Pointer to the output cipher text buffer.
    /// pout_len [out]
    /// Length of the output cipher text buffer.
    /// pin_data [in]
    /// Pointer to the input data buffer.
    /// pin_len [in]
    /// Length of the input data buffer.
    /// Return value
    /// SGX_SUCCESS
    /// All the outputs are generated successfully.
    /// SGX_ERROR_INVALID_PARAMETER
    /// Some of the pointers is NULL, or the input data size is 0.
    /// SGX_ERROR_UNEXPECTED
    /// Unexpected error occurs during performing encryption operation.
    pub fn encrypt_sha256(
        &self,
        out_data: &mut [u8],
        out_len: &mut usize,
        in_data: &[u8],
    ) -> SgxError {
        let mut rng = rand::thread_rng();
        let padding = Oaep::new::<sha2::Sha256>();
        let enc_data = self
            .key
            .encrypt(&mut rng, padding, in_data)
            .map_err(|err| sgx_status_t::SGX_ERROR_UNEXPECTED)?;

        *out_len = enc_data.len();

        Ok(())
    }
}

// impl Default for SgxRsaPubKey {
//     fn default() -> Self {
//         Self::new()
//     }
// }

pub struct SgxRsaPrivKey {
    key: RefCell<sgx_rsa_key_t>,
    mod_size: Cell<i32>,
    exp_size: Cell<i32>,
    createflag: Cell<bool>,
}
