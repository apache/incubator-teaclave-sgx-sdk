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

use super::sgx_status_t;
use crate::linux::x86_64::*;

extern "C" {
    pub fn u_open_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path: *const c_char,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_open64_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path: *const c_char,
        oflag: c_int,
        mode: mode_t,
    ) -> sgx_status_t;
    pub fn u_openat_ocall(
        result: *mut c_int,
        error: *mut c_int,
        dirfd: c_int,
        pathname: *const c_char,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_fstat_ocall(
        result: *mut c_int,
        error: *mut c_int,
        fd: c_int,
        buf: *mut stat,
    ) -> sgx_status_t;
    pub fn u_fstat64_ocall(
        result: *mut c_int,
        error: *mut c_int,
        fd: c_int,
        buf: *mut stat64,
    ) -> sgx_status_t;
    pub fn u_stat_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path: *const c_char,
        buf: *mut stat,
    ) -> sgx_status_t;
    pub fn u_stat64_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path: *const c_char,
        buf: *mut stat64,
    ) -> sgx_status_t;
    pub fn u_lstat_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path: *const c_char,
        buf: *mut stat,
    ) -> sgx_status_t;
    pub fn u_lstat64_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path: *const c_char,
        buf: *mut stat64,
    ) -> sgx_status_t;
    pub fn u_lseek_ocall(
        result: *mut off_t,
        error: *mut c_int,
        fd: c_int,
        offset: off_t,
        whence: c_int,
    ) -> sgx_status_t;
    pub fn u_lseek64_ocall(
        result: *mut off64_t,
        error: *mut c_int,
        fd: c_int,
        offset: off64_t,
        whence: c_int,
    ) -> sgx_status_t;
    pub fn u_ftruncate_ocall(
        result: *mut c_int,
        error: *mut c_int,
        fd: c_int,
        length: off_t,
    ) -> sgx_status_t;
    pub fn u_ftruncate64_ocall(
        result: *mut c_int,
        error: *mut c_int,
        fd: c_int,
        length: off64_t,
    ) -> sgx_status_t;
    pub fn u_truncate_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path: *const c_char,
        length: off_t,
    ) -> sgx_status_t;
    pub fn u_truncate64_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path: *const c_char,
        length: off64_t,
    ) -> sgx_status_t;
    pub fn u_fsync_ocall(result: *mut c_int, error: *mut c_int, fd: c_int) -> sgx_status_t;
    pub fn u_fdatasync_ocall(result: *mut c_int, error: *mut c_int, fd: c_int) -> sgx_status_t;
    pub fn u_fchmod_ocall(
        result: *mut c_int,
        error: *mut c_int,
        fd: c_int,
        mode: mode_t,
    ) -> sgx_status_t;
    pub fn u_unlink_ocall(
        result: *mut c_int,
        error: *mut c_int,
        pathname: *const c_char,
    ) -> sgx_status_t;
    pub fn u_link_ocall(
        result: *mut c_int,
        error: *mut c_int,
        oldpath: *const c_char,
        newpath: *const c_char,
    ) -> sgx_status_t;
    pub fn u_unlinkat_ocall(
        result: *mut c_int,
        error: *mut c_int,
        dirfd: c_int,
        pathname: *const c_char,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_linkat_ocall(
        result: *mut c_int,
        error: *mut c_int,
        olddirfd: c_int,
        oldpath: *const c_char,
        newdirfd: c_int,
        newpath: *const c_char,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_rename_ocall(
        result: *mut c_int,
        error: *mut c_int,
        oldpath: *const c_char,
        newpath: *const c_char,
    ) -> sgx_status_t;
    pub fn u_chmod_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path: *const c_char,
        mode: mode_t,
    ) -> sgx_status_t;
    pub fn u_readlink_ocall(
        result: *mut ssize_t,
        error: *mut c_int,
        path: *const c_char,
        buf: *mut c_char,
        bufsz: size_t,
    ) -> sgx_status_t;
    pub fn u_symlink_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path1: *const c_char,
        path2: *const c_char,
    ) -> sgx_status_t;
    pub fn u_realpath_ocall(
        result: *mut c_int,
        error: *mut c_int,
        path: *const c_char,
        resolved_buf: *mut c_char,
        bufsz: size_t,
    ) -> sgx_status_t;
    pub fn u_mkdir_ocall(
        result: *mut c_int,
        error: *mut c_int,
        pathname: *const c_char,
        mode: mode_t,
    ) -> sgx_status_t;
    pub fn u_rmdir_ocall(
        result: *mut c_int,
        error: *mut c_int,
        pathname: *const c_char,
    ) -> sgx_status_t;
    pub fn u_fdopendir_ocall(result: *mut *mut DIR, error: *mut c_int, fd: c_int) -> sgx_status_t;
    pub fn u_opendir_ocall(
        result: *mut *mut DIR,
        error: *mut c_int,
        pathname: *const c_char,
    ) -> sgx_status_t;
    pub fn u_readdir64_r_ocall(
        result: *mut c_int,
        dirp: *mut DIR,
        entry: *mut dirent64,
        eods: *mut c_int,
    ) -> sgx_status_t;
    pub fn u_closedir_ocall(result: *mut c_int, error: *mut c_int, dirp: *mut DIR) -> sgx_status_t;
    pub fn u_dirfd_ocall(result: *mut c_int, error: *mut c_int, dirp: *mut DIR) -> sgx_status_t;
    pub fn u_fstatat64_ocall(
        result: *mut c_int,
        error: *mut c_int,
        dirfd: c_int,
        pathname: *const c_char,
        buf: *mut stat64,
        flags: c_int,
    ) -> sgx_status_t;
}
