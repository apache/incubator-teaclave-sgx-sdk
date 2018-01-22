// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

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
    Active(sgx_key_128bit_t),
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