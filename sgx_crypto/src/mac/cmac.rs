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

use core::mem;
use core::ptr;
use sgx_crypto_sys::*;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::ContiguousMemory;
use sgx_types::memeq::ConstTimeEq;
use sgx_types::types::{AlignKey128bit, AlignMac128bit, CMacHandle, Key128bit, Mac128bit};

pub struct AesCMac {
    handle: CMacHandle,
    key: AlignKey128bit,
}

impl AesCMac {
    pub fn new(key: &Key128bit) -> SgxResult<AesCMac> {
        let mut handle: CMacHandle = ptr::null_mut();
        let key = AlignKey128bit::from(key);
        let status = unsafe {
            sgx_cmac128_init(&key.key as *const Key128bit, &mut handle as *mut CMacHandle)
        };

        ensure!(status.is_success(), status);
        Ok(AesCMac { handle, key })
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

        let status =
            unsafe { sgx_cmac128_update((data as *const T).cast(), size as u32, self.handle) };
        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn finalize(self) -> SgxResult<Mac128bit> {
        let mut mac = AlignMac128bit::default();
        self.finalize_into(&mut mac.mac)?;
        Ok(mac.mac)
    }

    pub fn finalize_align(self) -> SgxResult<AlignMac128bit> {
        let mut mac = AlignMac128bit::default();
        self.finalize_into(&mut mac.mac)?;
        Ok(mac)
    }

    pub fn finalize_into(self, mac: &mut Mac128bit) -> SgxResult {
        let status = unsafe { sgx_cmac128_final(self.handle, mac as *mut Mac128bit) };
        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn finalize_reset(&mut self) -> SgxResult<Mac128bit> {
        let mut mac = AlignMac128bit::default();
        self.finalize_into_reset(&mut mac.mac)?;
        Ok(mac.mac)
    }

    pub fn finalize_reset_align(&mut self) -> SgxResult<AlignMac128bit> {
        let mut mac = AlignMac128bit::default();
        self.finalize_into_reset(&mut mac.mac)?;
        Ok(mac)
    }

    pub fn finalize_into_reset(&mut self, mac: &mut Mac128bit) -> SgxResult {
        let status = unsafe { sgx_cmac128_final(self.handle, mac as *mut Mac128bit) };
        ensure!(status.is_success(), status);

        let status = unsafe { sgx_cmac128_close(self.handle) };
        debug_assert!(status.is_success());

        let status = unsafe {
            sgx_cmac128_init(
                &self.key.key as *const Key128bit,
                &mut self.handle as *mut CMacHandle,
            )
        };
        ensure!(status.is_success(), status);

        Ok(())
    }

    pub fn verify(self, mac: &Mac128bit) -> SgxResult {
        let mac_result = self.finalize_align()?;
        ensure!(&mac_result.mac.ct_eq(mac), SgxStatus::MacMismatch);
        Ok(())
    }

    pub fn cmac<T: ?Sized>(key: &Key128bit, data: &T) -> SgxResult<Mac128bit>
    where
        T: ContiguousMemory,
    {
        let mut mac = AlignMac128bit::default();
        Self::cmac_into(key, data, &mut mac.mac)?;
        Ok(mac.mac)
    }

    pub fn cmac_align<T: ?Sized>(key: &Key128bit, data: &T) -> SgxResult<AlignMac128bit>
    where
        T: ContiguousMemory,
    {
        let mut mac = AlignMac128bit::default();
        Self::cmac_into(key, data, &mut mac.mac)?;
        Ok(mac)
    }

    fn cmac_into<T: ?Sized>(key: &Key128bit, data: &T, mac: &mut Mac128bit) -> SgxResult
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let status = unsafe {
            sgx_rijndael128_cmac_msg(
                key as *const Key128bit,
                (data as *const T).cast(),
                size as u32,
                mac as *mut Mac128bit,
            )
        };
        ensure!(status.is_success(), status);

        Ok(())
    }
}

impl Drop for AesCMac {
    fn drop(&mut self) {
        unsafe { sgx_cmac128_close(self.handle) };
    }
}
