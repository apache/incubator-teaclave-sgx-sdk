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

use core::mem;
use core::ptr;
use sgx_crypto_sys::*;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::ContiguousMemory;
use sgx_types::types::{Sm3Handle, Sm3Hash};

pub struct Sm3 {
    handle: Sm3Handle,
}

impl Sm3 {
    pub fn new() -> SgxResult<Sm3> {
        let mut handle: Sm3Handle = ptr::null_mut();
        let status = unsafe { sgx_sm3_init(&mut handle as *mut Sm3Handle) };

        ensure!(status.is_success(), status);
        Ok(Sm3 { handle })
    }

    pub fn update<T: ?Sized>(&mut self, data: &T) -> SgxResult
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size <= u32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let status = unsafe { sgx_sm3_update((data as *const T).cast(), size as u32, self.handle) };
        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn finalize(self) -> SgxResult<Sm3Hash> {
        let mut hash = Sm3Hash::default();
        self.finalize_into(&mut hash)?;
        Ok(hash)
    }

    pub fn finalize_into(self, hash: &mut Sm3Hash) -> SgxResult {
        let status = unsafe { sgx_sm3_get_hash(self.handle, hash as *mut Sm3Hash) };
        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn finalize_reset(&mut self) -> SgxResult<Sm3Hash> {
        let mut hash = Sm3Hash::default();
        self.finalize_into_reset(&mut hash)?;
        Ok(hash)
    }

    pub fn finalize_into_reset(&mut self, hash: &mut Sm3Hash) -> SgxResult {
        let status = unsafe { sgx_sm3_get_hash(self.handle, hash as *mut Sm3Hash) };
        ensure!(status.is_success(), status);

        let status = unsafe { sgx_sm3_close(self.handle) };
        debug_assert!(status.is_success());

        let status = unsafe { sgx_sm3_init(&mut self.handle as *mut Sm3Handle) };
        ensure!(status.is_success(), status);

        Ok(())
    }

    pub fn digest<T: ?Sized>(data: &T) -> SgxResult<Sm3Hash>
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size <= u32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut hash = Sm3Hash::default();
        let status = unsafe {
            sgx_sm3_msg(
                (data as *const T).cast(),
                size as u32,
                &mut hash as *mut Sm3Hash,
            )
        };
        ensure!(status.is_success(), status);

        Ok(hash)
    }
}

impl Drop for Sm3 {
    fn drop(&mut self) {
        let _ = unsafe { sgx_sm3_close(self.handle) };
    }
}
