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

use super::*;
use alloc::vec::Vec;
use core::convert::{From, TryFrom};
use core::mem;
use sgx_types::marker::ContiguousMemory;

pub unsafe fn getaddrinfo(
    node: Option<&CStr>,
    service: Option<&CStr>,
    hints: Option<AddrInfoHints>,
) -> OCallResult<Vec<AddrInfo>> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    const ADDRINFO_MAX_COUNT: usize = 32;

    let addrinfo: addr_info = mem::zeroed();
    let mut addrinfo_vec = vec![addrinfo; ADDRINFO_MAX_COUNT];

    let in_count = ADDRINFO_MAX_COUNT;
    let mut out_count = 0_usize;

    let node_ptr = node.map(|n| n.as_ptr()).unwrap_or(ptr::null());
    let service_ptr = service.map(|s| s.as_ptr()).unwrap_or(ptr::null());

    let hints = hints.map(|h| h.to_addrinfo());
    let hints_ptr = hints
        .as_ref()
        .map(|h| h as *const addrinfo)
        .unwrap_or(ptr::null());

    let status = u_getaddrinfo_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        node_ptr,
        service_ptr,
        hints_ptr,
        addrinfo_vec.as_mut_ptr(),
        in_count,
        &mut out_count as *mut size_t,
    );

    ensure!(status.is_success(), esgx!(status));
    match result {
        0 => {}
        EAI_SYSTEM => return Err(eos!(error)),
        1 => return Err(ecust!("Untrusted side error")),
        e => return Err(egai!(e)),
    };

    ensure!(result == 0, ecust!("Unkonwn error"));
    ensure!(out_count <= in_count, ecust!("Malformed out_count"));

    let mut addr_vec = Vec::<AddrInfo>::with_capacity(out_count);
    for addr in addrinfo_vec.into_iter().take(out_count) {
        addr_vec.push(AddrInfo::try_from(addr)?);
    }

    Ok(addr_vec)
}

#[derive(Clone, Copy, Debug)]
pub struct AddrInfoHints {
    pub flags: i32,
    pub family: i32,
    pub socktype: i32,
    pub protocol: i32,
}

impl AddrInfoHints {
    fn to_addrinfo(self) -> addrinfo {
        let mut addrinfo: addrinfo = unsafe { mem::zeroed() };
        addrinfo.ai_flags = self.flags;
        addrinfo.ai_family = self.family;
        addrinfo.ai_socktype = self.socktype;
        addrinfo.ai_protocol = self.protocol;
        addrinfo
    }

    #[allow(dead_code)]
    fn to_addr_info(self) -> addr_info {
        let mut addrinfo: addr_info = unsafe { mem::zeroed() };
        addrinfo.ai_flags = self.flags;
        addrinfo.ai_family = self.family;
        addrinfo.ai_socktype = self.socktype;
        addrinfo.ai_protocol = self.protocol;
        addrinfo
    }
}

impl From<addrinfo> for AddrInfoHints {
    fn from(addrinfo: addrinfo) -> AddrInfoHints {
        AddrInfoHints {
            flags: addrinfo.ai_flags,
            family: addrinfo.ai_family,
            socktype: addrinfo.ai_socktype,
            protocol: addrinfo.ai_protocol,
        }
    }
}

impl From<addr_info> for AddrInfoHints {
    fn from(addrinfo: addr_info) -> AddrInfoHints {
        AddrInfoHints {
            flags: addrinfo.ai_flags,
            family: addrinfo.ai_family,
            socktype: addrinfo.ai_socktype,
            protocol: addrinfo.ai_protocol,
        }
    }
}

impl Default for AddrInfoHints {
    fn default() -> Self {
        AddrInfoHints {
            flags: 0,
            family: AF_UNSPEC,
            socktype: 0,
            protocol: 0,
        }
    }
}

unsafe impl ContiguousMemory for AddrInfoHints {}

#[derive(Clone, Copy)]
pub struct AddrInfo {
    pub flags: i32,
    pub family: i32,
    pub socktype: i32,
    pub protocol: i32,
    pub addr: SockAddr,
}

unsafe impl ContiguousMemory for AddrInfo {}

impl TryFrom<addr_info> for AddrInfo {
    type Error = OCallError;
    fn try_from(addrinfo: addr_info) -> Result<Self, Self::Error> {
        ensure!(
            addrinfo.ai_family == addrinfo.ai_addr.ss_family as i32,
            ecust!("Unsupported family info")
        );

        Ok(AddrInfo {
            flags: addrinfo.ai_flags,
            family: addrinfo.ai_family,
            socktype: addrinfo.ai_socktype,
            protocol: addrinfo.ai_protocol,
            addr: match addrinfo.ai_addr.ss_family as i32 {
                AF_INET | AF_INET6 => unsafe {
                    SockAddr::try_from_storage(addrinfo.ai_addr, addrinfo.ai_addrlen)?
                },
                _ => return Err(ecust!("Unsupported family info")),
            },
        })
    }
}
