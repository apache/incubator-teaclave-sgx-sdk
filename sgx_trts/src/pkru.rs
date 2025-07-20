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

use crate::feature::SysFeatures;
use core::arch::asm;
use sgx_types::types;

pub struct Pkru;

impl Pkru {
    pub fn read() -> Result<u32, ()> {
        if !Pkru::is_enabled() {
            return Err(());
        }

        unsafe {
            let pkru;
            asm!(
                "rdpkru",
                lateout("eax") pkru,
                lateout("edx") _,
                in("ecx") 0,
            );
            Ok(pkru)
        }
    }

    pub fn write(pkru: u32) -> Result<(), ()> {
        if !Pkru::is_enabled() {
            return Err(());
        }

        unsafe {
            asm!(
                "wrpkru",
                in("eax") pkru,
                in("edx") 0,
                in("ecx") 0,
            );
            Ok(())
        }
    }

    fn is_enabled() -> bool {
        SysFeatures::get().xfrm() & types::XFRM_PKRU == types::XFRM_PKRU
    }
}
