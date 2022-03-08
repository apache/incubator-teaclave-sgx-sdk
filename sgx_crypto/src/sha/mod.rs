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
use sgx_types::types::{Sha1Hash, Sha256Hash, ShaHandle};

pub struct Sha256 {
    handle: ShaHandle,
}

impl Sha256 {
    pub fn new() -> SgxResult<Sha256> {
        let mut handle: ShaHandle = ptr::null_mut();
        let status = unsafe { sgx_sha256_init(&mut handle as *mut ShaHandle) };

        ensure!(status.is_success(), status);
        Ok(Sha256 { handle })
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
            unsafe { sgx_sha256_update((data as *const T).cast(), size as u32, self.handle) };
        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn finalize(self) -> SgxResult<Sha256Hash> {
        let mut hash = Sha256Hash::default();
        self.finalize_into(&mut hash)?;
        Ok(hash)
    }

    pub fn finalize_into(self, hash: &mut Sha256Hash) -> SgxResult {
        let status = unsafe { sgx_sha256_get_hash(self.handle, hash as *mut Sha256Hash) };
        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn finalize_reset(&mut self) -> SgxResult<Sha256Hash> {
        let mut hash = Sha256Hash::default();
        self.finalize_into_reset(&mut hash)?;
        Ok(hash)
    }

    pub fn finalize_into_reset(&mut self, hash: &mut Sha256Hash) -> SgxResult {
        let status = unsafe { sgx_sha256_get_hash(self.handle, hash as *mut Sha256Hash) };
        ensure!(status.is_success(), status);

        let status = unsafe { sgx_sha256_close(self.handle) };
        debug_assert!(status.is_success());

        let status = unsafe { sgx_sha256_init(&mut self.handle as *mut ShaHandle) };
        ensure!(status.is_success(), status);

        Ok(())
    }

    pub fn digest<T: ?Sized>(data: &T) -> SgxResult<Sha256Hash>
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut hash = Sha256Hash::default();
        let status = unsafe {
            sgx_sha256_msg(
                (data as *const T).cast(),
                size as u32,
                &mut hash as *mut Sha256Hash,
            )
        };
        ensure!(status.is_success(), status);

        Ok(hash)
    }
}

impl Drop for Sha256 {
    fn drop(&mut self) {
        let _ = unsafe { sgx_sha256_close(self.handle) };
    }
}

pub struct Sha1 {
    handle: ShaHandle,
}

impl Sha1 {
    pub fn new() -> SgxResult<Sha1> {
        let mut handle: ShaHandle = ptr::null_mut();
        let status = unsafe { sgx_sha1_init(&mut handle as *mut ShaHandle) };

        ensure!(status.is_success(), status);
        Ok(Sha1 { handle })
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
            unsafe { sgx_sha1_update((data as *const T).cast(), size as u32, self.handle) };
        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn finalize(self) -> SgxResult<Sha1Hash> {
        let mut hash = Sha1Hash::default();
        self.finalize_into(&mut hash)?;
        Ok(hash)
    }

    pub fn finalize_into(self, hash: &mut Sha1Hash) -> SgxResult {
        let status = unsafe { sgx_sha1_get_hash(self.handle, hash as *mut Sha1Hash) };
        ensure!(status.is_success(), status);
        Ok(())
    }

    pub fn finalize_reset(&mut self) -> SgxResult<Sha1Hash> {
        let mut hash = Sha1Hash::default();
        self.finalize_into_reset(&mut hash)?;
        Ok(hash)
    }

    pub fn finalize_into_reset(&mut self, hash: &mut Sha1Hash) -> SgxResult {
        let status = unsafe { sgx_sha1_get_hash(self.handle, hash as *mut Sha1Hash) };
        ensure!(status.is_success(), status);

        let status = unsafe { sgx_sha1_close(self.handle) };
        debug_assert!(status.is_success());

        let status = unsafe { sgx_sha1_init(&mut self.handle as *mut ShaHandle) };
        ensure!(status.is_success(), status);

        Ok(())
    }

    pub fn digest<T: ?Sized>(data: &T) -> SgxResult<Sha1Hash>
    where
        T: ContiguousMemory,
    {
        let size = mem::size_of_val(data);
        ensure!(
            (size > 0 && size < i32::MAX as usize),
            SgxStatus::InvalidParameter
        );

        let mut hash = Sha1Hash::default();
        let status = unsafe {
            sgx_sha1_msg(
                (data as *const T).cast(),
                size as u32,
                &mut hash as *mut Sha1Hash,
            )
        };
        ensure!(status.is_success(), status);

        Ok(hash)
    }
}

impl Drop for Sha1 {
    fn drop(&mut self) {
        let _ = unsafe { sgx_sha1_close(self.handle) };
    }
}
