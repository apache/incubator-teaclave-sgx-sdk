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

use std::ptr;
use std::io::Error;
use libc::{self, c_int, c_char, size_t, ssize_t, off_t, off64_t, mode_t, stat, stat64};

#[no_mangle]
pub extern "C" fn u_open_ocall(error: * mut c_int,
                               pathname: * const c_char,
                               flags: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::open(pathname, flags) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_open64_ocall(error: * mut c_int,
                                 path: * const c_char,
                                 oflag: c_int,
                                 mode: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::open64(path, oflag, mode) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fstat_ocall(error: * mut c_int,
                                fd: c_int,
                                buf: * mut stat) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::fstat(fd, buf) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fstat64_ocall(error: * mut c_int,
                                  fd: c_int,
                                  buf: * mut stat64) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::fstat64(fd, buf) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_stat_ocall(error: * mut c_int,
                               path: * const c_char,
                               buf: * mut stat) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::stat(path, buf) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_stat64_ocall(error: * mut c_int,
                                 path: * const c_char,
                                 buf: * mut stat64) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::stat64(path, buf) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_lstat_ocall(error: * mut c_int,
                                path: * const c_char,
                                buf: * mut stat) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::stat(path, buf) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_lstat64_ocall(error: * mut c_int,
                                  path: * const c_char,
                                  buf: * mut stat64) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::stat64(path, buf) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_lseek_ocall(error: * mut c_int,
                                fd: c_int,
                                offset: off_t,
                                whence: c_int) -> off_t {
    let mut errno = 0;
    let ret = unsafe { libc::lseek(fd, offset, whence) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_lseek64_ocall(error: * mut c_int,
                                  fd: c_int,
                                  offset: off64_t,
                                  whence: c_int) -> off64_t {
    let mut errno = 0;
    let ret = unsafe { libc::lseek64(fd, offset, whence) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_ftruncate_ocall(error: * mut c_int,
                                    fd: c_int,
                                    length: off_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::ftruncate(fd, length) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_ftruncate64_ocall(error: * mut c_int,
                                      fd: c_int,
                                      length: off64_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::ftruncate64(fd, length) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_truncate_ocall(error: * mut c_int,
                                   path: * const c_char,
                                   length: off_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::truncate(path, length) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_truncate64_ocall(error: * mut c_int,
                                     path: * const c_char,
                                     length: off64_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::truncate64(path, length) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fsync_ocall(error: * mut c_int, fd: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::fsync(fd) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fdatasync_ocall(error: * mut c_int, fd: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::fdatasync(fd) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fchmod_ocall(error: * mut c_int, fd: c_int, mode: mode_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::fchmod(fd, mode) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_unlink_ocall(error: * mut c_int, pathname: * const c_char) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::unlink(pathname) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_link_ocall(error: * mut c_int,
                               oldpath: * const c_char,
                               newpath: * const c_char) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::link(oldpath, newpath) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_rename_ocall(error: * mut c_int,
                                 oldpath: * const c_char,
                                 newpath: * const c_char) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::rename(oldpath, newpath) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_chmod_ocall(error: * mut c_int,
                                path: * const c_char,
                                mode: mode_t) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::chmod(path, mode) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_readlink_ocall(error: * mut c_int,
                                   path: * const c_char,
                                   buf: * mut c_char,
                                   bufsz: size_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::readlink(path, buf, bufsz) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_symlink_ocall(error: * mut c_int,
                                  path1: * const c_char,
                                  path2: * const c_char) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::symlink(path1, path2) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_realpath_ocall(error: * mut c_int, pathname: * const c_char) -> * mut c_char {
    let mut errno = 0;
    let ret = unsafe { libc::realpath(pathname, ptr::null_mut()) };
    if ret.is_null() {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}