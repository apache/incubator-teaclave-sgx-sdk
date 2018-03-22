// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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
use libc::{self, c_int, c_char, c_void, size_t, ssize_t, off64_t, c_ulong, mode_t, stat64};

#[no_mangle]
pub extern "C" fn u_fs_open64_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_read_ocall(error: * mut c_int,
                                  fd: c_int,
                                  buf: * mut c_void,
                                  count: size_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::read(fd, buf, count) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fs_pread64_ocall(error: * mut c_int,
                                     fd: c_int,
                                     buf: * mut c_void,
                                     count: size_t,
                                     offset: off64_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::pread64(fd, buf, count, offset) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fs_write_ocall(error: * mut c_int,
                                   fd: c_int,
                                   buf: * const c_void,
                                   count: size_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::write(fd, buf, count) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fs_pwrite64_ocall(error: * mut c_int,
                                      fd: c_int,
                                      buf: * const c_void,
                                      count: size_t,
                                      offset: off64_t) -> ssize_t {
    let mut errno = 0;
    let ret = unsafe { libc::pwrite64(fd, buf, count, offset) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fs_close_ocall(error: * mut c_int, fd: c_int) -> c_int {

    let mut errno = 0;
    let ret = unsafe { libc::close(fd) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fs_fcntl_arg0_ocall(error: * mut c_int,
                                        fd: c_int,
                                        cmd: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::fcntl(fd, cmd) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fs_fcntl_arg1_ocall(error: * mut c_int,
                                        fd: c_int,
                                        cmd: c_int,
                                        arg: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::fcntl(fd, cmd, arg) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fs_ioctl_arg0_ocall(error: * mut c_int,
                                        fd: c_int,
                                        request: c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::ioctl(fd, request as c_ulong) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fs_ioctl_arg1_ocall(error: * mut c_int,
                                        fd: c_int,
                                        request: c_int,
                                        arg: * const c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::ioctl(fd, request as c_ulong, arg) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_fs_fstat64_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_fsync_ocall(error: * mut c_int, fd: c_int) -> c_int {

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
pub extern "C" fn u_fs_fdatasync_ocall(error: * mut c_int, fd: c_int) -> c_int {

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
pub extern "C" fn u_fs_ftruncate64_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_lseek64_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_fchmod_ocall(error: * mut c_int, fd: c_int, mode: mode_t) -> c_int {

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
pub extern "C" fn u_fs_unlink_ocall(error: * mut c_int, pathname: * const c_char) -> c_int {

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
pub extern "C" fn u_fs_link_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_rename_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_chmod_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_readlink_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_symlink_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_stat64_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_lstat64_ocall(error: * mut c_int,
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
pub extern "C" fn u_fs_realpath_ocall(error: * mut c_int, pathname: * const c_char) -> * mut c_char {

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

#[no_mangle]
pub extern "C" fn u_fs_free_ocall(p: * mut c_void) {

    unsafe { libc::free(p) }
}