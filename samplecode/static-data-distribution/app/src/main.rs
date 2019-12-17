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

extern crate serde_json;
extern crate sgx_crypto_helper;
extern crate sgx_types;
extern crate sgx_urts;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use sgx_crypto_helper::RsaKeyPair;
use sgx_crypto_helper::rsa3072::{Rsa3072KeyPair, Rsa3072PubKey};

static ENCLAVE_FILE: &'static str = "enclave.signed.so";
use std::io::Write;

extern "C" {
    fn say_something(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        some_string: *const u8,
        len: usize,
    ) -> sgx_status_t;
    fn fake_provisioning(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        key_ptr: *const u8,
        len: usize,
    ) -> sgx_status_t;
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    SgxEnclave::create(ENCLAVE_FILE,
                       debug,
                       &mut launch_token,
                       &mut launch_token_updated,
                       &mut misc_attr)
}

fn main() {
    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        }
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        }
    };

    // Step 1: Generate a pair of RSA key
    let rsa_keypair = Rsa3072KeyPair::new().unwrap();

    // Step 2: Provision it to an enclave. RA-TLS based solution is more practical.
    // The current solution is just for demo. Do not use it in production.
    let rsa_key_json = serde_json::to_string(&rsa_keypair).unwrap();

    let mut retval = sgx_status_t::SGX_SUCCESS;

    let result = unsafe {
        fake_provisioning(
            enclave.geteid(),
            &mut retval,
            rsa_key_json.as_ptr() as *const u8,
            rsa_key_json.len(),
        )
    };

    match result {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return;
        }
    }

    // Step 3: Generate a static data

    let text = String::from("Can you decrypt this").repeat(300);
    let text_slice = &text.into_bytes();

    let mut ciphertext = Vec::new();
    //match rsa_keypair.encrypt_buffer(text_slice, &mut ciphertext) {
    //    Ok(n) => println!("Generated payload {} bytes", n),
    //    Err(x) => println!("Error occured during encryption {}", x),
    //}

    let exported_pubkey: Rsa3072PubKey = rsa_keypair.export_pubkey().unwrap();
    let serialized_pubkey = serde_json::to_string(&exported_pubkey).unwrap();
    println!("exported pubkey = {}", serialized_pubkey);

    let imported_pubkey: Rsa3072PubKey = serde_json::from_str(&serialized_pubkey).unwrap();
    println!("imported pubkey = {:?}", imported_pubkey);
    match imported_pubkey.encrypt_buffer(text_slice, &mut ciphertext) {
        Ok(n) => println!("Generated payload {} bytes", n),
        Err(x) => println!("Error occured during encryption {}", x),
    }

    match std::fs::File::create("static_data.bin") {
        Ok(mut f) => {
            f.write_all(&ciphertext).unwrap();
            println!("File saved successfully!");
        }
        Err(x) => {
            println!("Create static_data.bin failed {}", x);
            return;
        }
    }

    let hello_string = "Hello world!".to_string();
    let result = unsafe {
        say_something(
            enclave.geteid(),
            &mut retval,
            hello_string.as_ptr() as *const u8,
            hello_string.len(),
        )
    };

    match result {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return;
        }
    }

    println!("[+] say_something success...");

    enclave.destroy();
}
