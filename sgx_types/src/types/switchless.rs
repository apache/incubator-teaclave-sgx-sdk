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

use crate::error::SgxStatus;

/* intel sgx sdk 2.2 */
//
// sgx_uswitchless.h
//
impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum SwitchlessWokerType {
        Untrusted = 0,
        Trusted = 1,
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum SwitchlessWokerEvent {
        Start = 0,
        Idel = 1,
        Miss = 2,
        Exit = 3,
        Num = 4,
    }
}

impl_struct! {
    #[repr(C)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct SwitchlessWokerStats {
        pub processed: u64,
        pub missed: u64,
    }
}

impl_asref_array! {
    SwitchlessWokerStats;
}
impl_asmut_array! {
    SwitchlessWokerStats;
}
impl_from_array! {
    SwitchlessWokerStats;
}

pub type SwitchlessWokerCallback = extern "C" fn(
    typ: SwitchlessWokerType,
    event: SwitchlessWokerEvent,
    stats: *const SwitchlessWokerStats,
) -> SgxStatus;

pub const SL_DEFAULT_FALLBACK_RETRIES: u32 = 20000;
pub const SL_DEFAULT_SLEEP_RETRIES: u32 = 20000;
pub const SL_DEFUALT_MAX_TASKS_QWORDS: u32 = 1;
pub const SL_MAX_TASKS_MAX_QWORDS: u32 = 8;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SwitchlessConfig {
    pub pool_size_qwords: u32,
    pub uworkers: u32,
    pub tworkers: u32,
    pub retries_before_fallback: u32,
    pub retries_before_sleep: u32,
    pub callback: [Option<SwitchlessWokerCallback>; 4],
}

impl Default for SwitchlessConfig {
    fn default() -> SwitchlessConfig {
        SwitchlessConfig {
            pool_size_qwords: 0,
            uworkers: 1,
            tworkers: 1,
            retries_before_fallback: 0,
            retries_before_sleep: 0,
            callback: [None; 4],
        }
    }
}
