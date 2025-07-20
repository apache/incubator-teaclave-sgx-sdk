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

use super::HostFs;
use crate::sys::error::FsResult;
use crate::sys::node::NODE_SIZE;
use sgx_types::error::errno::*;
use sgx_types::error::SgxStatus;
use sgx_types::types::{c_char, c_int, c_void, size_t, uint64_t, uint8_t};
use std::ffi::CString;
use std::path::Path;
use std::ptr;

type RawFileStream = *mut c_void;

extern "C" {
    fn u_sgxfs_open_ocall(
        file: *mut *mut c_void,
        error: *mut c_int,
        name: *const c_char,
        readonly: uint8_t,
        size: *mut size_t,
    ) -> SgxStatus;

    fn u_sgxfs_read_ocall(
        result: *mut c_int,
        error: *mut c_int,
        file: *mut c_void,
        number: uint64_t,
        node: *mut uint8_t,
        size: size_t,
    ) -> SgxStatus;

    fn u_sgxfs_write_ocall(
        result: *mut c_int,
        error: *mut c_int,
        file: *mut c_void,
        number: uint64_t,
        node: *const uint8_t,
        size: size_t,
    ) -> SgxStatus;

    fn u_sgxfs_flush_ocall(result: *mut c_int, error: *mut c_int, file: *mut c_void) -> SgxStatus;

    fn u_sgxfs_close_ocall(result: *mut c_int, error: *mut c_int, file: *mut c_void) -> SgxStatus;

    fn u_sgxfs_exists_ocall(
        result: *mut c_int,
        error: *mut c_int,
        name: *const c_char,
        is_exists: *mut uint8_t,
    ) -> SgxStatus;

    fn u_sgxfs_remove_ocall(
        result: *mut c_int,
        error: *mut c_int,
        name: *const c_char,
    ) -> SgxStatus;

    fn u_sgxfs_open_recovery_ocall(
        file: *mut *mut c_void,
        error: *mut c_int,
        name: *const c_char,
    ) -> SgxStatus;

    fn u_sgxfs_write_recovery_ocall(
        result: *mut c_int,
        error: *mut c_int,
        file: *mut c_void,
        node: *const uint8_t,
        size: size_t,
    ) -> SgxStatus;

    fn u_sgxfs_close_recovery_ocall(
        result: *mut c_int,
        error: *mut c_int,
        file: *mut c_void,
    ) -> SgxStatus;

    fn u_sgxfs_recovery_ocall(
        result: *mut c_int,
        error: *mut c_int,
        source: *const c_char,
        recovery: *const c_char,
    ) -> SgxStatus;
}

fn cstr(name: &Path) -> FsResult<CString> {
    CString::new(name.to_str().ok_or(EINVAL)?).map_err(|_| eos!(EINVAL))
}

#[derive(Debug)]
pub struct HostFile {
    file: RawFileStream,
    size: usize,
}

impl HostFile {
    pub fn open(name: &Path, readonly: bool) -> FsResult<HostFile> {
        let mut file: RawFileStream = ptr::null_mut();
        let mut size: size_t = 0;
        let mut error: c_int = 0;

        let name = cstr(name)?;
        let status = unsafe {
            u_sgxfs_open_ocall(
                &mut file as *mut *mut c_void,
                &mut error as *mut c_int,
                name.as_ptr(),
                readonly as uint8_t,
                &mut size as *mut size_t,
            )
        };

        ensure!(status.is_success(), esgx!(status));
        ensure!(
            !file.is_null(),
            eos!(if error != 0 { error } else { EACCES })
        );
        ensure!(
            size <= i64::MAX as usize && size % NODE_SIZE == 0,
            esgx!(SgxStatus::NotSgxFile)
        );
        Ok(HostFile { file, size })
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    fn close(&mut self) -> FsResult {
        let mut result: c_int = 0;
        let mut error: c_int = 0;

        let status = unsafe {
            u_sgxfs_close_ocall(
                &mut result as *mut c_int,
                &mut error as *mut c_int,
                self.file,
            )
        };

        ensure!(status.is_success(), esgx!(status));
        ensure!(
            result == 0,
            if error != 0 {
                eos!(error)
            } else {
                esgx!(SgxStatus::CloseFailed)
            }
        );
        Ok(())
    }
}

impl HostFs for HostFile {
    fn read(&mut self, number: u64, node: &mut dyn AsMut<[u8]>) -> FsResult {
        let mut result: c_int = 0;
        let mut error: c_int = 0;

        let status = unsafe {
            u_sgxfs_read_ocall(
                &mut result as *mut c_int,
                &mut error as *mut c_int,
                self.file,
                number,
                node.as_mut().as_mut_ptr(),
                node.as_mut().len(),
            )
        };

        ensure!(status.is_success(), esgx!(status));
        ensure!(result == 0, eos!(if error != 0 { error } else { EIO }));
        Ok(())
    }

    fn write(&mut self, number: u64, node: &dyn AsRef<[u8]>) -> FsResult {
        let mut result: c_int = 0;
        let mut error: c_int = 0;

        let status = unsafe {
            u_sgxfs_write_ocall(
                &mut result as *mut c_int,
                &mut error as *mut c_int,
                self.file,
                number,
                node.as_ref().as_ptr(),
                node.as_ref().len(),
            )
        };

        ensure!(status.is_success(), esgx!(status));
        ensure!(result == 0, eos!(if error != 0 { error } else { EIO }));
        Ok(())
    }

    fn flush(&mut self) -> FsResult {
        let mut result: c_int = 0;
        let mut error: c_int = 0;

        let status = unsafe {
            u_sgxfs_flush_ocall(
                &mut result as *mut c_int,
                &mut error as *mut c_int,
                self.file,
            )
        };

        ensure!(status.is_success(), esgx!(status));
        ensure!(result == 0, esgx!(SgxStatus::FluchFailed));
        Ok(())
    }
}

impl Drop for HostFile {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[derive(Debug)]
pub struct RecoveryFile {
    file: RawFileStream,
}

impl RecoveryFile {
    pub fn open(name: &Path) -> FsResult<RecoveryFile> {
        let mut file: RawFileStream = ptr::null_mut();
        let mut error: c_int = 0;

        let name = cstr(name)?;
        let status = unsafe {
            u_sgxfs_open_recovery_ocall(
                &mut file as *mut *mut c_void,
                &mut error as *mut c_int,
                name.as_ptr(),
            )
        };

        ensure!(status.is_success(), esgx!(status));
        ensure!(!file.is_null(), esgx!(SgxStatus::CantOpenRecoveryFile));
        Ok(RecoveryFile { file })
    }

    fn close(&mut self) -> FsResult {
        let mut result: c_int = 0;
        let mut error: c_int = 0;

        let status = unsafe {
            u_sgxfs_close_recovery_ocall(
                &mut result as *mut c_int,
                &mut error as *mut c_int,
                self.file,
            )
        };

        ensure!(status.is_success(), esgx!(status));
        ensure!(
            result == 0,
            if error != 0 {
                eos!(error)
            } else {
                esgx!(SgxStatus::CloseFailed)
            }
        );
        Ok(())
    }
}

impl HostFs for RecoveryFile {
    fn read(&mut self, _number: u64, _node: &mut dyn AsMut<[u8]>) -> FsResult {
        bail!(eos!(ENOTSUP))
    }

    fn write(&mut self, _number: u64, node: &dyn AsRef<[u8]>) -> FsResult {
        let mut result: c_int = 0;
        let mut error: c_int = 0;

        let status = unsafe {
            u_sgxfs_write_recovery_ocall(
                &mut result as *mut c_int,
                &mut error as *mut c_int,
                self.file,
                node.as_ref().as_ptr(),
                node.as_ref().len(),
            )
        };

        ensure!(status.is_success(), esgx!(status));
        ensure!(result == 0, esgx!(SgxStatus::CantWriteRecoveryFile));
        Ok(())
    }

    fn flush(&mut self) -> FsResult {
        bail!(eos!(ENOTSUP))
    }
}

impl Drop for RecoveryFile {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub fn try_exists(name: &Path) -> FsResult<bool> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let mut is_exists = 0_u8;
    let name = cstr(name)?;
    let status = unsafe {
        u_sgxfs_exists_ocall(
            &mut result as *mut c_int,
            &mut error as *mut c_int,
            name.as_ptr(),
            &mut is_exists as *mut uint8_t,
        )
    };

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    Ok(is_exists != 0)
}

pub fn remove(name: &Path) -> FsResult {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let name = cstr(name)?;
    let status = unsafe {
        u_sgxfs_remove_ocall(
            &mut result as *mut c_int,
            &mut error as *mut c_int,
            name.as_ptr(),
        )
    };

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(if error != 0 { error } else { EPERM }));
    Ok(())
}

pub fn recovery(source: &Path, recovery: &Path) -> FsResult {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let source = cstr(source)?;
    let recovery = cstr(recovery)?;

    let status = unsafe {
        u_sgxfs_recovery_ocall(
            &mut result as *mut c_int,
            &mut error as *mut c_int,
            source.as_ptr(),
            recovery.as_ptr(),
        )
    };

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(if error != 0 { error } else { EINVAL }));
    Ok(())
}
