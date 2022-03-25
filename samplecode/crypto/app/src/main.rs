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

extern crate sgx_types;
extern crate sgx_urts;
use sgx_types::error::SgxStatus;
use sgx_types::types::*;
use sgx_urts::enclave::SgxEnclave;
use itertools::Itertools;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern "C" {
    fn calc_sha256(eid: EnclaveId, retval: *mut SgxStatus,
                   s: * const u8, len: usize, output_hash: &mut [u8;32]) -> SgxStatus;
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

    let mut retval = SgxStatus::Success;
    let mut output_hash: [u8; 32] = [0; 32];

    let p: * const u8 = b"abc" as * const u8;

    println!("[+] sha256 input string is {}", "abc");
    println!("[+] Expected SHA256 hash: {}", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    let result = unsafe {
        calc_sha256(enclave.eid(),
            &mut retval,
            p,
            3,
            &mut output_hash,
        )
    };

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
