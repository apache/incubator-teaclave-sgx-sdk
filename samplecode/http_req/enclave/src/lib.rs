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
pub extern "C" fn send_http_request(fd: c_int, hostname: *const c_char) -> sgx_status_t {
    let hostname = unsafe { CStr::from_ptr(hostname).to_str() };
    let hostname = hostname.expect("Failed to recover hostname");

    //Parse uri and assign it to variable `addr`
    let addr: Uri = hostname.parse().unwrap();

    //Connect to remote host
    let stream = TcpStream::new(fd).unwrap();

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
