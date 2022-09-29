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

use crate::arch::{Align16, Align512};
use crate::enclave::EnclaveRange;
use crate::inst::{self, EncluInst};
use crate::se::AlignReport;
use core::convert::From;
use core::mem;
use core::ptr;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::ContiguousMemory;
use sgx_types::types::KEY_REQUEST_RESERVED2_BYTES;
use sgx_types::types::{AlignKey128bit, AttributesFlags, Key128bit, KeyPolicy, KeyRequest};

#[repr(C, align(512))]
#[derive(Clone, Copy, Debug, Default)]
pub struct AlignKeyRequest(pub KeyRequest);

pub type AlignKey = Align16<Key128bit>;

unsafe impl ContiguousMemory for AlignKeyRequest {}

impl AlignKeyRequest {
    pub fn egetkey(&self) -> SgxResult<AlignKey128bit> {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(self.0.reserved1 == 0, SgxStatus::InvalidParameter);
        ensure!(
            self.0.reserved2 == [0; KEY_REQUEST_RESERVED2_BYTES],
            SgxStatus::InvalidParameter
        );
        ensure!(self.0.key_policy.is_valid(), SgxStatus::InvalidParameter);

        // check if KSS flag is disabled but KSS related policy or config_svn is set
        let report = AlignReport::get_self();
        if !report
            .0
            .body
            .attributes
            .flags
            .intersects(AttributesFlags::KSS)
            && (self
                .0
                .key_policy
                .intersects(KeyPolicy::KSS | KeyPolicy::NOISVPRODID)
                || self.0.config_svn > 0)
        {
            bail!(SgxStatus::InvalidParameter);
        }

        EncluInst::egetkey(self)
            .map(|k| From::from(k.0))
            .map_err(|e| match e {
                inst::INVALID_ATTRIBUTE => SgxStatus::InvalidAttribute,
                inst::INVALID_CPUSVN => SgxStatus::InvalidCpusvn,
                inst::INVALID_ISVSVN => SgxStatus::InvalidIsvsvn,
                inst::INVALID_KEYNAME => SgxStatus::InvalidKeyname,
                _ => SgxStatus::Unexpected,
            })
    }
}

impl AlignKeyRequest {
    pub const UNPADDED_SIZE: usize = mem::size_of::<KeyRequest>();
    pub const ALIGN_SIZE: usize = mem::size_of::<AlignKeyRequest>();

    pub fn try_copy_from(src: &[u8]) -> Option<AlignKeyRequest> {
        if src.len() == Self::UNPADDED_SIZE {
            unsafe {
                let mut ret: Self = mem::zeroed();
                ptr::copy_nonoverlapping(
                    src.as_ptr(),
                    &mut ret as *mut _ as *mut _,
                    Self::UNPADDED_SIZE,
                );
                Some(ret)
            }
        } else {
            None
        }
    }
}

impl AsRef<Align512<[u8; AlignKeyRequest::UNPADDED_SIZE]>> for AlignKeyRequest {
    fn as_ref(&self) -> &Align512<[u8; AlignKeyRequest::UNPADDED_SIZE]> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl From<KeyRequest> for AlignKeyRequest {
    fn from(kr: KeyRequest) -> AlignKeyRequest {
        AlignKeyRequest(kr)
    }
}

impl From<&KeyRequest> for AlignKeyRequest {
    fn from(kr: &KeyRequest) -> AlignKeyRequest {
        AlignKeyRequest(*kr)
    }
}

impl From<AlignKeyRequest> for KeyRequest {
    fn from(kr: AlignKeyRequest) -> KeyRequest {
        kr.0
    }
}
