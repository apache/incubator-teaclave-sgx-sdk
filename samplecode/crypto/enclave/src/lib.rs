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

#![cfg_attr(not(target_vendor = "teaclave"), no_std)]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]

#[cfg(not(target_vendor = "teaclave"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_types;

use sgx_crypto::aes::gcm::{Aad, AesGcm, Nonce};
use sgx_crypto::mac::AesCMac;
use sgx_crypto::rsa::Rsa2048KeyPair;
use sgx_crypto::sha::Sha256;
use sgx_types::error::SgxStatus;
use sgx_types::memeq::ConstTimeEq;
use sgx_types::types::{AESGCM_IV_SIZE, KEY_128BIT_SIZE, MAC_128BIT_SIZE, SHA256_HASH_SIZE};
use std::vec::Vec;
use std::{ptr, slice};

/// A Ecall function takes a string and output its SHA256 digest.
///
/// # Parameters
///
/// **input_str**
///
/// A raw pointer to the string to be calculated.
///
/// **some_len**
///
/// An unsigned int indicates the length of input string
///
/// **hash**
///
/// A const reference to [u8;32] array, which is the destination buffer which contains the SHA256 digest, caller allocated.
///
/// # Return value
///
/// **Success** on success. The SHA256 digest is stored in the destination buffer.
///
/// # Requirements
///
/// Caller allocates the input buffer and output buffer.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Indicates the parameter is invalid
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sha256(
    input_str: *const u8,
    some_len: usize,
    hash: &mut [u8; SHA256_HASH_SIZE],
) -> SgxStatus {
    println!("calc_sha256 invoked!");

    // First, we need slices for input
    let input_slice = slice::from_raw_parts(input_str, some_len);

    // Second, calculate its SHA256
    let result = Sha256::digest(input_slice);

    // Third, copy back the result
    match result {
        Ok(sha256hash) => *hash = *sha256hash.as_ref(),
        Err(e) => return e,
    }

    SgxStatus::Success
}

/// An AES-GCM-128 encrypt function sample.
///
/// # Parameters
///
/// **key**
///
/// Key used in AES encryption, typed as &[u8;16].
///
/// **plaintext**
///
/// Plain text to be encrypted.
///
/// **text_len**
///
/// Length of plain text, unsigned int.
///
/// **iv**
///
/// Initialization vector of AES encryption, typed as &[u8;12].
///
/// **ciphertext**
///
/// A pointer to destination ciphertext buffer.
///
/// **mac**
///
/// A pointer to destination mac buffer, typed as &mut [u8;16].
///
/// # Return value
///
/// **Success** on success
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER** Indicates the parameter is invalid.
///
/// **SGX_ERROR_UNEXPECTED** Indicates that encryption failed.
///
/// # Requirements
///
/// The caller should allocate the ciphertext buffer. This buffer should be
/// at least same length as plaintext buffer. The caller should allocate the
/// mac buffer, at least 16 bytes.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn aes_gcm_128_encrypt(
    key: &[u8; KEY_128BIT_SIZE],
    plaintext: *const u8,
    text_len: usize,
    iv: &[u8; AESGCM_IV_SIZE],
    ciphertext: *mut u8,
    mac: &mut [u8; MAC_128BIT_SIZE],
) -> SgxStatus {
    println!("aes_gcm_128_encrypt invoked!");

    // First, we need slices for input
    let plaintext_slice = slice::from_raw_parts(plaintext, text_len);

    // Here we need to initiate the ciphertext buffer, though nothing in it.
    // Thus show the length of ciphertext buffer is equal to plaintext buffer.
    // If not, the length of ciphertext_vec will be 0, which leads to argument
    // illegal.
    let mut ciphertext_vec = vec![0_u8; text_len];
    let ciphertext_slice = &mut ciphertext_vec[..];
    println!(
        "aes_gcm_128_encrypt parameter prepared! {}, {}",
        plaintext_slice.len(),
        ciphertext_slice.len()
    );

    let aad = Aad::empty();
    let iv = Nonce::from(iv);
    let mut aes_gcm = match AesGcm::new(key, iv, aad) {
        Ok(aes_gcm) => aes_gcm,
        Err(e) => {
            return e;
        }
    };

    match aes_gcm.encrypt(plaintext_slice, ciphertext_slice) {
        Ok(mac_array) => {
            ptr::copy_nonoverlapping(ciphertext_slice.as_ptr(), ciphertext, text_len);
            *mac = mac_array;
        }
        Err(e) => {
            return e;
        }
    };

    SgxStatus::Success
}

/// An AES-GCM-128 decrypt function sample.
///
/// # Parameters
///
/// **key**
///
/// Key used in AES encryption, typed as &[u8;16].
///
/// **ciphertext**
///
/// Cipher text to be decrypted.
///
/// **text_len**
///
/// Length of cipher text.
///
/// **iv**
///
/// Initialization vector of AES encryption, typed as &[u8;12].
///
/// **mac**
///
/// A pointer to source mac buffer, typed as &[u8;16].
///
/// **plaintext**
///
/// A pointer to destination plaintext buffer.
///
/// # Return value
///
/// **Success** on success
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER** Indicates the parameter is invalid.
///
/// **SGX_ERROR_UNEXPECTED** means that decryption failed.
///
/// # Requirements
//
/// The caller should allocate the plaintext buffer. This buffer should be
/// at least same length as ciphertext buffer.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn aes_gcm_128_decrypt(
    key: &[u8; KEY_128BIT_SIZE],
    ciphertext: *const u8,
    text_len: usize,
    iv: &[u8; AESGCM_IV_SIZE],
    mac: &[u8; MAC_128BIT_SIZE],
    plaintext: *mut u8,
) -> SgxStatus {
    println!("aes_gcm_128_decrypt invoked!");

    // First, we need slices for input
    let ciphertext_slice = slice::from_raw_parts(ciphertext, text_len);

    // Second, for data with unknown length, we use vector as builder.
    let mut plaintext_vec: Vec<u8> = vec![0; text_len];
    let plaintext_slice = &mut plaintext_vec[..];

    println!(
        "aes_gcm_128_decrypt parameter prepared! {}, {}",
        ciphertext_slice.len(),
        plaintext_slice.len()
    );

    let aad = Aad::empty();
    let iv = Nonce::from(iv);
    let mut aes_gcm = match AesGcm::new(key, iv, aad) {
        Ok(aes_gcm) => aes_gcm,
        Err(e) => {
            return e;
        }
    };

    match aes_gcm.decrypt(ciphertext_slice, plaintext_slice, mac) {
        Ok(_) => ptr::copy_nonoverlapping(plaintext_slice.as_ptr(), plaintext, text_len),
        Err(e) => {
            return e;
        }
    };

    SgxStatus::Success
}

/// A sample aes-cmac function.
///
/// # Parameters
///
/// **text**
///
/// The text message to be calculated.
///
/// **text_len**
///
/// An unsigned int indicate the length of input text message.
///
/// **key**
///
/// The key used in AES-CMAC, 16 bytes sized.
///
/// **cmac**
///
/// The output buffer, at least 16 bytes available.
///
/// # Return value
///
/// **Success** on success.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER** indicates invalid input parameters
///
/// # Requirement
///
/// The caller should allocate the output cmac buffer.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn aes_cmac(
    text: *const u8,
    text_len: usize,
    key: &[u8; KEY_128BIT_SIZE],
    cmac: &mut [u8; MAC_128BIT_SIZE],
) -> SgxStatus {
    let text_slice = slice::from_raw_parts(text, text_len);

    match AesCMac::cmac(key, text_slice) {
        Ok(mac_array) => {
            *cmac = mac_array;
            SgxStatus::Success
        }
        Err(e) => e,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn rsa2048(text: *const u8, text_len: usize) -> SgxStatus {
    println!("rsa_key invoked!");

    let text_slice = slice::from_raw_parts(text, text_len);

    let key_pair = match Rsa2048KeyPair::create() {
        Ok(key_pair) => key_pair,
        Err(e) => {
            return e;
        }
    };

    let ciphertext = match key_pair.public_key().encrypt(text_slice) {
        Ok(ciphertext) => {
            println!("rsa chipertext len: {:?}", ciphertext.len());
            ciphertext
        }
        Err(e) => {
            return e;
        }
    };

    let plaintext = match key_pair.private_key().decrypt(&ciphertext) {
        Ok(plaintext) => {
            println!("rsa plaintext len: {:?}", plaintext.len());
            plaintext
        }
        Err(e) => {
            return e;
        }
    };

    if !plaintext[..].ct_eq(text_slice) {
        return SgxStatus::Unexpected;
    }

    SgxStatus::Success
}
