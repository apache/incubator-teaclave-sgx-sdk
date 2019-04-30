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

extern crate dirs;
extern crate serde_json;
extern crate sgx_crypto_helper;
extern crate sgx_types;
extern crate sgx_urts;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::fs;
use std::io::{Read, Write};
use std::path;

use sgx_crypto_helper::RsaKeyPair;
use sgx_crypto_helper::rsa3072::{Rsa3072KeyPair, Rsa3072PubKey};

static ENCLAVE_FILE: &'static str = "enclave.signed.so";
static ENCLAVE_TOKEN: &'static str = "enclave.token";

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
    // Step 1: try to retrieve the launch token saved by last transaction
    //         if there is no token, then create a new one.
    //
    // try to get the token saved in $HOME */
    let mut home_dir = path::PathBuf::new();
    let use_token = match dirs::home_dir() {
        Some(path) => {
            println!("[+] Home dir is {}", path.display());
            home_dir = path;
            true
        }
        None => {
            println!("[-] Cannot get home dir");
            false
        }
    };

    let token_file: path::PathBuf = home_dir.join(ENCLAVE_TOKEN);;
    if use_token == true {
        match fs::File::open(&token_file) {
            Err(_) => {
                println!(
                    "[-] Open token file {} error! Will create one.",
                    token_file.as_path().to_str().unwrap()
                );
            }
            Ok(mut f) => {
                println!("[+] Open token file success! ");
                match f.read(&mut launch_token) {
                    Ok(1024) => {
                        println!("[+] Token file valid!");
                    }
                    _ => println!("[+] Token file invalid, will create new token file"),
                }
            }
        }
    }

    // Step 2: call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    let enclave = try!(SgxEnclave::create(
        ENCLAVE_FILE,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr
    ));

    // Step 3: save the launch token if it is updated
    if use_token == true && launch_token_updated != 0 {
        // reopen the file with write capablity
        match fs::File::create(&token_file) {
            Ok(mut f) => match f.write_all(&launch_token) {
                Ok(()) => println!("[+] Saved updated launch token!"),
                Err(_) => println!("[-] Failed to save updated launch token!"),
            },
            Err(_) => {
                println!("[-] Failed to save updated enclave token, but doesn't matter");
            }
        }
    }

    Ok(enclave)
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
