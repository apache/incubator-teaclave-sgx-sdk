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

use sgx_types::*;
use sgx_tdh::*;
use std::default::Default;

pub const DH_KEY_SIZE: i32  = 20;
pub const NONCE_SIZE : i32  = 16;
pub const MAC_SIZE: i32     = 16;
pub const MAC_KEY_SIZE: i32 = 16;
pub const PADDING_SIZE: i32 = 16;

pub const TAG_SIZE: i32     = 16;
pub const IV_SIZE: i32      = 12;

pub const DERIVE_MAC_KEY: i32 = 0x0;
pub const DERIVE_SESSION_KEY: i32 = 0x1;
pub const DERIVE_VK1_KEY: i32 = 0x3;
pub const DERIVE_VK2_KEY: i32 = 0x4;

pub const CLOSED: u32 = 0x0;
pub const IN_PROGRESS: u32 = 0x1;
pub const ACTIVE: u32 = 0x2;

pub const MESSAGE_EXCHANGE: i32 = 0x0;
pub const ENCLAVE_TO_ENCLAVE_CALL: i32 = 0x1;

pub const INVALID_ARGUMENT: i32 = -2;   ///< Invalid function argument
pub const LOGIC_ERROR: i32 = -3 ;  ///< Functional logic error
pub const FILE_NOT_FOUND : i32 =  -4 ;  ///< File not found

pub const VMC_ATTRIBUTE_MASK: u64 = 0xFFFFFFFFFFFFFFCB;

pub const MAX_SESSION_COUNT: i32 = 16;

pub struct Callback {
    pub verify: fn(&sgx_dh_session_enclave_identity_t) -> u32,
}

pub enum DhSessionStatus {
    Closed,
    InProgress(SgxDhResponder),
    Active(sgx_align_key_128bit_t),
}

impl Default for DhSessionStatus {
    fn default() -> DhSessionStatus {
        DhSessionStatus::Closed
    }
}

#[derive(Default)]
pub struct DhSession {
    pub session_id: u32,
    pub session_status: DhSessionStatus
}

#[derive(Default)]
pub struct DhSessionInfo {
    pub enclave_id: sgx_enclave_id_t,
    pub session: DhSession,
}