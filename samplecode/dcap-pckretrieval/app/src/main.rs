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

extern crate itertools;
extern crate libloading;
extern crate sgx_types;
extern crate sgx_urts;
use itertools::*;
use sgx_types::*;
use sgx_urts::SgxEnclave;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern "C" {
    fn enclave_create_report(
        eid: sgx_enclave_id_t,
        retval: *mut i32,
        p_qe3_target: &sgx_target_info_t,
        p_report: *mut sgx_report_t,
    ) -> sgx_status_t;
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 0;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    SgxEnclave::create(
        ENCLAVE_FILE,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )
}

fn main() {
    // quote holds the generated quote
    let quote: Vec<u8> = generate_quote().unwrap();

    // this quote has type `sgx_quote3_t` and is structured as:
    // sgx_quote3_t {
    //     header: sgx_quote_header_t,
    //     report_body: sgx_report_body_t,
    //     signature_data_len: uint32_t,  // 1116
    //     signature_data {               // 1116 bytes payload
    //         sig_data: sgx_ql_ecdsa_sig_data_t { // 576 = 64x3 +384 header
    //             sig: [uint8_t; 64],
    //             attest_pub_key: [uint8_t; 64],
    //             qe3_report: sgx_report_body_t, //  384
    //             qe3_report_sig: [uint8_t; 64],
    //             auth_certification_data { // 2 + 32 = 34
    //                 sgx_ql_auth_data_t: u16 // observed 32, size of following auth_data
    //                 auth_data: [u8; sgx_ql_auth_data_t]
    //             }
    //             sgx_ql_certification_data_t {/ 2 + 4 + 500
    //                 cert_key_type: uint16_t,
    //                 size: uint32_t, // observed 500, size of following certificateion_data
    //                 certification_data { // 500 bytes
    //                 }
    //             }
    //         }
    //     }
    //  }
    let p_quote3: *const sgx_quote3_t = quote.as_ptr() as *const sgx_quote3_t;

    // copy heading bytes to a sgx_quote3_t type to simplify access
    let quote3: sgx_quote3_t = unsafe { *p_quote3 };

    let quote_signature_data_vec: Vec<u8> = quote[std::mem::size_of::<sgx_quote3_t>()..].into();

    //println!("quote3 header says signature data len = {}", quote3.signature_data_len);
    //println!("quote_signature_data len = {}", quote_signature_data_vec.len());

    assert_eq!(
        quote3.signature_data_len as usize,
        quote_signature_data_vec.len()
    );

    // signature_data has a header of sgx_ql_ecdsa_sig_data_t structure
    //let p_sig_data: * const sgx_ql_ecdsa_sig_data_t = quote_signature_data_vec.as_ptr() as _;
    // mem copy
    //let sig_data = unsafe { * p_sig_data };

    // sgx_ql_ecdsa_sig_data_t is followed by sgx_ql_auth_data_t
    // create a new vec for auth_data
    let auth_certification_data_offset = std::mem::size_of::<sgx_ql_ecdsa_sig_data_t>();
    let p_auth_data: *const sgx_ql_auth_data_t =
        (quote_signature_data_vec[auth_certification_data_offset..]).as_ptr() as _;
    let auth_data_header: sgx_ql_auth_data_t = unsafe { *p_auth_data };
    //println!("auth_data len = {}", auth_data_header.size);

    let auth_data_offset =
        auth_certification_data_offset + std::mem::size_of::<sgx_ql_auth_data_t>();

    // It should be [0,1,2,3...]
    // defined at https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/4605fae1c606de4ff1191719433f77f050f1c33c/QuoteGeneration/quote_wrapper/quote/qe_logic.cpp#L1452
    //let auth_data_vec: Vec<u8> = quote_signature_data_vec[auth_data_offset..auth_data_offset + auth_data_header.size as usize].into();
    //println!("Auth data:\n{:?}", auth_data_vec);

    let temp_cert_data_offset = auth_data_offset + auth_data_header.size as usize;
    let p_temp_cert_data: *const sgx_ql_certification_data_t =
        quote_signature_data_vec[temp_cert_data_offset..].as_ptr() as _;
    let temp_cert_data: sgx_ql_certification_data_t = unsafe { *p_temp_cert_data };

    //println!("certification data offset = {}", temp_cert_data_offset);
    //println!("certification data size = {}", temp_cert_data.size);

    let cert_info_offset =
        temp_cert_data_offset + std::mem::size_of::<sgx_ql_certification_data_t>();

    //println!("cert info offset = {}", cert_info_offset);
    // this should be the last structure
    assert_eq!(
        quote_signature_data_vec.len(),
        cert_info_offset + temp_cert_data.size as usize
    );

    let tail_content = quote_signature_data_vec[cert_info_offset..].to_vec();
    let enc_ppid_len = 384;
    let enc_ppid: &[u8] = &tail_content[0..enc_ppid_len];
    let pce_id: &[u8] = &tail_content[enc_ppid_len..enc_ppid_len + 2];
    let cpu_svn: &[u8] = &tail_content[enc_ppid_len + 2..enc_ppid_len + 2 + 16];
    let pce_isvsvn: &[u8] = &tail_content[enc_ppid_len + 2 + 16..enc_ppid_len + 2 + 18];
    println!("EncPPID:\n{:02x}", enc_ppid.iter().format(""));
    println!("PCE_ID:\n{:02x}", pce_id.iter().format(""));
    println!("TCBr - CPUSVN:\n{:02x}", cpu_svn.iter().format(""));
    println!("TCBr - PCE_ISVSVN:\n{:02x}", pce_isvsvn.iter().format(""));
    println!("QE_ID:\n{:02x}", quote3.header.user_data.iter().format(""));
}

// Re-invent App/utility.cpp
// int generate_quote(uint8_t **quote_buffer, uint32_t& quote_size)
fn generate_quote() -> Option<Vec<u8>> {
    let mut ti: sgx_target_info_t = sgx_target_info_t::default();

    let _l = unsafe { libloading::Library::new("./libdcap_quoteprov.so.1").unwrap() };
    println!("Step1: Call sgx_qe_get_target_info:");
    //println!("sgx_qe_get_target_info = {:p}", sgx_qe_get_target_info as * const _);

    let qe3_ret = unsafe { sgx_qe_get_target_info(&mut ti as *mut _) };

    if qe3_ret != sgx_quote3_error_t::SGX_QL_SUCCESS {
        println!("Error in sgx_qe_get_target_info. {:?}\n", qe3_ret);
        return None;
    }

    //println!("target_info.mr_enclave = {:?}", ti.mr_enclave.m);
    //println!("target_info.config_id = {:02x}", ti.config_id.iter().format(" "));

    let quote_size = std::mem::size_of::<sgx_target_info_t>();
    let mut v: Vec<u8> = vec![0; quote_size];
    unsafe {
        std::ptr::copy_nonoverlapping(
            &ti as *const sgx_target_info_t as *const u8,
            v.as_mut_ptr() as *mut u8,
            quote_size,
        );
    }

    //println!("quote = {:?}", v);

    println!("succeed!\nStep2: Call create_app_report:");
    let app_report: sgx_report_t = if let Some(r) = create_app_enclave_report(&ti) {
        println!("succeed! \nStep3: Call sgx_qe_get_quote_size:");
        r
    } else {
        println!("\nCall to create_app_report() failed\n");
        return None;
    };

    //println!("app_report.body.cpu_svn = {:02x}", app_report.body.cpu_svn.svn.iter().format(""));
    //println!("app_report.body.misc_select = {:08x}", app_report.body.misc_select);
    //println!("app_report.key_id = {:02x}", app_report.key_id.id.iter().format(""));
    //println!("app_report.mac = {:02x}", app_report.mac.iter().format(""));

    let mut quote_size: u32 = 0;
    let qe3_ret = unsafe { sgx_qe_get_quote_size(&mut quote_size as _) };

    if qe3_ret != sgx_quote3_error_t::SGX_QL_SUCCESS {
        println!("Error in sgx_qe_get_quote_size . {:?}\n", qe3_ret);
        return None;
    }

    println!("succeed!");

    let mut quote_vec: Vec<u8> = vec![0; quote_size as usize];

    println!("\nStep4: Call sgx_qe_get_quote:");

    let qe3_ret =
        unsafe { sgx_qe_get_quote(&app_report as _, quote_size, quote_vec.as_mut_ptr() as _) };

    if qe3_ret != sgx_quote3_error_t::SGX_QL_SUCCESS {
        println!("Error in sgx_qe_get_quote. {:?}\n", qe3_ret);
        return None;
    }

    Some(quote_vec)
}

fn create_app_enclave_report(qe_ti: &sgx_target_info_t) -> Option<sgx_report_t> {
    let enclave = if let Ok(r) = init_enclave() {
        r
    } else {
        return None;
    };

    let mut retval = 0;
    let mut ret_report: sgx_report_t = sgx_report_t::default();

    let result = unsafe {
        enclave_create_report(
            enclave.geteid(),
            &mut retval,
            qe_ti,
            &mut ret_report as *mut sgx_report_t,
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return None;
        }
    }
    enclave.destroy();
    Some(ret_report)
}
