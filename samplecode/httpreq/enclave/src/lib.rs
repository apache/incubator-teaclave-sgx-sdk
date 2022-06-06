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

extern crate sgx_libc;
extern crate sgx_oc;
extern crate sgx_trts;
extern crate sgx_types;

use http_req::{request::RequestBuilder, tls, uri::Uri};
use sgx_oc::ocall;
use sgx_trts::veh::{
    register, unregister, CpuContext, ExceptionInfo, ExceptionType, ExceptionVector, HandleResult,
};
use sgx_types::error::SgxStatus;
use sgx_types::types::c_char;
use std::ffi::CStr;
use std::net::TcpStream;

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn send_http_request(hostname: *const c_char) -> SgxStatus {
    if hostname.is_null() {
        return SgxStatus::Unexpected;
    }

    let handle = match register(handle_exception) {
        Ok(handle) => handle,
        Err(e) => return e,
    };

    let hostname = CStr::from_ptr(hostname).to_str();
    let hostname = hostname.expect("Failed to recover hostname");

    // Parse uri and assign it to variable `addr`
    let addr = Uri::try_from(hostname).unwrap();

    // Construct a domain:ip string for tcp connection
    let conn_addr = format!("{}:{}", addr.host().unwrap(), addr.port().unwrap());

    // Connect to remote host
    let stream = TcpStream::connect(conn_addr).unwrap();

    // Open secure connection over TlsStream, because of `addr` (https)
    let mut stream = tls::Config::default()
        .connect(addr.host().unwrap_or(""), stream)
        .unwrap();

    // Container for response's body
    let mut writer = Vec::new();

    // Add header `Connection: Close`
    let response = RequestBuilder::new(&addr)
        .header("Connection", "Close")
        .send(&mut stream, &mut writer)
        .unwrap();

    println!("{}", String::from_utf8_lossy(&writer));
    println!("Status: {} {}", response.status_code(), response.reason());

    unregister(handle);
    SgxStatus::Success
}

#[no_mangle]
extern "C" fn handle_exception(info: &mut ExceptionInfo) -> HandleResult {
    const CPUID_OPCODE: u16 = 0xA20F;

    let mut result = HandleResult::Search;
    if info.vector == ExceptionVector::UD && info.exception_type == ExceptionType::Hardware {
        let ip_opcode = unsafe { *(info.context.rip as *const u16) };
        if ip_opcode == CPUID_OPCODE {
            result = handle_cpuid_exception(&mut info.context);
        }
    }
    result
}

fn handle_cpuid_exception(context: &mut CpuContext) -> HandleResult {
    let leaf = context.rax as u32;
    match unsafe { ocall::cpuid(leaf) } {
        Ok(cpuid_result) => {
            context.rax = cpuid_result.eax as u64;
            context.rbx = cpuid_result.ebx as u64;
            context.rcx = cpuid_result.ecx as u64;
            context.rdx = cpuid_result.edx as u64;
            context.rip += 2;
            HandleResult::Execution
        }
        Err(_) => HandleResult::Search,
    }
}
