// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::io::Error;
use libc::{self, c_char, c_int, uid_t, size_t, passwd};

#[no_mangle]
pub extern "C" fn u_getuid_ocall() -> uid_t {
     unsafe { libc::getuid()}
}

#[no_mangle]
pub extern "C" fn u_environ_ocall() -> * const * const c_char {
    extern { static environ: * const * const c_char; }
    unsafe { environ }
}

#[no_mangle]
pub extern "C" fn u_getenv_ocall(name: * const c_char) -> * const c_char {
    unsafe { libc::getenv(name) }
}

#[no_mangle]
pub extern "C" fn u_setenv_ocall(error: * mut c_int,
                                 name: * const c_char,
                                 value: * const c_char,
                                 overwrite: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::setenv(name, value, overwrite) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_unsetenv_ocall(error: * mut c_int, name: * const c_char) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::unsetenv(name) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn  u_getcwd_ocall(error: * mut c_int, buf: *mut c_char, size: size_t) -> *mut c_char {
    let mut errno = 0;
    let ret = unsafe { libc::getcwd(buf, size) };
    if ret.is_null() {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
     if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_chdir_ocall(error: * mut c_int, dir: *const c_char) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::chdir(dir) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
     if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_getpwuid_r_ocall(uid: uid_t,
                                     pwd: *mut passwd,
                                     buf: *mut c_char,
                                     buflen: size_t,
                                     passwd_result: *mut *mut passwd) -> c_int {
    let ret = unsafe { libc::getpwuid_r(uid, pwd, buf, buflen, passwd_result) };
    if ret == 0 {
        let pwd_ret = unsafe { *passwd_result };
        if !pwd_ret.is_null() {
            unsafe {
                let mut temp_pwd = &mut *pwd;
                let p = -1_isize as usize;
                if !temp_pwd.pw_name.is_null() {
                    temp_pwd.pw_name = temp_pwd.pw_name.offset_from(buf) as * mut c_char;
                } else {
                    temp_pwd.pw_name = p as usize as * mut c_char;
                }
                if !temp_pwd.pw_passwd.is_null() {
                    temp_pwd.pw_passwd = temp_pwd.pw_passwd.offset_from(buf) as * mut c_char;
                } else {
                    temp_pwd.pw_passwd = p as usize as * mut c_char;
                }
                if !temp_pwd.pw_gecos.is_null() {
                    temp_pwd.pw_gecos = temp_pwd.pw_gecos.offset_from(buf) as * mut c_char;
                } else {
                    temp_pwd.pw_gecos = p as * mut c_char;
                }
                if !temp_pwd.pw_dir.is_null() {
                    temp_pwd.pw_dir = temp_pwd.pw_dir.offset_from(buf) as * mut c_char;
                } else {
                    temp_pwd.pw_dir = p as * mut c_char;
                }
                if !temp_pwd.pw_shell.is_null() {
                    temp_pwd.pw_shell = temp_pwd.pw_shell.offset_from(buf) as * mut c_char;
                } else {
                    temp_pwd.pw_shell = p as * mut c_char;
                }
            }
        }
    }
    ret
}