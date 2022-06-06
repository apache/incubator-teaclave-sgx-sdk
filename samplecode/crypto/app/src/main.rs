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

#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::print_literal)]

extern crate sgx_types;
extern crate sgx_urts;

use itertools::Itertools;
use sgx_types::error::SgxStatus;
use sgx_types::types::*;
use sgx_urts::enclave::SgxEnclave;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern "C" {
    fn sha256(
        eid: EnclaveId,
        retval: *mut SgxStatus,
        s: *const u8,
        len: usize,
        output_hash: &mut [u8; 32],
    ) -> SgxStatus;
    fn aes_gcm_128_encrypt(
        eid: EnclaveId,
        retval: *mut SgxStatus,
        key: &[u8; 16],
        plaintext: *const u8,
        text_len: usize,
        iv: &[u8; 12],
        ciphertext: *mut u8,
        mac: &mut [u8; 16],
    ) -> SgxStatus;
    fn aes_gcm_128_decrypt(
        eid: EnclaveId,
        retval: *mut SgxStatus,
        key: &[u8; 16],
        ciphertext: *const u8,
        text_len: usize,
        iv: &[u8; 12],
        mac: &[u8; 16],
        plaintext: *mut u8,
    ) -> SgxStatus;
    fn aes_cmac(
        eid: EnclaveId,
        retval: *mut SgxStatus,
        text: *const u8,
        text_len: usize,
        key: &[u8; 16],
        cmac: &mut [u8; 16],
    ) -> SgxStatus;
    fn rsa2048(
        eid: EnclaveId,
        retval: *mut SgxStatus,
        text: *const u8,
        text_len: usize,
    ) -> SgxStatus;
}

fn main() {
    let enclave = match SgxEnclave::create(ENCLAVE_FILE, true) {
        Ok(enclave) => {
            println!("[+] Init Enclave Successful {}!", enclave.eid());
            enclave
        }
        Err(err) => {
            println!("[-] Init Enclave Failed {}!", err.as_str());
            return;
        }
    };

    run_sha256(&enclave);
    run_aes_gcm_128(&enclave);
    run_aes_cmac(&enclave);
    run_rsa2048(&enclave);
}

fn run_sha256(enclave: &SgxEnclave) {
    let mut retval = SgxStatus::Success;
    let mut output_hash = [0_u8; SHA256_HASH_SIZE];

    let p: *const u8 = b"abc" as *const u8;

    println!("[+] sha256 input string is {}", "abc");
    println!(
        "[+] Expected SHA256 hash: {}",
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let result = unsafe { sha256(enclave.eid(), &mut retval, p, 3, &mut output_hash) };

    match result {
        SgxStatus::Success => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return;
        }
    }

    println!("[+] SHA256 result is {:02x}", output_hash.iter().format(""));
    println!("[+] calc_sha256 success ...");
}

fn run_aes_gcm_128(enclave: &SgxEnclave) {
    let mut retval = SgxStatus::Success;

    println!("[+] Starting aes-gcm-128 encrypt calculation");
    println!("[+] aes-gcm-128 args prepared!");
    println!(
        "[+] aes-gcm-128 expected ciphertext: {}",
        "0388dace60b6a392f328c2b971b2fe78"
    );
    let aes_gcm_plaintext = [0_u8; 16];
    let mut aes_gcm_ciphertext = [0_u8; 16];
    let aes_gcm_key = [0_u8; KEY_128BIT_SIZE];
    let aes_gcm_iv = [0_u8; AESGCM_IV_SIZE];
    let mut aes_gcm_mac = [0_u8; MAC_128BIT_SIZE];

    let result = unsafe {
        aes_gcm_128_encrypt(
            enclave.eid(),
            &mut retval,
            &aes_gcm_key,
            aes_gcm_plaintext.as_ptr(),
            aes_gcm_plaintext.len(),
            &aes_gcm_iv,
            aes_gcm_ciphertext.as_mut_ptr(),
            &mut aes_gcm_mac,
        )
    };
    println!("[+] aes-gcm-128 returned from enclave!");
    match result {
        SgxStatus::Success => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return;
        }
    }

    println!(
        "[+] aes-gcm-128 ciphnertext is: {:02x}",
        aes_gcm_ciphertext.iter().format("")
    );
    println!(
        "[+] aes-gcm-128 result mac is: {:02x}",
        aes_gcm_mac.iter().format("")
    );

    let mut aes_gcm_decrypted_text = [0_u8; 16];
    let result = unsafe {
        aes_gcm_128_decrypt(
            enclave.eid(),
            &mut retval,
            &aes_gcm_key,
            aes_gcm_ciphertext.as_ptr(),
            aes_gcm_ciphertext.len(),
            &aes_gcm_iv,
            &aes_gcm_mac,
            aes_gcm_decrypted_text.as_mut_ptr(),
        )
    };
    match result {
        SgxStatus::Success => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return;
        }
    }

    println!(
        "[+] aes-gcm-128 decrypted plaintext is: {:02x}",
        aes_gcm_decrypted_text.iter().format("")
    );
    println!("[+] aes-gcm-128 decrypted complete");
}

fn run_aes_cmac(enclave: &SgxEnclave) {
    println!("[+] Starting aes-cmac test");
    println!(
        "[+] aes-cmac expected digest: {}",
        "51f0bebf7e3b9d92fc49741779363cfe"
    );

    let mut retval = SgxStatus::Success;
    let cmac_key: [u8; KEY_128BIT_SIZE] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
        0x3c,
    ];

    let cmac_msg = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17,
        0x2a, 0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf,
        0x8e, 0x51, 0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb, 0xc1, 0x19, 0x1a,
        0x0a, 0x52, 0xef, 0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b,
        0xe6, 0x6c, 0x37, 0x10,
    ];

    let mut cmac = [0_u8; MAC_128BIT_SIZE];
    let result = unsafe {
        aes_cmac(
            enclave.eid(),
            &mut retval,
            cmac_msg.as_ptr(),
            cmac_msg.len(),
            &cmac_key,
            &mut cmac,
        )
    };
    match result {
        SgxStatus::Success => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return;
        }
    }

    println!("[+] aes-cmac result is: {:2x}", cmac.iter().format(""));
}

fn run_rsa2048(enclave: &SgxEnclave) {
    println!("[+] Starting rsa2048 test");

    let mut retval = SgxStatus::Success;
    let rsa_msg = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17,
        0x2a, 0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf,
        0x8e, 0x51, 0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb, 0xc1, 0x19, 0x1a,
        0x0a, 0x52, 0xef, 0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b,
        0xe6, 0x6c, 0x37, 0x10, 0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e,
        0x11, 0x73, 0x93, 0x17, 0x2a, 0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7,
        0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51, 0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5,
        0xfb, 0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef, 0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17,
        0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10,
    ];

    let result = unsafe { rsa2048(enclave.eid(), &mut retval, rsa_msg.as_ptr(), rsa_msg.len()) };
    match result {
        SgxStatus::Success => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return;
        }
    }

    println!("[+] rsa2048 success");
}
