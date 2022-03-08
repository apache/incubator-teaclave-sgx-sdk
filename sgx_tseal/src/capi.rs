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

use crate::aad::MacAad;
use crate::internal::InnerSealedData;
use crate::seal::SealedData;
use core::mem;
use core::ptr;
use core::slice;
use sgx_trts::trts::{is_within_enclave, is_within_host};
use sgx_types::error::SgxStatus;
use sgx_types::types::{Attributes, CSealedData, KeyPolicy};

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_calc_sealed_data_size(aad_size: u32, encrypt_size: u32) -> u32 {
    InnerSealedData::raw_sealed_data_size(aad_size, encrypt_size).unwrap_or(u32::MAX)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_get_add_mac_txt_len(c_sealed_data: *const CSealedData) -> u32 {
    if c_sealed_data.is_null() {
        return u32::MAX;
    }

    let sealed_data = &*c_sealed_data;
    let data_size = sealed_data.aes_data.payload_size - sealed_data.plaintext_offset;
    if data_size > sealed_data.aes_data.payload_size {
        u32::MAX
    } else {
        data_size
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_get_encrypt_txt_len(c_sealed_data: *const CSealedData) -> u32 {
    if c_sealed_data.is_null() {
        u32::MAX
    } else {
        (*c_sealed_data).plaintext_offset
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_seal_data(
    aad_size: u32,
    aad: *const u8,
    encrypt_size: u32,
    encrypt_txt: *const u8,
    sealed_data_size: u32,
    c_sealed_data: *mut CSealedData,
) -> SgxStatus {
    let raw_size =
        InnerSealedData::raw_sealed_data_size(aad_size, encrypt_size).unwrap_or(u32::MAX);
    if raw_size == u32::MAX || raw_size != sealed_data_size {
        return SgxStatus::InvalidParameter;
    }

    if c_sealed_data.is_null() {
        return SgxStatus::InvalidParameter;
    }
    if !is_within_enclave(c_sealed_data as *const u8, raw_size as usize) {
        return SgxStatus::InvalidParameter;
    }

    if aad_size > 0 && aad.is_null() {
        return SgxStatus::InvalidParameter;
    }
    if encrypt_size == 0 || encrypt_txt.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let aad = if aad_size != 0 {
        Some(slice::from_raw_parts(aad, aad_size as usize))
    } else {
        None
    };
    let plaintext = slice::from_raw_parts(encrypt_txt, encrypt_size as usize);

    let sealed_data = match SealedData::seal(plaintext, aad) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let bytes = match sealed_data.into_bytes() {
        Ok(b) => b,
        Err(e) => return e,
    };

    ptr::copy_nonoverlapping(
        bytes.as_ptr(),
        c_sealed_data.cast(),
        sealed_data_size as usize,
    );

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_seal_data_ex(
    key_policy: u16,
    attribute_mask: Attributes,
    misc_mask: u32,
    aad_size: u32,
    aad: *const u8,
    encrypt_size: u32,
    encrypt_txt: *const u8,
    sealed_data_size: u32,
    c_sealed_data: *mut CSealedData,
) -> SgxStatus {
    let raw_size =
        InnerSealedData::raw_sealed_data_size(aad_size, encrypt_size).unwrap_or(u32::MAX);
    if raw_size == u32::MAX || raw_size != sealed_data_size {
        return SgxStatus::InvalidParameter;
    }

    if c_sealed_data.is_null() {
        return SgxStatus::InvalidParameter;
    }
    if !is_within_enclave(c_sealed_data as *const u8, raw_size as usize) {
        return SgxStatus::InvalidParameter;
    }

    let key_policy = match KeyPolicy::from_bits(key_policy) {
        Some(policy) => policy,
        None => return SgxStatus::InvalidParameter,
    };

    if aad_size > 0 && aad.is_null() {
        return SgxStatus::InvalidParameter;
    }
    if encrypt_size == 0 || encrypt_txt.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let aad = if aad_size != 0 {
        Some(slice::from_raw_parts(aad, aad_size as usize))
    } else {
        None
    };
    let plaintext = slice::from_raw_parts(encrypt_txt, encrypt_size as usize);

    let sealed_data = match SealedData::seal_with_key_policy(
        key_policy,
        attribute_mask,
        misc_mask,
        plaintext,
        aad,
    ) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let bytes = match sealed_data.into_bytes() {
        Ok(b) => b,
        Err(e) => return e,
    };

    ptr::copy_nonoverlapping(
        bytes.as_ptr(),
        c_sealed_data.cast(),
        sealed_data_size as usize,
    );

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_unseal_data(
    c_sealed_data: *const CSealedData,
    aad: *mut u8,
    aad_size: *mut u32,
    decrypt_txt: *mut u8,
    decrypt_size: *mut u32,
) -> SgxStatus {
    if c_sealed_data.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let ciphertext_len = sgx_get_encrypt_txt_len(c_sealed_data);
    let aad_len = sgx_get_add_mac_txt_len(c_sealed_data);
    if ciphertext_len == u32::MAX || aad_len == u32::MAX {
        return SgxStatus::InvalidParameter;
    }

    let raw_size = sgx_calc_sealed_data_size(aad_len, ciphertext_len);
    if raw_size == u32::MAX {
        return SgxStatus::InvalidParameter;
    }

    if (aad_len > 0) && (aad.is_null() || aad_size.is_null()) {
        return SgxStatus::InvalidParameter;
    }

    if ciphertext_len < 1 || decrypt_txt.is_null() || decrypt_size.is_null() {
        return SgxStatus::InvalidParameter;
    }

    if !is_within_enclave(decrypt_size as *const u8, mem::size_of::<u32>()) {
        return SgxStatus::InvalidParameter;
    }

    let input_decrypt_size = *decrypt_size;
    if input_decrypt_size < ciphertext_len {
        return SgxStatus::InvalidParameter;
    }

    if !is_within_enclave(decrypt_txt, input_decrypt_size as usize) {
        return SgxStatus::InvalidParameter;
    }

    if !(aad_size.is_null()
        || is_within_enclave(aad_size as *const u8, mem::size_of::<u32>())
        || is_within_host(aad_size as *const u8, mem::size_of::<u32>()))
    {
        return SgxStatus::InvalidParameter;
    }

    let input_aad_size = if !aad_size.is_null() { *aad_size } else { 0 };
    if input_aad_size < aad_len {
        return SgxStatus::InvalidParameter;
    }

    if (aad_len > 0)
        && (!(is_within_enclave(aad, input_aad_size as usize)
            || is_within_host(aad, input_aad_size as usize)))
    {
        return SgxStatus::InvalidParameter;
    }

    let raw_slice = slice::from_raw_parts(c_sealed_data as *const u8, raw_size as usize);
    let sealed_data = match InnerSealedData::from_slice(raw_slice) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let unsealed_data = match sealed_data.unseal() {
        Ok(data) => data,
        Err(e) => return e,
    };

    let decrypt_slice = slice::from_raw_parts_mut(decrypt_txt, ciphertext_len as usize);
    decrypt_slice.fill(0);
    decrypt_slice.copy_from_slice(&unsealed_data.plaintext);
    *decrypt_size = ciphertext_len;

    if aad_len > 0 {
        let aad_slice = slice::from_raw_parts_mut(aad, aad_len as usize);
        aad_slice.fill(0);
        aad_slice.copy_from_slice(&unsealed_data.aad);
        *aad_size = aad_len;
    }

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_mac_aadata(
    aad_size: u32,
    aad: *const u8,
    sealed_data_size: u32,
    c_sealed_data: *mut CSealedData,
) -> SgxStatus {
    let raw_size = InnerSealedData::raw_sealed_data_size(aad_size, 0).unwrap_or(u32::MAX);
    if raw_size == u32::MAX || raw_size != sealed_data_size {
        return SgxStatus::InvalidParameter;
    }

    if c_sealed_data.is_null() {
        return SgxStatus::InvalidParameter;
    }
    if !is_within_enclave(c_sealed_data as *const u8, raw_size as usize) {
        return SgxStatus::InvalidParameter;
    }

    if aad_size == 0 || aad.is_null() {
        return SgxStatus::InvalidParameter;
    }
    let aad = slice::from_raw_parts(aad, aad_size as usize);

    let mac_aad = match MacAad::mac(aad) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let bytes = match mac_aad.into_bytes() {
        Ok(b) => b,
        Err(e) => return e,
    };

    ptr::copy_nonoverlapping(
        bytes.as_ptr(),
        c_sealed_data.cast(),
        sealed_data_size as usize,
    );

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_mac_aadata_ex(
    key_policy: u16,
    attribute_mask: Attributes,
    misc_mask: u32,
    aad_size: u32,
    aad: *const u8,
    sealed_data_size: u32,
    c_sealed_data: *mut CSealedData,
) -> SgxStatus {
    let raw_size = InnerSealedData::raw_sealed_data_size(aad_size, 0).unwrap_or(u32::MAX);
    if raw_size == u32::MAX || raw_size != sealed_data_size {
        return SgxStatus::InvalidParameter;
    }

    if c_sealed_data.is_null() {
        return SgxStatus::InvalidParameter;
    }
    if !is_within_enclave(c_sealed_data as *const u8, raw_size as usize) {
        return SgxStatus::InvalidParameter;
    }

    let key_policy = match KeyPolicy::from_bits(key_policy) {
        Some(policy) => policy,
        None => return SgxStatus::InvalidParameter,
    };

    if aad_size == 0 || aad.is_null() {
        return SgxStatus::InvalidParameter;
    }
    let aad = slice::from_raw_parts(aad, aad_size as usize);

    let mac_aad = match MacAad::mac_with_key_policy(key_policy, attribute_mask, misc_mask, aad) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let bytes = match mac_aad.into_bytes() {
        Ok(b) => b,
        Err(e) => return e,
    };

    ptr::copy_nonoverlapping(
        bytes.as_ptr(),
        c_sealed_data.cast(),
        sealed_data_size as usize,
    );

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_unmac_aadata(
    c_sealed_data: *const CSealedData,
    aad: *mut u8,
    aad_size: *mut u32,
) -> SgxStatus {
    let ciphertext_len = sgx_get_encrypt_txt_len(c_sealed_data);
    if ciphertext_len != 0 {
        return SgxStatus::InvalidParameter;
    }

    let aad_len = sgx_get_add_mac_txt_len(c_sealed_data);
    if aad_len == u32::MAX || aad_len == 0 {
        return SgxStatus::InvalidParameter;
    }

    let raw_size = sgx_calc_sealed_data_size(aad_len, ciphertext_len);
    if raw_size == u32::MAX {
        return SgxStatus::InvalidParameter;
    }

    if aad.is_null() || aad_size.is_null() {
        return SgxStatus::InvalidParameter;
    }

    if !(is_within_enclave(aad_size as *const u8, mem::size_of::<u32>())
        || is_within_host(aad_size as *const u8, mem::size_of::<u32>()))
    {
        return SgxStatus::InvalidParameter;
    }

    let input_aad_size = *aad_size;
    if input_aad_size < aad_len {
        return SgxStatus::InvalidParameter;
    }

    if !(is_within_enclave(aad, input_aad_size as usize)
        || is_within_host(aad, input_aad_size as usize))
    {
        return SgxStatus::InvalidParameter;
    }

    let raw_slice = slice::from_raw_parts(c_sealed_data as *const u8, raw_size as usize);
    let sealed_data = match InnerSealedData::from_slice(raw_slice) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let unsealed_data = match sealed_data.verify() {
        Ok(data) => data,
        Err(e) => return e,
    };

    let aad_slice = slice::from_raw_parts_mut(aad, aad_len as usize);
    aad_slice.fill(0);
    aad_slice.copy_from_slice(&unsealed_data.aad);
    *aad_size = aad_len;

    SgxStatus::Success
}
