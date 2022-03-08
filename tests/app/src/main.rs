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

extern crate raw_cpuid;
extern crate sgx_types;
extern crate sgx_urts;

use raw_cpuid::CpuId;
use sgx_types::error::SgxStatus;
use sgx_types::types::*;
use sgx_urts::enclave::SgxEnclave;
use std::env;

static ENCLAVE_FILE: &str = "enclave.signed.so";

extern "C" {
    fn run_test_cases(eid: EnclaveId, retval: *mut SgxStatus) -> SgxStatus;
    fn run_bench_cases(eid: EnclaveId, retval: *mut SgxStatus, freq: u64) -> SgxStatus;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaseType {
    Test,
    Bench,
}

fn usage() {
    println!("USAGE:\n\t./test test\n\t./test bench");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage();
        return;
    }

    let case_type = match args[1].as_str() {
        "test" => CaseType::Test,
        "bench" => CaseType::Bench,
        _ => {
            usage();
            return;
        }
    };

    let mut freq = 0_u64;
    if case_type == CaseType::Bench {
        let cpuid = CpuId::new();

        match SgxEnclave::mode() {
            EnclaveMode::Hw => {
                let sgx_info = match cpuid.get_sgx_info() {
                    Some(info) => info,
                    None => {
                        println!("[-] Get SGX information Failed!");
                        return;
                    }
                };
                if !sgx_info.has_sgx2() {
                    println!("[-] Benchmarking is only supported on SGX2 processors!");
                    return;
                }
            }
            EnclaveMode::Sim => (),
            EnclaveMode::Hyper => {
                println!("[-] Benchmarking is not supported in Hyper mode!");
                return;
            }
        }

        let freq_info = match cpuid.get_processor_frequency_info() {
            Some(info) => info,
            None => {
                println!("[-] Get processor frequency Failed!");
                return;
            }
        };
        freq = freq_info.processor_base_frequency() as u64;
    }

    let enclave = match SgxEnclave::create(ENCLAVE_FILE, true) {
        Ok(enclave) => enclave,
        Err(err) => {
            println!("[-] Init Enclave Failed {}!", err.as_str());
            return;
        }
    };

    let mut retval = SgxStatus::Success;
    let result = unsafe {
        match case_type {
            CaseType::Test => run_test_cases(enclave.eid(), &mut retval),
            CaseType::Bench => run_bench_cases(enclave.eid(), &mut retval, freq),
        }
    };

    match result {
        SgxStatus::Success => (),
        _ => println!("[-] ECALL Enclave Failed {}!", result.as_str()),
    }
}
