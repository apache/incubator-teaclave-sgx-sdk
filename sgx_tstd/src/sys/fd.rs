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

use sgx_trts::libc::{c_int, ssize_t, c_void};
use core::cmp;
use core::mem;
use core::sync::atomic::{AtomicBool, Ordering};
use io::{self, Read};
use sys::cvt;
use sys_common::AsInner;

#[derive(Debug)]
pub struct FileDesc {
    fd: c_int,
}

fn max_len() -> usize {
    // The maximum read limit on most posix-like systems is `SSIZE_MAX`,
    // with the man page quoting that if the count of bytes to read is
    // greater than `SSIZE_MAX` the result is "unspecified".
    //
    // On macOS, however, apparently the 64-bit libc is either buggy or
    // intentionally showing odd behavior by rejecting any read with a size
    // larger than or equal to INT_MAX. To handle both of these the read
    // size is capped on both platforms.
    if cfg!(target_os = "macos") {
        <c_int>::max_value() as usize - 1
    } else {
        <ssize_t>::max_value() as usize
    }
}

impl FileDesc {
    pub fn new(fd: c_int) -> FileDesc {
        FileDesc { fd: fd }
    }

    pub fn raw(&self) -> c_int { self.fd }

    /// Extracts the actual filedescriptor without closing it.
    pub fn into_raw(self) -> c_int {
        let fd = self.fd;
        mem::forget(self);
        fd
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::read(self.fd,
                       buf.as_mut_ptr() as *mut c_void,
                       cmp::min(buf.len(), max_len()))
        })?;
        Ok(ret as usize)
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {

        unsafe fn cvt_pread64(fd: c_int, buf: *mut c_void, count: usize, offset: i64)
            -> io::Result<isize>
        {
            cvt(libc::pread64(fd, buf, count, offset))
        }

        unsafe {
            cvt_pread64(self.fd,
                        buf.as_mut_ptr() as *mut c_void,
                        cmp::min(buf.len(), max_len()),
                        offset as i64)
                .map(|n| n as usize)
        }
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::write(self.fd,
                        buf.as_ptr() as *const c_void,
                        cmp::min(buf.len(), max_len()))
        })?;
        Ok(ret as usize)
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {

        unsafe fn cvt_pwrite64(fd: c_int, buf: *const c_void, count: usize, offset: i64)
            -> io::Result<isize>
        {
            cvt(libc::pwrite64(fd, buf, count, offset))
        }

        unsafe {
            cvt_pwrite64(self.fd,
                         buf.as_ptr() as *const c_void,
                         cmp::min(buf.len(), max_len()),
                         offset as i64)
                .map(|n| n as usize)
        }
    }

    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            cvt(libc::ioctl_arg0(self.fd, libc::FIOCLEX))?;
            Ok(())
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        unsafe {
            let v = nonblocking as c_int;
            cvt(libc::ioctl_arg1(self.fd, libc::FIONBIO, &v as * const c_int))?;
            Ok(())
        }
    }

    pub fn duplicate(&self) -> io::Result<FileDesc> {
        // We want to atomically duplicate this file descriptor and set the
        // CLOEXEC flag, and currently that's done via F_DUPFD_CLOEXEC. This
        // flag, however, isn't supported on older Linux kernels (earlier than
        // 2.6.24).
        //
        // To detect this and ensure that CLOEXEC is still set, we
        // follow a strategy similar to musl [1] where if passing
        // F_DUPFD_CLOEXEC causes `fcntl` to return EINVAL it means it's not
        // supported (the third parameter, 0, is always valid), so we stop
        // trying that.
        //
        // Also note that Android doesn't have F_DUPFD_CLOEXEC, but get it to
        // resolve so we at least compile this.
        //
        // [1]: http://comments.gmane.org/gmane.linux.lib.musl.general/2963

        let make_filedesc = |fd| {
            let fd = FileDesc::new(fd);
            fd.set_cloexec()?;
            Ok(fd)
        };
        static TRY_CLOEXEC: AtomicBool =
            AtomicBool::new(!cfg!(target_os = "android"));
        let fd = self.raw();
        if TRY_CLOEXEC.load(Ordering::Relaxed) {
            match cvt(unsafe { libc::fcntl_arg1(fd, libc::F_DUPFD_CLOEXEC, 0) }) {
                // We *still* call the `set_cloexec` method as apparently some
                // linux kernel at some point stopped setting CLOEXEC even
                // though it reported doing so on F_DUPFD_CLOEXEC.
                Ok(fd) => {
                    return Ok(if cfg!(target_os = "linux") {
                        make_filedesc(fd)?
                    } else {
                        FileDesc::new(fd)
                    })
                }
                Err(ref e) if e.raw_os_error() == Some(libc::EINVAL) => {
                    TRY_CLOEXEC.store(false, Ordering::Relaxed);
                }
                Err(e) => return Err(e),
            }
        }
        cvt(unsafe { libc::fcntl_arg1(fd, libc::F_DUPFD, 0) }).and_then(make_filedesc)
    }
}

impl<'a> Read for &'a FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }
}

impl AsInner<c_int> for FileDesc {
    fn as_inner(&self) -> &c_int { &self.fd }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        // Note that errors are ignored when closing a file descriptor. The
        // reason for this is that if an error occurs we don't actually know if
        // the file descriptor was closed or not, and if we retried (for
        // something like EINTR), we might close another valid file descriptor
        // (opened after we closed ours.
        let _ = unsafe { libc::close(self.fd) };
    }
}

mod libc {
    use sgx_types::sgx_status_t;
    use io;
    pub use sgx_trts::libc::*;
    
    extern "C" {
        pub fn u_fs_read_ocall(result: * mut ssize_t,
                               errno: * mut c_int,
                               fd: c_int,
                               buf: * mut c_void,
                               count: size_t) -> sgx_status_t;

        pub fn u_fs_pread64_ocall(result: * mut ssize_t,
                                  errno: * mut c_int,
                                  fd: c_int,
                                  buf: * mut c_void,
                                  count: size_t,
                                  offset: off64_t) -> sgx_status_t;

        pub fn u_fs_write_ocall(result: * mut ssize_t,
                                errno: * mut c_int,
                                fd: c_int,
                                buf: * const c_void,
                                count: size_t) -> sgx_status_t;

        pub fn u_fs_pwrite64_ocall(result: * mut ssize_t,
                                   errno: * mut c_int,
                                   fd: c_int,
                                   buf: * const c_void,
                                   count: size_t,
                                   offset: off64_t) -> sgx_status_t;

        pub fn u_fs_close_ocall(result: * mut c_int,
                                errno: * mut c_int,
                                fd: c_int) -> sgx_status_t;

        pub fn u_fs_ioctl_arg0_ocall(result: * mut c_int,
                                     errno: * mut c_int,
                                     fd: c_int,
                                     request: c_int) -> sgx_status_t;

        pub fn u_fs_ioctl_arg1_ocall(result: * mut c_int,
                                     errno: * mut c_int,
                                     fd: c_int,
                                     request: c_int,
                                     arg: * const c_int) -> sgx_status_t;

        pub fn u_fs_fcntl_arg1_ocall(result: * mut c_int,
                                     errno: * mut c_int,
                                     fd: c_int,
                                     cmd: c_int,
                                     arg: c_int) -> sgx_status_t;
    }

    pub unsafe fn read(fd: c_int, buf: * mut c_void, count: size_t) -> ssize_t {

        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let status = u_fs_read_ocall(&mut result as * mut ssize_t,
                                     &mut error as * mut c_int,
                                     fd,
                                     buf,
                                     count);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn pread64(fd: c_int, buf: * mut c_void, count: size_t, offset: off64_t) -> ssize_t {
        
        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let status = u_fs_pread64_ocall(&mut result as * mut ssize_t,
                                        &mut error as * mut c_int,
                                        fd,
                                        buf,
                                        count,
                                        offset);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn write(fd: c_int, buf: * const c_void, count: size_t) -> ssize_t {
        
        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let status = u_fs_write_ocall(&mut result as * mut ssize_t,
                                      &mut error as * mut c_int,
                                      fd,
                                      buf,
                                      count);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn pwrite64(fd: c_int, buf: * const c_void, count: size_t, offset: off64_t) -> ssize_t {
        
        let mut result: ssize_t = 0;
        let mut error: c_int = 0;
        let status = u_fs_pwrite64_ocall(&mut result as * mut ssize_t,
                                         &mut error as * mut c_int,
                                         fd,
                                         buf,
                                         count,
                                         offset);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn close(fd: c_int) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_close_ocall(&mut result as * mut c_int,
                                      &mut error as * mut c_int,
                                      fd);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn ioctl_arg0(fd: c_int, request: c_int) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_ioctl_arg0_ocall(&mut result as * mut c_int,
                                           &mut error as * mut c_int,
                                           fd,
                                           request);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn ioctl_arg1(fd: c_int, request: c_int, arg: * const c_int) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_ioctl_arg1_ocall(&mut result as * mut c_int,
                                           &mut error as * mut c_int,
                                           fd,
                                           request,
                                           arg);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }

    pub unsafe fn fcntl_arg1(fd: c_int, cmd: c_int, arg: c_int) -> c_int {

        let mut result: c_int = 0;
        let mut error: c_int = 0;
        let status = u_fs_fcntl_arg1_ocall(&mut result as * mut c_int,
                                           &mut error as * mut c_int,
                                           fd,
                                           cmd,
                                           arg);

        if status == sgx_status_t::SGX_SUCCESS {
            if result == -1 {
                io::set_errno(error);
            }
        } else {
            io::set_errno(ESGX);
            result = -1;
        }
        result
    }
}
