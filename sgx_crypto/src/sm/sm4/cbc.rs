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

use core::array::TryFromSliceError;
use core::convert::{From, TryFrom, TryInto};
use sgx_crypto_sys::*;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::{BytewiseEquality, ContiguousMemory};
use sgx_types::types::{AlignKey128bit, Key128bit, SM4CBC_IV_SIZE};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Sm4Cbc {
    key: AlignKey128bit,
    iv: Nonce,
}

impl Sm4Cbc {
    const MBS_SMS4: usize = 16;

    pub fn new(key: &Key128bit, iv: Nonce) -> Sm4Cbc {
        Sm4Cbc {
            key: AlignKey128bit::from(key),
            iv,
        }
    }

    pub fn encrypt(&mut self, src: &[u8], dst: &mut [u8]) -> SgxResult {
        let src_len = src.len();
        let dst_len = dst.len();

        ensure!(
            (1..i32::MAX as usize).contains(&src_len),
            SgxStatus::InvalidParameter
        );
        ensure!(src_len % Self::MBS_SMS4 == 0, SgxStatus::InvalidParameter);
        ensure!(src_len == dst_len, SgxStatus::InvalidParameter);

        let status = unsafe {
            sgx_sm4_cbc_encrypt(
                &self.key.key as *const Key128bit,
                src.as_ptr(),
                src_len as u32,
                dst.as_mut_ptr(),
                self.iv.as_ref().as_ptr(),
                self.iv.as_ref().len() as u32,
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
        ensure!(src_len % Self::MBS_SMS4 == 0, SgxStatus::InvalidParameter);
        ensure!(src_len == dst_len, SgxStatus::InvalidParameter);

        let status = unsafe {
            sgx_sm4_cbc_decrypt(
                &self.key.key as *const Key128bit,
                src.as_ptr(),
                src_len as u32,
                dst.as_mut_ptr(),
                self.iv.as_ref().as_ptr(),
                self.iv.as_ref().len() as u32,
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

impl Default for Sm4Cbc {
    fn default() -> Sm4Cbc {
        let mut key = AlignKey128bit::default();
        super::rand(&mut key.key);

        Sm4Cbc {
            key,
            iv: Nonce::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Nonce([u8; SM4CBC_IV_SIZE]);

impl Nonce {
    pub fn new() -> Nonce {
        let mut nonce = [0_u8; SM4CBC_IV_SIZE];
        super::rand(&mut nonce);
        Nonce(nonce)
    }

    #[inline]
    pub fn zeroed() -> Nonce {
        Nonce([0_u8; SM4CBC_IV_SIZE])
    }
}

impl Default for Nonce {
    #[inline]
    fn default() -> Nonce {
        Nonce::new()
    }
}

impl AsRef<[u8; SM4CBC_IV_SIZE]> for Nonce {
    #[inline]
    fn as_ref(&self) -> &[u8; SM4CBC_IV_SIZE] {
        &self.0
    }
}

impl From<[u8; SM4CBC_IV_SIZE]> for Nonce {
    #[inline]
    fn from(nonce: [u8; SM4CBC_IV_SIZE]) -> Nonce {
        Nonce(nonce)
    }
}

impl From<&[u8; SM4CBC_IV_SIZE]> for Nonce {
    #[inline]
    fn from(nonce: &[u8; SM4CBC_IV_SIZE]) -> Nonce {
        Nonce(*nonce)
    }
}

impl TryFrom<&[u8]> for Nonce {
    type Error = TryFromSliceError;

    fn try_from(nonce: &[u8]) -> Result<Nonce, TryFromSliceError> {
        let nonce: &[u8; SM4CBC_IV_SIZE] = nonce.try_into()?;
        Ok(Nonce(*nonce))
    }
}

unsafe impl ContiguousMemory for Nonce {}

unsafe impl BytewiseEquality for Nonce {}
