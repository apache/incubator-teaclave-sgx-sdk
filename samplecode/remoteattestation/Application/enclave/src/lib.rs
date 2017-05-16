/*
 * Copyright (C) 2011-2016 2017 Baidu, Inc. All Rights Reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 *   * Redistributions of source code must retain the above copyright
 *     notice, this list of conditions and the following disclaimer.
 *   * Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in
 *     the documentation and/or other materials provided with the
 *     distribution.
 *   * Neither the name of Baidu, Inc., nor the names of its
 *     contributors may be used to endorse or promote products derived
 *     from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 */
#![crate_name = "raenclave"]
#![crate_type = "staticlib"]

#![no_std]
#![feature(collections)]

#[macro_use]
extern crate collections;

extern crate sgx_types;
extern crate sgx_tdh;
extern crate sgx_tcrypto;
extern crate sgx_tservice;
extern crate sgx_tkey_exchange;

use sgx_types::*;
use sgx_tcrypto::*;
use sgx_tservice::*;
use sgx_tkey_exchange::*;
use collections::slice;
use collections::vec::Vec;

const G_SP_PUB_KEY : sgx_ec256_public_t = sgx_ec256_public_t {
    gx : [0x72, 0x12, 0x8a, 0x7a, 0x17, 0x52, 0x6e, 0xbf,
          0x85, 0xd0, 0x3a, 0x62, 0x37, 0x30, 0xae, 0xad,
          0x3e, 0x3d, 0xaa, 0xee, 0x9c, 0x60, 0x73, 0x1d,
          0xb0, 0x5b, 0xe8, 0x62, 0x1c, 0x4b, 0xeb, 0x38],
    gy : [0xd4, 0x81, 0x40, 0xd9, 0x50, 0xe2, 0x57, 0x7b,
          0x26, 0xee, 0xb7, 0x41, 0xe7, 0xc6, 0x14, 0xe2,
          0x24, 0xb7, 0xbd, 0xc9, 0x03, 0xf2, 0x9a, 0x28,
          0xa8, 0x3c, 0xc8, 0x10, 0x11, 0x14, 0x5e, 0x06]
};

#[no_mangle]
pub extern "C"
fn enclave_init_ra(b_pse : i32,
                   p_context : &mut sgx_ra_context_t)
                   -> sgx_status_t {
    let mut ret:sgx_status_t = sgx_status_t::SGX_SUCCESS;

    if b_pse != 0 {
        for _ in 0..2 {
            match rsgx_create_pse_session() {
                Ok(()) => {
                    ret = sgx_status_t::SGX_SUCCESS;
                    break;
                },
                Err(x) => {ret = x;}
            }

        }

        if ret != sgx_status_t::SGX_SUCCESS {
            return ret;
        }
    }

    match rsgx_ra_init(&G_SP_PUB_KEY, b_pse) {
        Ok(p) => {
            *p_context = p; 
            ret = sgx_status_t::SGX_SUCCESS;
        },
        Err(x) => {
            ret = x;
            return ret;
        }
    }

    if b_pse != 0 {
        let _ = rsgx_close_pse_session();
    }
    ret
}

#[no_mangle]
pub extern "C"
fn enclave_ra_close(context : sgx_ra_context_t) -> sgx_status_t {
    match rsgx_ra_close(context) {
        Ok(()) => sgx_status_t::SGX_SUCCESS,
        Err(x) => x
    }
}

#[no_mangle]
pub extern "C"
fn verify_att_result_mac(context : sgx_ra_context_t,
                         message : * const u8,
                         msg_size: size_t,
                         mac     : * const u8,
                         mac_size: size_t) -> sgx_status_t {

    let ret:sgx_status_t;
    let mk_key:sgx_ec_key_128bit_t;
    let mac_slice;
    let message_slice;
    let mac_result:sgx_cmac_128bit_tag_t;

    if mac_size != SGX_MAC_SIZE || msg_size > u32::max_value as usize {
        ret = sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
        return ret;
    }

    unsafe {
        mac_slice = slice::from_raw_parts(mac, mac_size as usize);
        message_slice = slice::from_raw_parts(message, msg_size as usize);
    }

    if mac_slice.len() != SGX_MAC_SIZE as usize  ||
       message_slice.len() != msg_size as usize {
        ret = sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
        return ret;
    }

    match rsgx_ra_get_keys(context, sgx_ra_key_type_t::SGX_RA_KEY_MK) {
        Ok(k) => mk_key = k,
        Err(x) => return x
    }

    match rsgx_rijndael128_cmac_slice(&mk_key, message_slice) {
        Ok(tag) => mac_result = tag,
        Err(x) => return x
    }

    let mut diff:u8 = 0;

    // consttime_memequal
    for i in 0..SGX_CMAC_MAC_SIZE {
        diff |= mac_slice[i] ^ mac_result[i];
    }

    if diff != 0 {
        ret = sgx_status_t::SGX_ERROR_MAC_MISMATCH;
        return ret;
    }

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C"
fn verify_secret_data(context : sgx_ra_context_t,
                      p_secret: * const u8,
                      sec_size: u32,
                      gcm_mac : &[u8;16],
                      max_vlen: u32,
                      p_ret   : & mut [u8;16]) -> sgx_status_t {

    let ret:sgx_status_t;
    let sk_key:sgx_ec_key_128bit_t;
    
    match rsgx_ra_get_keys(context, sgx_ra_key_type_t::SGX_RA_KEY_SK) {
        Ok(key) => sk_key = key,
        Err(x) => return x
    }

    let secret_slice;
//    let gcm_mac_slice;

    unsafe {
        secret_slice = slice::from_raw_parts(p_secret, sec_size as usize);
//        gcm_mac_slice = slice::from_raw_parts(gcm_mac, SGX_AESGCM_MAC_SIZE);
    }

    if secret_slice.len() != sec_size as usize {
//       gcm_mac_slice.len() != SGX_AESGCM_MAC_SIZE {
        ret = sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
        return ret;
    }

    let mut decrypted_vec: Vec<u8> = vec![0; sec_size as usize];
    let mut decrypted_slice = &mut decrypted_vec[..];
    let iv = [0;12];
    let aad:[u8;0] = [0;0];

//    let debug_str = fmt!("{}", aad.len());
//
//    unsafe {
//        ocall_print_string(debug_str as *const c_uchar, debug_str.len() as size_t);
//    }

    let ret = rsgx_rijndael128GCM_decrypt(&sk_key,
                                          &secret_slice,
                                          &iv,
                                          &aad,
                                          gcm_mac,
                                          decrypted_slice);

    match ret {
        Ok(()) => {
            if decrypted_slice[0] == 0 && decrypted_slice[1] != 1 {
                return sgx_status_t::SGX_ERROR_INVALID_SIGNATURE;
            }
            else {
                return sgx_status_t::SGX_SUCCESS;
            }
        },
        Err(_) => {
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }

    }
}
