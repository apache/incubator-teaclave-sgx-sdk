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
// under the License.

extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::error::SgxStatus;
use sgx_types::types::*;
use sgx_urts::enclave::SgxEnclave;
use std::ffi::CString;

static ENCLAVE_FILE: &str = "enclave.signed.so";

extern "C" {
    fn send_http_request(
        eid: EnclaveId,
        retval: *mut SgxStatus,
        hostname: *const c_char,
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

    let hostname = "www.rust-lang.org";
    let port = 443;

    let hostname = format!("https://{}:{}", hostname, port);
    let c_hostname = CString::new(hostname.to_string()).unwrap();

    let mut retval = SgxStatus::Success;
    let result = unsafe { send_http_request(enclave.eid(), &mut retval, c_hostname.as_ptr()) };
    match result {
        SgxStatus::Success => println!("[+] ECall Success..."),
        _ => println!("[-] ECall Enclave Failed {}!", result.as_str()),
    }
}
