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

use sgx_types::sgx_status_t;
use super::*;

extern "C" {
    // memory
    pub fn u_malloc_ocall(result: * mut * mut c_void, error: * mut c_int, size: size_t) -> sgx_status_t;
    pub fn u_free_ocall(p: * mut c_void) -> sgx_status_t;
    pub fn u_mmap_ocall(result: * mut * mut c_void,
                        error: * mut c_int,
                        start: * mut c_void,
                        length: size_t,
                        prot: c_int,
                        flags: c_int,
                        fd: c_int,
                        offset: off_t) -> sgx_status_t;
    pub fn u_munmap_ocall(result: * mut c_int,
                          error: * mut c_int,
                          start: * mut c_void,
                          length: size_t) -> sgx_status_t;

    pub fn u_msync_ocall(result: * mut c_int,
                         error: * mut c_int,
                         addr: * mut c_void,
                         length: size_t,
                         flags: c_int) -> sgx_status_t;

    pub fn u_mprotect_ocall(result: * mut c_int,
                            error: * mut c_int,
                            addr: * mut c_void,
                            length: size_t,
                            prot: c_int) -> sgx_status_t;
    // env
    pub fn u_environ_ocall(result: * mut * const * const c_char) -> sgx_status_t;
    pub fn u_getenv_ocall(result: * mut * const c_char,
                          name: * const c_char) -> sgx_status_t;
    pub fn u_setenv_ocall(result: * mut c_int,
                          error: * mut c_int,
                          name: * const c_char,
                          value: * const c_char,
                          overwrite: c_int) -> sgx_status_t;
    pub fn u_unsetenv_ocall(result: * mut c_int,
                            error: * mut c_int,
                            name: * const c_char) -> sgx_status_t;
    // file
    pub fn u_open_ocall(result: * mut c_int,
                        error: * mut c_int,
                        path: * const c_char,
                        flags: c_int) -> sgx_status_t;
    pub fn u_open64_ocall(result: * mut c_int,
                          error: * mut c_int,
                          path: * const c_char,
                          oflag: c_int,
                          mode: c_int) -> sgx_status_t;            
    pub fn u_fstat_ocall(result: * mut c_int,
                         error: * mut c_int,
                         fd: c_int,
                         buf: * mut stat) -> sgx_status_t;
    pub fn u_fstat64_ocall(result: * mut c_int,
                           error: * mut c_int,
                           fd: c_int,
                           buf: * mut stat64) -> sgx_status_t;
    pub fn u_stat_ocall(result: * mut c_int,
                        error: * mut c_int,
                        path: * const c_char,
                        buf: * mut stat) -> sgx_status_t;
    pub fn u_stat64_ocall(result: * mut c_int,
                          error: * mut c_int,
                          path: * const c_char,
                          buf: * mut stat64) -> sgx_status_t;
    pub fn u_lstat_ocall(result: * mut c_int,
                         error: * mut c_int,
                         path: * const c_char,
                         buf: * mut stat) -> sgx_status_t;
    pub fn u_lstat64_ocall(result: * mut c_int,
                           error: * mut c_int,
                           path: * const c_char,
                           buf: * mut stat64) -> sgx_status_t;
    pub fn u_lseek_ocall(result: * mut off_t,
                         error: * mut c_int,
                         fd: c_int,
                         offset: off_t,
                         whence: c_int) -> sgx_status_t;
    pub fn u_lseek64_ocall(result: * mut off64_t,
                           error: * mut c_int,
                           fd: c_int,
                           offset: off64_t,
                           whence: c_int) -> sgx_status_t;
    pub fn u_ftruncate_ocall(result: * mut c_int,
                             error: * mut c_int,
                             fd: c_int,
                             length: off_t) -> sgx_status_t;
    pub fn u_ftruncate64_ocall(result: * mut c_int,
                               error: * mut c_int,
                               fd: c_int,
                               length: off64_t) -> sgx_status_t;
    pub fn u_truncate_ocall(result: * mut c_int,
                            error: * mut c_int,
                            path: * const c_char,
                            length: off_t) -> sgx_status_t;
    pub fn u_truncate64_ocall(result: * mut c_int,
                              error: * mut c_int,
                              path: * const c_char,
                              length: off64_t) -> sgx_status_t;
    pub fn u_fsync_ocall(result: * mut c_int,
                         error: * mut c_int,
                         fd: c_int) -> sgx_status_t;
    pub fn u_fdatasync_ocall(result: * mut c_int,
                             error: * mut c_int,
                             fd: c_int) -> sgx_status_t;
    pub fn u_fchmod_ocall(result: * mut c_int,
                          error: * mut c_int,
                          fd: c_int,
                          mode: mode_t) -> sgx_status_t;
    pub fn u_unlink_ocall(result: * mut c_int,
                          error: * mut c_int,
                          pathname: * const c_char) -> sgx_status_t;
    pub fn u_link_ocall(result: * mut c_int,
                        error: * mut c_int,
                        oldpath: * const c_char,
                        newpath: * const c_char) -> sgx_status_t;
    pub fn u_rename_ocall(result: * mut c_int,
                          error: * mut c_int,
                          oldpath: * const c_char,
                          newpath: * const c_char) -> sgx_status_t;
    pub fn u_chmod_ocall(result: * mut c_int,
                         error: * mut c_int,
                         path: * const c_char,
                         mode: mode_t) -> sgx_status_t;
    pub fn u_readlink_ocall(result: * mut ssize_t,
                            error: * mut c_int,
                            path: * const c_char,
                            buf: * mut c_char,
                            bufsz: size_t) -> sgx_status_t;
    pub fn u_symlink_ocall(result: * mut c_int,
                           error: * mut c_int,
                           path1: * const c_char,
                           path2: * const c_char) -> sgx_status_t;
    pub fn u_realpath_ocall(result: * mut * mut c_char,
                            error: * mut c_int,
                            pathname: * const c_char) -> sgx_status_t;
    // fd
    pub fn u_read_ocall(result: * mut ssize_t,
                        errno: * mut c_int,
                        fd: c_int,
                        buf: * mut c_void,
                        count: size_t) -> sgx_status_t;
    pub fn u_pread64_ocall(result: * mut ssize_t,
                           errno: * mut c_int,
                           fd: c_int,
                           buf: * mut c_void,
                           count: size_t,
                           offset: off64_t) -> sgx_status_t;
    pub fn u_write_ocall(result: * mut ssize_t,
                         errno: * mut c_int,
                         fd: c_int,
                         buf: * const c_void,
                         count: size_t) -> sgx_status_t;
    pub fn u_pwrite64_ocall(result: * mut ssize_t,
                            errno: * mut c_int,
                            fd: c_int,
                            buf: * const c_void,
                            count: size_t,
                            offset: off64_t) -> sgx_status_t;
    pub fn u_fcntl_arg0_ocall(result: * mut c_int,
                              errno: * mut c_int,
                              fd: c_int,
                              cmd: c_int) -> sgx_status_t;
    
    pub fn u_fcntl_arg1_ocall(result: * mut c_int,
                              errno: * mut c_int,
                              fd: c_int,
                              cmd: c_int,
                              arg: c_int) -> sgx_status_t;
    pub fn u_ioctl_arg0_ocall(result: * mut c_int,
                              errno: * mut c_int,
                              fd: c_int,
                              request: c_int) -> sgx_status_t;
    pub fn u_ioctl_arg1_ocall(result: * mut c_int,
                              errno: * mut c_int,
                              fd: c_int,
                              request: c_int,
                              arg: * const c_int) -> sgx_status_t;
    pub fn u_close_ocall(result: * mut c_int,
                         errno: * mut c_int,
                         fd: c_int) -> sgx_status_t;
    // time
    pub fn u_clock_gettime_ocall(result: * mut c_int,
                                 errno: * mut c_int,
                                 clk_id: clockid_t,
                                 tp: * mut timespec) -> sgx_status_t;
    // socket
    pub fn u_socket_ocall(result: *mut c_int,
                          errno: *mut c_int,
                          domain: c_int,
                          ty: c_int,
                          protocol: c_int) -> sgx_status_t;
    pub fn u_socketpair_ocall(result: *mut c_int,
                              errno: *mut c_int,
                              domain: c_int,
                              ty: c_int,
                              protocol: c_int,
                              sv: *mut c_int) -> sgx_status_t;
    pub fn u_bind_ocall(result: * mut c_int,
                        errno: * mut c_int,
                        sockfd: c_int,
                        address: * const sockaddr,
                        addrlen: socklen_t) -> sgx_status_t;
    pub fn u_listen_ocall(result: *mut c_int,
                          error: *mut c_int,
                          sockfd: c_int,
                          backlog: c_int) -> sgx_status_t;
    pub fn u_accept4_ocall(result: *mut c_int,
                           errno: *mut c_int,
                           sockfd: c_int,
                           addr: *mut sockaddr,
                           addrlen_in: socklen_t,
                           addrlen_out: *mut socklen_t,
                           flags: c_int) -> sgx_status_t;
    pub fn u_connect_ocall(result: * mut c_int,
                           errno: * mut c_int,
                           sockfd: c_int,
                           address: * const sockaddr,
                           addrlen: socklen_t) -> sgx_status_t;
    pub fn u_send_ocall(result: * mut ssize_t,
                        errno: * mut c_int,
                        sockfd: c_int,
                        buf: * const c_void,
                        len: size_t,
                        flags: c_int) -> sgx_status_t;
    pub fn u_sendto_ocall(result: * mut ssize_t,
                          errno: * mut c_int,
                          sockfd: c_int,
                          buf: * const c_void,
                          len: size_t,
                          flags: c_int,
                          addr: * const sockaddr,
                          addrlen: socklen_t) -> sgx_status_t;
    pub fn u_recv_ocall(result: * mut ssize_t,
                        errno: * mut c_int,
                        sockfd: c_int,
                        buf: * mut c_void,
                        len: size_t,
                        flags: c_int) -> sgx_status_t;
    pub fn u_recvfrom_ocall(result: * mut ssize_t,
                            errno: * mut c_int,
                            sockfd: c_int,
                            buf: * mut c_void,
                            len: size_t,
                            flags: c_int,
                            addr: * mut sockaddr,
                            addrlen_in: socklen_t,
                            addrlen_out: * mut socklen_t) -> sgx_status_t;
    pub fn u_setsockopt_ocall(result: * mut c_int,
                              errno: * mut c_int,
                              sockfd: c_int,
                              level: c_int,
                              optname: c_int,
                              optval: * const c_void,
                              optlen: socklen_t) -> sgx_status_t;
    pub fn u_getsockopt_ocall(result: * mut c_int,
                              errno: * mut c_int,
                              sockfd: c_int,
                              level: c_int,
                              optname: c_int,
                              optval: * mut c_void,
                              optlen_in: socklen_t,
                              optlen_out: * mut socklen_t) -> sgx_status_t;
    pub fn u_getpeername_ocall(result: * mut c_int,
                               errno: * mut c_int,
                               sockfd: c_int,
                               address: * mut sockaddr,
                               addrlen_in: socklen_t,
                               addrlen_out: * mut socklen_t) -> sgx_status_t;
    pub fn u_getsockname_ocall(result: * mut c_int,
                               errno: * mut c_int,
                               sockfd: c_int,
                               address: * mut sockaddr,
                               addrlen_in: socklen_t,
                               addrlen_out: * mut socklen_t) -> sgx_status_t;
    pub fn u_shutdown_ocall(result: * mut c_int,
                            errno: * mut c_int,
                            sockfd: c_int,
                            how: c_int) -> sgx_status_t;
    // async io
    pub fn u_poll_ocall(result: * mut c_int,
                        errno: * mut c_int,
                        fds: * mut pollfd,
                        nfds: nfds_t,
                        timeout: c_int) -> sgx_status_t;

    pub fn u_epoll_create1_ocall(result: * mut c_int,
                                 error: * mut c_int,
                                 flags: c_int) -> sgx_status_t;
    pub fn u_epoll_ctl_ocall(result: * mut c_int,
                             error: * mut c_int,
                             epfd: c_int,
                             op: c_int,
                             fd: c_int,
                             event: * mut epoll_event) -> sgx_status_t;
    pub fn u_epoll_wait_ocall(result: * mut c_int,
                              error: * mut c_int,
                              epfd: c_int,
                              events: * mut epoll_event,
                              maxevents: c_int,
                              timeout: c_int) -> sgx_status_t;
    // sys
    pub fn u_sysconf_ocall(result: * mut c_long,
                           error: * mut c_int,
                           name: c_int) -> sgx_status_t;
}

pub unsafe fn malloc(size: size_t) -> * mut c_void {
    let mut result: * mut c_void = ptr::null_mut();
    let mut error: c_int = 0;
    let status = u_malloc_ocall(&mut result as * mut * mut c_void,
                                &mut error as * mut c_int,
                                size);

    if status == sgx_status_t::SGX_SUCCESS {
        if result.is_null() {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = ptr::null_mut();
    }
    result
}

pub unsafe fn free(p: * mut c_void) {
    let _ = u_free_ocall(p);
}

pub unsafe fn mmap(start: * mut c_void, length: size_t, prot: c_int, flags: c_int, fd: c_int, offset: off_t) -> * mut c_void {
    let mut result: * mut c_void = ptr::null_mut();
    let mut error: c_int = 0;
    let status = u_mmap_ocall(&mut result as * mut * mut c_void,
                              &mut error as * mut c_int,
                              start,
                              length,
                              prot,
                              flags,
                              fd,
                              offset);

    if status == sgx_status_t::SGX_SUCCESS {
        if result as isize == -1 {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = -1 as isize as * mut c_void;
    }
    result
}

pub unsafe fn munmap(start: * mut c_void, length: size_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_munmap_ocall(&mut result as * mut c_int,
                                &mut error as * mut c_int,
                                start,
                                length);

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

pub unsafe fn msync(addr: * mut c_void, length: size_t, flags: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_msync_ocall(&mut result as * mut c_int,
                               &mut error as * mut c_int,
                               addr,
                               length,
                               flags);

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

pub unsafe fn mprotect(addr: * mut c_void, length: size_t, prot: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_mprotect_ocall(&mut result as * mut c_int,
                                  &mut error as * mut c_int,
                                  addr,
                                  length,
                                  prot);

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

pub unsafe fn environ() -> * const * const c_char {
    let mut result: * const * const c_char = ptr::null();
    let status = u_environ_ocall(&mut result as * mut * const * const c_char);

    if status != sgx_status_t::SGX_SUCCESS {
        result = ptr::null();
    }
    result
}

pub unsafe fn getenv(name: * const c_char) -> * const c_char {
    let mut result: * const c_char = ptr::null();
    let status = u_getenv_ocall(&mut result as * mut * const c_char, name);

    if status != sgx_status_t::SGX_SUCCESS {
        result = ptr::null();
    }
    result
}

pub unsafe fn setenv(name: * const c_char, value: * const c_char, overwrite: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_setenv_ocall(&mut result as * mut c_int,
                                &mut error as * mut c_int,
                                name,
                                value,
                                overwrite);

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

pub unsafe fn unsetenv(name: * const c_char) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_unsetenv_ocall(&mut result as * mut c_int,
                                  &mut error as * mut c_int,
                                  name);

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

pub unsafe fn open(path: * const c_char, flags: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_open_ocall(&mut result as * mut c_int,
                              &mut error as * mut c_int,
                              path,
                              flags);

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
pub unsafe fn open64(path: * const c_char, oflag: c_int, mode: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_open64_ocall(&mut result as * mut c_int,
                                &mut error as * mut c_int,
                                path,
                                oflag,
                                mode);

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

pub unsafe fn fstat(fd: c_int, buf: * mut stat) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_fstat_ocall(&mut result as * mut c_int,
                               &mut error as * mut c_int,
                               fd,
                               buf);

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

pub unsafe fn fstat64(fd: c_int, buf: * mut stat64) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_fstat64_ocall(&mut result as * mut c_int,
                                 &mut error as * mut c_int,
                                 fd,
                                 buf);

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

pub unsafe fn stat(path: * const c_char, buf: * mut stat) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_stat_ocall(&mut result as * mut c_int,
                              &mut error as * mut c_int,
                              path,
                              buf);

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

pub unsafe fn stat64(path: * const c_char, buf: * mut stat64) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_stat64_ocall(&mut result as * mut c_int,
                                &mut error as * mut c_int,
                                path,
                                buf);

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

pub unsafe fn lstat(path: * const c_char, buf: * mut stat) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_lstat_ocall(&mut result as * mut c_int,
                               &mut error as * mut c_int,
                               path,
                               buf);

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

pub unsafe fn lstat64(path: * const c_char, buf: * mut stat64) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_lstat64_ocall(&mut result as * mut c_int,
                                 &mut error as * mut c_int,
                                 path,
                                 buf);

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

pub unsafe fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t {
    let mut result: off_t = 0;
    let mut error: c_int = 0;
    let status = u_lseek_ocall(&mut result as * mut off_t,
                               &mut error as * mut c_int,
                               fd,
                               offset,
                               whence);

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

pub unsafe fn lseek64(fd: c_int, offset: off64_t, whence: c_int) -> off64_t {
    let mut result: off64_t = 0;
    let mut error: c_int = 0;
    let status = u_lseek64_ocall(&mut result as * mut off64_t,
                                 &mut error as * mut c_int,
                                 fd,
                                 offset,
                                 whence);

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

pub unsafe fn ftruncate(fd: c_int, length: off_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_ftruncate_ocall(&mut result as * mut c_int,
                                   &mut error as * mut c_int,
                                   fd,
                                   length);

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

pub unsafe fn ftruncate64(fd: c_int, length: off64_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_ftruncate64_ocall(&mut result as * mut c_int,
                                     &mut error as * mut c_int,
                                     fd,
                                     length);

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

pub unsafe fn truncate(path: * const c_char, length: off_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_truncate_ocall(&mut result as * mut c_int,
                                  &mut error as * mut c_int,
                                  path,
                                  length);

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

pub unsafe fn truncate64(path: * const c_char, length: off64_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_truncate64_ocall(&mut result as * mut c_int,
                                    &mut error as * mut c_int,
                                    path,
                                    length);

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

pub unsafe fn fsync(fd: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_fsync_ocall(&mut result as * mut c_int,
                               &mut error as * mut c_int,
                               fd);

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

pub unsafe fn fdatasync(fd: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_fdatasync_ocall(&mut result as * mut c_int,
                                   &mut error as * mut c_int,
                                   fd);

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

pub unsafe fn fchmod(fd: c_int, mode: mode_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_fchmod_ocall(&mut result as * mut c_int,
                                &mut error as * mut c_int,
                                fd,
                                mode);

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

pub unsafe fn unlink(pathname: * const c_char) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_unlink_ocall(&mut result as * mut c_int,
                                &mut error as * mut c_int,
                                pathname);

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

pub unsafe fn link(oldpath: * const c_char, newpath: * const c_char) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_link_ocall(&mut result as * mut c_int,
                              &mut error as * mut c_int,
                              oldpath,
                              newpath);

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

pub unsafe fn rename(oldpath: * const c_char, newpath: * const c_char) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_rename_ocall(&mut result as * mut c_int,
                                &mut error as * mut c_int,
                                oldpath,
                                newpath);

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

pub unsafe fn chmod(path: * const c_char, mode: mode_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_chmod_ocall(&mut result as * mut c_int,
                               &mut error as * mut c_int,
                               path,
                               mode);

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

pub unsafe fn readlink(path: * const c_char, buf: * mut c_char, bufsz: size_t) -> ssize_t {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let status = u_readlink_ocall(&mut result as * mut ssize_t,
                                  &mut error as * mut c_int,
                                  path,
                                  buf,
                                  bufsz);

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

pub unsafe fn symlink(path1: * const c_char, path2: * const c_char) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_symlink_ocall(&mut result as * mut c_int,
                                 &mut error as * mut c_int,
                                 path1,
                                 path2);

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

pub unsafe fn realpath(pathname: * const c_char) -> * mut c_char {
    let mut result: * mut c_char = ptr::null_mut();
    let mut error: c_int = 0;
    let status = u_realpath_ocall(&mut result as * mut * mut c_char,
                                  &mut error as * mut c_int,
                                  pathname);

    if status == sgx_status_t::SGX_SUCCESS {
        if result.is_null() {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = ptr::null_mut();
    }
    result
}

pub unsafe fn read(fd: c_int, buf: * mut c_void, count: size_t) -> ssize_t {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let status = u_read_ocall(&mut result as * mut ssize_t,
                              &mut error as * mut c_int,
                              fd,
                              buf,
                              count);

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

pub unsafe fn pread64(fd: c_int, buf: * mut c_void, count: size_t, offset: off64_t) -> ssize_t {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let status = u_pread64_ocall(&mut result as * mut ssize_t,
                                 &mut error as * mut c_int,
                                 fd,
                                 buf,
                                 count,
                                 offset);

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

pub unsafe fn write(fd: c_int, buf: * const c_void, count: size_t) -> ssize_t {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let status = u_write_ocall(&mut result as * mut ssize_t,
                               &mut error as * mut c_int,
                               fd,
                               buf,
                               count);

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

pub unsafe fn pwrite64(fd: c_int, buf: * const c_void, count: size_t, offset: off64_t) -> ssize_t {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let status = u_pwrite64_ocall(&mut result as * mut ssize_t,
                                  &mut error as * mut c_int,
                                  fd,
                                  buf,
                                  count,
                                  offset);

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

pub unsafe fn fcntl_arg0(fd: c_int, cmd: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_fcntl_arg0_ocall(&mut result as * mut c_int,
                                    &mut error as * mut c_int,
                                    fd,
                                    cmd);

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

pub unsafe fn fcntl_arg1(fd: c_int, cmd: c_int, arg: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_fcntl_arg1_ocall(&mut result as * mut c_int,
                                    &mut error as * mut c_int,
                                    fd,
                                    cmd,
                                    arg);

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

pub unsafe fn ioctl_arg0(fd: c_int, request: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_ioctl_arg0_ocall(&mut result as * mut c_int,
                                    &mut error as * mut c_int,
                                    fd,
                                    request);

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

pub unsafe fn ioctl_arg1(fd: c_int, request: c_int, arg: * const c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_ioctl_arg1_ocall(&mut result as * mut c_int,
                                    &mut error as * mut c_int,
                                    fd,
                                    request,arg);

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

pub unsafe fn close(fd: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_close_ocall(&mut result as * mut c_int,
                               &mut error as * mut c_int,
                               fd);

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

pub unsafe fn clock_gettime(clk_id: clockid_t, tp: * mut timespec) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_clock_gettime_ocall(&mut result as * mut c_int,
                                       &mut error as * mut c_int,
                                       clk_id,
                                       tp);

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


pub unsafe fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_socket_ocall(&mut result as *mut c_int,
                                &mut error as *mut c_int,
                                domain,
                                ty,
                                protocol);

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

pub unsafe fn socketpair(domain: c_int, ty: c_int, protocol: c_int, sv: *mut c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_socketpair_ocall(&mut result as *mut c_int,
                                    &mut error as *mut c_int,
                                    domain,
                                    ty,
                                    protocol,
                                    sv);

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

pub unsafe fn bind(sockfd: c_int, address: * const sockaddr, addrlen: socklen_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_bind_ocall(&mut result as * mut c_int,
                              &mut error as * mut c_int,
                              sockfd,
                              address,
                              addrlen);

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

pub unsafe fn listen(sockfd: c_int, backlog: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_listen_ocall(&mut result as *mut c_int,
                                &mut error as *mut c_int,
                                sockfd,
                                backlog);

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

pub unsafe fn accept4(sockfd: c_int, addr: *mut sockaddr, addrlen: *mut socklen_t, flags: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let len_in: socklen_t = if !addrlen.is_null() { *addrlen } else { 0 };
    let mut len_out: socklen_t = 0 as socklen_t;
    let status = u_accept4_ocall(&mut result as *mut c_int,
                                 &mut error as *mut c_int,
                                 sockfd,
                                 addr,
                                 len_in, // This additional arg is just for EDL
                                 &mut len_out as * mut socklen_t,
                                 flags);

    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = -1;
    }

    if !addrlen.is_null() {
        *addrlen = len_out;
    }
    result
}

pub unsafe fn connect(sockfd: c_int, address: * const sockaddr, addrlen: socklen_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_connect_ocall(&mut result as * mut c_int,
                                 &mut error as * mut c_int,
                                 sockfd,
                                 address,
                                 addrlen);

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

pub unsafe fn send(sockfd: c_int, buf: * const c_void, len: size_t, flags: c_int) -> ssize_t {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let status = u_send_ocall(&mut result as * mut ssize_t,
                              &mut error as * mut c_int,
                              sockfd,
                              buf,
                              len,
                              flags);

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

pub unsafe fn sendto(sockfd: c_int,
                     buf: * const c_void,
                     len: size_t,
                     flags: c_int,
                     addr: * const sockaddr,
                     addrlen: socklen_t) -> ssize_t {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let status = u_sendto_ocall(&mut result as * mut ssize_t,
                                &mut error as * mut c_int,
                                sockfd,
                                buf,
                                len,
                                flags,
                                addr,
                                addrlen);

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

pub unsafe fn recv(sockfd: c_int, buf: * mut c_void, len: size_t, flags: c_int) -> ssize_t {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let status = u_recv_ocall(&mut result as * mut ssize_t,
                              &mut error as * mut c_int,
                              sockfd,
                              buf,
                              len,
                              flags);

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

pub unsafe fn recvfrom(sockfd: c_int,
                       buf: * mut c_void,
                       len: size_t,
                       flags: c_int,
                       addr: * mut sockaddr,
                       addrlen: * mut socklen_t) -> ssize_t {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let len_in: socklen_t = if !addrlen.is_null() { *addrlen } else { 0 };
    let mut len_out: socklen_t = 0 as socklen_t;
    let status = u_recvfrom_ocall(&mut result as * mut ssize_t,
                                  &mut error as * mut c_int,
                                  sockfd,
                                  buf,
                                  len,
                                  flags,
                                  addr,
                                  len_in, // This additional arg is just for EDL
                                  &mut len_out as * mut socklen_t);

    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = -1;
    }

    if !addrlen.is_null() {
        *addrlen = len_out;
    }
    result
}

pub unsafe fn setsockopt(sockfd: c_int,
                         level: c_int,
                         optname: c_int,
                         optval: * const c_void,
                         optlen: socklen_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_setsockopt_ocall(&mut result as * mut c_int,
                                    &mut error as * mut c_int,
                                    sockfd,
                                    level,
                                    optname,
                                    optval,
                                    optlen);

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

pub unsafe fn getsockopt(sockfd: c_int,
                         level: c_int,
                         optname: c_int,
                         optval: * mut c_void,
                         optlen: * mut socklen_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let len_in: socklen_t = if !optlen.is_null() { *optlen } else { 0 };
    let mut len_out: socklen_t = 0 as socklen_t;

    let status = u_getsockopt_ocall(&mut result as * mut c_int,
                                    &mut error as * mut c_int,
                                    sockfd,
                                    level,
                                    optname,
                                    optval,
                                    len_in,
                                    &mut len_out as * mut socklen_t);

    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = -1;
    }

    if !optlen.is_null() {
        *optlen = len_out;
    }
    result
}

pub unsafe fn getpeername(sockfd: c_int, address: * mut sockaddr, addrlen: * mut socklen_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let len_in: socklen_t = if !addrlen.is_null() { *addrlen } else { 0 };
    let mut len_out: socklen_t = 0 as socklen_t;
    let status = u_getpeername_ocall(&mut result as * mut c_int,
                                     &mut error as * mut c_int,
                                     sockfd,
                                     address,
                                     len_in,
                                     &mut len_out as * mut socklen_t);

    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = -1;
    }

    if !addrlen.is_null() {
        *addrlen = len_out;
    }
    result
}

pub unsafe fn getsockname(sockfd: c_int, address: * mut sockaddr, addrlen: * mut socklen_t) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let len_in: socklen_t = if !addrlen.is_null() { *addrlen } else { 0 };
    let mut len_out: socklen_t = 0 as socklen_t;
    let status = u_getsockname_ocall(&mut result as * mut c_int,
                                     &mut error as * mut c_int,
                                     sockfd,
                                     address,
                                     len_in,
                                     &mut len_out as * mut socklen_t);

    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 {
            set_errno(error);
        }
    } else {
        set_errno(ESGX);
        result = -1;
    }

    if !addrlen.is_null() {
        *addrlen = len_out;
    }
    result
}

pub unsafe fn shutdown(sockfd: c_int, how: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_shutdown_ocall(&mut result as * mut c_int,
                                  &mut error as * mut c_int,
                                  sockfd,
                                  how);

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

pub unsafe fn poll(fds: * mut pollfd, nfds: nfds_t, timeout: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_poll_ocall(&mut result as * mut c_int,
                              &mut error as * mut c_int,
                              fds,
                              nfds,
                              timeout);

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

pub unsafe fn epoll_create1(flags: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_epoll_create1_ocall(&mut result as * mut c_int,
                                       &mut error as * mut c_int,
                                       flags);
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

pub unsafe fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: * mut epoll_event) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_epoll_ctl_ocall(&mut result as * mut c_int,
                                   &mut error as * mut c_int,
                                   epfd,
                                   op,
                                   fd,
                                   event);
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

pub unsafe fn epoll_wait(epfd: c_int, events: * mut epoll_event, maxevents: c_int, timeout: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_epoll_wait_ocall(&mut result as * mut c_int,
                                    &mut error as * mut c_int,
                                    epfd,
                                    events,
                                    maxevents,
                                    timeout);
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

pub unsafe fn sysconf(name: c_int) -> c_long {
    let mut result: c_long = 0;
    let mut error: c_int = 0;
    let status = u_sysconf_ocall(&mut result as * mut c_long,
                                 &mut error as * mut c_int,
                                 name);

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