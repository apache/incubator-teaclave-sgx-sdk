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
use crate::c_str::CStr;
use crate::memchr;
use alloc::vec::Vec;
use core::cmp;
use core::mem;
use core::ptr;
use sgx_types::*;

extern "C" {
    // memory
    pub fn u_malloc_ocall(
        result: *mut *mut c_void,
        error: *mut c_int,
        size: size_t,
    ) -> sgx_status_t;
    pub fn u_free_ocall(p: *mut c_void) -> sgx_status_t;
    pub fn u_mmap_ocall(
        result: *mut *mut c_void,
        error: *mut c_int,
        start: *mut c_void,
        length: size_t,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: off_t,
    ) -> sgx_status_t;
    pub fn u_munmap_ocall(
        result: *mut c_int,
        error: *mut c_int,
        start: *mut c_void,
        length: size_t,
    ) -> sgx_status_t;
    pub fn u_msync_ocall(
        result: *mut c_int,
        error: *mut c_int,
        addr: *mut c_void,
        length: size_t,
        flags: c_int,
    ) -> sgx_status_t;

    pub fn u_mprotect_ocall(
        result: *mut c_int,
        error: *mut c_int,
        addr: *mut c_void,
        length: size_t,
        prot: c_int,
    ) -> sgx_status_t;
    // env
    pub fn u_getuid_ocall(result: *mut uid_t) -> sgx_status_t;
    pub fn u_getenv_ocall(
        result: *mut c_int,
        error: *mut c_int,
        name: *const c_char,
        buf: *mut c_char,
        size: size_t,
        isset: *mut c_int,
    ) -> sgx_status_t;
    pub fn u_getcwd_ocall(
        result: *mut c_int,
        error: *mut c_int,
        buf: *mut c_char,
        size: size_t,
    ) -> sgx_status_t;
    pub fn u_chdir_ocall(result: *mut c_int, error: *mut c_int, dir: *const c_char)
        -> sgx_status_t;
    // file
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
        mode: c_int,
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
        pathname: *const c_char,
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
    // fd
    pub fn u_read_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        fd: c_int,
        buf: *mut c_void,
        count: size_t,
    ) -> sgx_status_t;
    pub fn u_pread64_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        fd: c_int,
        buf: *mut c_void,
        count: size_t,
        offset: off64_t,
    ) -> sgx_status_t;
    pub fn u_write_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        fd: c_int,
        buf: *const c_void,
        count: size_t,
    ) -> sgx_status_t;
    pub fn u_pwrite64_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        fd: c_int,
        buf: *const c_void,
        count: size_t,
        offset: off64_t,
    ) -> sgx_status_t;
    pub fn u_fcntl_arg0_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fd: c_int,
        cmd: c_int,
    ) -> sgx_status_t;
    pub fn u_fcntl_arg1_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fd: c_int,
        cmd: c_int,
        arg: c_int,
    ) -> sgx_status_t;
    pub fn u_ioctl_arg0_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fd: c_int,
        request: c_int,
    ) -> sgx_status_t;
    pub fn u_ioctl_arg1_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fd: c_int,
        request: c_int,
        arg: *mut c_int,
    ) -> sgx_status_t;
    pub fn u_close_ocall(result: *mut c_int, errno: *mut c_int, fd: c_int) -> sgx_status_t;
    // time
    pub fn u_clock_gettime_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        clk_id: clockid_t,
        tp: *mut timespec,
    ) -> sgx_status_t;
    // socket
    pub fn u_socket_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        domain: c_int,
        ty: c_int,
        protocol: c_int,
    ) -> sgx_status_t;
    pub fn u_socketpair_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        domain: c_int,
        ty: c_int,
        protocol: c_int,
        sv: *mut c_int,
    ) -> sgx_status_t;
    pub fn u_bind_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        address: *const sockaddr,
        addrlen: socklen_t,
    ) -> sgx_status_t;
    pub fn u_listen_ocall(
        result: *mut c_int,
        error: *mut c_int,
        sockfd: c_int,
        backlog: c_int,
    ) -> sgx_status_t;
    pub fn u_accept4_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        addr: *mut sockaddr,
        addrlen_in: socklen_t,
        addrlen_out: *mut socklen_t,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_connect_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        address: *const sockaddr,
        addrlen: socklen_t,
    ) -> sgx_status_t;
    pub fn u_send_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        sockfd: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_sendto_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        sockfd: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
        addr: *const sockaddr,
        addrlen: socklen_t,
    ) -> sgx_status_t;
    pub fn u_sendmsg_ocall(
        result: *mut ssize_t,
        error: *mut c_int,
        sockfd: c_int,
        msg: *const msghdr,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_recv_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        sockfd: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_recvfrom_ocall(
        result: *mut ssize_t,
        errno: *mut c_int,
        sockfd: c_int,
        buf: *mut c_void,
        len: size_t,
        flags: c_int,
        addr: *mut sockaddr,
        addrlen_in: socklen_t,
        addrlen_out: *mut socklen_t,
    ) -> sgx_status_t;
    pub fn u_recvmsg_ocall(
        result: *mut ssize_t,
        error: *mut c_int,
        sockfd: c_int,
        msg: *mut msghdr,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_setsockopt_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        level: c_int,
        optname: c_int,
        optval: *const c_void,
        optlen: socklen_t,
    ) -> sgx_status_t;
    pub fn u_getsockopt_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        level: c_int,
        optname: c_int,
        optval: *mut c_void,
        optlen_in: socklen_t,
        optlen_out: *mut socklen_t,
    ) -> sgx_status_t;
    pub fn u_getpeername_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        address: *mut sockaddr,
        addrlen_in: socklen_t,
        addrlen_out: *mut socklen_t,
    ) -> sgx_status_t;
    pub fn u_getsockname_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        address: *mut sockaddr,
        addrlen_in: socklen_t,
        addrlen_out: *mut socklen_t,
    ) -> sgx_status_t;
    pub fn u_shutdown_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        sockfd: c_int,
        how: c_int,
    ) -> sgx_status_t;
    // net
    pub fn u_getaddrinfo_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        node: *const c_char,
        service: *const c_char,
        hints: *const addrinfo,
        entry_size: size_t,
        buf: *mut u8,
        bufsz: size_t,
        out_entry_count: *mut size_t,
    ) -> sgx_status_t;
    // async io
    pub fn u_poll_ocall(
        result: *mut c_int,
        errno: *mut c_int,
        fds: *mut pollfd,
        nfds: nfds_t,
        timeout: c_int,
    ) -> sgx_status_t;
    pub fn u_epoll_create1_ocall(
        result: *mut c_int,
        error: *mut c_int,
        flags: c_int,
    ) -> sgx_status_t;
    pub fn u_epoll_ctl_ocall(
        result: *mut c_int,
        error: *mut c_int,
        epfd: c_int,
        op: c_int,
        fd: c_int,
        event: *mut epoll_event,
    ) -> sgx_status_t;
    pub fn u_epoll_wait_ocall(
        result: *mut c_int,
        error: *mut c_int,
        epfd: c_int,
        events: *mut epoll_event,
        maxevents: c_int,
        timeout: c_int,
    ) -> sgx_status_t;
    // sys
    pub fn u_sysconf_ocall(result: *mut c_long, error: *mut c_int, name: c_int) -> sgx_status_t;
    pub fn u_prctl_ocall(
        result: *mut c_int,
        error: *mut c_int,
        option: c_int,
        arg2: c_ulong,
        arg3: c_ulong,
        arg4: c_ulong,
        arg5: c_ulong,
    ) -> sgx_status_t;
    pub fn u_sched_setaffinity_ocall(
        result: *mut c_int,
        error: *mut c_int,
        pid: pid_t,
        cpusetsize: size_t,
        mask: *const cpu_set_t,
    ) -> sgx_status_t;
    pub fn u_sched_getaffinity_ocall(
        result: *mut c_int,
        error: *mut c_int,
        pid: pid_t,
        cpusetsize: size_t,
        mask: *mut cpu_set_t,
    ) -> sgx_status_t;
    // pipe
    pub fn u_pipe2_ocall(
        result: *mut c_int,
        error: *mut c_int,
        fds: *mut c_int,
        flags: c_int,
    ) -> sgx_status_t;
    //thread
    pub fn u_sched_yield_ocall(result: *mut c_int, error: *mut c_int) -> sgx_status_t;
    pub fn u_nanosleep_ocall(
        result: *mut c_int,
        error: *mut c_int,
        rqtp: *const timespec,
        rmtp: *mut timespec,
    ) -> sgx_status_t;
    //signal
    pub fn u_sigaction_ocall(
        result: *mut c_int,
        error: *mut c_int,
        signum: c_int,
        act: *const sigaction,
        oldact: *mut sigaction,
        enclave_id: uint64_t,
    ) -> sgx_status_t;
    pub fn u_sigprocmask_ocall(
        result: *mut c_int,
        error: *mut c_int,
        signum: c_int,
        set: *const sigset_t,
        oldset: *mut sigset_t,
    ) -> sgx_status_t;
    pub fn u_raise_ocall(result: *mut c_int, signum: c_int) -> sgx_status_t;
    //process
    pub fn u_getpid_ocall(result: *mut pid_t) -> sgx_status_t;
}

pub unsafe fn shrink_to_fit_os_string<T: Into<Vec<u8>>>(buf: T) -> OCallResult<Vec<u8>> {
    let mut buf = buf.into();
    if let Some(i) = memchr::memchr(0, &buf) {
        buf.set_len(i);
        buf.shrink_to_fit();
        return Ok(buf);
    };
    return Err(ecust!("Malformed Linux OsString Vec: not null terminated"));
}

pub unsafe fn getuid() -> OCallResult<uid_t> {
    let mut result: uid_t = 0;

    let status = u_getuid_ocall(&mut result as *mut uid_t);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    Ok(result)
}

const MAX_ENV_VALUE_SIZE: usize = 1024;
pub unsafe fn getenv(name: &CStr) -> OCallResult<Option<Vec<u8>>> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let mut isset: c_int = 0;

    let bufsz = MAX_ENV_VALUE_SIZE;
    let mut buf = vec![0; bufsz];

    // a) isset is used to indicate the NULL return value of conventional libc::getenv function
    // b) if isset, a '\0' terminated string is expected in the buf
    let status = u_getenv_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        name.as_ptr(),
        buf.as_mut_ptr() as *mut c_char,
        bufsz,
        &mut isset as *mut c_int,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));

    if isset == 1 {
        let v = shrink_to_fit_os_string(buf)?;
        Ok(Some(v))
    } else {
        Ok(None)
    }
}

pub unsafe fn getcwd() -> OCallResult<Vec<u8>> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let bufsz = PATH_MAX as usize;
    let mut buf = vec![0; bufsz];

    let status = u_getcwd_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        buf.as_mut_ptr() as *mut c_char,
        bufsz,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));

    shrink_to_fit_os_string(buf)
}

pub unsafe fn chdir(dir: &CStr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_chdir_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        dir.as_ptr(),
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn open(path: &CStr, flags: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_open_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        flags,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn open64(path: &CStr, oflag: c_int, mode: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_open64_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        oflag,
        mode,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn fsync(fd: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_fsync_ocall(&mut result as *mut c_int, &mut error as *mut c_int, fd);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn fdatasync(fd: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_fdatasync_ocall(&mut result as *mut c_int, &mut error as *mut c_int, fd);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn readlink(path: &CStr) -> OCallResult<Vec<u8>> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let mut buf: Vec<u8> = Vec::with_capacity(256);

    loop {
        let bufsz = buf.capacity();

        let status = u_readlink_ocall(
            &mut result as *mut ssize_t,
            &mut error as *mut c_int,
            path.as_ptr(),
            buf.as_mut_ptr() as *mut c_char,
            bufsz,
        );

        ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn realpath(path: &CStr) -> OCallResult<Vec<u8>> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let bufsz = PATH_MAX as usize;
    let mut resolved_buf = vec![0u8; bufsz];

    let status = u_realpath_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        path.as_ptr(),
        resolved_buf.as_mut_ptr() as *mut c_char,
        bufsz,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));
    shrink_to_fit_os_string(resolved_buf)
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

// todo: change DIR to handle
pub unsafe fn opendir(pathname: &CStr) -> OCallResult<*mut DIR> {
    let mut result: *mut DIR = ptr::null_mut();
    let mut error: c_int = 0;

    let status = u_opendir_ocall(
        &mut result as *mut *mut DIR,
        &mut error as *mut c_int,
        pathname.as_ptr(),
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(!result.is_null(), eos!(error));
    Ok(result)
}

// todo: change the interface, dirresult
// result: success or error
// hint: end-of-dir or continue
pub unsafe fn readdir64_r(
    dirp: *mut DIR,
    entry: &mut dirent64,
    dirresult: *mut *mut dirent64,
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut eods: c_int = 0;

    // if there is any error or or end-of-dir, the result is always null.
    *dirresult = ptr::null_mut();

    let status = u_readdir64_r_ocall(
        &mut result as *mut c_int,
        dirp,
        entry,
        &mut eods as *mut c_int,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(result));

    // inspect contents of dirent64
    // d_reclen is 8 bytes aligned on 64-bit platform
    let dn_slice = as_u8_slice!(entry.d_name.as_ptr(), entry.d_name.len());
    if let Some(slen) = memchr::memchr(0, dn_slice) {
        ensure!(
            (slen == 0 && entry.d_reclen == 0)
                || align8!(slen + DIRENT64_NAME_OFFSET) == entry.d_reclen as usize,
            ecust!("readdir64_r: inconsistant dirent content")
        );
    } else {
        return Err(ecust!("readdir64_r: d_name is not null terminated"));
    }

    if eods == 0 {
        *dirresult = entry;
    }

    Ok(())
}

pub unsafe fn closedir(dirp: *mut DIR) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_closedir_ocall(&mut result as *mut c_int, &mut error as *mut c_int, dirp);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

// todo: change the interface
pub unsafe fn dirfd(dirp: *mut DIR) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_dirfd_ocall(&mut result as *mut c_int, &mut error as *mut c_int, dirp);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
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

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

fn check_trusted_enclave_buffer(buf: *const u8, count: size_t) -> OCallResult<()> {
    ensure!(!buf.is_null(), ecust!("Invalid null buffer."));
    ensure!(count > 0, ecust!("Invalid buffer checking length."));
    ensure!(
        unsafe { sgx_is_within_enclave(buf as *const c_void, count) } == 1,
        ecust!("Buffer is not strictly inside enclave")
    );
    Ok(())
}

struct HostBuffer {
    data: *mut u8,
    len: usize,
}

unsafe fn host_malloc(size: usize) -> OCallResult<*mut c_void> {
    let mut result: *mut c_void = ptr::null_mut();
    let mut error: c_int = 0;
    let status = u_malloc_ocall(
        &mut result as *mut *mut c_void,
        &mut error as *mut c_int,
        size,
    );
    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(!result.is_null(), ecust!("Out of memory"));
    ensure!(
        sgx_is_outside_enclave(result, size) == 1,
        ecust!("Malformed malloc address")
    );

    Ok(result as _)
}

impl HostBuffer {
    fn zeroed(size: usize) -> OCallResult<Self> {
        ensure!(
            size != 0,
            ecust!("Trying to allocate zero byte host memory.")
        );
        let host_ptr = unsafe { host_malloc(size)? };
        unsafe { host_ptr.write_bytes(0_u8, size) };

        Ok(HostBuffer {
            data: host_ptr as _,
            len: size,
        })
    }

    fn from_enclave(buf: &[u8]) -> OCallResult<Self> {
        let bufsz = buf.len();
        ensure!(
            bufsz != 0,
            ecust!("Trying to allocate zero byte host memory.")
        );
        check_trusted_enclave_buffer(buf.as_ptr(), bufsz)?;
        let host_ptr = unsafe { host_malloc(bufsz)? };
        unsafe { ptr::copy_nonoverlapping(buf.as_ptr() as *const u8, host_ptr as *mut u8, bufsz) };

        Ok(HostBuffer {
            data: host_ptr as _,
            len: bufsz,
        })
    }

    fn as_ptr(&self) -> *const c_void {
        self.data as _
    }

    fn as_mut_ptr(&mut self) -> *mut c_void {
        self.data as _
    }

    fn len(&self) -> usize {
        self.len
    }

    fn to_enclave_slice(&self, ecl_slice: &mut [u8]) -> OCallResult<usize> {
        if ecl_slice.len() == 0 {
            return Ok(0);
        }
        ensure!(ecl_slice.len() <= self.len);
        check_trusted_enclave_buffer(ecl_slice.as_ptr(), ecl_slice.len())?;
        unsafe {
            ptr::copy_nonoverlapping(
                self.data as *const u8,
                ecl_slice.as_mut_ptr(),
                ecl_slice.len(),
            );
        }
        Ok(ecl_slice.len())
    }
}

impl Drop for HostBuffer {
    fn drop(&mut self) {
        let _ = unsafe { u_free_ocall(self.data as _) };
    }
}

pub unsafe fn read(fd: c_int, buf: &mut [u8]) -> OCallResult<usize> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let bufsz = buf.len();
    check_trusted_enclave_buffer(buf.as_ptr(), bufsz)?;

    let mut host_buf = HostBuffer::zeroed(bufsz)?;

    let status = u_read_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_mut_ptr(),
        host_buf.len(),
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nread = result as usize;
    ensure!(nread <= bufsz, ecust!("Malformed return size"));

    host_buf.to_enclave_slice(&mut buf[..nread])
}

pub unsafe fn pread64(fd: c_int, buf: &mut [u8], offset: off64_t) -> OCallResult<usize> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let bufsz = buf.len();

    check_trusted_enclave_buffer(buf.as_ptr(), bufsz)?;
    let mut host_buf = HostBuffer::zeroed(bufsz)?;

    let status = u_pread64_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_mut_ptr(),
        host_buf.len(),
        offset,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nread = result as usize;
    ensure!(nread <= bufsz, ecust!("Malformed return size"));

    host_buf.to_enclave_slice(&mut buf[..nread])
}

// Caller is responsible to ensure the iov parameter and its content (slices) are inside enclave memory.
pub unsafe fn readv(fd: c_int, mut iov: Vec<&mut [u8]>) -> OCallResult<size_t> {
    let mut total_size: usize = 0;
    for s in iov.iter() {
        check_trusted_enclave_buffer(s.as_ptr(), s.len())?;
        total_size = total_size
            .checked_add(s.len())
            .ok_or_else(|| OCallError::from_custom_error("Overflow"))?;
    }

    let bufsz = total_size;
    if total_size == 0 {
        return Ok(0);
    }

    let mut host_buf = HostBuffer::zeroed(bufsz)?;
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let status = u_read_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_mut_ptr(),
        host_buf.len(),
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    let u_read = result as usize;
    ensure!(u_read <= bufsz, ecust!("Malformed result"));

    if u_read == 0 {
        return Ok(0);
    }

    // Buffers are processed in array order.  This means that readv() com‐
    // pletely fills iov[0] before proceeding to iov[1], and so on.  (If
    // there is insufficient data, then not all buffers pointed to by iov
    // may be filled.)  Similarly, writev() writes out the entire contents
    // of iov[0] before proceeding to iov[1], and so on.
    let mut bytes_left: usize = u_read;
    let mut copied: usize = 0;
    let mut i = 0;

    let mut v = vec![0; u_read];
    host_buf.to_enclave_slice(v.as_mut_slice())?;
    while i < iov.len() && bytes_left > 0 {
        let n = cmp::min(iov[i].len(), bytes_left);
        iov[i][..n].copy_from_slice(&v[copied..copied + n]);
        bytes_left -= n;
        copied += n;
        i += 1;
    }

    Ok(u_read)
}

pub unsafe fn write(fd: c_int, buf: &[u8]) -> OCallResult<usize> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let bufsz = buf.len();
    if bufsz == 0 {
        return Ok(0);
    }

    let host_buf = HostBuffer::from_enclave(buf)?;

    let status = u_write_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_ptr(),
        host_buf.len(),
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nwritten = result as usize;
    ensure!(nwritten <= bufsz, ecust!("Malformed return size"));
    Ok(nwritten)
}

pub unsafe fn pwrite64(fd: c_int, buf: &[u8], offset: off64_t) -> OCallResult<usize> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let bufsz = buf.len();
    if bufsz == 0 {
        return Ok(0);
    }

    let host_buf = HostBuffer::from_enclave(buf)?;

    let status = u_pwrite64_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_ptr(),
        host_buf.len(),
        offset,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nwritten = result as usize;
    ensure!(nwritten <= bufsz, ecust!("Malformed return size"));

    Ok(nwritten)
}

pub unsafe fn writev(fd: c_int, iov: Vec<&[u8]>) -> OCallResult<size_t> {
    let mut total_size: usize = 0;
    for s in iov.iter() {
        check_trusted_enclave_buffer(s.as_ptr(), s.len())?;
        total_size = total_size
            .checked_add(s.len())
            .ok_or_else(|| OCallError::from_custom_error("Overflow"))?;
    }

    if total_size == 0 {
        return Ok(0);
    }

    let bufsz = total_size;
    let mut buffer = vec![0; total_size];

    let mut copied = 0;
    for io_slice in iov.iter() {
        buffer[copied..copied + io_slice.len()].copy_from_slice(io_slice);
        copied += io_slice.len();
    }

    let host_buf = HostBuffer::from_enclave(&buffer)?;
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let status = u_write_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_ptr(),
        host_buf.len(),
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nwritten = result as size_t;
    ensure!(nwritten <= bufsz, ecust!("Malformed result"));

    Ok(nwritten)
}

pub unsafe fn fcntl_arg0(fd: c_int, cmd: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_fcntl_arg0_ocall(&mut result as *mut c_int, &mut error as *mut c_int, fd, cmd);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn fcntl_arg1(fd: c_int, cmd: c_int, arg: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_fcntl_arg1_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fd,
        cmd,
        arg,
    );
    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn ioctl_arg0(fd: c_int, request: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_ioctl_arg0_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fd,
        request,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn ioctl_arg1(fd: c_int, request: c_int, arg: &mut c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_ioctl_arg1_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fd,
        request,
        arg,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn close(fd: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_close_ocall(&mut result as *mut c_int, &mut error as *mut c_int, fd);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));

    Ok(())
}

pub unsafe fn clock_gettime(clk_id: clockid_t, tp: &mut timespec) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_clock_gettime_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        clk_id,
        tp,
    );
    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));

    Ok(())
}

pub unsafe fn socket(domain: c_int, ty: c_int, protocol: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_socket_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        domain,
        ty,
        protocol,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    Ok(result)
}

pub unsafe fn socketpair(
    domain: c_int,
    ty: c_int,
    protocol: c_int,
    sv: &mut [c_int; 2],
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let mut out_sv = [0 as c_int; 2];

    let status = u_socketpair_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        domain,
        ty,
        protocol,
        out_sv.as_mut_ptr() as *mut c_int,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));

    sv.copy_from_slice(&out_sv);
    Ok(())
}

pub unsafe fn bind(sockfd: c_int, sock_addr: &SockAddr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let address = sock_addr.as_bytes();

    let status = u_bind_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        address.as_ptr() as *const sockaddr,
        address.len() as socklen_t,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn listen(sockfd: c_int, backlog: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_listen_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        backlog,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn accept4(sockfd: c_int, flags: c_int) -> OCallResult<(c_int, SockAddr)> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let mut storage: sockaddr_storage = mem::zeroed();
    let len_in = mem::size_of_val(&storage) as socklen_t;

    let mut len_out: socklen_t = 0 as socklen_t;
    let status = u_accept4_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        &mut storage as *mut _ as *mut _,
        len_in, // This additional arg is just for EDL
        &mut len_out as *mut socklen_t,
        flags,
    );
    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));
    ensure!(
        len_out <= len_in,
        ecust!("Caller should alloc enough addr buffer.")
    );

    let addr = SockAddr::try_from_storage(storage, len_out)?;
    Ok((result, addr))
}

pub unsafe fn connect(sockfd: c_int, address: &SockAddr) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let address = address.as_bytes();

    let status = u_connect_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        address.as_ptr() as *const sockaddr,
        address.len() as socklen_t,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn send(sockfd: c_int, buf: &[u8], flags: c_int) -> OCallResult<usize> {
    let bufsz = buf.len();
    // size zero

    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let host_buf = HostBuffer::from_enclave(buf)?;

    let status = u_send_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        sockfd,
        host_buf.as_ptr(),
        host_buf.len(),
        flags,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nsent = result as usize;
    ensure!(nsent <= bufsz, ecust!("Malformed return size"));

    Ok(nsent)
}

pub unsafe fn sendto(
    sockfd: c_int,
    buf: &[u8],
    flags: c_int,
    sock_addr: &SockAddr,
) -> OCallResult<usize> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let bufsz = buf.len();
    let host_buf = HostBuffer::from_enclave(buf)?;

    let addr = sock_addr.as_bytes();

    //todo: add check for addrlen
    let status = u_sendto_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        sockfd,
        host_buf.as_ptr(),
        host_buf.len(),
        flags,
        addr.as_ptr() as *const sockaddr,
        addr.len() as socklen_t,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nsent = result as usize;
    ensure!(nsent <= bufsz, ecust!("Malformed return size"));

    Ok(nsent)
}

pub unsafe fn recv(sockfd: c_int, buf: &mut [u8], flags: c_int) -> OCallResult<usize> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let buf_ptr = buf.as_ptr();
    let bufsz = buf.len();
    check_trusted_enclave_buffer(buf_ptr, bufsz)?;
    let mut host_buf = HostBuffer::zeroed(bufsz)?;

    let status = u_recv_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        sockfd,
        host_buf.as_mut_ptr(),
        host_buf.len(),
        flags,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nrecv = result as usize;
    ensure!(nrecv <= bufsz, ecust!("Malformed return size"));
    host_buf.to_enclave_slice(&mut buf[..nrecv])
}

pub unsafe fn recvfrom(
    sockfd: c_int,
    buf: &mut [u8],
    flags: c_int,
) -> OCallResult<(size_t, SockAddr)> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let mut storage: sockaddr_storage = mem::zeroed();
    let len_in = mem::size_of_val(&storage) as socklen_t;
    let mut len_out: socklen_t = 0 as socklen_t;

    let bufsz = buf.len();
    let mut host_buf = HostBuffer::zeroed(bufsz)?;

    // todo: add check for len_in
    let status = u_recvfrom_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        sockfd,
        host_buf.as_mut_ptr(),
        host_buf.len(),
        flags,
        &mut storage as *mut _ as *mut _,
        len_in, // This additional arg is just for EDL
        &mut len_out as *mut socklen_t,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));
    ensure!(
        len_out <= len_in,
        ecust!("Caller should alloc enough addr buffer.")
    );

    let nrecv = result as usize;
    ensure!(nrecv <= bufsz, ecust!("Malformed return value"));

    host_buf.to_enclave_slice(&mut buf[..nrecv])?;
    let addr = SockAddr::try_from_storage(storage, len_out)?;
    Ok((nrecv, addr))
}

pub unsafe fn setsockopt(
    sockfd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: socklen_t,
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_setsockopt_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        level,
        optname,
        optval,
        optlen,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn getsockopt(
    sockfd: c_int,
    level: c_int,
    optname: c_int,
    optval: *mut c_void,
    optlen: *mut socklen_t,
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let len_in: socklen_t = if !optlen.is_null() { *optlen } else { 0 };
    let mut len_out: socklen_t = 0 as socklen_t;

    let status = u_getsockopt_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        level,
        optname,
        optval,
        len_in,
        &mut len_out as *mut socklen_t,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    ensure!(
        len_out <= len_in,
        ecust!("Caller should alloc enough opt buffer.")
    );
    if !optlen.is_null() {
        *optlen = len_out;
    }
    Ok(())
}

pub unsafe fn getpeername(sockfd: c_int) -> OCallResult<SockAddr> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let mut storage: sockaddr_storage = mem::zeroed();
    let len_in = mem::size_of_val(&storage) as socklen_t;
    let mut len_out: socklen_t = 0 as socklen_t;

    let status = u_getpeername_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        &mut storage as *mut _ as *mut _,
        len_in,
        &mut len_out as *mut socklen_t,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    ensure!(
        len_out <= len_in,
        ecust!("Caller should alloc enough addr buffer.")
    );

    let addr = SockAddr::try_from_storage(storage, len_out)?;
    Ok(addr)
}

pub unsafe fn getsockname(sockfd: c_int) -> OCallResult<SockAddr> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let mut storage: sockaddr_storage = mem::zeroed();
    let len_in = mem::size_of_val(&storage) as socklen_t;
    let mut len_out: socklen_t = 0 as socklen_t;

    let status = u_getsockname_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        &mut storage as *mut _ as *mut _,
        len_in,
        &mut len_out as *mut socklen_t,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    ensure!(
        len_out <= len_in,
        ecust!("Caller should alloc enough addr buffer.")
    );

    let addr = SockAddr::try_from_storage(storage, len_out)?;
    Ok(addr)
}

pub unsafe fn shutdown(sockfd: c_int, how: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_shutdown_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        sockfd,
        how,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

// exchange only struct sockaddr *ai_addr; information across the boundary
// information filtering by hint is also not supported
pub unsafe fn getaddrinfo(
    node: Option<&CStr>,
    service: Option<&CStr>,
    hints: Option<AddrInfoHints>,
) -> OCallResult<Vec<SockAddr>> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let entry_size = cmp::max(
        mem::size_of::<sockaddr_in6>(),
        mem::size_of::<sockaddr_in>(),
    );

    let entry_count = 32;
    let buf_sz = entry_size * entry_count;

    let mut buffer = vec![0u8; buf_sz];
    let mut out_count = 0usize;

    let node_ptr = match node {
        Some(n) => n.as_ptr(),
        None => ptr::null(),
    };

    let service_ptr = match service {
        Some(s) => s.as_ptr(),
        None => ptr::null(),
    };

    let hints = hints.map(|h| h.to_addrinfo());
    let hints_ptr = match &hints {
        Some(h) => h,
        None => ptr::null(),
    };

    let status = u_getaddrinfo_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        node_ptr,
        service_ptr,
        hints_ptr,
        entry_size,
        buffer.as_mut_ptr() as *mut u8,
        buf_sz,
        &mut out_count as *mut size_t,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    match result {
        0 => {}
        EAI_SYSTEM => return Err(eos!(error)),
        1 => return Err(ecust!("Untrusted side error")),
        e => return Err(egai!(e)),
    };

    ensure!(result == 0, ecust!("Unkonwn error"));
    ensure!(out_count <= entry_count, ecust!("Malformed out_count"));

    let mut addr_vec = Vec::<SockAddr>::with_capacity(out_count);
    let mut cur_ptr = buffer.as_ptr() as *const u8;
    for _ in 0..out_count {
        let family: *const sa_family_t = mem::transmute(cur_ptr);
        match *family as i32 {
            AF_INET => {
                let sockaddr: *const sockaddr_in = mem::transmute(cur_ptr);
                addr_vec.push(SockAddr::IN4(*sockaddr))
            }
            AF_INET6 => {
                let sockaddr: *const sockaddr_in6 = mem::transmute(cur_ptr);
                addr_vec.push(SockAddr::IN6(*sockaddr))
            }
            _ => bail!(ecust!("Unsupported family info")),
        }
        cur_ptr = cur_ptr.add(entry_size);
    }
    Ok(addr_vec)
}

pub unsafe fn poll(fds: &mut [pollfd], timeout: c_int) -> OCallResult<PolledOk> {
    let nfds = fds.len() as u64;

    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_poll_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fds.as_mut_ptr(),
        nfds,
        timeout,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    match result {
        1..=i32::MAX => {
            if result as u64 > nfds {
                Err(ecust!("Malformed return value"))
            } else {
                Ok(PolledOk::ReadyDescsCount(result as usize))
            }
        }
        0 => Ok(PolledOk::TimeLimitExpired),
        _ => Err(eos!(error)),
    }
}

pub unsafe fn epoll_create1(flags: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_epoll_create1_ocall(&mut result as *mut c_int, &mut error as *mut c_int, flags);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));

    Ok(result)
}

pub unsafe fn epoll_ctl(
    epfd: c_int,
    op: c_int,
    fd: c_int,
    event: &mut epoll_event,
) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_epoll_ctl_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        epfd,
        op,
        fd,
        event,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));

    Ok(())
}

pub unsafe fn epoll_wait(
    epfd: c_int,
    events: &mut Vec<epoll_event>,
    timeout: c_int,
) -> OCallResult<usize> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let maxevents: c_int = events.capacity() as c_int;
    ensure!(maxevents > 0, ecust!("Invalid events capacity"));

    let status = u_epoll_wait_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        epfd,
        events.as_mut_ptr(),
        maxevents,
        timeout,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));
    ensure!(result <= maxevents, ecust!("Malformed event count"));

    events.set_len(result as usize);
    Ok(result as usize)
}

pub unsafe fn sysconf(name: c_int) -> OCallResult<c_long> {
    let mut result: c_long = 0;
    let mut error: c_int = 0;

    let status = u_sysconf_ocall(&mut result as *mut c_long, &mut error as *mut c_int, name);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn prctl(
    option: c_int,
    arg2: c_ulong,
    arg3: c_ulong,
    arg4: c_ulong,
    arg5: c_ulong,
) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_prctl_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        option,
        arg2,
        arg3,
        arg4,
        arg5,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn pipe2(fds: &mut [c_int; 2], flags: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_pipe2_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fds.as_mut_ptr(),
        flags,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn sched_yield() -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_sched_yield_ocall(&mut result as *mut c_int, &mut error as *mut c_int);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn getpid() -> OCallResult<pid_t> {
    let mut result = -1;

    let status = u_getpid_ocall(&mut result as *mut pid_t);

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));
    Ok(result)
}

pub unsafe fn nanosleep(req: &mut timespec) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let mut rem = req.clone();

    let status = u_nanosleep_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        req,
        &mut rem,
    );

    ensure!(status == sgx_status_t::SGX_SUCCESS, esgx!(status));

    ensure!(rem.tv_sec <= req.tv_sec, ecust!("Malformed remaining time"));
    if rem.tv_sec == req.tv_sec {
        ensure!(
            rem.tv_nsec < req.tv_nsec,
            ecust!("Malformed remaining time")
        );
    }

    req.tv_sec = rem.tv_sec;
    req.tv_nsec = rem.tv_nsec;

    ensure!(result == 0, eos!(error));

    req.tv_sec = 0;
    req.tv_nsec = 0;
    Ok(())
}

pub unsafe fn sigaction(
    signum: c_int,
    act: *const sigaction,
    oldact: *mut sigaction,
    enclave_id: uint64_t,
) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_sigaction_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        signum,
        act,
        oldact,
        enclave_id,
    );

    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = -1;
    }
    result
}

pub unsafe fn sigprocmask(signum: c_int, set: *const sigset_t, oldset: *mut sigset_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_sigprocmask_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        signum,
        set,
        oldset,
    );

    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = -1;
    }
    result
}

pub unsafe fn raise(signum: c_int) -> c_int {
    let mut result: c_int = -1;

    let status = u_raise_ocall(&mut result as *mut c_int, signum);

    if status != sgx_status_t::SGX_SUCCESS {
        result = -1;
    }
    result
}

pub unsafe fn pthread_sigmask(signum: c_int, set: &sigset_t, oldset: &mut sigset_t) -> c_int {
    sigprocmask(signum, set, oldset)
}

pub unsafe fn sched_setaffinity(pid: pid_t, cpusetsize: size_t, mask: *const cpu_set_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_sched_setaffinity_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        pid,
        cpusetsize,
        mask,
    );

    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = -1;
    }
    result
}

pub unsafe fn sched_getaffinity(pid: pid_t, cpusetsize: size_t, mask: &mut cpu_set_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_sched_getaffinity_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        pid,
        cpusetsize,
        mask,
    );

    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = -1;
    }
    result
}
