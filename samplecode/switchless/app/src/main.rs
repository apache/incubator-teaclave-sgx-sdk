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
use std::time::Instant;

static ENCLAVE_FILE: &str = "enclave.signed.so";

extern "C" {
    fn ecall_repeat_ocalls(eid: EnclaveId, nrepeats: u64, use_switchless: u32) -> SgxStatus;
    fn ecall_empty(eid: EnclaveId) -> SgxStatus;
    fn ecall_empty_switchless(eid: EnclaveId) -> SgxStatus;
}

#[no_mangle]
pub extern "C" fn ocall_empty() {}

#[no_mangle]
pub extern "C" fn ocall_empty_switchless() {}

const REPEATS: u64 = 500000;

fn main() {
    let enclave = match SgxEnclave::create_with_switchless(ENCLAVE_FILE, true, 2, 2) {
        Ok(enclave) => {
            println!("[+] Init Enclave Successful {}!", enclave.eid());
            enclave
        }
        Err(err) => {
            println!("[-] Init Enclave Failed {}!", err.as_str());
            return;
        }
    };

    println!("Running a benchmark that compares **ordinary** and **switchless** OCalls...");
    benchmark_empty_ocall(enclave.eid(), 0);
    benchmark_empty_ocall(enclave.eid(), 1);
    println!("Done.\n");

    println!("Running a benchmark that compares **ordinary** and **switchless** ECalls...");
    benchmark_empty_ecall(enclave.eid(), 0);
    benchmark_empty_ecall(enclave.eid(), 1);
    println!("Done.\n");

    println!("[+] ECall Success...");
}

fn benchmark_empty_ocall(eid: EnclaveId, is_switchless: u32) {
    let info = match is_switchless {
        0 => "ordinary",
        _ => "switchless",
    };

    println!(
        "Repeating an **{}** OCall that does nothing for {} times...",
        info, REPEATS
    );

    let start = Instant::now();
    let _ = unsafe { ecall_repeat_ocalls(eid, REPEATS, is_switchless) };
    let elapsed = start.elapsed();
    println!("Time elapsed {:?}", elapsed);
}

fn benchmark_empty_ecall(eid: EnclaveId, is_switchless: u32) {
    let info = match is_switchless {
        0 => "ordinary",
        _ => "switchless",
    };

    let ecall_fn: unsafe extern "C" fn(EnclaveId) -> SgxStatus = match is_switchless {
        0 => ecall_empty,
        _ => ecall_empty_switchless,
    };

    println!(
        "Repeating an **{}** ECall that does nothing for {} times...",
        info, REPEATS
    );

    let start = Instant::now();
    for _ in 0..REPEATS {
        let _ = unsafe { ecall_fn(eid) };
    }
    let elapsed = start.elapsed();
    println!("Time elapsed {:?}", elapsed);
}
