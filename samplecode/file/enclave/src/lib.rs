// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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