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

use core::array::TryFromSliceError;
use core::convert::{From, TryFrom, TryInto};
use sgx_crypto_sys::*;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::{BytewiseEquality, ContiguousMemory};
use sgx_types::types::{AlignKey128bit, Key128bit, AESCTR_CTR_SIZE};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AesCtr {
    key: AlignKey128bit,
    ctr: Counter,
}

impl AesCtr {
    const CTR_INC_BITS: u32 = 128;

    pub fn new(key: &Key128bit, ctr: Counter) -> AesCtr {
        AesCtr {
            key: AlignKey128bit::from(key),
            ctr,
        }
    }

    pub fn encrypt(&mut self, src: &[u8], dst: &mut [u8]) -> SgxResult {
        let src_len = src.len();
        let dst_len = dst.len();

        ensure!(
            (1..i32::MAX as usize).contains(&src_len),
            SgxStatus::InvalidParameter
        );
        ensure!(src_len == dst_len, SgxStatus::InvalidParameter);

        let mut ctr = self.ctr;
        let status = unsafe {
            sgx_aes_ctr_encrypt(
                &self.key.key as *const Key128bit,
                src.as_ptr(),
                src_len as u32,
                ctr.as_mut() as *mut u8,
                Self::CTR_INC_BITS,
                dst.as_mut_ptr(),
            )
        };

        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn encrypt_in_place(&mut self, in_out: &mut [u8]) -> SgxResult {
        let mut dst = vec![0_u8; in_out.len()];
        self.encrypt(in_out, dst.as_mut_slice())?;
        in_out.clone_from_slice(dst.as_slice());
        Ok(())
    }

    pub fn decrypt(&mut self, src: &[u8], dst: &mut [u8]) -> SgxResult {
        let src_len = src.len();
        let dst_len = dst.len();

        ensure!(
            (1..i32::MAX as usize).contains(&src_len),
            SgxStatus::InvalidParameter
        );
        ensure!(src_len == dst_len, SgxStatus::InvalidParameter);

        let mut ctr = self.ctr;
        let status = unsafe {
            sgx_aes_ctr_decrypt(
                &self.key.key as *const Key128bit,
                src.as_ptr(),
                src_len as u32,
                ctr.as_mut() as *mut u8,
                Self::CTR_INC_BITS,
                dst.as_mut_ptr(),
            )
        };

        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn decrypt_in_place(&mut self, in_out: &mut [u8]) -> SgxResult {
        let mut dst = vec![0_u8; in_out.len()];
        self.decrypt(in_out, dst.as_mut_slice())?;
        in_out.clone_from_slice(dst.as_slice());
        Ok(())
    }
}

impl Default for AesCtr {
    fn default() -> AesCtr {
        let mut key = AlignKey128bit::default();
        super::rand(&mut key.key);

        AesCtr {
            key,
            ctr: Counter::nonce(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Counter([u8; AESCTR_CTR_SIZE]);

impl Counter {
    pub fn nonce() -> Counter {
        let mut nonce = [0_u8; AESCTR_CTR_SIZE];
        super::rand(&mut nonce);
        Counter(nonce)
    }

    #[inline]
    pub fn zeroed() -> Counter {
        Counter([0; AESCTR_CTR_SIZE])
    }
}

impl Default for Counter {
    fn default() -> Counter {
        Counter::nonce()
    }
}

impl AsRef<[u8; AESCTR_CTR_SIZE]> for Counter {
    fn as_ref(&self) -> &[u8; AESCTR_CTR_SIZE] {
        &self.0
    }
}

impl AsMut<[u8; AESCTR_CTR_SIZE]> for Counter {
    fn as_mut(&mut self) -> &mut [u8; AESCTR_CTR_SIZE] {
        &mut self.0
    }
}

impl From<[u8; AESCTR_CTR_SIZE]> for Counter {
    fn from(ctr: [u8; AESCTR_CTR_SIZE]) -> Counter {
        Counter(ctr)
    }
}

impl TryFrom<&[u8]> for Counter {
    type Error = TryFromSliceError;

    fn try_from(ctr: &[u8]) -> Result<Counter, TryFromSliceError> {
        let ctr: &[u8; AESCTR_CTR_SIZE] = ctr.try_into()?;
        Ok(Counter(*ctr))
    }
}

unsafe impl ContiguousMemory for Counter {}

unsafe impl BytewiseEquality for Counter {}
