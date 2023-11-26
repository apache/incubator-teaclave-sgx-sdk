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

// Tests for this module
#[cfg(feature = "unit_test")]
mod tests;

use crate::sys_common::{FromInner, IntoInner};

use sgx_oc as c;

pub use core::net::IpAddr;

pub use core::net::{Ipv4Addr, Ipv6Addr};

pub use core::net::Ipv6MulticastScope;

impl IntoInner<c::in_addr> for Ipv4Addr {
    #[inline]
    fn into_inner(self) -> c::in_addr {
        // `s_addr` is stored as BE on all machines and the array is in BE order.
        // So the native endian conversion method is used so that it's never swapped.
        c::in_addr { s_addr: u32::from_ne_bytes(self.octets()) }
    }
}
impl FromInner<c::in_addr> for Ipv4Addr {
    fn from_inner(addr: c::in_addr) -> Ipv4Addr {
        Ipv4Addr::from(addr.s_addr.to_ne_bytes())
    }
}

impl IntoInner<c::in6_addr> for Ipv6Addr {
    fn into_inner(self) -> c::in6_addr {
        c::in6_addr { s6_addr: self.octets() }
    }
}
impl FromInner<c::in6_addr> for Ipv6Addr {
    #[inline]
    fn from_inner(addr: c::in6_addr) -> Ipv6Addr {
        Ipv6Addr::from(addr.s6_addr)
    }
}
