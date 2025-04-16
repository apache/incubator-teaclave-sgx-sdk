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

#![allow(deprecated)]

use crate::ocall::util::*;
use libc::{self, c_char, c_int, c_void, size_t, uint64_t, uint8_t};
use sgx_uprotected_fs::{self as ufs, HostFile, RawFileStream, RecoveryFile};
use std::ffi::CStr;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::ptr;
use std::slice;

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_open_ocall(
    error: *mut c_int,
    name: *const c_char,
    readonly: uint8_t,
    size: *mut size_t,
) -> *mut c_void {
    if name.is_null() || size.is_null() {
        set_error(error, libc::EINVAL);
        return ptr::null_mut();
    }

    let name = match CStr::from_ptr(name).to_str() {
        Ok(name) => name,
        Err(_) => {
            set_error(error, libc::EINVAL);
            return ptr::null_mut();
        }
    };

    let file = match HostFile::open(Path::new(name), readonly != 0) {
        Ok(file) => file,
        Err(errno) => {
            set_error(error, errno);
            return ptr::null_mut();
        }
    };
    let sz = match file.size() {
        Ok(size) => size,
        Err(errno) => {
            set_error(error, errno);
            return ptr::null_mut();
        }
    };
    unsafe { size.write_unaligned(sz) };

    file.into_raw_stream() as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_read_ocall(
    error: *mut c_int,
    file: *mut c_void,
    node_number: uint64_t,
    node: *mut uint8_t,
    size: size_t,
) -> c_int {
    if file.is_null() || node.is_null() || size == 0 {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let mut file = match HostFile::from_raw_stream(file as RawFileStream) {
        Ok(file) => ManuallyDrop::new(file),
        Err(errno) => {
            set_error(error, errno);
            return -1;
        }
    };

    let node = slice::from_raw_parts_mut(node, size);
    match file.read(node_number, node) {
        Ok(_) => 0,
        Err(errno) => {
            set_error(error, errno);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_write_ocall(
    error: *mut c_int,
    file: *mut c_void,
    node_number: uint64_t,
    node: *const uint8_t,
    size: size_t,
) -> c_int {
    if file.is_null() || node.is_null() || size == 0 {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let mut file = match HostFile::from_raw_stream(file as RawFileStream) {
        Ok(file) => ManuallyDrop::new(file),
        Err(errno) => {
            set_error(error, errno);
            return -1;
        }
    };

    let node = slice::from_raw_parts(node, size);
    match file.write(node_number, node) {
        Ok(_) => 0,
        Err(errno) => {
            set_error(error, errno);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_flush_ocall(error: *mut c_int, file: *mut c_void) -> c_int {
    let mut file = match HostFile::from_raw_stream(file as RawFileStream) {
        Ok(file) => ManuallyDrop::new(file),
        Err(errno) => {
            set_error(error, errno);
            return -1;
        }
    };

    match file.flush() {
        Ok(_) => 0,
        Err(errno) => {
            set_error(error, errno);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_close_ocall(error: *mut c_int, file: *mut c_void) -> c_int {
    match HostFile::from_raw_stream(file as RawFileStream) {
        Ok(file) => {
            drop(file);
            0
        }
        Err(errno) => {
            set_error(error, errno);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_open_recovery_ocall(
    error: *mut c_int,
    name: *const c_char,
) -> *mut c_void {
    if name.is_null() {
        set_error(error, libc::EINVAL);
        return ptr::null_mut();
    }

    let name = match CStr::from_ptr(name).to_str() {
        Ok(name) => name,
        Err(_) => {
            set_error(error, libc::EINVAL);
            return ptr::null_mut();
        }
    };

    match RecoveryFile::open(Path::new(name)) {
        Ok(file) => file.into_raw_stream() as *mut c_void,
        Err(errno) => {
            set_error(error, errno);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_write_recovery_ocall(
    error: *mut c_int,
    file: *mut c_void,
    node: *const uint8_t,
    size: size_t,
) -> c_int {
    if file.is_null() || node.is_null() || size == 0 {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let mut file = match RecoveryFile::from_raw_stream(file as RawFileStream) {
        Ok(file) => ManuallyDrop::new(file),
        Err(errno) => {
            set_error(error, errno);
            return -1;
        }
    };

    let node = slice::from_raw_parts(node, size);
    match file.write(node) {
        Ok(_) => 0,
        Err(errno) => {
            set_error(error, errno);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_close_recovery_ocall(
    error: *mut c_int,
    file: *mut c_void,
) -> c_int {
    match RecoveryFile::from_raw_stream(file as RawFileStream) {
        Ok(file) => {
            drop(file);
            0
        }
        Err(errno) => {
            set_error(error, errno);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_exists_ocall(
    error: *mut c_int,
    name: *const c_char,
    is_exists: *mut uint8_t,
) -> c_int {
    if name.is_null() || is_exists.is_null() {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let name = match CStr::from_ptr(name).to_str() {
        Ok(name) => name,
        Err(_) => {
            set_error(error, libc::EINVAL);
            return -1;
        }
    };

    *is_exists = 0;
    match ufs::try_exists(Path::new(name)) {
        Ok(exists) => {
            if exists {
                *is_exists = 1;
            }
            0
        }
        Err(errno) => {
            set_error(error, errno);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_remove_ocall(error: *mut c_int, name: *const c_char) -> c_int {
    if name.is_null() {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let name = match CStr::from_ptr(name).to_str() {
        Ok(name) => name,
        Err(_) => {
            set_error(error, libc::EINVAL);
            return -1;
        }
    };

    match ufs::remove(Path::new(name)) {
        Ok(_) => 0,
        Err(errno) => {
            set_error(error, errno);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_sgxfs_recovery_ocall(
    error: *mut c_int,
    source: *const c_char,
    recovery: *const c_char,
) -> c_int {
    if source.is_null() || recovery.is_null() {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let source = match CStr::from_ptr(source).to_str() {
        Ok(name) => name,
        Err(_) => {
            set_error(error, libc::EINVAL);
            return -1;
        }
    };

    let recovery = match CStr::from_ptr(recovery).to_str() {
        Ok(name) => name,
        Err(_) => {
            set_error(error, libc::EINVAL);
            return -1;
        }
    };

    match ufs::recovery(Path::new(source), Path::new(recovery)) {
        Ok(_) => 0,
        Err(errno) => {
            set_error(error, errno);
            -1
        }
    }
}
