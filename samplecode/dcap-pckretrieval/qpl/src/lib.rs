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

#![allow(non_snake_case)]

extern crate sgx_types;
#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;
use sgx_types::*;

//const MAX_URL_LENGTH: usize = 2083;
const QE3_ID_SIZE: usize = 16;
const ENC_PPID_SIZE: usize = 384;
const CPUSVN_SIZE: usize = 16;
const PCESVN_SIZE: usize = 2;
const PCEID_SIZE: usize = 2;
//const FMSPC_SIZE: usize = 6;
const MIN_CERT_DATA_SIZE: usize = 500;

#[no_mangle]
pub extern "C" fn sgx_ql_free_quote_config(
    p_quote_config: *mut sgx_ql_config_t,
) -> sgx_quote3_error_t {
    //println!("sgx_ql_free_quote_config: free {:p}", p_quote_config);
    if !p_quote_config.is_null() {
        let p_cert_data = unsafe { (*p_quote_config).p_cert_data };
        if !p_cert_data.is_null() {
            let _s = unsafe { std::slice::from_raw_parts(p_cert_data, (*p_quote_config).cert_data_size as usize) };
            drop(_s);// this is done implicitly. the explicit drop here is just for demon purpose
        }
        let _b: Box<sgx_ql_config_t> = unsafe { Box::from_raw(p_quote_config) };
        drop(_b);// this is done implicitly. the explicit drop here is just for demon purpose
    }
    sgx_quote3_error_t::SGX_QL_SUCCESS
}

// The original sgx_ql_get_quote_config is not mt-safe. It writes to a global mutable array
// `encrypted_ppid` and read from it later.
// In this impl, we use a Mutex to guard the global `encrypted_ppid`.
// The calling sequence is 

lazy_static! {
    static ref ENCRYPTED_PPID: Mutex<[u8;ENC_PPID_SIZE]> = Mutex::new([0;ENC_PPID_SIZE]);
}

#[no_mangle]
pub extern "C" fn sgx_ql_get_quote_config(
    p_cert_id: *const sgx_ql_pck_cert_id_t,
    pp_quote_config: *mut *mut sgx_ql_config_t,
) -> sgx_quote3_error_t {
    //println!("sgx_ql_get_quote_config: {:p}", p_cert_id);

    if p_cert_id.is_null() || pp_quote_config.is_null() {
        return sgx_quote3_error_t::SGX_QL_ERROR_INVALID_PARAMETER;
    }

    if unsafe { (*p_cert_id).p_qe3_id }.is_null()
        || unsafe { (*p_cert_id).qe3_id_size } != QE3_ID_SIZE as u32
        || unsafe { (*p_cert_id).p_platform_cpu_svn }.is_null()
        || unsafe { (*p_cert_id).p_platform_pce_isv_svn }.is_null()
        || unsafe { (*p_cert_id).crypto_suite } != PCE_ALG_RSA_OAEP_3072
    {
        return sgx_quote3_error_t::SGX_QL_ERROR_INVALID_PARAMETER;
    }

    let encrypted_ppid: [u8; ENC_PPID_SIZE] = if !unsafe { (*p_cert_id).p_encrypted_ppid }.is_null()
    {
        if unsafe { (*p_cert_id).encrypted_ppid_size } != ENC_PPID_SIZE as u32 {
            return sgx_quote3_error_t::SGX_QL_ERROR_INVALID_PARAMETER;
        } else {
            let mut eppid = [0; ENC_PPID_SIZE];
            unsafe {
                let p: *const u8 = (*p_cert_id).p_encrypted_ppid as *const u8;
                p.copy_to_nonoverlapping(eppid.as_mut_ptr(), ENC_PPID_SIZE);
            }

            if let Ok(mut l) = ENCRYPTED_PPID.lock() {
                *l = eppid;
            }

            eppid
        }
    } else {
        *ENCRYPTED_PPID.lock().unwrap()
    };

    let version:sgx_ql_config_version_t = sgx_ql_config_version_t::SGX_QL_CONFIG_VERSION_1;
    let cert_cpu_svn:sgx_cpu_svn_t = unsafe { *(*p_cert_id).p_platform_cpu_svn };
    let cert_pce_isv_svn: sgx_isv_svn_t = unsafe { *(*p_cert_id).p_platform_pce_isv_svn};
    // previously we asserted enc_ppid_size = ENC_PPID_SIZE, qe3_id_size = QE3_ID_SIZE
    // so the sum here is smaller than MIN_CERT_DATA_SIZE. cert_data_size is MIN_CERT_DATA_SIZE
    let cert_data_size: uint32_t = std::cmp::max(
        ENC_PPID_SIZE + QE3_ID_SIZE + PCEID_SIZE + CPUSVN_SIZE + PCESVN_SIZE,
        MIN_CERT_DATA_SIZE) as u32;

    // cert data is:
    // ENC_PPID || PCEID || CPUSVN || PCESVN || QEID || 0x00...
    let pce_id: [u8;PCEID_SIZE] = unsafe { (*p_cert_id).pce_id }.to_le_bytes();
    let cpu_svn: [u8; CPUSVN_SIZE] = unsafe { *(*p_cert_id).p_platform_cpu_svn }.svn;
    let pce_svn: [u8; PCESVN_SIZE] = unsafe { *(*p_cert_id).p_platform_pce_isv_svn}.to_le_bytes();
    let qe_id: &[u8] = unsafe { std::slice::from_raw_parts((*p_cert_id).p_qe3_id, QE3_ID_SIZE) };

    let mut cert_data_vec: Vec<u8> = encrypted_ppid.to_vec();
    cert_data_vec.extend_from_slice(&pce_id[..]);
    cert_data_vec.extend_from_slice(&cpu_svn[..]);
    cert_data_vec.extend_from_slice(&pce_svn[..]);
    cert_data_vec.extend_from_slice(qe_id);

    cert_data_vec.resize_with(cert_data_size as usize, Default::default);

    let mut b = cert_data_vec.into_boxed_slice();
    let p_cert_data = b.as_mut_ptr();
    let _ = Box::into_raw(b); // memory leak here.

    let ql_config = sgx_ql_config_t {
        version: version,
        cert_cpu_svn: cert_cpu_svn,
        cert_pce_isv_svn: cert_pce_isv_svn,
        cert_data_size: cert_data_size,
        p_cert_data: p_cert_data,
    };

    let p_ret_ql_config = Box::into_raw(Box::new(ql_config));

    unsafe {
        *pp_quote_config = p_ret_ql_config;
    }

    sgx_quote3_error_t::SGX_QL_SUCCESS
}
