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

#![allow(unused_attributes)]

extern crate sgx_types;
extern crate sgx_urts;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::time::Instant;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern {
    fn ecall_repeat_ocalls(eid: sgx_enclave_id_t,
                           nrepeats: u64,
                           use_switchless: u32) -> sgx_status_t;

    fn ecall_empty(eid: sgx_enclave_id_t) -> sgx_status_t;
    fn ecall_empty_switchless(eid: sgx_enclave_id_t) -> sgx_status_t;
}

fn init_enclave(num_uworker : u32, num_tworker : u32) -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    SgxEnclave::create_with_workers(ENCLAVE_FILE,
                                    debug,
                                    &mut launch_token,
                                    &mut launch_token_updated,
                                    &mut misc_attr,
                                    num_uworker,
                                    num_tworker)
}

#[no_mangle]
pub extern "C"
fn ocall_empty() {
}

#[no_mangle]
pub extern "C"
fn ocall_empty_switchless() {
}

const REPEATS:u64 = 500000;

fn benchmark_empty_ocall(eid : sgx_enclave_id_t,
                         is_switchless : u32) {
    let info = match is_switchless {
        0 => "ordinary",
        _ => "switchless",
    };

    println!("Repeating an **{}** OCall that does nothing for {} times", info, REPEATS);

    let start = Instant::now();
    let _ = unsafe {
        ecall_repeat_ocalls(eid, REPEATS, is_switchless)
    };
    let elapsed = start.elapsed();
    println!("Time elapsed {:?}", elapsed);
}

fn benchmark_empty_ecall(eid : sgx_enclave_id_t,
                         is_switchless : u32) {
    let info = match is_switchless {
        0 => "ordinary",
        _ => "switchless",
    };

    let func : unsafe extern "C" fn(sgx_enclave_id_t) -> sgx_status_t = match is_switchless {
        0 => ecall_empty,
        _ => ecall_empty_switchless,
    };

    println!("Repeating an **{}** OCall that does nothing for {} times", info, REPEATS);

    let start = Instant::now();
    for _ in 0..REPEATS {
        let _ = unsafe {
            func(eid)
        };
    }
    let elapsed = start.elapsed();
    println!("Time elapsed {:?}", elapsed);
}

fn main() {

    let enclave = match init_enclave(2,2) {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        },
    };

    benchmark_empty_ocall(enclave.geteid(),0);
    benchmark_empty_ocall(enclave.geteid(),1);
    benchmark_empty_ecall(enclave.geteid(),0);
    benchmark_empty_ecall(enclave.geteid(),1);

    println!("[+] say_something success...");

    enclave.destroy();
}
