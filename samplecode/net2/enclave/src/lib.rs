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

extern crate net2;

use sgx_types::*;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::slice;
use std::string::String;
use std::vec::Vec;

fn tcplistener_test() {
    let listener = TcpListener::bind("127.0.0.1:50080").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 512];

                stream.read(&mut buffer).unwrap();

                println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
                let response = "HTTP/1.1 200 OK\r\n\r\n";

                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("Error on accept {}", e);
            }
        }
    }
}

fn net2_test() {
    use net2::TcpBuilder;

    let tcp = TcpBuilder::new_v4().unwrap();
    tcp.reuse_address(true).unwrap();

    let mut stream = tcp.connect("139.59.236.18:80").unwrap();

    stream.write(b"GET / HTTP/1.1\r\nHost: localhost:50080\r\nUser-Agent: curl/7.47.0\r\nAccept: */*\r\n\r\n").unwrap();

    stream.flush().unwrap();
    let mut buffer = [0;1024];
    stream.read(&mut buffer).unwrap();
    println!("Got response:\n{}", String::from_utf8_lossy(&buffer[..]));
}

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {
    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    // A sample &'static string
    let rust_raw_string = "This is a in-Enclave ";
    // An array
    let word: [u8; 4] = [82, 117, 115, 116];
    // An vector
    let word_vec: Vec<u8> = vec![32, 115, 116, 114, 105, 110, 103, 33];

    // Construct a string from &'static string
    let mut hello_string = String::from(rust_raw_string);

    // Iterate on word array
    for c in word.iter() {
        hello_string.push(*c as char);
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8").as_str();

    // Ocall to normal world for output
    println!("{}", &hello_string);

    net2_test();
    tcplistener_test();

    sgx_status_t::SGX_SUCCESS
}
