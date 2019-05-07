// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

extern crate sgx_types;
extern crate sgx_urts;
extern crate dirs;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::io::{Read, Write};
use std::fs;
use std::path;
use std::time::Instant;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";
static ENCLAVE_TOKEN: &'static str = "enclave.token";

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
        },
        None => {
            println!("[-] Cannot get home dir");
            false
        }
    };

    let token_file: path::PathBuf = home_dir.join(ENCLAVE_TOKEN);;
    if use_token == true {
        match fs::File::open(&token_file) {
            Err(_) => {
                println!("[-] Open token file {} error! Will create one.", token_file.as_path().to_str().unwrap());
            },
            Ok(mut f) => {
                println!("[+] Open token file success! ");
                match f.read(&mut launch_token) {
                    Ok(1024) => {
                        println!("[+] Token file valid!");
                    },
                    _ => println!("[+] Token file invalid, will create new token file"),
                }
            }
        }
    }

    // Step 2: call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};

    let enclave = try!(SgxEnclave::create_with_workers(ENCLAVE_FILE,
                                                       debug,
                                                       &mut launch_token,
                                                       &mut launch_token_updated,
                                                       &mut misc_attr,
                                                       num_uworker,
                                                       num_tworker));

    // Step 3: save the launch token if it is updated
    if use_token == true && launch_token_updated != 0 {
        // reopen the file with write capablity
        match fs::File::create(&token_file) {
            Ok(mut f) => {
                match f.write_all(&launch_token) {
                    Ok(()) => println!("[+] Saved updated launch token!"),
                    Err(_) => println!("[-] Failed to save updated launch token!"),
                }
            },
            Err(_) => {
                println!("[-] Failed to save updated enclave token, but doesn't matter");
            },
        }
    }

    Ok(enclave)
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
