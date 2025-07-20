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

use crate::linux::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr;
use core::slice;
use sgx_ffi::c_str::CStr;
use sgx_oc::linux::ocall;
use sgx_oc::linux::ocall::{AddrInfoHints, OCallError};
use sgx_trts::trts::is_within_enclave;

#[no_mangle]
pub unsafe extern "C" fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const addrinfo,
    res: *mut *mut addrinfo,
) -> c_int {
    let node_opt = if !node.is_null() {
        Some(CStr::from_ptr(node))
    } else {
        None
    };

    let service_opt = if !service.is_null() {
        Some(CStr::from_ptr(service))
    } else {
        None
    };

    let hints_opt = if !hints.is_null() {
        if !is_within_enclave(hints as *const u8, mem::size_of::<addrinfo>()) {
            set_errno(EINVAL);
            return EAI_SYSTEM;
        }
        Some(AddrInfoHints::from(*hints))
    } else {
        None
    };

    if res.is_null() || !is_within_enclave(res as *const u8, mem::size_of::<*mut addrinfo>()) {
        set_errno(EINVAL);
        return EAI_SYSTEM;
    }

    let result = ocall::getaddrinfo(node_opt, service_opt, hints_opt).map_err(|e| match e {
        OCallError::GaiError(err) => err,
        _ => EAI_SYSTEM,
    });

    if let Ok(addrinfo) = result {
        let mut addrinfo_vec: Vec<ManuallyDrop<Box<addrinfo>>> = Vec::new();

        for addr in addrinfo.iter() {
            let mut info = addrinfo {
                ai_flags: addr.flags,
                ai_family: addr.family,
                ai_socktype: addr.socktype,
                ai_protocol: addr.protocol,
                ai_addrlen: 0,
                ai_addr: ptr::null_mut(),
                ai_canonname: ptr::null_mut(),
                ai_next: ptr::null_mut(),
            };

            let mut addr_box = ManuallyDrop::new(Box::<[u8]>::from(addr.addr.as_bytes()));
            info.ai_addrlen = addr.addr.addr_len() as u32;
            info.ai_addr = addr_box.as_mut_ptr() as *mut sockaddr;

            addrinfo_vec.push(ManuallyDrop::new(Box::new(info)));
        }

        *res = ptr::null_mut();
        if !addrinfo_vec.is_empty() {
            for i in 0..addrinfo_vec.len() - 1 {
                addrinfo_vec[i].ai_next = addrinfo_vec[i + 1].as_mut() as *mut addrinfo;
            }

            let res_ptr = addrinfo_vec[0].as_mut() as *mut addrinfo;
            *res = res_ptr;
        }
        0
    } else {
        result.err().unwrap()
    }
}

#[no_mangle]
pub unsafe extern "C" fn freeaddrinfo(res: *mut addrinfo) {
    if res.is_null() {
        return;
    }

    let mut cur_ptr: *mut addrinfo = res;
    let mut addrinfo_vec: Vec<Box<addrinfo>> = Vec::new();
    while !cur_ptr.is_null() {
        if !is_within_enclave(cur_ptr as *const u8, mem::size_of::<addrinfo>()) {
            continue;
        }

        let cur: &addrinfo = &*cur_ptr;

        if !cur.ai_addr.is_null()
            && cur.ai_addrlen > 0
            && is_within_enclave(cur.ai_addr as *const u8, cur.ai_addrlen as usize)
        {
            let addr_slice =
                slice::from_raw_parts_mut(cur.ai_addr as *mut u8, cur.ai_addrlen as usize);
            let addr_box = Box::from_raw(addr_slice as *mut [u8]);
            drop(addr_box);
        }

        addrinfo_vec.push(Box::from_raw(cur_ptr));
        cur_ptr = cur.ai_next;
    }
    drop(addrinfo_vec);
}
