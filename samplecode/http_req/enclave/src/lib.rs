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

#![crate_name = "httpreqenclave"]
#![crate_type = "staticlib"]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

#[cfg(target_env = "sgx")]
extern crate sgx_types;

use http_req::{request::RequestBuilder, tls, uri::Uri};
use sgx_types::*;
use std::ffi::CStr;
use std::net::TcpStream;
use std::os::raw::c_char;
use std::prelude::v1::*;

#[no_mangle]
pub extern "C" fn send_http_request(hostname: *const c_char) -> sgx_status_t {
    if hostname.is_null() {
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    }

    let hostname = unsafe { CStr::from_ptr(hostname).to_str() };
    let hostname = hostname.expect("Failed to recover hostname");

    //Parse uri and assign it to variable `addr`
    let addr: Uri = hostname.parse().unwrap();

    //Construct a domain:ip string for tcp connection
    let conn_addr = format!("{}:{}", addr.host().unwrap(), addr.port().unwrap());

    //Connect to remote host
    let stream = TcpStream::connect(conn_addr).unwrap();

    //Open secure connection over TlsStream, because of `addr` (https)
    let mut stream = tls::Config::default()
        .connect(addr.host().unwrap_or(""), stream)
        .unwrap();

    //Container for response's body
    let mut writer = Vec::new();

    //Add header `Connection: Close`
    let response = RequestBuilder::new(&addr)
        .header("Connection", "Close")
        .send(&mut stream, &mut writer)
        .unwrap();

    println!("{}", String::from_utf8_lossy(&writer));
    println!("Status: {} {}", response.status_code(), response.reason());

    sgx_status_t::SGX_SUCCESS
}
