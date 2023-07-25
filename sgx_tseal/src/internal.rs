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

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem;
use core::ptr;
use sgx_tcrypto::*;
use sgx_trts::trts::*;
use sgx_tse::*;
use sgx_types::*;

/* intel sgx sdk 2.4 */
const KEY_POLICY_KSS: uint16_t =
    SGX_KEYPOLICY_CONFIGID | SGX_KEYPOLICY_ISVFAMILYID | SGX_KEYPOLICY_ISVEXTPRODID;

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
    _reserved: [u8; 12],
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
        let max = u32::MAX;
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
        if data_size > self.payload_data.payload_size as usize || data_size >= u32::MAX as usize {
            u32::MAX
        } else {
            data_size as u32
        }
    }

    pub fn get_encrypt_txt_len(&self) -> u32 {
        let data_size = self.payload_data.encrypt.len();
        if data_size > self.payload_data.payload_size as usize || data_size >= u32::MAX as usize {
            u32::MAX
        } else {
            data_size as u32
        }
    }

    pub unsafe fn to_raw_sealed_data_t(
        &self,
        p: *mut sgx_sealed_data_t,
        len: u32,
    ) -> Option<*mut sgx_sealed_data_t> {
        if p.is_null() {
            return None;
        }
        if !rsgx_raw_is_within_enclave(p as *mut u8, len as usize)
            && !rsgx_raw_is_outside_enclave(p as *mut u8, len as usize)
        {
            return None;
        }

        let additional_len = self.get_add_mac_txt_len();
        let encrypt_len = self.get_encrypt_txt_len();
        if (additional_len == u32::MAX) || (encrypt_len == u32::MAX) {
            return None;
        }
        if (additional_len + encrypt_len) != self.get_payload_size() {
            return None;
        }

        let sealed_data_size = sgx_calc_sealed_data_size(additional_len, encrypt_len);
        if sealed_data_size == u32::MAX {
            return None;
        }
        if len < sealed_data_size {
            return None;
        }

        let ptr_sealed_data = p as *mut u8;
        let ptr_encrypt = ptr_sealed_data.add(mem::size_of::<sgx_sealed_data_t>());
        if encrypt_len > 0 {
            ptr::copy_nonoverlapping(
                self.payload_data.encrypt.as_ptr(),
                ptr_encrypt,
                encrypt_len as usize,
            );
        }
        if additional_len > 0 {
            let ptr_additional = ptr_encrypt.offset(encrypt_len as isize);
            ptr::copy_nonoverlapping(
                self.payload_data.additional.as_ptr(),
                ptr_additional,
                additional_len as usize,
            );
        }

        let raw_sealed_data = &mut *p;
        raw_sealed_data.key_request = self.key_request;
        raw_sealed_data.plain_text_offset = encrypt_len;
        raw_sealed_data.aes_data.payload_size = self.payload_data.payload_size;
        raw_sealed_data.aes_data.payload_tag = self.payload_data.payload_tag;

        Some(p)
    }

    #[allow(clippy::cast_ptr_alignment)]
    pub unsafe fn from_raw_sealed_data_t(p: *mut sgx_sealed_data_t, len: u32) -> Option<Self> {
        if p.is_null() {
            return None;
        }
        if !rsgx_raw_is_within_enclave(p as *mut u8, len as usize)
            && !rsgx_raw_is_outside_enclave(p as *mut u8, len as usize)
        {
            return None;
        }

        if (len as usize) < mem::size_of::<sgx_sealed_data_t>() {
            return None;
        }

        let raw_sealed_data = &*p;
        if raw_sealed_data.plain_text_offset > raw_sealed_data.aes_data.payload_size {
            return None;
        }

        let ptr_sealed_data = p as *mut u8;
        let additional_len = sgx_get_add_mac_txt_len(ptr_sealed_data as *const sgx_sealed_data_t);
        let encrypt_len = sgx_get_encrypt_txt_len(ptr_sealed_data as *const sgx_sealed_data_t);
        if (additional_len == u32::MAX) || (encrypt_len == u32::MAX) {
            return None;
        }
        if (additional_len + encrypt_len) != raw_sealed_data.aes_data.payload_size {
            return None;
        }

        let sealed_data_size = sgx_calc_sealed_data_size(additional_len, encrypt_len);
        if sealed_data_size == u32::MAX {
            return None;
        }
        if len < sealed_data_size {
            return None;
        }

        let ptr_encrypt = ptr_sealed_data.add(mem::size_of::<sgx_sealed_data_t>());

        let encrypt: Vec<u8> = if encrypt_len > 0 {
            let mut temp: Vec<u8> = Vec::with_capacity(encrypt_len as usize);
            temp.set_len(encrypt_len as usize);
            ptr::copy_nonoverlapping(
                ptr_encrypt as *const u8,
                temp.as_mut_ptr(),
                encrypt_len as usize,
            );
            temp
        } else {
            Vec::new()
        };

        let additional: Vec<u8> = if additional_len > 0 {
            let ptr_additional = ptr_encrypt.offset(encrypt_len as isize);
            let mut temp: Vec<u8> = Vec::with_capacity(additional_len as usize);
            temp.set_len(additional_len as usize);
            ptr::copy_nonoverlapping(
                ptr_additional as *const u8,
                temp.as_mut_ptr(),
                additional_len as usize,
            );
            temp
        } else {
            Vec::new()
        };

        Some(Self {
            key_request: raw_sealed_data.key_request,
            payload_data: SgxPayload {
                payload_size: raw_sealed_data.aes_data.payload_size,
                _reserved: [0; 12],
                payload_tag: raw_sealed_data.aes_data.payload_tag,
                additional: additional.into_boxed_slice(),
                encrypt: encrypt.into_boxed_slice(),
            },
        })
    }

    pub fn seal_data(additional_text: &[u8], encrypt_text: &[u8]) -> SgxResult<Self> {
        /* intel sgx sdk 1.8 */
        let attribute_mask = sgx_attributes_t {
            flags: TSEAL_DEFAULT_FLAGSMASK,
            xfrm: 0,
        };
        /* intel sgx sdk 2.4 */
        let mut key_policy = SGX_KEYPOLICY_MRSIGNER;
        let report = rsgx_self_report();
        if (report.body.attributes.flags & SGX_FLAGS_KSS) != 0 {
            key_policy = SGX_KEYPOLICY_MRSIGNER | KEY_POLICY_KSS;
        }

        Self::seal_data_ex(
            key_policy,
            attribute_mask,
            TSEAL_DEFAULT_MISCMASK,
            additional_text,
            encrypt_text,
        )
    }

    pub fn seal_data_ex(
        key_policy: u16,
        attribute_mask: sgx_attributes_t,
        misc_mask: sgx_misc_select_t,
        additional_text: &[u8],
        encrypt_text: &[u8],
    ) -> SgxResult<Self> {
        let additional_len = additional_text.len();
        let encrypt_len = encrypt_text.len();

        if (additional_len >= u32::MAX as usize) || (encrypt_len >= u32::MAX as usize) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if Self::calc_raw_sealed_data_size(additional_len as u32, encrypt_len as u32) == u32::MAX {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if encrypt_len == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if (key_policy
            & (!(SGX_KEYPOLICY_MRENCLAVE
                | SGX_KEYPOLICY_MRSIGNER
                | KEY_POLICY_KSS
                | SGX_KEYPOLICY_NOISVPRODID))
            != 0)
            || ((key_policy & (SGX_KEYPOLICY_MRENCLAVE | SGX_KEYPOLICY_MRSIGNER)) == 0)
        {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if ((attribute_mask.flags & SGX_FLAGS_INITTED) == 0)
            || ((attribute_mask.flags & SGX_FLAGS_DEBUG) == 0)
        {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if !rsgx_slice_is_within_enclave(encrypt_text) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if additional_len > 0
            && !rsgx_slice_is_within_enclave(additional_text)
            && !rsgx_slice_is_outside_enclave(additional_text)
        {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        //let target_info = sgx_target_info_t::default();
        //let report_data = sgx_report_data_t::default();
        let mut key_id = sgx_key_id_t::default();

        /* intel sgx sdk 2.4 */
        let mut report = rsgx_self_report();

        let error = rsgx_read_rand(&mut key_id.id);
        if let Err(e) = error {
            report = sgx_report_t::default();
            key_id = sgx_key_id_t::default();
            return Err(e);
        }

        let key_request = sgx_key_request_t {
            key_name: SGX_KEYSELECT_SEAL,
            key_policy,
            isv_svn: report.body.isv_svn,
            reserved1: 0_u16,
            cpu_svn: report.body.cpu_svn,
            attribute_mask,
            key_id,
            misc_mask,
            config_svn: report.body.config_svn,
            reserved2: [0_u8; SGX_KEY_REQUEST_RESERVED2_BYTES],
        };

        let payload_iv = [0_u8; SGX_SEAL_IV_SIZE];
        let mut result =
            Self::seal_data_iv(additional_text, encrypt_text, &payload_iv, &key_request);

        if let Ok(ref mut sealed_data) = result {
            sealed_data.key_request = key_request
        };

        report = sgx_report_t::default();
        key_id = sgx_key_id_t::default();

        result
    }

    pub fn unseal_data(&self) -> SgxResult<SgxInternalUnsealedData> {
        let additional_len = self.get_add_mac_txt_len();
        let encrypt_len = self.get_encrypt_txt_len();

        if (additional_len == u32::MAX) || (encrypt_len == u32::MAX) {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if Self::calc_raw_sealed_data_size(additional_len, encrypt_len) == u32::MAX {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if encrypt_len < 1 {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if (additional_len + encrypt_len) != self.get_payload_size() {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if !rsgx_raw_is_within_enclave(self as *const _ as *const u8, mem::size_of::<Self>()) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if !rsgx_slice_is_within_enclave(self.get_encrypt_txt()) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if additional_len > 0 && !rsgx_slice_is_within_enclave(self.get_additional_txt()) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        self.unseal_data_helper()
    }

    pub fn mac_aadata(additional_text: &[u8]) -> SgxResult<Self> {
        let attribute_mask = sgx_attributes_t {
            flags: TSEAL_DEFAULT_FLAGSMASK,
            xfrm: 0,
        };
        let mut key_policy: u16 = SGX_KEYPOLICY_MRSIGNER;
        let report = rsgx_self_report();

        if (report.body.attributes.flags & SGX_FLAGS_KSS) != 0 {
            key_policy = SGX_KEYPOLICY_MRSIGNER | KEY_POLICY_KSS;
        }

        Self::mac_aadata_ex(
            key_policy,
            attribute_mask,
            TSEAL_DEFAULT_MISCMASK,
            additional_text,
        )
    }

    pub fn mac_aadata_ex(
        key_policy: u16,
        attribute_mask: sgx_attributes_t,
        misc_mask: sgx_misc_select_t,
        additional_text: &[u8],
    ) -> SgxResult<Self> {
        let additional_len = additional_text.len();
        if additional_len >= u32::MAX as usize {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if Self::calc_raw_sealed_data_size(additional_len as u32, 0_u32) == u32::MAX {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if additional_len == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if (key_policy
            & (!(SGX_KEYPOLICY_MRENCLAVE
                | SGX_KEYPOLICY_MRSIGNER
                | KEY_POLICY_KSS
                | SGX_KEYPOLICY_NOISVPRODID))
            != 0)
            || ((key_policy & (SGX_KEYPOLICY_MRENCLAVE | SGX_KEYPOLICY_MRSIGNER)) == 0)
        {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if ((attribute_mask.flags & SGX_FLAGS_INITTED) == 0)
            || ((attribute_mask.flags & SGX_FLAGS_DEBUG) == 0)
        {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if !rsgx_slice_is_within_enclave(additional_text)
            && !rsgx_slice_is_outside_enclave(additional_text)
        {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        //let target_info = sgx_target_info_t::default();
        //let report_data = sgx_report_data_t::default();
        let mut key_id = sgx_key_id_t::default();

        /* intel sgx sdk 2.4 */
        let mut report = rsgx_self_report();

        let error = rsgx_read_rand(&mut key_id.id);
        if let Err(e) = error {
            report = sgx_report_t::default();
            key_id = sgx_key_id_t::default();
            return Err(e);
        }

        let key_request = sgx_key_request_t {
            key_name: SGX_KEYSELECT_SEAL,
            key_policy,
            isv_svn: report.body.isv_svn,
            reserved1: 0_u16,
            cpu_svn: report.body.cpu_svn,
            attribute_mask,
            key_id,
            misc_mask,
            config_svn: report.body.config_svn,
            reserved2: [0_u8; SGX_KEY_REQUEST_RESERVED2_BYTES],
        };

        let payload_iv = [0_u8; SGX_SEAL_IV_SIZE];
        let mut result = Self::seal_data_iv(additional_text, &[0_u8; 0], &payload_iv, &key_request);
        if let Ok(ref mut sealed_data) = result {
            sealed_data.key_request = key_request
        };

        report = sgx_report_t::default();
        key_id = sgx_key_id_t::default();

        result
    }

    pub fn unmac_aadata(&self) -> SgxResult<SgxInternalUnsealedData> {
        let additional_len = self.get_add_mac_txt_len();
        let encrypt_len = self.get_encrypt_txt_len();

        if (additional_len == u32::MAX) || (encrypt_len == u32::MAX) {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if additional_len < 1 {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if encrypt_len != 0 {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if Self::calc_raw_sealed_data_size(additional_len, encrypt_len) == u32::MAX {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if (additional_len + encrypt_len) != self.get_payload_size() {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }

        if !rsgx_raw_is_within_enclave(self as *const _ as *const u8, mem::size_of::<Self>()) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if !rsgx_slice_is_within_enclave(self.get_additional_txt()) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        self.unseal_data_helper()
    }

    fn seal_data_iv(
        additional_text: &[u8],
        encrypt_text: &[u8],
        payload_iv: &[u8],
        key_request: &sgx_key_request_t,
    ) -> SgxResult<Self> {
        let mut seal_key = rsgx_get_align_key(key_request).map_err(|ret| {
            if ret != sgx_status_t::SGX_ERROR_OUT_OF_MEMORY {
                sgx_status_t::SGX_ERROR_UNEXPECTED
            } else {
                ret
            }
        })?;

        let mut sealed_data = SgxInternalSealedData::default();
        sealed_data.payload_data.encrypt = vec![0_u8; encrypt_text.len()].into_boxed_slice();

        let error = rsgx_rijndael128GCM_encrypt(
            &seal_key.key,
            encrypt_text,
            payload_iv,
            additional_text,
            &mut sealed_data.payload_data.encrypt,
            &mut sealed_data.payload_data.payload_tag,
        );
        if let Err(e) = error {
            seal_key.key = sgx_key_128bit_t::default();
            return Err(e);
        }

        sealed_data.payload_data.payload_size = (encrypt_text.len() + additional_text.len()) as u32;
        if !additional_text.is_empty() {
            sealed_data.payload_data.additional = additional_text.to_vec().into_boxed_slice();
        }

        seal_key.key = sgx_key_128bit_t::default();

        Ok(sealed_data)
    }

    fn unseal_data_helper(&self) -> SgxResult<SgxInternalUnsealedData> {
        let mut seal_key = rsgx_get_align_key(self.get_key_request()).map_err(|ret| {
            if (ret == sgx_status_t::SGX_ERROR_INVALID_CPUSVN)
                || (ret == sgx_status_t::SGX_ERROR_INVALID_ISVSVN)
                || (ret == sgx_status_t::SGX_ERROR_OUT_OF_MEMORY)
            {
                ret
            } else {
                sgx_status_t::SGX_ERROR_MAC_MISMATCH
            }
        })?;

        //
        // code that calls sgx_unseal_data commonly does some sanity checks
        // related to plain_text_offset.  We add fence here since we don't
        // know what crypto code does and if plain_text_offset-related
        // checks mispredict the crypto code could operate on unintended data
        //
        rsgx_lfence();

        let payload_iv = [0_u8; SGX_SEAL_IV_SIZE];
        let mut unsealed_data = SgxInternalUnsealedData {
            decrypt: vec![0_u8; self.payload_data.encrypt.len()].into_boxed_slice(),
            ..Default::default()
        };

        let error = rsgx_rijndael128GCM_decrypt(
            &seal_key.key,
            self.get_encrypt_txt(),
            &payload_iv,
            self.get_additional_txt(),
            self.get_payload_tag(),
            &mut unsealed_data.decrypt,
        );
        if let Err(e) = error {
            seal_key.key = sgx_key_128bit_t::default();
            return Err(e);
        }

        if self.payload_data.additional.len() > 0 {
            unsealed_data.additional = self.get_additional_txt().to_vec().into_boxed_slice();
        }
        unsealed_data.payload_size = self.get_payload_size();

        seal_key.key = sgx_key_128bit_t::default();

        Ok(unsealed_data)
    }
}
