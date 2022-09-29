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

use crate::sys::error::FsResult;
use crate::sys::{self as fs_imp, EncryptMode, OpenOptions, RawProtectedFile, SgxFile};
use sgx_types::error::errno::*;
use sgx_types::types::c_char;
#[cfg(feature = "tfs")]
use sgx_types::types::KeyPolicy;
use sgx_types::types::{Key128bit, Mac128bit};
use std::ffi::CStr;
use std::io::SeekFrom;
use std::mem::ManuallyDrop;
use std::ptr;
use std::slice;

pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_fopen(
    filename: *const c_char,
    mode: *const c_char,
    key: *const Key128bit,
) -> RawProtectedFile {
    if filename.is_null() || mode.is_null() {
        eos!(EINVAL).set_errno();
        return ptr::null_mut();
    }

    let name = match CStr::from_ptr(filename).to_str() {
        Ok(name) => name,
        Err(_) => {
            eos!(EINVAL).set_errno();
            return ptr::null_mut();
        }
    };

    let opts = match parse_mode(CStr::from_ptr(mode)) {
        Ok(mode) => mode,
        Err(error) => {
            error.set_errno();
            return ptr::null_mut();
        }
    };

    let encrypt_mode = if key.is_null() {
        cfg_if! {
            if #[cfg(feature = "tfs")] {
                EncryptMode::EncryptAutoKey(KeyPolicy::MRSIGNER)
            } else {
                eos!(EINVAL).set_errno();
                return ptr::null_mut();
            }
        }
    } else {
        EncryptMode::EncryptUserKey(*key)
    };
    match SgxFile::open(name, &opts, &encrypt_mode, None) {
        Ok(file) => file.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// # Safety
#[cfg(feature = "tfs")]
#[no_mangle]
pub unsafe extern "C" fn sgx_fopen_auto_key(
    filename: *const c_char,
    mode: *const c_char,
) -> RawProtectedFile {
    if filename.is_null() || mode.is_null() {
        eos!(EINVAL).set_errno();
        return ptr::null_mut();
    }

    let name = match CStr::from_ptr(filename).to_str() {
        Ok(name) => name,
        Err(_) => {
            eos!(EINVAL).set_errno();
            return ptr::null_mut();
        }
    };

    let opts = match parse_mode(CStr::from_ptr(mode)) {
        Ok(mode) => mode,
        Err(error) => {
            error.set_errno();
            return ptr::null_mut();
        }
    };

    let encrypt_mode = EncryptMode::EncryptAutoKey(KeyPolicy::MRSIGNER);
    match SgxFile::open(name, &opts, &encrypt_mode, None) {
        Ok(file) => file.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_fopen_integrity_only(
    filename: *const c_char,
    mode: *const c_char,
) -> RawProtectedFile {
    if filename.is_null() || mode.is_null() {
        eos!(EINVAL).set_errno();
        return ptr::null_mut();
    }

    let name = match CStr::from_ptr(filename).to_str() {
        Ok(name) => name,
        Err(_) => {
            eos!(EINVAL).set_errno();
            return ptr::null_mut();
        }
    };

    let opts = match parse_mode(CStr::from_ptr(mode)) {
        Ok(mode) => mode,
        Err(error) => {
            error.set_errno();
            return ptr::null_mut();
        }
    };

    let encrypt_mode = EncryptMode::IntegrityOnly;
    match SgxFile::open(name, &opts, &encrypt_mode, None) {
        Ok(file) => file.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// # Safety
#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn sgx_fopen_ex(
    filename: *const c_char,
    mode: *const c_char,
    key: *const Key128bit,
    key_policy: u16,
    cache_size: u64,
) -> RawProtectedFile {
    if filename.is_null() || mode.is_null() || cache_size < fs_imp::DEFAULT_CACHE_SIZE as u64 {
        eos!(EINVAL).set_errno();
        return ptr::null_mut();
    }

    let name = match CStr::from_ptr(filename).to_str() {
        Ok(name) => name,
        Err(_) => {
            eos!(EINVAL).set_errno();
            return ptr::null_mut();
        }
    };

    let opts = match parse_mode(CStr::from_ptr(mode)) {
        Ok(mode) => mode,
        Err(error) => {
            error.set_errno();
            return ptr::null_mut();
        }
    };

    let encrypt_mode = if key.is_null() {
        cfg_if! {
            if #[cfg(feature = "tfs")] {
                let key_policy = if key_policy != 0 {
                    match KeyPolicy::from_bits(key_policy) {
                        Some(key_policy) => key_policy,
                        None => {
                            eos!(EINVAL).set_errno();
                            return ptr::null_mut();
                        }
                    }
                } else {
                    KeyPolicy::MRSIGNER
                };
                EncryptMode::EncryptAutoKey(key_policy)
            } else {
                eos!(EINVAL).set_errno();
                return ptr::null_mut();
            }
        }
    } else {
        EncryptMode::EncryptUserKey(*key)
    };
    match SgxFile::open(name, &opts, &encrypt_mode, Some(cache_size as usize)) {
        Ok(file) => file.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_fwrite(
    buf: *const u8,
    size: usize,
    count: usize,
    file: RawProtectedFile,
) -> usize {
    if buf.is_null() || file.is_null() || size == 0 || count == 0 {
        eos!(EINVAL).set_errno();
        return 0;
    }
    let data_size = match size.checked_mul(count) {
        Some(size) => size,
        None => {
            eos!(EINVAL).set_errno();
            return 0;
        }
    };

    let buf = slice::from_raw_parts(buf, data_size);
    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    file.write(buf).map(|nwritten| nwritten / size).unwrap_or(0)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_fread(
    buf: *mut u8,
    size: usize,
    count: usize,
    file: RawProtectedFile,
) -> usize {
    if buf.is_null() || file.is_null() || size == 0 || count == 0 {
        eos!(EINVAL).set_errno();
        return 0;
    }
    let data_size = match size.checked_mul(count) {
        Some(size) => size,
        None => {
            eos!(EINVAL).set_errno();
            return 0;
        }
    };

    let buf = slice::from_raw_parts_mut(buf, data_size);
    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    file.read(buf).map(|nread| nread / size).unwrap_or(0)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ftell(file: RawProtectedFile) -> i64 {
    if file.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    match file.tell() {
        Ok(offset) => offset as i64,
        Err(_) => -1,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_fseek(file: RawProtectedFile, offset: i64, origin: i32) -> i32 {
    if file.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let pos = match origin {
        SEEK_SET => SeekFrom::Start(offset as u64),
        SEEK_CUR => SeekFrom::Current(offset),
        SEEK_END => SeekFrom::End(offset),
        _ => {
            eos!(EINVAL).set_errno();
            return -1;
        }
    };

    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    match file.seek(pos) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_fflush(file: RawProtectedFile) -> i32 {
    if file.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    match file.flush() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ferror(file: RawProtectedFile) -> i32 {
    if file.is_null() {
        eos!(EINVAL).set_errno();
        return EINVAL;
    }

    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    file.get_error().to_errno()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_fget_file_size(file: RawProtectedFile, file_size: *mut u64) -> i32 {
    if file.is_null() || file_size.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    match file.file_size() {
        Ok(size) => {
            *file_size = size;
            0
        }
        Err(_) => -1,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_feof(file: RawProtectedFile) -> i32 {
    if file.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    if file.is_eof() {
        1
    } else {
        0
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_clearerr(file: RawProtectedFile) {
    if file.is_null() {
        return;
    }

    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    let _ = file.clear_error();
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_fclear_cache(file: RawProtectedFile) -> i32 {
    if file.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    match file.clear_cache() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_fget_mac(file: RawProtectedFile, mac: *mut Mac128bit) -> i32 {
    if file.is_null() || mac.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    match file.get_mac() {
        Ok(root_mac) => {
            *mac = root_mac;
            0
        }
        Err(_) => -1,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_frename(
    file: RawProtectedFile,
    old_name: *const c_char,
    new_name: *const c_char,
) -> i32 {
    if file.is_null() || old_name.is_null() || new_name.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let old_name = match CStr::from_ptr(old_name).to_str() {
        Ok(name) => name,
        Err(_) => {
            eos!(EINVAL).set_errno();
            return -1;
        }
    };
    let new_name = match CStr::from_ptr(new_name).to_str() {
        Ok(name) => name,
        Err(_) => {
            eos!(EINVAL).set_errno();
            return -1;
        }
    };

    let file = ManuallyDrop::new(SgxFile::from_raw(file));
    match file.rename(old_name, new_name) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_fclose(file: RawProtectedFile) -> i32 {
    if file.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let file = SgxFile::from_raw(file);
    drop(file);
    0
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_remove(filename: *const c_char) -> i32 {
    if filename.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let name = match CStr::from_ptr(filename).to_str() {
        Ok(name) => name,
        Err(_) => {
            eos!(EINVAL).set_errno();
            return -1;
        }
    };

    match fs_imp::remove(name) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// # Safety
#[cfg(feature = "tfs")]
#[no_mangle]
pub unsafe extern "C" fn sgx_fexport_auto_key(
    filename: *const c_char,
    export_key: *mut Key128bit,
) -> i32 {
    if filename.is_null() || export_key.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let name = match CStr::from_ptr(filename).to_str() {
        Ok(name) => name,
        Err(_) => {
            eos!(EINVAL).set_errno();
            return -1;
        }
    };

    match fs_imp::export_key(name) {
        Ok(key) => {
            *export_key = key;
            0
        }
        Err(_) => -1,
    }
}

/// # Safety
#[cfg(feature = "tfs")]
#[no_mangle]
pub unsafe extern "C" fn sgx_fimport_auto_key(
    filename: *const c_char,
    import_key: *const Key128bit,
    key_policy: u16,
) -> i32 {
    if filename.is_null() || import_key.is_null() {
        eos!(EINVAL).set_errno();
        return -1;
    }

    let name = match CStr::from_ptr(filename).to_str() {
        Ok(name) => name,
        Err(_) => {
            eos!(EINVAL).set_errno();
            return -1;
        }
    };

    let key_policy = if key_policy != 0 {
        match KeyPolicy::from_bits(key_policy) {
            Some(key_policy) => key_policy,
            None => {
                eos!(EINVAL).set_errno();
                return -1;
            }
        }
    } else {
        KeyPolicy::MRSIGNER
    };

    match fs_imp::import_key(name, *import_key, key_policy) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

fn parse_mode(mode: &CStr) -> FsResult<OpenOptions> {
    const MAX_MODE_STRING_LEN: usize = 5;

    let mode = mode.to_str().map_err(|_| eos!(EINVAL))?;
    ensure!(
        !mode.is_empty() && mode.len() <= MAX_MODE_STRING_LEN,
        eos!(EINVAL)
    );

    let mut opts = OpenOptions::new();
    for c in mode.chars() {
        match c {
            'r' => opts.read(true),
            'w' => opts.write(true),
            'a' => opts.append(true),
            '+' => opts.update(true),
            'b' => opts.binary(true),
            _ => bail!(eos!(EINVAL)),
        }
    }
    Ok(opts)
}
