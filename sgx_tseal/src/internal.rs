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

use sgx_types::*;
use sgx_trts::trts::*;
use sgx_tcrypto::*;
use sgx_tse::*;
use core::mem;
use core::ptr;
use alloc::boxed::Box;
use alloc::vec::Vec;

//const SGX_MISCSEL_EXINFO: uint32_t     = 0x00000001;
//const TSEAL_DEFAULT_MISCMASK: uint32_t = (!SGX_MISCSEL_EXINFO);

/* intel sgx sdk 1.8 */
/* Set the bits which have no security implications to 0 for sealed data migration */
/* Bits which have no security implications in attributes.flags:
 *    Reserved bit[55:6]  - 0xFFFFFFFFFFFFC0ULL
 *    SGX_FLAGS_MODE64BIT
 *    SGX_FLAGS_PROVISION_KEY
 *    SGX_FLAGS_EINITTOKEN_KEY */
const FLAGS_NON_SECURITY_BITS: uint64_t = (0x00FFFFFFFFFFFFC0 | SGX_FLAGS_MODE64BIT | SGX_FLAGS_PROVISION_KEY| SGX_FLAGS_EINITTOKEN_KEY);
const TSEAL_DEFAULT_FLAGSMASK: uint64_t = (!FLAGS_NON_SECURITY_BITS);

const MISC_NON_SECURITY_BITS: uint32_t =  0x0FFFFFFF;  /* bit[27:0]: have no security implications */
const TSEAL_DEFAULT_MISCMASK: uint32_t =  (!MISC_NON_SECURITY_BITS);

#[derive(Clone, Default)]
pub struct SgxInternalUnsealedData {
    pub payload_size: u32,
    pub decrypt: Box<[u8]>,
    pub additional: Box<[u8]>,
}

impl SgxInternalUnsealedData {
    ///
    /// Get the payload size of the SgxInternalUnsealedData.
    ///
    #[allow(dead_code)]
    pub fn get_payload_size(&self) -> u32 {
        self.payload_size
    }
    ///
    /// Get the pointer of decrypt buffer in SgxInternalUnsealedData.
    ///
    #[allow(dead_code)]
    pub fn get_decrypt_txt(&self) -> &[u8] {
        &*self.decrypt
    }
    ///
    /// Get the pointer of additional buffer in SgxInternalUnsealedData.
    ///
    #[allow(dead_code)]
    pub fn get_additional_txt(&self) -> &[u8] {
        &*self.additional
    }
}

#[derive(Clone, Default)]
struct SgxPayload {
    payload_size: u32,
    reserved: [u8; 12],
    payload_tag: [u8; SGX_SEAL_TAG_SIZE],
    encrypt: Box<[u8]>,
    additional: Box<[u8]>,
}

#[derive(Clone, Default)]
pub struct SgxInternalSealedData {
    key_request: sgx_key_request_t,
    payload_data: SgxPayload,
}

impl SgxInternalSealedData {

    pub fn new() -> Self {
        SgxInternalSealedData::default()
    }
    pub fn get_payload_size(&self) -> u32 {
        self.payload_data.payload_size
    }
    pub fn get_payload_tag(&self) -> &[u8; SGX_SEAL_TAG_SIZE] {
        &self.payload_data.payload_tag
    }
    pub fn get_key_request(&self) -> &sgx_key_request_t {
        &self.key_request
    }
    pub fn get_encrypt_txt(&self) -> &[u8] {
        &*self.payload_data.encrypt
    }
    pub fn get_additional_txt(&self) -> &[u8] {
        &*self.payload_data.additional
    }

    pub fn calc_raw_sealed_data_size(add_mac_txt_size: u32, encrypt_txt_size: u32) -> u32 {

        let max = u32::max_value();
        let sealed_data_size = mem::size_of::<sgx_sealed_data_t>() as u32;

        if add_mac_txt_size > max - encrypt_txt_size {
            return max;
        }
        let payload_size: u32 = add_mac_txt_size + encrypt_txt_size;
        if payload_size > max - sealed_data_size {
            return max;
        }
        sealed_data_size + payload_size
    }

    pub fn get_add_mac_txt_len(&self) -> u32 {

        let data_size = self.payload_data.additional.len();
        if data_size > self.payload_data.payload_size as usize {
            u32::max_value()
        } else if data_size >= u32::max_value() as usize {
            u32::max_value()
        } else {
            data_size as u32
        }
    }

    pub fn get_encrypt_txt_len(&self) -> u32 {

        let data_size = self.payload_data.encrypt.len();
        if data_size > self.payload_data.payload_size as usize {
            u32::max_value()
        } else if data_size >= u32::max_value() as usize {
            u32::max_value()
        } else {
            data_size as u32
        }
    }

    pub unsafe fn to_raw_sealed_data_t(&self, p: * mut sgx_sealed_data_t, len: u32) -> Option<* mut sgx_sealed_data_t> {

        if p.is_null() {
            return None;
        }
        if rsgx_raw_is_within_enclave(p as * mut u8, len as usize) == false &&
           rsgx_raw_is_outside_enclave(p as * mut u8, len as usize) == false {
            return None;
        }

        let additional_len = self.get_add_mac_txt_len();
        let encrypt_len = self.get_encrypt_txt_len();
        if (additional_len == u32::max_value()) || (encrypt_len == u32::max_value()) {
            return None;
        }
        if (additional_len + encrypt_len) != self.get_payload_size() {
            return None;
        }

        let sealed_data_size = sgx_calc_sealed_data_size(additional_len, encrypt_len);
        if sealed_data_size == u32::max_value() {
            return None;
        }
        if len < sealed_data_size {
            return None;
        }

        let ptr_sealed_data = p as *mut u8;
        let ptr_encrypt = ptr_sealed_data.offset(mem::size_of::<sgx_sealed_data_t>() as isize);
        if encrypt_len > 0 {
            ptr::copy_nonoverlapping(self.payload_data.encrypt.as_ptr(), ptr_encrypt, encrypt_len as usize);
        }
        if additional_len > 0 {
            let ptr_additional = ptr_encrypt.offset(encrypt_len as isize);
            ptr::copy_nonoverlapping(self.payload_data.additional.as_ptr(), ptr_additional, additional_len as usize);
        }

        let raw_sealed_data = &mut *p;
        raw_sealed_data.key_request = self.key_request;
        raw_sealed_data.plain_text_offset = encrypt_len;
        raw_sealed_data.aes_data.payload_size = self.payload_data.payload_size;
        raw_sealed_data.aes_data.payload_tag = self.payload_data.payload_tag;

        Some(p)
    }

    pub unsafe fn from_raw_sealed_data_t(p: * mut sgx_sealed_data_t, len: u32) -> Option<Self> {

        if p.is_null() {
            return None;
        }
        if rsgx_raw_is_within_enclave(p as * mut u8, len as usize) == false &&
           rsgx_raw_is_outside_enclave(p as * mut u8, len as usize) == false {
            return None;
        }

        if (len as usize) < mem::size_of::<sgx_sealed_data_t>() {
            return None;
        }

        let raw_sealed_data = &*p;
        if raw_sealed_data.plain_text_offset > raw_sealed_data.aes_data.payload_size {
            return None;
        }

        let ptr_sealed_data = p as * mut u8;
        let additional_len = sgx_get_add_mac_txt_len(ptr_sealed_data as * const sgx_sealed_data_t);
        let encrypt_len = sgx_get_encrypt_txt_len(ptr_sealed_data as * const sgx_sealed_data_t);
        if (additional_len == u32::max_value()) || (encrypt_len == u32::max_value()) {
            return None;
        }
        if (additional_len + encrypt_len) != raw_sealed_data.aes_data.payload_size {
            return None;
        }

        let sealed_data_size = sgx_calc_sealed_data_size(additional_len, encrypt_len);
        if sealed_data_size == u32::max_value() {
            return None;
        }
        if len < sealed_data_size {
            return None;
        }

        let ptr_encrypt = ptr_sealed_data.offset(mem::size_of::<sgx_sealed_data_t>() as isize);
        let mut encrypt: Vec<u8> =  Vec::new();
        if encrypt_len > 0 {
            let mut temp: Vec<u8> = Vec::with_capacity(encrypt_len as usize);
            temp.set_len(encrypt_len as usize);
            ptr::copy_nonoverlapping(ptr_encrypt as * const u8, temp.as_mut_ptr(), encrypt_len as usize);
            encrypt = temp;
        }

        let mut additional: Vec<u8> = Vec::new();
        if additional_len > 0 {
            let ptr_additional = ptr_encrypt.offset(encrypt_len as isize);
            let mut temp: Vec<u8> = Vec::with_capacity(additional_len as usize);
            temp.set_len(additional_len as usize);
            ptr::copy_nonoverlapping(ptr_additional as * const u8, temp.as_mut_ptr(), additional_len as usize);
            additional = temp;
        }

        let mut sealed_data = Self::default();
        sealed_data.key_request = raw_sealed_data.key_request;
        sealed_data.payload_data.payload_size = raw_sealed_data.aes_data.payload_size;
        sealed_data.payload_data.payload_tag = raw_sealed_data.aes_data.payload_tag;
        sealed_data.payload_data.additional = additional.into_boxed_slice();
        sealed_data.payload_data.encrypt = encrypt.into_boxed_slice();

        Some(sealed_data)
    }

    pub fn seal_data(additional_text: &[u8], encrypt_text: &[u8]) -> SgxResult<Self> {

        //let attribute_mask = sgx_attributes_t{flags: SGX_FLAGS_RESERVED | SGX_FLAGS_INITTED | SGX_FLAGS_DEBUG, xfrm: 0};
        /* intel sgx sdk 1.8 */
        let attribute_mask = sgx_attributes_t{flags: TSEAL_DEFAULT_FLAGSMASK, xfrm: 0};
        Self::seal_data_ex(SGX_KEYPOLICY_MRSIGNER,
                           attribute_mask,
                           TSEAL_DEFAULT_MISCMASK,
                           additional_text,
                           encrypt_text)
    }

    pub fn seal_data_ex(key_policy: u16,
                        attribute_mask: sgx_attributes_t,
                        misc_mask: sgx_misc_select_t,
                        additional_text: &[u8],
                        encrypt_text: &[u8]) -> SgxResult<Self> {

        let additional_len = additional_text.len();
        let encrypt_len = encrypt_text.len();

        if (additional_len >= u32::max_value() as usize) || (encrypt_len >= u32::max_value() as usize) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if Self::calc_raw_sealed_data_size(additional_len as u32, encrypt_len as u32) == u32::max_value() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if encrypt_len == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if (key_policy & (!(SGX_KEYPOLICY_MRENCLAVE | SGX_KEYPOLICY_MRSIGNER)) != 0) ||
           ((key_policy &  (SGX_KEYPOLICY_MRENCLAVE | SGX_KEYPOLICY_MRSIGNER)) == 0) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if ((attribute_mask.flags & SGX_FLAGS_INITTED) == 0) ||
           ((attribute_mask.flags & SGX_FLAGS_DEBUG) == 0) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if rsgx_slice_is_within_enclave(encrypt_text) == false {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if additional_len > 0 {
            if (rsgx_slice_is_within_enclave(additional_text) == false) &&
               (rsgx_slice_is_outside_enclave(additional_text) == false) {
                return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
            }
        }

        let target_info = sgx_target_info_t::default();
        let report_data = sgx_report_data_t::default();
        let mut key_id = sgx_key_id_t::default();

        let mut report = try!(rsgx_create_report(&target_info, &report_data));

        let error = rsgx_read_rand(&mut key_id.id);
        if error.is_err() {
            report = sgx_report_t::default();
            key_id = sgx_key_id_t::default();
            return Err(error.unwrap_err());
        }

        let key_request = sgx_key_request_t{key_name: SGX_KEYSELECT_SEAL,
                                            key_policy: key_policy,
                                            isv_svn: report.body.isv_svn,
                                            reserved1: 0_u16,
                                            cpu_svn: report.body.cpu_svn,
                                            attribute_mask: attribute_mask,
                                            key_id: key_id,
                                            misc_mask: misc_mask,
                                            reserved2: [0_u8; SGX_KEY_REQUEST_RESERVED2_BYTES]};

        let payload_iv = [0_u8; SGX_SEAL_IV_SIZE];
        let mut result = Self::seal_data_iv(additional_text, encrypt_text, &payload_iv, &key_request);
        match result {
            Ok(ref mut sealed_data) => sealed_data.key_request = key_request,
            _ => {},
        };

        report = sgx_report_t::default();
        key_id = sgx_key_id_t::default();

        result
    }

    pub fn unseal_data(&self) -> SgxResult<SgxInternalUnsealedData> {

        let additional_len = self.get_add_mac_txt_len();
        let encrypt_len = self.get_encrypt_txt_len();

        if (additional_len == u32::max_value()) || (encrypt_len == u32::max_value()) {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if Self::calc_raw_sealed_data_size(additional_len, encrypt_len) == u32::max_value() {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if encrypt_len < 1 {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if (additional_len + encrypt_len) != self.get_payload_size() {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if rsgx_raw_is_within_enclave(self as * const _ as * const u8,  mem::size_of::<Self>()) == false {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if rsgx_slice_is_within_enclave(self.get_encrypt_txt()) == false {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if additional_len > 0 {
            if rsgx_slice_is_within_enclave(self.get_additional_txt()) == false {
                return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
            }
        }

        self.unseal_data_helper()
    }

    pub fn mac_aadata(additional_text: &[u8]) -> SgxResult<Self> {

        let attribute_mask = sgx_attributes_t{flags: SGX_FLAGS_RESERVED | SGX_FLAGS_INITTED | SGX_FLAGS_DEBUG, xfrm: 0};

        Self::mac_aadata_ex(SGX_KEYPOLICY_MRSIGNER,
                            attribute_mask,
                            TSEAL_DEFAULT_MISCMASK,
                            additional_text)
    }

    pub fn mac_aadata_ex(key_policy: u16,
                         attribute_mask: sgx_attributes_t,
                         misc_mask: sgx_misc_select_t,
                         additional_text: &[u8]) -> SgxResult<Self> {

        let additional_len = additional_text.len();
        if additional_len >= u32::max_value() as usize {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if Self::calc_raw_sealed_data_size(additional_len as u32, 0_u32) == u32::max_value() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if additional_len == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if (key_policy & (!(SGX_KEYPOLICY_MRENCLAVE | SGX_KEYPOLICY_MRSIGNER)) != 0) ||
           ((key_policy &  (SGX_KEYPOLICY_MRENCLAVE | SGX_KEYPOLICY_MRSIGNER)) == 0) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if (attribute_mask.flags & 0x3) != 0x3 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if (rsgx_slice_is_within_enclave(additional_text) == false) &&
           (rsgx_slice_is_outside_enclave(additional_text) == false) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        let target_info = sgx_target_info_t::default();
        let report_data = sgx_report_data_t::default();
        let mut key_id = sgx_key_id_t::default();

        let mut report = try!(rsgx_create_report(&target_info, &report_data));

        let error = rsgx_read_rand(&mut key_id.id);
        if error.is_err() {
            report = sgx_report_t::default();
            key_id = sgx_key_id_t::default();
            return Err(error.unwrap_err());
        }

        let key_request = sgx_key_request_t{key_name: SGX_KEYSELECT_SEAL,
                                            key_policy: key_policy,
                                            isv_svn: report.body.isv_svn,
                                            reserved1: 0_u16,
                                            cpu_svn: report.body.cpu_svn,
                                            attribute_mask: attribute_mask,
                                            key_id: key_id,
                                            misc_mask: misc_mask,
                                            reserved2: [0_u8; SGX_KEY_REQUEST_RESERVED2_BYTES]};

        let payload_iv = [0_u8; SGX_SEAL_IV_SIZE];
        let mut result = Self::seal_data_iv(additional_text, &[0_u8; 0], &payload_iv, &key_request);
        match result {
            Ok(ref mut sealed_data) => sealed_data.key_request = key_request,
            _ => {},
        };

        report = sgx_report_t::default();
        key_id = sgx_key_id_t::default();

        result
    }

    pub fn unmac_aadata(&self) -> SgxResult<SgxInternalUnsealedData> {

        let additional_len = self.get_add_mac_txt_len();
        let encrypt_len = self.get_encrypt_txt_len();

        if (additional_len == u32::max_value()) || (encrypt_len == u32::max_value()) {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if additional_len < 1 {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if encrypt_len != 0 {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if Self::calc_raw_sealed_data_size(additional_len, encrypt_len) == u32::max_value() {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if (additional_len + encrypt_len) != self.get_payload_size() {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }

        if rsgx_raw_is_within_enclave(self as * const _ as * const u8,  mem::size_of::<Self>()) == false {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if rsgx_slice_is_within_enclave(self.get_additional_txt()) == false {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        self.unseal_data_helper()
    }

    fn seal_data_iv(additional_text: &[u8],
                    encrypt_text: &[u8],
                    payload_iv: &[u8],
                    key_request: &sgx_key_request_t) -> SgxResult<Self>  {


        let mut seal_key = try!(rsgx_get_key(key_request).map_err(|ret| {
            if ret != sgx_status_t::SGX_ERROR_OUT_OF_MEMORY {
                sgx_status_t::SGX_ERROR_UNEXPECTED
            } else {
                ret
            }
        }));

        let mut sealed_data = SgxInternalSealedData::default();
        sealed_data.payload_data.encrypt = vec![0_u8; encrypt_text.len()].into_boxed_slice();

        let error = rsgx_rijndael128GCM_encrypt(&seal_key,
                                                encrypt_text,
                                                payload_iv,
                                                &additional_text,
                                                &mut sealed_data.payload_data.encrypt,
                                                &mut sealed_data.payload_data.payload_tag);
        if error.is_err() {
            seal_key = sgx_key_128bit_t::default();
            return Err(error.unwrap_err());
        }

        sealed_data.payload_data.payload_size = (encrypt_text.len() + additional_text.len()) as u32;
        if additional_text.len() > 0 {
            sealed_data.payload_data.additional = additional_text.to_vec().into_boxed_slice();
        }

        seal_key = sgx_key_128bit_t::default();

        Ok(sealed_data)
    }

    fn unseal_data_helper(&self) -> SgxResult<SgxInternalUnsealedData> {

        let mut seal_key = try!(rsgx_get_key(self.get_key_request()).map_err(|ret| {
            if (ret == sgx_status_t::SGX_ERROR_INVALID_CPUSVN) ||
               (ret == sgx_status_t::SGX_ERROR_INVALID_ISVSVN) ||
               (ret == sgx_status_t::SGX_ERROR_OUT_OF_MEMORY) {
                ret
            } else {
                sgx_status_t::SGX_ERROR_MAC_MISMATCH
            }
        }));

        let payload_iv = [0_u8; SGX_SEAL_IV_SIZE];
        let mut unsealed_data: SgxInternalUnsealedData = SgxInternalUnsealedData::default();
        unsealed_data.decrypt = vec![0_u8; self.payload_data.encrypt.len()].into_boxed_slice();

        let error = rsgx_rijndael128GCM_decrypt(&seal_key,
                                                self.get_encrypt_txt(),
                                                &payload_iv,
                                                self.get_additional_txt(),
                                                self.get_payload_tag(),
                                                &mut unsealed_data.decrypt);
        if error.is_err() {
            seal_key = sgx_key_128bit_t::default();
            return Err(error.unwrap_err());
        }

        if self.payload_data.additional.len() > 0 {
            unsealed_data.additional = self.get_additional_txt().to_vec().into_boxed_slice();
        }
        unsealed_data.payload_size = self.get_payload_size();

        seal_key = sgx_key_128bit_t::default();

        Ok(unsealed_data)
    }
}