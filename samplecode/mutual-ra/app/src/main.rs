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

#![allow(dead_code)]
#![allow(unused_assignments)]

extern crate sgx_types;
extern crate sgx_urts;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::os::unix::io::{IntoRawFd, AsRawFd};
use std::env;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::str;

const BUFFER_SIZE: usize = 1024;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";
static ENCLAVE_TOKEN: &'static str = "enclave.token";

extern {
    fn run_server(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
        socket_fd: c_int, sign_type: sgx_quote_sign_type_t) -> sgx_status_t;
    fn run_client(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
        socket_fd: c_int, sign_type: sgx_quote_sign_type_t) -> sgx_status_t;
}

#[no_mangle]
pub extern "C"
fn ocall_sgx_init_quote(ret_ti: *mut sgx_target_info_t,
                        ret_gid : *mut sgx_epid_group_id_t) -> sgx_status_t {
    println!("Entering ocall_sgx_init_quote");
    unsafe {sgx_init_quote(ret_ti, ret_gid)}
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
fn ocall_get_ias_socket(ret_fd : *mut c_int) -> sgx_status_t {
    let port = 443;
    let hostname = "api.trustedservices.intel.com";
    let addr = lookup_ipv4(hostname, port);
    let sock = TcpStream::connect(&addr).expect("[-] Connect tls server failed!");

    unsafe {*ret_fd = sock.into_raw_fd();}

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C"
fn ocall_get_quote (p_sigrl            : *const u8,
                    sigrl_len          : u32,
                    p_report           : *const sgx_report_t,
                    quote_type         : sgx_quote_sign_type_t,
                    p_spid             : *const sgx_spid_t,
                    p_nonce            : *const sgx_quote_nonce_t,
                    p_qe_report        : *mut sgx_report_t,
                    p_quote            : *mut u8,
                    _maxlen             : u32,
                    p_quote_len        : *mut u32) -> sgx_status_t {
    println!("Entering ocall_get_quote");

    let mut real_quote_len : u32 = 0;

    let ret = unsafe {
        sgx_calc_quote_size(p_sigrl, sigrl_len, &mut real_quote_len as *mut u32)
    };

    if ret != sgx_status_t::SGX_SUCCESS {
        println!("sgx_calc_quote_size returned {}", ret);
        return ret;
    }

    println!("quote size = {}", real_quote_len);
    unsafe { *p_quote_len = real_quote_len; }

    let ret = unsafe {
        sgx_get_quote(p_report,
                      quote_type,
                      p_spid,
                      p_nonce,
                      p_sigrl,
                      sigrl_len,
                      p_qe_report,
                      p_quote as *mut sgx_quote_t,
                      real_quote_len)
    };

    if ret != sgx_status_t::SGX_SUCCESS {
        println!("sgx_calc_quote_size returned {}", ret);
        return ret;
    }

    println!("sgx_calc_quote_size returned {}", ret);
    ret
}

#[no_mangle]
pub extern "C"
fn ocall_get_update_info (platform_blob: * const sgx_platform_info_t,
                          enclave_trusted: i32,
                          update_info: * mut sgx_update_info_bit_t) -> sgx_status_t {
    unsafe{
        sgx_report_attestation_status(platform_blob, enclave_trusted, update_info)
    }
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    SgxEnclave::create(ENCLAVE_FILE,
                       debug,
                       &mut launch_token,
                       &mut launch_token_updated,
                       &mut misc_attr)
}

enum Mode {
    Client,
    Server,
}

fn main() {
    let mut mode:Mode = Mode::Server;
    let mut args: Vec<_> = env::args().collect();
    let mut sign_type = sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE;
    args.remove(0);
    while !args.is_empty() {
        match args.remove(0).as_ref() {
            "--client" => mode = Mode::Client,
            "--server" => mode = Mode::Server,
            "--unlink" => sign_type = sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
            _ => {
                panic!("Only --client/server/unlink is accepted");
            }
        }
    }

    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        },
    };

    match mode {
        Mode::Server => {
            println!("Running as server...");
            let listener = TcpListener::bind("0.0.0.0:3443").unwrap();
            //loop{
            match listener.accept() {
                Ok((socket, addr)) => {
                    println!("new client from {:?}", addr);
                    let mut retval = sgx_status_t::SGX_SUCCESS;
                    let result = unsafe {
                        run_server(enclave.geteid(), &mut retval, socket.as_raw_fd(), sign_type)
                    };
                    match result {
                        sgx_status_t::SGX_SUCCESS => {
                            println!("ECALL success!");
                        },
                        _ => {
                            println!("[-] ECALL Enclave Failed {}!", result.as_str());
                            return;
                        }
                    }
                }
                Err(e) => println!("couldn't get client: {:?}", e),
            }
            //} //loop
        }
        Mode::Client => {
            println!("Running as client...");
            let socket = TcpStream::connect("localhost:3443").unwrap();
            let mut retval = sgx_status_t::SGX_SUCCESS;
            let result = unsafe {
                run_client(enclave.geteid(), &mut retval, socket.as_raw_fd(), sign_type)
            };
            match result {
                sgx_status_t::SGX_SUCCESS => {
                    println!("ECALL success!");
                },
                _ => {
                    println!("[-] ECALL Enclave Failed {}!", result.as_str());
                    return;
                }
            }
        }
    }

    println!("[+] Done!");

    enclave.destroy();
}
