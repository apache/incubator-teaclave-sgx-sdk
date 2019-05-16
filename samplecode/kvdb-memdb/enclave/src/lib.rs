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

#![crate_name = "helloworldsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

extern crate elastic_array;
extern crate parity_bytes;

use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::io::{self, Write};
use std::slice;

use elastic_array::ElasticArray2;
use parity_bytes::BytesRef;

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {

    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    // A sample &'static string
    let rust_raw_string = "This is a in-Enclave ";
    // An array
    let word:[u8;4] = [82, 117, 115, 116];
    // An vector
    let word_vec:Vec<u8> = vec![32, 115, 116, 114, 105, 110, 103, 33];

    // Construct a string from &'static string
    let mut hello_string = String::from(rust_raw_string);

    // Iterate on word array
    for c in word.iter() {
        hello_string.push(*c as char);
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8")
                                               .as_str();

    // Ocall to normal world for output
    println!("{}", &hello_string);

    println!("{}", &hello_string);

    // test elastic_array import
    type BytesShort = ElasticArray2<u8>;
    let mut bytes = BytesShort::new();
    bytes.push(1);
    bytes.push(2);
    bytes.insert_slice(1, &[3, 4]);
    assert_eq!(bytes.len(), 4);
    let r: &[u8] = &bytes;
    assert_eq!(r, &[1, 3, 4, 2]);

    //test parity_byte import
    let mut data1 = vec![0, 0, 0];
    let mut data2 = vec![0, 0, 0];
    let mut data3 = vec![0, 0, 0];
    let (res1, res2, res3) = {
        let mut bytes1 = BytesRef::Flexible(&mut data1);
        let mut bytes2 = BytesRef::Flexible(&mut data2);
        let mut bytes3 = BytesRef::Flexible(&mut data3);

        // when
        let res1 = bytes1.write(1, &[1, 1, 1]);
        let res2 = bytes2.write(3, &[1, 1, 1]);
        let res3 = bytes3.write(5, &[1, 1, 1]);
        (res1, res2, res3)
    };

    // then
    assert_eq!(&data1, &[0, 1, 1, 1]);
    assert_eq!(res1, 3);

    assert_eq!(&data2, &[0, 0, 0, 1, 1, 1]);
    assert_eq!(res2, 3);

    assert_eq!(&data3, &[0, 0, 0, 0, 0, 1, 1, 1]);
    assert_eq!(res3, 5);


    sgx_status_t::SGX_SUCCESS
}