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

use super::*;
use core::ptr;
use core::slice;
use sgx_ffi::c_str::CStr;

pub unsafe fn open(path: &CStr, flags: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_open_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn open64(path: &CStr, oflag: c_int, mode: mode_t) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_open64_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        oflag,
        mode,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn openat(dirfd: c_int, pathname: &CStr, flags: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_openat_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        dirfd,
        pathname.as_ptr(),
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn fstat(fd: c_int, buf: &mut stat) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_fstat_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fd,
        buf as *mut stat,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn fstat64(fd: c_int, buf: &mut stat64) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_fstat64_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fd,
        buf as *mut stat64,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn stat(path: &CStr, buf: &mut stat) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_stat_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        buf as *mut stat,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn stat64(path: &CStr, buf: &mut stat64) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_stat64_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        buf as *mut stat64,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn lstat(path: &CStr, buf: &mut stat) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_lstat_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        buf as *mut stat,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn lstat64(path: &CStr, buf: &mut stat64) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_lstat64_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        buf as *mut stat64,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn lseek(fd: c_int, offset: off_t, whence: c_int) -> OCallResult<u64> {
    let mut result: off_t = 0;
    let mut error: c_int = 0;

    let status = u_lseek_ocall(
        &mut result as *mut off_t,
        &mut error as *mut c_int,
        fd,
        offset,
        whence,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result as u64)
}

pub unsafe fn lseek64(fd: c_int, offset: off64_t, whence: c_int) -> OCallResult<u64> {
    let mut result: off64_t = 0;
    let mut error: c_int = 0;

    let status = u_lseek64_ocall(
        &mut result as *mut off64_t,
        &mut error as *mut c_int,
        fd,
        offset,
        whence,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result as u64)
}

pub unsafe fn ftruncate(fd: c_int, length: off_t) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_ftruncate_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fd,
        length,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn ftruncate64(fd: c_int, length: off64_t) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_ftruncate64_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fd,
        length,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn truncate(path: &CStr, length: off_t) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_truncate_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        length,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn truncate64(path: &CStr, length: off64_t) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_truncate64_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        length,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn fsync(fd: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_fsync_ocall(&mut result as *mut c_int, &mut error as *mut c_int, fd);

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn fdatasync(fd: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_fdatasync_ocall(&mut result as *mut c_int, &mut error as *mut c_int, fd);

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn fchmod(fd: c_int, mode: mode_t) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_fchmod_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fd,
        mode,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn unlink(pathname: &CStr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_unlink_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        pathname.as_ptr(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn link(oldpath: &CStr, newpath: &CStr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_link_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        oldpath.as_ptr(),
        newpath.as_ptr(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn unlinkat(dirfd: c_int, pathname: &CStr, flags: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_unlinkat_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        dirfd,
        pathname.as_ptr(),
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn linkat(
    olddirfd: c_int,
    oldpath: &CStr,
    newdirfd: c_int,
    newpath: &CStr,
    flags: c_int,
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_linkat_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        olddirfd,
        oldpath.as_ptr(),
        newdirfd,
        newpath.as_ptr(),
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn rename(oldpath: &CStr, newpath: &CStr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_rename_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        oldpath.as_ptr(),
        newpath.as_ptr(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn chmod(path: &CStr, mode: mode_t) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_chmod_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        mode,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn readlink(path: &CStr) -> OCallResult<Vec<u8>> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let mut buf: Vec<u8> = Vec::with_capacity(512);

    loop {
        let bufsz = buf.capacity();

        let status = u_readlink_ocall(
            &mut result as *mut ssize_t,
            &mut error as *mut c_int,
            path.as_ptr(),
            buf.as_mut_ptr() as *mut c_char,
            bufsz,
        );

        ensure!(status.is_success(), esgx!(status));
        ensure!(result >= 0, eos!(error));

        // On success, these calls return the number of bytes placed in buf.
        // If the returned value equals bufsz, then truncation may have occurred.
        let buf_read = result as usize;
        ensure!(buf_read <= bufsz, ecust!("Malformed return value."));
        buf.set_len(buf_read);

        if buf_read < buf.capacity() {
            buf.shrink_to_fit();
            return Ok(buf);
        }

        // Trigger the internal buffer resizing logic of `Vec` by requiring
        // more space than the current capacity. The length is guaranteed to be
        // the same as the capacity due to the if statement above.
        buf.reserve(1);
    }
}

pub unsafe fn symlink(path1: &CStr, path2: &CStr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_symlink_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path1.as_ptr(),
        path2.as_ptr(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn realpath(path: &CStr) -> OCallResult<CString> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let bufsz = PATH_MAX as usize;
    let mut resolved_buf = vec![0_u8; bufsz];

    let status = u_realpath_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        resolved_buf.as_mut_ptr() as *mut c_char,
        bufsz,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    shrink_to_fit_cstring(resolved_buf)
}

pub unsafe fn mkdir(pathname: &CStr, mode: mode_t) -> OCallResult<()> {
    let mut error: c_int = 0;
    let mut result: c_int = 0;

    let status = u_mkdir_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        pathname.as_ptr(),
        mode,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn rmdir(pathname: &CStr) -> OCallResult<()> {
    let mut error: c_int = 0;
    let mut result: c_int = 0;

    let status = u_rmdir_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        pathname.as_ptr(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub type DirPtr = *mut DIR;

pub unsafe fn fdopendir(fd: c_int) -> OCallResult<DirPtr> {
    let mut result: DirPtr = ptr::null_mut();
    let mut error: c_int = 0;

    let status = u_fdopendir_ocall(&mut result as *mut DirPtr, &mut error as *mut c_int, fd);

    ensure!(status.is_success(), esgx!(status));
    ensure!(!result.is_null(), eos!(error));
    Ok(result)
}

pub unsafe fn opendir(pathname: &CStr) -> OCallResult<DirPtr> {
    let mut result: DirPtr = ptr::null_mut();
    let mut error: c_int = 0;

    let status = u_opendir_ocall(
        &mut result as *mut DirPtr,
        &mut error as *mut c_int,
        pathname.as_ptr(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(!result.is_null(), eos!(error));
    Ok(result)
}

pub unsafe fn readdir64_r(
    dir: DirPtr,
    entry: &mut dirent64,
    dirresult: &mut *mut dirent64,
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut eods: c_int = 0;

    // if there is any error or end-of-dir, the result is always null.
    *dirresult = ptr::null_mut();
    let status = u_readdir64_r_ocall(
        &mut result as *mut c_int,
        dir,
        entry,
        &mut eods as *mut c_int,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(result));

    // inspect contents of dirent64
    // d_reclen is 8 bytes aligned on 64-bit platform
    const DIRENT64_NAME_OFFSET: usize = 20;
    let dn_slice = slice::from_raw_parts(entry.d_name.as_ptr() as *const u8, entry.d_name.len());
    if let Some(slen) = memchr::memchr(0, dn_slice) {
        ensure!(
            (slen == 0 && entry.d_reclen == 0)
                || align8!(slen + DIRENT64_NAME_OFFSET) == entry.d_reclen as usize,
            ecust!("inconsistant dirent content")
        );
    } else {
        bail!(ecust!("dirent.d_name is not null terminated"));
    }

    if eods == 0 {
        *dirresult = entry;
    }
    Ok(())
}

pub unsafe fn closedir(dir: DirPtr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_closedir_ocall(&mut result as *mut c_int, &mut error as *mut c_int, dir);

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn dirfd(dir: DirPtr) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_dirfd_ocall(&mut result as *mut c_int, &mut error as *mut c_int, dir);

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn fstatat64(
    dirfd: c_int,
    pathname: &CStr,
    buf: &mut stat64,
    flags: c_int,
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_fstatat64_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        dirfd,
        pathname.as_ptr(),
        buf,
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}
