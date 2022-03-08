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

use crate::linux::*;
use core::ptr;
use sgx_ffi::c_str::CStr;
use sgx_oc::linux::ocall;
use sgx_trts::trts::is_within_enclave;

pub use sgx_oc::linux::ocall::environ;

#[no_mangle]
pub unsafe extern "C" fn getuid() -> uid_t {
    if let Ok(uid) = ocall::getuid() {
        uid
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn getgid() -> gid_t {
    if let Ok(gid) = ocall::getgid() {
        gid
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn getenv(name: *const c_char) -> *mut c_char {
    if name.is_null() {
        return ptr::null_mut();
    }

    ocall::getenv_ref(CStr::from_ptr(name))
        .unwrap_or(None)
        .map_or(ptr::null_mut(), |v| v.as_ptr() as *mut c_char)
}

#[no_mangle]
pub unsafe extern "C" fn setenv(
    name: *const c_char,
    value: *const c_char,
    overwrite: c_int,
) -> c_int {
    if name.is_null() || value.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::setenv(CStr::from_ptr(name), CStr::from_ptr(value), overwrite).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn unsetenv(name: *const c_char) -> c_int {
    if name.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::unsetenv(CStr::from_ptr(name)).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn getcwd(buf: *mut c_char, bufsz: size_t) -> *mut c_char {
    if (buf.is_null() && bufsz != 0)
        || (!buf.is_null() && (bufsz == 0 || !is_within_enclave(buf as *const u8, bufsz)))
    {
        set_errno(EINVAL);
        return ptr::null_mut();
    }

    if let Ok(cwd) = ocall::getcwd() {
        if buf.is_null() {
            cwd.into_raw()
        } else if cwd.as_bytes_with_nul().len() <= bufsz {
            ptr::copy_nonoverlapping(cwd.as_ptr(), buf, cwd.as_bytes_with_nul().len());
            buf
        } else {
            set_errno(ERANGE);
            ptr::null_mut()
        }
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn chdir(dir: *const c_char) -> c_int {
    if dir.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::chdir(CStr::from_ptr(dir)).is_ok() {
        0
    } else {
        -1
    }
}
