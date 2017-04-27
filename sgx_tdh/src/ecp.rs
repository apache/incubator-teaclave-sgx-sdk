// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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
use sgx_tcrypto::*;


pub const EC_LABEL_LENGTH: usize = 3;
pub const EC_SMK_LABEL: [u8; EC_LABEL_LENGTH] = [0x53, 0x4D, 0x4B];
pub const EC_AEK_LABEL: [u8; EC_LABEL_LENGTH] = [0x41, 0x45, 0x4B];
pub const EC_DERIVATION_BUFFER_SIZE: usize = 7;

pub fn derive_key(shared_key: &sgx_ec256_dh_shared_t, 
                  label: &[u8; EC_LABEL_LENGTH]) -> SgxResult<sgx_ec_key_128bit_t> {

    let cmac_key = sgx_cmac_128bit_key_t::default();
    let mut key_derive_key = try!(rsgx_rijndael128_cmac_msg(&cmac_key, shared_key).map_err(set_error));

    //derivation_buffer = counter(0x01) || label || 0x00 || output_key_len(0x0080)
    let mut derivation_buffer = [0_u8; EC_DERIVATION_BUFFER_SIZE];
    derivation_buffer[0] = 0x01;
    derivation_buffer[1] = label[0];
    derivation_buffer[2] = label[1];
    derivation_buffer[3] = label[2];
    derivation_buffer[4] = 0x00;
    derivation_buffer[5] = 0x80;
    derivation_buffer[6] = 0x00;

    let result = rsgx_rijndael128_cmac_slice(&key_derive_key, &derivation_buffer).map_err(set_error);
    key_derive_key = Default::default();
    result
}

fn set_error(sgx_ret: sgx_status_t) -> sgx_status_t {

    let ret = match sgx_ret {
        sgx_status_t::SGX_ERROR_OUT_OF_MEMORY => sgx_status_t::SGX_ERROR_OUT_OF_MEMORY,
        _ => sgx_status_t::SGX_ERROR_UNEXPECTED,
    };
    ret   
}