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

use core::mem;
use core::slice;

pub const TSEAL_DEFAULT_MISCMASK: u32 = !0x0FFF_FFFF;

//
// sgx_tseal.h
//
pub const SEAL_TAG_SIZE: usize = MAC_128BIT_SIZE;
pub const SEAL_IV_SIZE: usize = 12;

impl_struct! {
    #[repr(C)]
    #[derive(Debug)]
    pub struct CAesGcmData {
        pub payload_size: u32,
        pub reserved: [u8; 12],
        pub payload_tag: [u8; SEAL_TAG_SIZE],
        pub payload: [u8; 0],
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct CSealedData {
        pub key_request: KeyRequest,
        pub plaintext_offset: u32,
        pub reserved: [u8; 12],
        pub aes_data: CAesGcmData,
    }
}

impl AsRef<[u8]> for CSealedData {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<CSealedData>() + self.aes_data.payload_size as usize,
            )
        }
    }
}
