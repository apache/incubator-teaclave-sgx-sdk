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

#![crate_name = "filesampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_rand;
#[macro_use]
extern crate sgx_rand_derive;
extern crate sgx_serialize;
#[macro_use]
extern crate sgx_serialize_derive;

use std::sgxfs::SgxFile;
use std::io::{Read, Write};

use sgx_serialize::{SerializeHelper, DeSerializeHelper};

#[derive(Copy, Clone, Default, Debug, Serializable, DeSerializable, Rand)]
struct RandData {
    key: u32,
    rand: u32,
}

#[no_mangle]
pub extern "C" fn write_file() -> i32 {

    let rand = sgx_rand::random::<RandData>();

    let helper = SerializeHelper::new();
    let data = match helper.encode(rand) {
        Some(d) => d,
        None => {
            println!("encode data failed.");
            return 1;
        },
    };

    let mut file = match SgxFile::create("sgx_file") {
        Ok(f) => f,
        Err(_) => {
            println!("SgxFile::create failed.");
            return 2;
        },
    };

    let write_size = match file.write(data.as_slice()) {
        Ok(len) => len,
        Err(_) => {
            println!("SgxFile::write failed.");
            return 3;
        },
    };

    println!("write file success, write size: {}, {:?}.", write_size, rand);
    0
}

#[no_mangle]
pub extern "C" fn read_file() -> i32 {

    let mut data = [0_u8; 10];

    let mut file = match SgxFile::open("sgx_file") {
        Ok(f) => f,
        Err(_) => {
            println!("SgxFile::open failed.");
            return 1;
        },
    };

    let read_size = match file.read(&mut data) {
        Ok(len) => len,
        Err(_) => {
            println!("SgxFile::read failed.");
            return 2;
        },
    };

    let helper = DeSerializeHelper::<RandData>::new(data.to_vec());
    let rand = match helper.decode() {
        Some(d) => d,
        None => {
            println!("decode data failed.");
            return 3;
        },
    };

    println!("read file success, read size: {}, {:?}.", read_size, rand);
    0
}