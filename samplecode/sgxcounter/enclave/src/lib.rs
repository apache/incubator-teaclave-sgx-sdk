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

#![crate_name = "sgxcounterenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_tservice;
extern crate itertools;

use sgx_types::*;
use sgx_tservice::*;

use itertools::Itertools;

#[no_mangle]
pub extern "C" fn sgx_counter_sample() -> sgx_status_t {

    match rsgx_create_pse_session() {
        Ok(_) => println!("Create PSE session done"),
        _ => {
            println!("Cannot create PSE session");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    }
    let mut init_val: u32 = 100;
    let tcounter = sgxcounter::SgxMonotonicCounter::new(&mut init_val).unwrap();

    for _ in 0..10 {
        tcounter.increment().unwrap();
        println!("value after increment = {}", tcounter.read().unwrap());
    }

    let uuid = tcounter.get_uuid().unwrap();

    println!("acquired uuid = {:02X}", uuid.counter_id.iter().format(""));
    println!("acquired nonce = {:02X}", uuid.nonce.iter().format(""));

    match rsgx_close_pse_session() {
        Ok(_) => println!("close PSE session done"),
        _ => {
            println!("Cannot close PSE session");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    }

    // Emulate the 2nd session
    match rsgx_create_pse_session() {
        Ok(_) => println!("Create PSE session done"),
        _ => {
            println!("Cannot create PSE session");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    }

    let recovered_counter = sgxcounter::SgxMonotonicCounter::from_uuid(uuid);
    println!("recovered uuid value = {}", recovered_counter.read().unwrap());

    match rsgx_close_pse_session() {
        Ok(_) => println!("close PSE session done"),
        _ => {
            println!("Cannot close PSE session");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    }

    sgx_status_t::SGX_SUCCESS
}
