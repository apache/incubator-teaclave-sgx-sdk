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

use super::*;
use sgx_types::marker::ContiguousMemory;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

unsafe impl ContiguousMemory for CpuidResult {}

pub unsafe fn cpuid_count(leaf: u32, sub_leaf: u32) -> OCallResult<CpuidResult> {
    let mut cpuinfo = [0_i32; 4];
    let status = sgx_oc_cpuidex(&mut cpuinfo as *mut [i32; 4], leaf as i32, sub_leaf as i32);

    ensure!(status.is_success(), esgx!(status));

    Ok(CpuidResult {
        eax: cpuinfo[0] as u32,
        ebx: cpuinfo[1] as u32,
        ecx: cpuinfo[2] as u32,
        edx: cpuinfo[3] as u32,
    })
}

#[inline]
pub unsafe fn cpuid(leaf: u32) -> OCallResult<CpuidResult> {
    cpuid_count(leaf, 0)
}
