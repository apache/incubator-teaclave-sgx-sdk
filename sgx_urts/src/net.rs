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

use libc::{self, addrinfo, c_char, c_int, size_t};
use std::io::Error;

use core::ptr;

#[no_mangle]
pub extern "C" fn u_getaddrinfo_ocall(
    error: *mut c_int,
    node: *const c_char,
    service: *const c_char,
    hints: *mut addrinfo,
    entry_size: size_t,
    buf: *mut u8,
    bufsz: size_t,
    out_cnt: *mut size_t,
) -> c_int {
    let mut errno = 0;
    let mut res: *mut addrinfo = ptr::null_mut();

    let ret = unsafe { libc::getaddrinfo(node, service, hints, &mut res) };

    if ret == libc::EAI_SYSTEM {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }

    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }

    if ret != 0 {
        return ret;
    }

    unsafe {
        let mut i = 0;
        let mut cur_ptr = res;
        while cur_ptr != ptr::null_mut() && (i + entry_size) < bufsz {
            let cur: &addrinfo = &*cur_ptr;
            let len = cur.ai_addrlen as usize;
            if len > entry_size {
                return 1;
            }
            std::ptr::copy_nonoverlapping(cur.ai_addr as *const u8, buf.add(i), len);
            i += entry_size;
            cur_ptr = cur.ai_next;
        }
        *out_cnt = i / entry_size;
    }

    unsafe { libc::freeaddrinfo(res) };
    return 0;
}
