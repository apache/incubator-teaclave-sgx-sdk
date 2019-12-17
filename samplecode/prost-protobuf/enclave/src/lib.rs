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

#![crate_name = "helloworldsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_types::*;
use std::slice;

extern crate prost;
extern crate prost_types;
extern crate bytes;

use prost::Message;
use prost_types::Timestamp;

mod person {
    include!(concat!(env!("OUT_DIR"), "/person.rs"));
}

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {
    let person_slice = unsafe { slice::from_raw_parts(some_string, some_len) };

    let the_one: person::Person = person::Person::decode(person_slice).unwrap();
    println!("name: {}, id: 0x{:08X}, email at: {}",
        the_one.name,
        the_one.id,
        the_one.email);
    println!("{:?}", the_one);

    let ts = Timestamp { seconds: 0x1234, nanos: 0x5678 };
    println!("well known types ts = {:?}", ts);

    sgx_status_t::SGX_SUCCESS
}
