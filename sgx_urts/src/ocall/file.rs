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

use crate::ocall::util::*;
use libc::{
    self, c_char, c_int, dirent64, mode_t, off64_t, off_t, size_t, ssize_t, stat, stat64, DIR,
};
use std::io::Error;
use std::ptr;

#[no_mangle]
pub unsafe extern "C" fn u_open_ocall(
    error: *mut c_int,
    path: *const c_char,
    flags: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::open(path, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_open64_ocall(
    error: *mut c_int,
    path: *const c_char,
    oflag: c_int,
    mode: mode_t,
) -> c_int {
    let mut errno = 0;
    let ret = libc::open64(path, oflag, mode as c_int);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_openat_ocall(
    error: *mut c_int,
    dirfd: c_int,
    pathname: *const c_char,
    flags: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::openat(dirfd, pathname, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_fstat_ocall(error: *mut c_int, fd: c_int, buf: *mut stat) -> c_int {
    let mut errno = 0;
    let ret = libc::fstat(fd, buf);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_fstat64_ocall(error: *mut c_int, fd: c_int, buf: *mut stat64) -> c_int {
    let mut errno = 0;
    let ret = libc::fstat64(fd, buf);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_stat_ocall(
    error: *mut c_int,
    path: *const c_char,
    buf: *mut stat,
) -> c_int {
    let mut errno = 0;
    let ret = libc::stat(path, buf);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_stat64_ocall(
    error: *mut c_int,
    path: *const c_char,
    buf: *mut stat64,
) -> c_int {
    let mut errno = 0;
    let ret = libc::stat64(path, buf);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_lstat_ocall(
    error: *mut c_int,
    path: *const c_char,
    buf: *mut stat,
) -> c_int {
    let mut errno = 0;
    let ret = libc::lstat(path, buf);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_lstat64_ocall(
    error: *mut c_int,
    path: *const c_char,
    buf: *mut stat64,
) -> c_int {
    let mut errno = 0;
    let ret = libc::lstat64(path, buf);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_lseek_ocall(
    error: *mut c_int,
    fd: c_int,
    offset: off_t,
    whence: c_int,
) -> off_t {
    let mut errno = 0;
    let ret = libc::lseek(fd, offset, whence);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_lseek64_ocall(
    error: *mut c_int,
    fd: c_int,
    offset: off64_t,
    whence: c_int,
) -> off64_t {
    let mut errno = 0;
    let ret = libc::lseek64(fd, offset, whence);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_ftruncate_ocall(error: *mut c_int, fd: c_int, length: off_t) -> c_int {
    let mut errno = 0;
    let ret = libc::ftruncate(fd, length);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_ftruncate64_ocall(
    error: *mut c_int,
    fd: c_int,
    length: off64_t,
) -> c_int {
    let mut errno = 0;
    let ret = libc::ftruncate64(fd, length);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_truncate_ocall(
    error: *mut c_int,
    path: *const c_char,
    length: off_t,
) -> c_int {
    let mut errno = 0;
    let ret = libc::truncate(path, length);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_truncate64_ocall(
    error: *mut c_int,
    path: *const c_char,
    length: off64_t,
) -> c_int {
    let mut errno = 0;
    let ret = libc::truncate64(path, length);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_fsync_ocall(error: *mut c_int, fd: c_int) -> c_int {
    let mut errno = 0;
    let ret = libc::fsync(fd);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_fdatasync_ocall(error: *mut c_int, fd: c_int) -> c_int {
    let mut errno = 0;
    let ret = libc::fdatasync(fd);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_fchmod_ocall(error: *mut c_int, fd: c_int, mode: mode_t) -> c_int {
    let mut errno = 0;
    let ret = libc::fchmod(fd, mode);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_unlink_ocall(error: *mut c_int, pathname: *const c_char) -> c_int {
    let mut errno = 0;
    let ret = libc::unlink(pathname);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_link_ocall(
    error: *mut c_int,
    oldpath: *const c_char,
    newpath: *const c_char,
) -> c_int {
    let mut errno = 0;
    let ret = libc::link(oldpath, newpath);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_unlinkat_ocall(
    error: *mut c_int,
    dirfd: c_int,
    pathname: *const c_char,
    flags: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::unlinkat(dirfd, pathname, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_linkat_ocall(
    error: *mut c_int,
    olddirfd: c_int,
    oldpath: *const c_char,
    newdirfd: c_int,
    newpath: *const c_char,
    flags: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::linkat(olddirfd, oldpath, newdirfd, newpath, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_rename_ocall(
    error: *mut c_int,
    oldpath: *const c_char,
    newpath: *const c_char,
) -> c_int {
    let mut errno = 0;
    let ret = libc::rename(oldpath, newpath);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_chmod_ocall(
    error: *mut c_int,
    path: *const c_char,
    mode: mode_t,
) -> c_int {
    let mut errno = 0;
    let ret = libc::chmod(path, mode);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_readlink_ocall(
    error: *mut c_int,
    path: *const c_char,
    buf: *mut c_char,
    bufsz: size_t,
) -> ssize_t {
    let mut errno = 0;
    let ret = libc::readlink(path, buf, bufsz);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_symlink_ocall(
    error: *mut c_int,
    path1: *const c_char,
    path2: *const c_char,
) -> c_int {
    let mut errno = 0;
    let ret = libc::symlink(path1, path2);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_realpath_ocall(
    error: *mut c_int,
    path: *const c_char,
    resolved_buf: *mut c_char,
    bufsz: size_t,
) -> c_int {
    if bufsz == 0 || resolved_buf.is_null() {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let mut errno = 0;
    let mut result = 0;
    let ret = libc::realpath(path, resolved_buf);
    if ret.is_null() {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
        result = -1;
    }
    set_error(error, errno);
    result
}

#[no_mangle]
pub unsafe extern "C" fn u_mkdir_ocall(
    error: *mut c_int,
    pathname: *const c_char,
    mode: mode_t,
) -> c_int {
    let mut errno = 0;
    let ret = libc::mkdir(pathname, mode);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_rmdir_ocall(error: *mut c_int, pathname: *const c_char) -> c_int {
    let mut errno = 0;
    let ret = libc::rmdir(pathname);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_fdopendir_ocall(error: *mut c_int, fd: c_int) -> *mut DIR {
    let mut errno = 0;
    let ret = libc::fdopendir(fd);
    if ret.is_null() {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_opendir_ocall(error: *mut c_int, pathname: *const c_char) -> *mut DIR {
    let mut errno = 0;
    let ret = libc::opendir(pathname);
    if ret.is_null() {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_readdir64_r_ocall(
    dirp: *mut DIR,
    entry: *mut dirent64,
    eods: *mut c_int, // end of directory stream
) -> c_int {
    let mut result_ptr = ptr::null_mut();
    let ret = libc::readdir64_r(dirp, entry, &mut result_ptr);
    assert!(!eods.is_null());
    if result_ptr.is_null() {
        *eods = 1;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_closedir_ocall(error: *mut c_int, dirp: *mut DIR) -> c_int {
    let mut errno = 0;
    let ret = libc::closedir(dirp);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_dirfd_ocall(error: *mut c_int, dirp: *mut DIR) -> c_int {
    let mut errno = 0;
    let ret = libc::dirfd(dirp);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_fstatat64_ocall(
    error: *mut c_int,
    dirfd: c_int,
    pathname: *const c_char,
    buf: *mut stat64,
    flags: c_int,
) -> c_int {
    let mut errno = 0;
    let ret = libc::fstatat64(dirfd, pathname, buf, flags);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}
