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
// under the License.

extern crate sgx_types;

use inflate::inflate_bytes_zlib;
use libflate::zlib::Encoder;
use sgx_types::error::SgxStatus;
use std::io::Write;
use std::str::from_utf8;
use std::sync::LazyLock;

static HELLOSTR: LazyLock<String> =
    LazyLock::new(|| String::from("This is a global rust String init by LazyLock!"));

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn zlib_sample() -> SgxStatus {
    println!("Source string is : {:?}", *HELLOSTR);
    println!("Source data is : {:?}", HELLOSTR.as_bytes());

    let mut encoder = Encoder::new(Vec::new()).unwrap();
    encoder.write_all(HELLOSTR.as_bytes()).unwrap();
    let encoded_data = encoder.finish().into_result().unwrap();

    println!("After zlib compress : {encoded_data:?}");

    let decoded = inflate_bytes_zlib(&encoded_data[..]).unwrap();

    let decoded_string = from_utf8(&decoded[..]);
    println!("After zlib decompress: {:?}", decoded_string.unwrap());

    SgxStatus::Success
}
