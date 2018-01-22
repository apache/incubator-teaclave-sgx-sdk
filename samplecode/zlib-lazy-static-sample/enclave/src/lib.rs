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

#![crate_name = "zlibsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate inflate;
extern crate libflate;
#[macro_use]
extern crate lazy_static;

use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::str::from_utf8;

use inflate::inflate_bytes_zlib;
use libflate::zlib::Encoder;
use std::io::Write;

lazy_static! {
    static ref HELLOSTR : String = String::from("This is a global rust String init by lazy_static!");
}

#[no_mangle]
pub extern "C"
fn zlib_sample() -> sgx_status_t {
    println!("Source string is : {:?}", *HELLOSTR);
    println!("Source data is : {:?}", HELLOSTR.as_bytes());

    let mut encoder = Encoder::new(Vec::new()).unwrap();
    encoder.write_all(HELLOSTR.as_bytes()).unwrap();
    let encoded_data = encoder.finish().into_result().unwrap();

    println!("After zlib compress : {:?}", encoded_data);

    let decoded = inflate_bytes_zlib(&encoded_data[..]).unwrap();

    let decoded_string = from_utf8(&decoded[..]);
    println!("After zlib decompress: {:?}", decoded_string.unwrap());

    sgx_status_t::SGX_SUCCESS
}

