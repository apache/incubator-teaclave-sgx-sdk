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

use sgx_tcrypto::*;
use sgx_types::*;

pub const EC_LABEL_LENGTH: usize = 3;
pub const EC_SMK_LABEL: [u8; EC_LABEL_LENGTH] = [0x53, 0x4D, 0x4B];
pub const EC_AEK_LABEL: [u8; EC_LABEL_LENGTH] = [0x41, 0x45, 0x4B];
pub const EC_DERIVATION_BUFFER_SIZE: usize = 7;

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn derive_key(
    shared_key: &sgx_ec256_dh_shared_t,
    label: &[u8; EC_LABEL_LENGTH],
) -> SgxResult<sgx_align_key_128bit_t> {
    let cmac_key = sgx_cmac_128bit_key_t::default();
    let mut key_derive_key = rsgx_rijndael128_cmac_msg(&cmac_key, shared_key).map_err(set_error)?;

    //derivation_buffer = counter(0x01) || label || 0x00 || output_key_len(0x0080)
    let mut derivation_buffer = [0_u8; EC_DERIVATION_BUFFER_SIZE];
    derivation_buffer[0] = 0x01;
    derivation_buffer[1] = label[0];
    derivation_buffer[2] = label[1];
    derivation_buffer[3] = label[2];
    derivation_buffer[4] = 0x00;
    derivation_buffer[5] = 0x80;
    derivation_buffer[6] = 0x00;

    let result = rsgx_rijndael128_align_cmac_slice(&key_derive_key, &derivation_buffer)
        .map(|align_mac| {
            let mut align_key = sgx_align_key_128bit_t::default();
            align_key.key = align_mac.mac;
            align_key
        })
        .map_err(set_error);
    key_derive_key = Default::default();
    result
}

fn set_error(sgx_ret: sgx_status_t) -> sgx_status_t {
    match sgx_ret {
        sgx_status_t::SGX_ERROR_OUT_OF_MEMORY => sgx_status_t::SGX_ERROR_OUT_OF_MEMORY,
        _ => sgx_status_t::SGX_ERROR_UNEXPECTED,
    }
}
