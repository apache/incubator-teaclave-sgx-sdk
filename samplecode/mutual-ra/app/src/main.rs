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

extern crate sgx_types;
extern crate sgx_urts;

use std::net::{SocketAddr, TcpStream};
use std::os::unix::io::IntoRawFd;

use sgx_types::error::SgxStatus;
use sgx_types::types::*;
use sgx_urts::enclave::SgxEnclave;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

fn main() {
    let enclave = match SgxEnclave::create(ENCLAVE_FILE, true) {
        Ok(enclave) => {
            println!("[+] Init Enclave Successful {}!", enclave.eid());
            enclave
        }
        Err(err) => {
            println!("[-] Init Enclave Failed {}!", err.as_str());
            return;
        }
    };

    let mut retval = SgxStatus::Success;

    //match result {
    //    SgxStatus::Success => {}
    //    _ => {
    //        println!("[-] ECALL Enclave Failed {}!", result.as_str());
    //        return;
    //    }
    //}

    println!("[+] sample ended!");
}

extern {
    fn run_server(eid: EnclaveId, retval: *mut SgxStatus,
        socket_fd: c_int, sign_type: QuoteSignType) -> SgxStatus;
    fn run_client(eid: EnclaveId, retval: *mut SgxStatus,
        socket_fd: c_int, sign_type: QuoteSignType) -> SgxStatus;
}

#[no_mangle]
pub extern "C"
fn ocall_sgx_init_quote(ret_ti: *mut TargetInfo,
                        ret_gid : *mut EpidGroupId) -> SgxStatus {
    println!("Entering ocall_sgx_init_quote");
    //unsafe {sgx_init_quote(ret_ti, ret_gid)}
    SgxStatus::Success
}


pub fn lookup_ipv4(host: &str, port: u16) -> SocketAddr {
    use std::net::ToSocketAddrs;

    let addrs = (host, port).to_socket_addrs().unwrap();
    for addr in addrs {
        if let SocketAddr::V4(_) = addr {
            return addr;
        }
    }

    unreachable!("Cannot lookup address");
}


#[no_mangle]
pub extern "C"
fn ocall_get_ias_socket(ret_fd : *mut c_int) -> SgxStatus {
    let port = 443;
    let hostname = "api.trustedservices.intel.com";
    let addr = lookup_ipv4(hostname, port);
    let sock = TcpStream::connect(&addr).expect("[-] Connect tls server failed!");

    unsafe {*ret_fd = sock.into_raw_fd();}

    SgxStatus::Success
}

#[no_mangle]
pub extern "C"
fn ocall_get_quote (p_sigrl            : *const u8,
                    sigrl_len          : u32,
                    p_report           : *const Report,
                    quote_type         : QuoteSignType,
                    p_spid             : *const Spid,
                    p_nonce            : *const QuoteNonce,
                    p_qe_report        : *mut Report,
                    p_quote            : *mut u8,
                    _maxlen             : u32,
                    p_quote_len        : *mut u32) -> SgxStatus {
    println!("Entering ocall_get_quote");

    //let mut real_quote_len : u32 = 0;

    //let ret = unsafe {
    //    sgx_calc_quote_size(p_sigrl, sigrl_len, &mut real_quote_len as *mut u32)
    //};

    //if ret != sgx_status_t::SGX_SUCCESS {
    //    println!("sgx_calc_quote_size returned {}", ret);
    //    return ret;
    //}

    //println!("quote size = {}", real_quote_len);
    //unsafe { *p_quote_len = real_quote_len; }

    //let ret = unsafe {
    //    sgx_get_quote(p_report,
    //                  quote_type,
    //                  p_spid,
    //                  p_nonce,
    //                  p_sigrl,
    //                  sigrl_len,
    //                  p_qe_report,
    //                  p_quote as *mut sgx_quote_t,
    //                  real_quote_len)
    //};

    //if ret != sgx_status_t::SGX_SUCCESS {
    //    println!("sgx_calc_quote_size returned {}", ret);
    //    return ret;
    //}

    //println!("sgx_calc_quote_size returned {}", ret);
    //ret
    SgxStatus::Success
}

#[no_mangle]
pub extern "C"
fn ocall_get_update_info (platform_blob: * const PlatformInfo,
                          enclave_trusted: i32,
                          update_info: * mut UpdateInfoBit) -> SgxStatus {
    //unsafe{
    //    sgx_report_attestation_status(platform_blob, enclave_trusted, update_info)
    //}
    SgxStatus::Success
}

