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

use sgx_trts::libc::{c_int, c_void, ssize_t};
use core::cmp;
use core::mem;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::io::{self, Initializer, IoSlice, IoSliceMut, Read};
use crate::sys::cvt;
use crate::sys_common::AsInner;

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
        FileDesc { fd }
    }

    pub fn raw(&self) -> c_int {
        self.fd
    }

    /// Extracts the actual file descriptor without closing it.
    pub fn into_raw(self) -> c_int {
        let fd = self.fd;
        mem::forget(self);
        fd
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::read(self.fd, buf.as_mut_ptr() as *mut c_void, cmp::min(buf.len(), max_len()))
        })?;
        Ok(ret as usize)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::readv(
                self.fd,
                bufs.as_ptr() as *const libc::iovec,
                cmp::min(bufs.len(), c_int::max_value() as usize) as c_int,
            )
        })?;
        Ok(ret as usize)
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        unsafe fn cvt_pread64(
            fd: c_int,
            buf: *mut c_void,
            count: usize,
            offset: i64,
        ) -> io::Result<isize> {
            use libc::pread64;
            cvt(pread64(fd, buf, count, offset))
        }

        unsafe {
            cvt_pread64(
                self.fd,
                buf.as_mut_ptr() as *mut c_void,
                cmp::min(buf.len(), max_len()),
                offset as i64,
            )
            .map(|n| n as usize)
        }
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::write(self.fd, buf.as_ptr() as *const c_void, cmp::min(buf.len(), max_len()))
        })?;
        Ok(ret as usize)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::writev(
                self.fd,
                bufs.as_ptr() as *const libc::iovec,
                cmp::min(bufs.len(), c_int::max_value() as usize) as c_int,
            )
        })?;
        Ok(ret as usize)
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        unsafe fn cvt_pwrite64(
            fd: c_int,
            buf: *const c_void,
            count: usize,
            offset: i64,
        ) -> io::Result<isize> {
            use libc::pwrite64;
            cvt(pwrite64(fd, buf, count, offset))
        }

        unsafe {
            cvt_pwrite64(
                self.fd,
                buf.as_ptr() as *const c_void,
                cmp::min(buf.len(), max_len()),
                offset as i64,
            )
            .map(|n| n as usize)
        }
    }

    pub fn get_cloexec(&self) -> io::Result<bool> {
        unsafe { Ok((cvt(libc::fcntl_arg0(self.fd, libc::F_GETFD))? & libc::FD_CLOEXEC) != 0) }
    }

    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            let previous = cvt(libc::fcntl_arg0(self.fd, libc::F_GETFD))?;
            let new = previous | libc::FD_CLOEXEC;
            if new != previous {
                cvt(libc::fcntl_arg1(self.fd, libc::F_SETFD, new))?;
            }
            Ok(())
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        unsafe {
            let mut v = nonblocking as c_int;
            cvt(libc::ioctl_arg1(self.fd, libc::FIONBIO, &mut v as *mut c_int))?;
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
        use libc::F_DUPFD_CLOEXEC;

        let make_filedesc = |fd| {
            let fd = FileDesc::new(fd);
            fd.set_cloexec()?;
            Ok(fd)
        };
        static TRY_CLOEXEC: AtomicBool = AtomicBool::new(!cfg!(target_os = "android"));
        let fd = self.raw();
        if TRY_CLOEXEC.load(Ordering::Relaxed) {
            match cvt(unsafe { libc::fcntl_arg1(fd, F_DUPFD_CLOEXEC, 0) }) {
                // We *still* call the `set_cloexec` method as apparently some
                // linux kernel at some point stopped setting CLOEXEC even
                // though it reported doing so on F_DUPFD_CLOEXEC.
                Ok(fd) => {
                    return Ok(if cfg!(target_os = "linux") {
                        make_filedesc(fd)?
                    } else {
                        FileDesc::new(fd)
                    });
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

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

impl AsInner<c_int> for FileDesc {
    fn as_inner(&self) -> &c_int {
        &self.fd
    }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        // Note that errors are ignored when closing a file descriptor. The
        // reason for this is that if an error occurs we don't actually know if
        // the file descriptor was closed or not, and if we retried (for
        // something like EINTR), we might close another valid file descriptor
        // opened after we closed ours.
        let _ = unsafe { libc::close(self.fd) };
    }
}

mod libc {
    pub use sgx_trts::libc::*;
    pub use sgx_trts::libc::ocall::{read, pread64, write, pwrite64, readv, writev,
                                    fcntl_arg0, fcntl_arg1, ioctl_arg0, ioctl_arg1,
                                    close};
}
