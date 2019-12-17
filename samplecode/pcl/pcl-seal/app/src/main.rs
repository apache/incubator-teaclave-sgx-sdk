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
extern crate itertools;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::os::unix::io::{IntoRawFd, AsRawFd};
use std::env;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::str;
use itertools::Itertools;

const PCL_SEALED_KEY_SIZE: usize = SGX_AESGCM_KEY_SIZE + SGX_PCL_GUID_SIZE;
static ENCLAVE_FILE: &'static str = "enclave.signed.so";
static ENCRYPTED_ENCLAVE_FILE: &'static str = "payload.signed.so";

extern {
    fn key_provision(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
        socket_fd: c_int, sign_type: sgx_quote_sign_type_t) -> sgx_status_t;
    fn get_sealed_pcl_key_len(eid: sgx_enclave_id_t, retval: *mut u32) -> sgx_status_t;
    fn get_sealed_pcl_key(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
        key_buf : *mut u8, key_len: u32) -> sgx_status_t;
}

extern {
    fn f_01ff85f39c4c46adba815dec85546b5579ad1b56(eid: sgx_enclave_id_t,
                                                  retval: *mut sgx_status_t,
                                                  arg1: *const uint8_t,
                                                  arg2: usize) -> sgx_status_t;
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
    let hostname = "test-as.sgx.trustedservices.intel.com";
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

fn init_encrypted_enclave(payload_file: &str,
                          payload_pcl_key: &Vec<u8>) -> SgxResult<SgxEnclave> {

    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    SgxEnclave::create_encrypt(payload_file,
                               debug,
                               &mut launch_token,
                               &mut launch_token_updated,
                               &mut misc_attr,
                               payload_pcl_key.as_ptr() as *const sgx_sealed_data_t)
}

fn main() {
    let mut args: Vec<_> = env::args().collect();
    let mut sign_type = sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE;
    args.remove(0);
    while !args.is_empty() {
        match args.remove(0).as_ref() {
            "--unlink" => sign_type = sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
            _ => {
                panic!("Only --unlink is accepted");
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

    println!("[+] Running as server...");
    let listener = TcpListener::bind("0.0.0.0:3443").unwrap();
    match listener.accept() {
        Ok((socket, addr)) => {
            println!("[+] new client from {:?}", addr);
            let mut retval = sgx_status_t::SGX_SUCCESS;
            let result = unsafe {
                key_provision(enclave.geteid(), &mut retval, socket.as_raw_fd(), sign_type)
            };
            match result {
                sgx_status_t::SGX_SUCCESS => {
                    println!("[+] ECALL success!");
                },
                _ => {
                    println!("[-] ECALL Enclave Failed {}!", result.as_str());
                    return;
                }
            }
            match retval {
                sgx_status_t::SGX_SUCCESS => {
                    println!("[+] provisioning successed!");
                },
                x => {
                    println!("[-] key provisioning failed! {}", x.as_str());
                },
            }
        },
        Err(e) => println!("[-] couldn't get client: {:?}", e),
    }

    let mut pcl_sealed_key_len : u32 = 0;
    let result = unsafe { get_sealed_pcl_key_len(enclave.geteid(),
                                                 &mut pcl_sealed_key_len as *mut u32) };
    if result != sgx_status_t::SGX_SUCCESS {
        println!("[-] Call get_sealed_pcl_key_len error {}", result);
    }
    println!("[+] Get sealed_pcl_key_len = {}", pcl_sealed_key_len);

    let mut sealed_key_vec: Vec<u8> = vec![0u8;pcl_sealed_key_len as usize];
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        get_sealed_pcl_key(enclave.geteid(),
                           &mut retval,
                           sealed_key_vec.as_mut_ptr(),
                           pcl_sealed_key_len)
    };

    if result != sgx_status_t::SGX_SUCCESS {
      println!("[-] Call get_sealed_pcl_key error {}", result);
    }

    if retval != sgx_status_t::SGX_SUCCESS {
        println!("[-] get_sealed_pcl_key returned error {}", retval);
    }

    println!("get_sealed_pcl_key {:02X}", sealed_key_vec.iter().format(""));

    let result = init_encrypted_enclave(ENCRYPTED_ENCLAVE_FILE,
                                        &sealed_key_vec);
    let payload_enclave = match result {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        },
    };

    let mut retval : sgx_status_t = sgx_status_t::SGX_SUCCESS;
    let input_string = String::from("This is a normal world string passed into Enclave!\n");

    let result = unsafe {
        f_01ff85f39c4c46adba815dec85546b5579ad1b56(
            payload_enclave.geteid(),
            &mut retval,
            input_string.as_ptr() as * const u8,
            input_string.len())
    };

    if result != sgx_status_t::SGX_SUCCESS {
      println!("[-] Call encrypted func error {}", result);
    }

    if retval != sgx_status_t::SGX_SUCCESS {
        println!("[-] encrypted func returned error {}", retval);
    }

    println!("[+] Done!");

    enclave.destroy();
}
