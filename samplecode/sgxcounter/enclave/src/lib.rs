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
