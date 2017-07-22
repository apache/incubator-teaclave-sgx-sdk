#![crate_name = "Cryptosampleenclave"]
#![crate_type = "staticlib"]

#![no_std]
#![feature(collections)]

#[macro_use]
extern crate collections;

extern crate sgx_types;
extern crate sgx_trts;
extern crate sgx_tcrypto;

use sgx_types::*;
use sgx_tcrypto::*;
use collections::vec::Vec;
use collections::slice;
use core::ptr;

/// The Ocall declared in Enclave.edl and implemented in app.c
///
/// # Parameters
///
/// **str**
///
/// A pointer to the string to be printed
///
/// **len**
///
/// An unsigned int indicates the length of str
///
/// # Return value
///
/// None
extern "C" {
    fn ocall_print_string(str: *const c_uchar, len: size_t);
}

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
/// **SGX_SUCCESS** on success. The SHA256 digest is stored in the destination buffer.
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
pub extern "C" fn calc_sha256(input_str: *const u8,
                              some_len: u32,
                              hash: &mut [u8;32]) -> sgx_status_t {
    let rust_raw_string = "calc_sha256 invoked!";

    unsafe {
        ocall_print_string(rust_raw_string.as_ptr() as *const c_uchar,
                           rust_raw_string.len() as size_t);
    }

    // First, build a slice for input_str
    let input_slice;

    unsafe {
        input_slice = slice::from_raw_parts(input_str, some_len as usize);
    }

    // slice::from_raw_parts does not guarantee the length, we need a check
    if input_slice.len() != some_len as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let debug_str = format!("Input string len = {}, input len = {}",
                            input_slice.len(),
                            some_len);

    unsafe {
        ocall_print_string(debug_str.as_ptr() as *const c_uchar,
                           debug_str.len() as size_t);
    }

    // Second, convert the vector to a slice and calculate its SHA256
    let result = rsgx_sha256_slice(&input_slice);

    // Third, copy back the result
    match result {
        Ok(output_hash) => *hash = output_hash,
        Err(x) => return x
    }

    sgx_status_t::SGX_SUCCESS
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
/// **SGX_SUCCESS** on success
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
pub extern "C" fn aes_gcm_128_encrypt(key: &[u8;16],
                                      plaintext: *const u8,
                                      text_len: u32,
                                      iv: &[u8;12],
                                      ciphertext: *mut u8,
                                      mac: &mut [u8;16]) -> sgx_status_t {

    let rust_raw_string = "aes_gcm_128_encrypt invoked!";

    unsafe {
        ocall_print_string(rust_raw_string.as_ptr() as *const c_uchar,
                           rust_raw_string.len() as size_t);
    }

    // First, we need slices for input
    let plaintext_slice;

    // Here we need to initiate the ciphertext buffer, though nothing in it.
    // Thus show the length of ciphertext buffer is equal to plaintext buffer.
    // If not, the length of ciphertext_vec will be 0, which leads to argument
    // illegal.
    let mut ciphertext_vec: Vec<u8> = vec![0; text_len as usize];

    // Second, for data with known length, we use array with fixed length.
    // Here we cannot use slice::from_raw_parts because it provides &[u8]
    // instead of &[u8,16].
    let aad_array: [u8; 0] = [0; 0];
    let mut mac_array: [u8; SGX_AESGCM_MAC_SIZE] = [0; SGX_AESGCM_MAC_SIZE];

    unsafe {
        plaintext_slice = slice::from_raw_parts(plaintext, text_len as usize);
    }

    // Always check the length after slice::from_raw_parts
    if plaintext_slice.len() != text_len as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let mut ciphertext_slice = &mut ciphertext_vec[..];
    let rust_raw_string = format!("aes_gcm_128_encrypt parameter prepared! {}, {}",
                                  plaintext_slice.len(),
                                  ciphertext_slice.len());

    unsafe {
        ocall_print_string(rust_raw_string.as_ptr() as *const c_uchar,
                           rust_raw_string.len() as size_t);
    }

    // After everything has been set, call API
    let result = rsgx_rijndael128GCM_encrypt(key,
                                             &plaintext_slice,
                                             iv,
                                             &aad_array,
                                             ciphertext_slice,
                                             &mut mac_array);

    let rust_raw_string = "rsgx calling returned!";

    unsafe {
        ocall_print_string(rust_raw_string.as_ptr() as *const c_uchar,
                           rust_raw_string.len() as size_t);
    }

    // Match the result and copy result back to normal world.
    match result {
        Err(x) => {
            return x;
        }
        Ok(()) => {
            unsafe{
                ptr::copy_nonoverlapping(ciphertext_slice.as_ptr(),
                                         ciphertext,
                                         text_len as usize);
            }
            *mac = mac_array;
        }
    }

    sgx_status_t::SGX_SUCCESS
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
/// Cipher text to be encrypted.
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
/// **SGX_SUCCESS** on success
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
pub extern "C" fn aes_gcm_128_decrypt(key: &[u8;16],
                                      ciphertext: *const u8,
                                      text_len: u32,
                                      iv: &[u8;12],
                                      mac: &[u8;16],
                                      plaintext: *mut u8) -> sgx_status_t {

    let rust_raw_string = "aes_gcm_128_decrypt invoked!";

    unsafe {
        ocall_print_string(rust_raw_string.as_ptr() as *const c_uchar,
                           rust_raw_string.len() as size_t);
    }

    // First, for data with unknown length, we use vector as builder.
    let ciphertext_slice;
    let mut plaintext_vec: Vec<u8> = vec![0; text_len as usize];

    // Second, for data with known length, we use array with fixed length.
    let aad_array: [u8; 0] = [0; 0];

    unsafe {
        ciphertext_slice = slice::from_raw_parts(ciphertext, text_len as usize);
    }

    if ciphertext_slice.len() != text_len as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let mut plaintext_slice = &mut plaintext_vec[..];
    let rust_raw_string = format!("aes_gcm_128_decrypt parameter prepared! {},{}",
                                  ciphertext_slice.len(),
                                  plaintext_slice.len());

    unsafe {
        ocall_print_string(rust_raw_string.as_ptr() as *const c_uchar,
                           rust_raw_string.len() as size_t);
    }

    // After everything has been set, call API
    let result = rsgx_rijndael128GCM_decrypt(key,
                                             &ciphertext_slice,
                                             iv,
                                             &aad_array,
                                             mac,
                                             plaintext_slice);

    let rust_raw_string = "rsgx calling returned!";

    unsafe {
        ocall_print_string(rust_raw_string.as_ptr() as *const c_uchar,
                           rust_raw_string.len() as size_t);
    }
    // Match the result and copy result back to normal world.
    match result {
        Err(x) => {
            return x;
        }
        Ok(()) => {
            unsafe {
                ptr::copy_nonoverlapping(plaintext_slice.as_ptr(),
                                         plaintext,
                                         text_len as usize);
            }
        }
    }

    sgx_status_t::SGX_SUCCESS
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
/// **SGX_SUCCESS** on success.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER** indicates invalid input parameters
///
/// # Requirement
///
/// The caller should allocate the output cmac buffer.
#[no_mangle]
pub extern "C" fn aes_cmac(text: *const u8,
                           text_len: u32,
                           key: &[u8;16],
                           cmac: &mut [u8;16]) -> sgx_status_t {
    let text_slice;

    unsafe {
        text_slice = slice::from_raw_parts(text, text_len as usize);
    }

    if text_slice.len() != text_len as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let result = rsgx_rijndael128_cmac_slice(key, &text_slice);

    match result {
        Err(x) => return x,
        Ok(m) => *cmac = m
    }

    sgx_status_t::SGX_SUCCESS
}
