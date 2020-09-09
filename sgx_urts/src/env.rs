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

use libc::{self, c_char, c_int, passwd, size_t, uid_t};
use std::io::Error;

#[no_mangle]
pub extern "C" fn u_getuid_ocall() -> uid_t {
    unsafe { libc::getuid() }
}

#[no_mangle]
pub extern "C" fn u_environ_ocall() -> *const *const c_char {
    extern "C" {
        static environ: *const *const c_char;
    }
    unsafe { environ }
}

#[no_mangle]
pub extern "C" fn u_getenv_ocall(name: *const c_char) -> *const c_char {
    unsafe { libc::getenv(name) }
}

#[no_mangle]
pub extern "C" fn u_setenv_ocall(
    error: *mut c_int,
    name: *const c_char,
    value: *const c_char,
    overwrite: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::setenv(name, value, overwrite) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_unsetenv_ocall(error: *mut c_int, name: *const c_char) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::unsetenv(name) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_getcwd_ocall(error: *mut c_int, buf: *mut c_char, size: size_t) -> *mut c_char {
    let mut errno = 0;
    let ret = unsafe { libc::getcwd(buf, size) };
    if ret.is_null() {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_chdir_ocall(error: *mut c_int, dir: *const c_char) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::chdir(dir) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_getpwuid_r_ocall(
    uid: uid_t,
    pwd: *mut passwd,
    buf: *mut c_char,
    buflen: size_t,
    passwd_result: *mut *mut passwd,
) -> c_int {
    let ret = unsafe { libc::getpwuid_r(uid, pwd, buf, buflen, passwd_result) };
    if ret == 0 {
        let pwd_ret = unsafe { *passwd_result };
        if !pwd_ret.is_null() {
            unsafe {
                let mut temp_pwd = &mut *pwd;
                let p = -1_isize as usize;
                if !temp_pwd.pw_name.is_null() {
                    temp_pwd.pw_name = usize::checked_sub(temp_pwd.pw_name as _, buf as _)
                        .unwrap_or(p) as *mut c_char;
                } else {
                    temp_pwd.pw_name = p as *mut c_char;
                }
                if !temp_pwd.pw_passwd.is_null() {
                    temp_pwd.pw_passwd = usize::checked_sub(temp_pwd.pw_passwd as _, buf as _)
                        .unwrap_or(p) as *mut c_char;
                } else {
                    temp_pwd.pw_passwd = p as *mut c_char;
                }
                if !temp_pwd.pw_gecos.is_null() {
                    temp_pwd.pw_gecos = usize::checked_sub(temp_pwd.pw_gecos as _, buf as _)
                        .unwrap_or(p) as *mut c_char;
                } else {
                    temp_pwd.pw_gecos = p as *mut c_char;
                }
                if !temp_pwd.pw_dir.is_null() {
                    temp_pwd.pw_dir = usize::checked_sub(temp_pwd.pw_dir as _, buf as _)
                        .unwrap_or(p) as *mut c_char;
                } else {
                    temp_pwd.pw_dir = p as *mut c_char;
                }
                if !temp_pwd.pw_shell.is_null() {
                    temp_pwd.pw_shell = usize::checked_sub(temp_pwd.pw_shell as _, buf as _)
                        .unwrap_or(p) as *mut c_char;
                } else {
                    temp_pwd.pw_shell = p as *mut c_char;
                }
            }
        }
    }
    ret
}
