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

use crate::ocall::util::*;
use libc::{self, addrinfo, c_char, c_int, c_uint, size_t, sockaddr_storage};
use std::io::Error;
use std::mem;
use std::ptr;
use std::slice;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct addr_info {
    pub ai_flags: c_int,
    pub ai_family: c_int,
    pub ai_socktype: c_int,
    pub ai_protocol: c_int,
    pub ai_addrlen: c_uint,
    pub ai_addr: sockaddr_storage,
}

#[no_mangle]
pub unsafe extern "C" fn u_getaddrinfo_ocall(
    error: *mut c_int,
    node: *const c_char,
    service: *const c_char,
    hints: *mut addrinfo,
    addrinfo: *mut addr_info,
    in_count: size_t,
    out_count: *mut size_t,
) -> c_int {
    let mut errno = 0;
    let mut res: *mut addrinfo = ptr::null_mut();
    let ret = libc::getaddrinfo(node, service, hints, &mut res);
    if ret == libc::EAI_SYSTEM {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    if ret != 0 {
        return ret;
    }

    let addrinfo_slice = slice::from_raw_parts_mut(addrinfo, in_count);
    let mut i = 0;
    let mut cur_ptr = res;
    while !cur_ptr.is_null() && i < in_count {
        let cur: &addrinfo = &*cur_ptr;
        let len = cur.ai_addrlen as usize;
        if len > mem::size_of::<sockaddr_storage>() {
            return 1;
        }

        addrinfo_slice[i].ai_flags = cur.ai_flags;
        addrinfo_slice[i].ai_family = cur.ai_family;
        addrinfo_slice[i].ai_socktype = cur.ai_socktype;
        addrinfo_slice[i].ai_protocol = cur.ai_protocol;
        addrinfo_slice[i].ai_addrlen = cur.ai_addrlen;

        ptr::copy_nonoverlapping(
            cur.ai_addr as *const u8,
            &mut addrinfo_slice[i].ai_addr as *mut _ as *mut u8,
            len,
        );
        i += 1;
        cur_ptr = cur.ai_next;
    }

    *out_count = i;
    libc::freeaddrinfo(res);
    0
}
