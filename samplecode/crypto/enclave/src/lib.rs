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

extern crate sgx_crypto;
extern crate sgx_crypto_sys;
extern crate sgx_types;

use std::{ptr, slice};

use sgx_crypto::*;
use sgx_crypto_sys::*;
use sgx_types::error::*;
use sgx_types::memeq::ConstTimeEq;
use sgx_types::types::{Key128bit, Mac128bit, Sha256Hash, MAC_128BIT_SIZE};

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
#[no_mangle]
pub extern "C" fn calc_sha256(
    input_str: *const u8,
    some_len: usize,
    hash: &mut [u8; 32],
) -> SgxStatus {
    println!("calc_sha256 invoked!");

    // First, build a slice for input_str
    let input_slice = unsafe { slice::from_raw_parts(input_str, some_len) };

    // slice::from_raw_parts does not guarantee the length, we need a check
    if input_slice.len() != some_len {
        return SgxStatus::InvalidParameter;
    }

    println!(
        "Input string len = {}, input len = {}",
        input_slice.len(),
        some_len
    );

    // Second, convert the vector to a slice and calculate its SHA256
    let mut sha256hash = Sha256Hash::default();
    let result = unsafe {
        sgx_sha256_msg(
            (input_slice.as_ptr() as *const u8).cast(),
            some_len as u32,
            &mut sha256hash as *mut Sha256Hash,
        )
    };

    // Third, copy back the result
    match result {
        SgxStatus::Success => *hash = sha256hash,
        x => return x,
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
#[no_mangle]
pub extern "C" fn aes_gcm_128_encrypt(
    key: &[u8; 16],
    plaintext: *const u8,
    text_len: usize,
    iv: &[u8; 12],
    ciphertext: *mut u8,
    mac: &mut [u8; 16],
) -> SgxStatus {
    println!("aes_gcm_128_encrypt invoked!");

    // First, we need slices for input
    let plaintext_slice = unsafe { slice::from_raw_parts(plaintext, text_len) };

    // Here we need to initiate the ciphertext buffer, though nothing in it.
    // Thus show the length of ciphertext buffer is equal to plaintext buffer.
    // If not, the length of ciphertext_vec will be 0, which leads to argument
    // illegal.
    let mut ciphertext_vec: Vec<u8> = vec![0; text_len];

    // Second, for data with known length, we use array with fixed length.
    // Here we cannot use slice::from_raw_parts because it provides &[u8]
    // instead of &[u8,16].
    let aad_array: [u8; 0] = [0; 0];
    let mut mac_array: [u8; MAC_128BIT_SIZE] = [0; MAC_128BIT_SIZE];

    // Always check the length after slice::from_raw_parts
    if plaintext_slice.len() != text_len {
        return SgxStatus::InvalidParameter;
    }

    let ciphertext_slice = &mut ciphertext_vec[..];
    println!(
        "aes_gcm_128_encrypt parameter prepared! {}, {}",
        plaintext_slice.len(),
        ciphertext_slice.len()
    );

    // After everything has been set, call API
    //let result = rsgx_rijndael128GCM_encrypt(key,
    //                                         &plaintext_slice,
    //                                         iv,
    //                                         &aad_array,
    //                                         ciphertext_slice,
    //                                         &mut mac_array);
    //println!("rsgx calling returned!");
    let result = unsafe {
        sgx_rijndael128GCM_encrypt(
            key as *const _ as *const Key128bit,
            plaintext_slice.as_ptr(),
            plaintext_slice.len() as u32,
            ciphertext_slice.as_mut_ptr(),
            iv.as_ptr(),
            iv.as_ref().len() as u32,
            ptr::null(),
            0,
            &mut mac_array as *mut u8 as *mut Mac128bit,
        )
    };

    // Match the result and copy result back to normal world.
    match result {
        SgxStatus::Success => {
            unsafe {
                ptr::copy_nonoverlapping(ciphertext_slice.as_ptr(), ciphertext, text_len);
            }
            *mac = mac_array;
        }
        e => {
            return e;
        }
    }

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
#[no_mangle]
pub extern "C" fn aes_gcm_128_decrypt(
    key: &[u8; 16],
    ciphertext: *const u8,
    text_len: usize,
    iv: &[u8; 12],
    mac: &[u8; 16],
    plaintext: *mut u8,
) -> SgxStatus {
    println!("aes_gcm_128_decrypt invoked!");

    // First, for data with unknown length, we use vector as builder.
    let ciphertext_slice = unsafe { slice::from_raw_parts(ciphertext, text_len) };
    let mut plaintext_vec: Vec<u8> = vec![0; text_len];

    // Second, for data with known length, we use array with fixed length.
    let aad_array: [u8; 0] = [0; 0];

    if ciphertext_slice.len() != text_len {
        return SgxStatus::InvalidParameter;
    }

    let plaintext_slice = &mut plaintext_vec[..];
    println!(
        "aes_gcm_128_decrypt parameter prepared! {}, {}",
        ciphertext_slice.len(),
        plaintext_slice.len()
    );

    // After everything has been set, call API
    let result = unsafe {
        sgx_rijndael128GCM_decrypt(
            key.as_ptr() as *const Key128bit,
            ciphertext,
            text_len as u32,
            plaintext,
            iv.as_ptr(),
            iv.len() as u32,
            aad_array.as_ptr(),
            0,
            mac.as_ptr() as *const Mac128bit,
        )
    };

    println!("rsgx calling returned!");

    // Match the result and copy result back to normal world.
    match result {
        SgxStatus::Success => unsafe {
            ptr::copy_nonoverlapping(plaintext_slice.as_ptr(), plaintext, text_len);
        },
        x => {
            return x;
        }
    }

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
#[no_mangle]
pub extern "C" fn aes_cmac(
    text: *const u8,
    text_len: usize,
    key: &[u8; 16],
    cmac: &mut [u8; 16],
) -> SgxStatus {
    let text_slice = unsafe { slice::from_raw_parts(text, text_len) };

    if text_slice.len() != text_len {
        return SgxStatus::InvalidParameter;
    }

    let result = unsafe {
        sgx_rijndael128_cmac_msg(
            key.as_ptr() as *const Key128bit,
            text,
            text_len as u32,
            cmac.as_mut_ptr() as *mut Mac128bit,
        )
    };

    match result {
        SgxStatus::Success => {}
        x => return x,
    }

    SgxStatus::Success
}

#[no_mangle]
pub extern "C" fn rsa_key(text: *const u8, text_len: usize) -> SgxStatus {
    //let text_slice = unsafe { slice::from_raw_parts(text, text_len) };

    //if text_slice.len() != text_len {
    //    return SgxStatus::SGX_ERROR_INVALID_PARAMETER;
    //}

    //let mod_size: i32 = 256;
    //let exp_size: i32 = 4;
    //let mut n: Vec<u8> = vec![0_u8; mod_size as usize];
    //let mut d: Vec<u8> = vec![0_u8; mod_size as usize];
    //let mut e: Vec<u8> = vec![1, 0, 1, 0];
    //let mut p: Vec<u8> = vec![0_u8; mod_size as usize / 2];
    //let mut q: Vec<u8> = vec![0_u8; mod_size as usize / 2];
    //let mut dmp1: Vec<u8> = vec![0_u8; mod_size as usize / 2];
    //let mut dmq1: Vec<u8> = vec![0_u8; mod_size as usize / 2];
    //let mut iqmp: Vec<u8> = vec![0_u8; mod_size as usize / 2];

    //let result = sgx_create_rsa_key_pair(mod_size,
    //                                      exp_size,
    //                                      n.as_mut_slice(),
    //                                      d.as_mut_slice(),
    //                                      e.as_mut_slice(),
    //                                      p.as_mut_slice(),
    //                                      q.as_mut_slice(),
    //                                      dmp1.as_mut_slice(),
    //                                      dmq1.as_mut_slice(),
    //                                      iqmp.as_mut_slice());

    //match result {
    //    Err(x) => {
    //        return x;
    //    },
    //    Ok(()) => {},
    //}

    //let privkey = SgxRsaPrivKey::new();
    //let pubkey = SgxRsaPubKey::new();

    //let result = pubkey.create(mod_size,
    //                           exp_size,
    //                           n.as_slice(),
    //                           e.as_slice());
    //match result {
    //    Err(x) => return x,
    //    Ok(()) => {},
    //};

    //let result = privkey.create(mod_size,
    //                            exp_size,
    //                            e.as_slice(),
    //                            p.as_slice(),
    //                            q.as_slice(),
    //                            dmp1.as_slice(),
    //                            dmq1.as_slice(),
    //                            iqmp.as_slice());
    //match result {
    //    Err(x) => return x,
    //    Ok(()) => {},
    //};

    //let mut ciphertext: Vec<u8> = vec![0_u8; 256];
    //let mut chipertext_len: usize = ciphertext.len();
    //let ret = pubkey.encrypt_sha256(ciphertext.as_mut_slice(),
    //                                &mut chipertext_len,
    //                                text_slice);
    //match ret {
    //    Err(x) => {
    //        return x;
    //    },
    //    Ok(()) => {
    //        println!("rsa chipertext_len: {:?}", chipertext_len);
    //    },
    //};

    //let mut plaintext: Vec<u8> = vec![0_u8; 256];
    //let mut plaintext_len: usize = plaintext.len();
    //let ret = privkey.decrypt_sha256(plaintext.as_mut_slice(),
    //                                 &mut plaintext_len,
    //                                 ciphertext.as_slice());
    //match ret {
    //    Err(x) => {
    //        return x;
    //    },
    //    Ok(()) => {
    //        println!("rsa plaintext_len: {:?}", plaintext_len);
    //    },
    //};

    //if plaintext[..plaintext_len].consttime_memeq(text_slice) == false {
    //    return SgxStatus::Unexpected;
    //}

    SgxStatus::Success
}
