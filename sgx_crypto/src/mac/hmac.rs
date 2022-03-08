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

use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::mem;
use core::ptr;
use sgx_crypto_sys::*;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::ContiguousMemory;
use sgx_types::memeq::ConstTimeEq;
use sgx_types::types::{AlignMac256bit, HMacHandle, Mac256bit, MAC_256BIT_SIZE};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HashType {
    Sha256,
    Sm3,
}

pub struct HMac {
    hash_type: HashType,
    handle: HMacHandle,
    key: Vec<u8>,
}

impl HMac {
    pub fn new(key: &[u8], hash_type: HashType) -> SgxResult<HMac> {
        ensure!(
            (!key.is_empty() && key.len() <= i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut handle: HMacHandle = ptr::null_mut();
        let key = key.to_owned();
        let status = match hash_type {
            HashType::Sha256 => unsafe {
                sgx_hmac_sha256_init(
                    key.as_ptr(),
                    key.len() as i32,
                    &mut handle as *mut HMacHandle,
                )
            },
            HashType::Sm3 => unsafe {
                sgx_hmac_sm3_init(
                    key.as_ptr(),
                    key.len() as i32,
                    &mut handle as *mut HMacHandle,
                )
            },
        };

        ensure!(status.is_success(), status);
        Ok(HMac {
            hash_type,
            handle,
            key,
        })
    }

    pub fn update<T: ?Sized>(&mut self, data: &T) -> SgxResult
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let status = match self.hash_type {
            HashType::Sha256 => unsafe {
                sgx_hmac_sha256_update((data as *const T).cast(), size as i32, self.handle)
            },
            HashType::Sm3 => unsafe {
                sgx_hmac_sm3_update((data as *const T).cast(), size as i32, self.handle)
            },
        };
        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn finalize(self) -> SgxResult<Mac256bit> {
        let mut mac = AlignMac256bit::default();
        self.finalize_into(&mut mac.mac)?;
        Ok(mac.mac)
    }

    pub fn finalize_align(self) -> SgxResult<AlignMac256bit> {
        let mut mac = AlignMac256bit::default();
        self.finalize_into(&mut mac.mac)?;
        Ok(mac)
    }

    pub fn finalize_into(self, mac: &mut Mac256bit) -> SgxResult {
        let status = match self.hash_type {
            HashType::Sha256 => unsafe {
                sgx_hmac_sha256_final(
                    (mac as *mut Mac256bit).cast(),
                    MAC_256BIT_SIZE as i32,
                    self.handle,
                )
            },
            HashType::Sm3 => unsafe {
                sgx_hmac_sm3_final(
                    (mac as *mut Mac256bit).cast(),
                    MAC_256BIT_SIZE as i32,
                    self.handle,
                )
            },
        };
        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn finalize_reset(&mut self) -> SgxResult<Mac256bit> {
        let mut mac = AlignMac256bit::default();
        self.finalize_into_reset(&mut mac.mac)?;
        Ok(mac.mac)
    }

    pub fn finalize_reset_align(&mut self) -> SgxResult<AlignMac256bit> {
        let mut mac = AlignMac256bit::default();
        self.finalize_into_reset(&mut mac.mac)?;
        Ok(mac)
    }

    pub fn finalize_into_reset(&mut self, mac: &mut Mac256bit) -> SgxResult {
        match self.hash_type {
            HashType::Sha256 => {
                let status = unsafe {
                    sgx_hmac_sha256_final(
                        (mac as *mut Mac256bit).cast(),
                        MAC_256BIT_SIZE as i32,
                        self.handle,
                    )
                };
                ensure!(status.is_success(), status);

                let status = unsafe { sgx_hmac_sha256_close(self.handle) };
                debug_assert!(status.is_success());

                let status = unsafe {
                    sgx_hmac_sha256_init(
                        self.key.as_ptr(),
                        self.key.len() as i32,
                        &mut self.handle as *mut HMacHandle,
                    )
                };
                ensure!(status.is_success(), status);

                Ok(())
            }
            HashType::Sm3 => {
                let status = unsafe {
                    sgx_hmac_sm3_final(
                        (mac as *mut Mac256bit).cast(),
                        MAC_256BIT_SIZE as i32,
                        self.handle,
                    )
                };
                ensure!(status.is_success(), status);

                let status = unsafe { sgx_hmac_sm3_close(self.handle) };
                debug_assert!(status.is_success());

                let status = unsafe {
                    sgx_hmac_sm3_init(
                        self.key.as_ptr(),
                        self.key.len() as i32,
                        &mut self.handle as *mut HMacHandle,
                    )
                };
                ensure!(status.is_success(), status);

                Ok(())
            }
        }
    }

    pub fn verify(self, mac: &Mac256bit) -> SgxResult {
        let mac_result = self.finalize_align()?;
        ensure!(&mac_result.mac.ct_eq(mac), SgxStatus::MacMismatch);
        Ok(())
    }

    pub fn hmac<T: ?Sized>(key: &[u8], hash_type: HashType, data: &T) -> SgxResult<Mac256bit>
    where
        T: ContiguousMemory,
    {
        let mut mac = AlignMac256bit::default();
        Self::hmac_into(key, hash_type, data, &mut mac.mac)?;
        Ok(mac.mac)
    }

    pub fn hmac_align<T: ?Sized>(
        key: &[u8],
        hash_type: HashType,
        data: &T,
    ) -> SgxResult<AlignMac256bit>
    where
        T: ContiguousMemory,
    {
        let mut mac = AlignMac256bit::default();
        Self::hmac_into(key, hash_type, data, &mut mac.mac)?;
        Ok(mac)
    }

    pub fn hmac_into<T: ?Sized>(
        key: &[u8],
        hash_type: HashType,
        data: &T,
        mac: &mut Mac256bit,
    ) -> SgxResult
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );
        ensure!(
            (!key.is_empty() && key.len() <= i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let status = match hash_type {
            HashType::Sha256 => unsafe {
                sgx_hmac_sha256_msg(
                    (data as *const T).cast(),
                    size as i32,
                    key.as_ptr(),
                    key.len() as i32,
                    (mac as *mut Mac256bit).cast(),
                    MAC_256BIT_SIZE as i32,
                )
            },
            HashType::Sm3 => unsafe {
                sgx_hmac_sm3_msg(
                    (data as *const T).cast(),
                    size as i32,
                    key.as_ptr(),
                    key.len() as i32,
                    (mac as *mut Mac256bit).cast(),
                    MAC_256BIT_SIZE as i32,
                )
            },
        };
        ensure!(status.is_success(), status);

        Ok(())
    }
}

impl Drop for HMac {
    fn drop(&mut self) {
        match self.hash_type {
            HashType::Sha256 => unsafe { sgx_hmac_sha256_close(self.handle) },
            HashType::Sm3 => unsafe { sgx_hmac_sm3_close(self.handle) },
        };
    }
}
