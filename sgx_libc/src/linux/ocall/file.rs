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

use crate::linux::*;
use core::ptr;
use sgx_ffi::c_str::CStr;
use sgx_oc::linux::ocall;
use sgx_trts::trts::is_within_enclave;

#[no_mangle]
pub unsafe extern "C" fn open(path: *const c_char, flags: c_int) -> c_int {
    if path.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if let Ok(fd) = ocall::open(CStr::from_ptr(path), flags) {
        fd
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn open64(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    if path.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if let Ok(fd) = ocall::open64(CStr::from_ptr(path), oflag, mode) {
        fd
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn openat(dirfd: c_int, pathname: *const c_char, flags: c_int) -> c_int {
    if dirfd <= 0 || pathname.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if let Ok(fd) = ocall::openat(dirfd, CStr::from_ptr(pathname), flags) {
        fd
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn fstat(fd: c_int, buf: *mut stat) -> c_int {
    if buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::fstat(fd, &mut *buf).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn fstat64(fd: c_int, buf: *mut stat64) -> c_int {
    if buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::fstat64(fd, &mut *buf).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn stat(path: *const c_char, buf: *mut stat) -> c_int {
    if path.is_null() || buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::stat(CStr::from_ptr(path), &mut *buf).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn stat64(path: *const c_char, buf: *mut stat64) -> c_int {
    if path.is_null() || buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::stat64(CStr::from_ptr(path), &mut *buf).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn lstat(path: *const c_char, buf: *mut stat) -> c_int {
    if path.is_null() || buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::lstat(CStr::from_ptr(path), &mut *buf).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn lstat64(path: *const c_char, buf: *mut stat64) -> c_int {
    if path.is_null() || buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::lstat64(CStr::from_ptr(path), &mut *buf).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t {
    if let Ok(offset) = ocall::lseek(fd, offset, whence) {
        offset as off_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn lseek64(fd: c_int, offset: off64_t, whence: c_int) -> off64_t {
    if let Ok(offset) = ocall::lseek64(fd, offset, whence) {
        offset as off64_t
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn ftruncate(fd: c_int, length: off_t) -> c_int {
    if ocall::ftruncate(fd, length).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn ftruncate64(fd: c_int, length: off64_t) -> c_int {
    if ocall::ftruncate64(fd, length).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn truncate(path: *const c_char, length: off_t) -> c_int {
    if path.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::truncate(CStr::from_ptr(path), length).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn truncate64(path: *const c_char, length: off64_t) -> c_int {
    if path.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::truncate64(CStr::from_ptr(path), length).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn fsync(fd: c_int) -> c_int {
    if ocall::fsync(fd).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn fdatasync(fd: c_int) -> c_int {
    if ocall::fdatasync(fd).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn fchmod(fd: c_int, mode: mode_t) -> c_int {
    if ocall::fchmod(fd, mode).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn unlink(pathname: *const c_char) -> c_int {
    if pathname.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::unlink(CStr::from_ptr(pathname)).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn link(oldpath: *const c_char, newpath: *const c_char) -> c_int {
    if oldpath.is_null() || newpath.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::link(CStr::from_ptr(oldpath), CStr::from_ptr(newpath)).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn unlinkat(dirfd: c_int, pathname: *const c_char, flags: c_int) -> c_int {
    if dirfd <= 0 || pathname.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::unlinkat(dirfd, CStr::from_ptr(pathname), flags).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn linkat(
    olddirfd: c_int,
    oldpath: *const c_char,
    newdirfd: c_int,
    newpath: *const c_char,
    flags: c_int,
) -> c_int {
    if oldpath.is_null() || newpath.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::linkat(
        olddirfd,
        CStr::from_ptr(oldpath),
        newdirfd,
        CStr::from_ptr(newpath),
        flags,
    )
    .is_ok()
    {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn rename(oldpath: *const c_char, newpath: *const c_char) -> c_int {
    if oldpath.is_null() || newpath.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::rename(CStr::from_ptr(oldpath), CStr::from_ptr(newpath)).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn chmod(path: *const c_char, mode: mode_t) -> c_int {
    if path.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::chmod(CStr::from_ptr(path), mode).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn readlink(path: *const c_char, buf: *mut c_char, bufsz: size_t) -> ssize_t {
    if (path.is_null())
        || buf.is_null()
        || bufsz == 0
        || !is_within_enclave(buf as *const u8, bufsz)
    {
        set_errno(EINVAL);
        return -1;
    }

    if let Ok(path) = ocall::readlink(CStr::from_ptr(path)) {
        let path_len = path.len();
        if path_len <= bufsz {
            ptr::copy_nonoverlapping(path.as_ptr(), buf as *mut u8, path_len);
            path_len as ssize_t
        } else {
            set_errno(ERANGE);
            -1
        }
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn symlink(path1: *const c_char, path2: *const c_char) -> c_int {
    if path1.is_null() || path2.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::symlink(CStr::from_ptr(path1), CStr::from_ptr(path2)).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn realpath(
    path: *const c_char,
    resolved_buf: *mut c_char,
    bufsz: size_t,
) -> *mut c_char {
    if (path.is_null())
        || (resolved_buf.is_null() && bufsz != 0)
        || (!resolved_buf.is_null()
            && (bufsz == 0 || !is_within_enclave(resolved_buf as *const u8, bufsz)))
    {
        set_errno(EINVAL);
        return ptr::null_mut();
    }

    if let Ok(path) = ocall::realpath(CStr::from_ptr(path)) {
        if resolved_buf.is_null() {
            path.into_raw()
        } else if path.as_bytes_with_nul().len() <= bufsz {
            ptr::copy_nonoverlapping(path.as_ptr(), resolved_buf, path.as_bytes_with_nul().len());
            resolved_buf
        } else {
            set_errno(ERANGE);
            ptr::null_mut()
        }
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn mkdir(pathname: *const c_char, mode: mode_t) -> c_int {
    if pathname.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::mkdir(CStr::from_ptr(pathname), mode).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn rmdir(pathname: *const c_char) -> c_int {
    if pathname.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::rmdir(CStr::from_ptr(pathname)).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn fdopendir(fd: c_int) -> *mut DIR {
    if fd <= 0 {
        set_errno(EINVAL);
        return ptr::null_mut();
    }

    if let Ok(dir) = ocall::fdopendir(fd) {
        dir
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn opendir(pathname: *const c_char) -> *mut DIR {
    if pathname.is_null() {
        set_errno(EINVAL);
        return ptr::null_mut();
    }

    if let Ok(dir) = ocall::opendir(CStr::from_ptr(pathname)) {
        dir
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn readdir64_r(
    dirp: *mut DIR,
    entry: *mut dirent64,
    dirresult: *mut *mut dirent64,
) -> c_int {
    if entry.is_null() || dirresult.is_null() {
        set_errno(EINVAL);
        return EINVAL;
    }

    if ocall::readdir64_r(dirp, &mut *entry, &mut *dirresult).is_ok() {
        0
    } else {
        errno()
    }
}

#[no_mangle]
pub unsafe extern "C" fn closedir(dirp: *mut DIR) -> c_int {
    if ocall::closedir(dirp).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn dirfd(dirp: *mut DIR) -> c_int {
    if ocall::dirfd(dirp).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn fstatat64(
    dirfd: c_int,
    pathname: *const c_char,
    buf: *mut stat64,
    flags: c_int,
) -> c_int {
    if pathname.is_null() || buf.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::fstatat64(dirfd, CStr::from_ptr(pathname), &mut *buf, flags).is_ok() {
        0
    } else {
        -1
    }
}
