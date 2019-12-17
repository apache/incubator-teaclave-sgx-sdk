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

#![crate_name = "staticdatadistribution"]
#![crate_type = "staticlib"]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate serde_json;
extern crate sgx_crypto_helper;

pub const KEYFILE: &'static str = "prov_key.bin";

use sgx_types::*;
use std::io::{self, Read, Write};
use std::prelude::v1::*;
use std::sgxfs::SgxFile;
use std::slice;
use std::string::String;
use std::vec::Vec;

use sgx_crypto_helper::RsaKeyPair;
use sgx_crypto_helper::rsa3072::Rsa3072KeyPair;

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {
    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    // A sample &'static string
    let rust_raw_string = "This is a in-Enclave ";
    // An array
    let word: [u8; 4] = [82, 117, 115, 116];
    // An vector
    let word_vec: Vec<u8> = vec![32, 115, 116, 114, 105, 110, 103, 33];

    // Construct a string from &'static string
    let mut hello_string = String::from(rust_raw_string);

    // Iterate on word array
    for c in word.iter() {
        hello_string.push(*c as char);
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8").as_str();

    // Ocall to normal world for output
    //println!("{}", &hello_string);
    let mut keyvec: Vec<u8> = Vec::new();

    let key_json_str = match SgxFile::open(KEYFILE) {
        Ok(mut f) => match f.read_to_end(&mut keyvec) {
            Ok(len) => {
                println!("Read {} bytes from Key file", len);
                std::str::from_utf8(&keyvec).unwrap()
            }
            Err(x) => {
                println!("Read keyfile failed {}", x);
                return sgx_status_t::SGX_ERROR_UNEXPECTED;
            }
        },
        Err(x) => {
            println!("get_sealed_pcl_key cannot open keyfile, please check if key is provisioned successfully! {}", x);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };
    //println!("key_json = {}", key_json_str);
    let rsa_keypair: Rsa3072KeyPair = serde_json::from_str(&key_json_str).unwrap();
    //println!("Recovered key = {:?}", rsa_keypair);

    let mut ciphertext_bin = Vec::new();
    match std::untrusted::fs::File::open("static_data.bin") {
        Ok(mut f) => match f.read_to_end(&mut ciphertext_bin) {
            Ok(len) => {
                println!("Read {} bytes from static data", len);
            }
            Err(x) => {
                println!("Read static data failed {}", x);
                return sgx_status_t::SGX_ERROR_UNEXPECTED;
            }
        },
        Err(x) => {
            println!("cannot open static data file! {}", x);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let mut plaintext = Vec::new();
    rsa_keypair.decrypt_buffer(&ciphertext_bin, &mut plaintext).unwrap();

    let decrypted_string = String::from_utf8(plaintext).unwrap();
    println!("decrypted data = {}", decrypted_string);
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn fake_provisioning(key_ptr: *const u8, some_len: usize) -> sgx_status_t {
    let key_slice = unsafe { slice::from_raw_parts(key_ptr, some_len) };
    //let key_str = std::str::from_utf8(key_slice).unwrap();
    //let keys:Rsa2048KeyPair = serde_json::from_str(&key_str).unwrap();

    //println!("Received keys: {:?}", keys);
    match SgxFile::create(KEYFILE) {
        Ok(mut f) => match f.write_all(key_slice) {
            Ok(()) => {
                println!("SgxFile write key file success!");
                sgx_status_t::SGX_SUCCESS
            }
            Err(x) => {
                println!("SgxFile write key file failed! {}", x);
                sgx_status_t::SGX_ERROR_UNEXPECTED
            }
        },
        Err(x) => {
            println!("SgxFile create file {} error {}", KEYFILE, x);
            sgx_status_t::SGX_ERROR_UNEXPECTED
        }
    }
}
