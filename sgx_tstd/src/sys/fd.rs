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

#[cfg(feature = "unit_test")]
mod tests;

use crate::cmp;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, Read};
use crate::mem::MaybeUninit;
use crate::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
use crate::sys::cvt_ocall;
use crate::sys_common::{AsInner, FromInner, IntoInner};

use sgx_oc::c_int;

#[derive(Debug)]
pub struct FileDesc(OwnedFd);

const fn max_iov() -> usize {
    sgx_oc::UIO_MAXIOV as usize
}

impl FileDesc {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = cvt_ocall(unsafe { libc::read(self.as_raw_fd(), buf) })?;
        Ok(ret)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let len = cmp::min(bufs.len(), max_iov());
        let vbufs: Vec<&mut [u8]> = bufs[..len].iter_mut().map(|msl| &mut **msl).collect();
        let ret = cvt_ocall(unsafe { libc::readv(self.as_raw_fd(), vbufs) })?;
        Ok(ret)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        true
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        cvt_ocall(unsafe { libc::pread64(self.as_raw_fd(), buf, offset as _) })
    }

    pub fn read_buf(&self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let ret = cvt_ocall(unsafe {
            libc::read(self.as_raw_fd(), MaybeUninit::slice_assume_init_mut(cursor.as_mut()))
        })?;

        // Safety: `ret` bytes were written to the initialized portion of the buffer
        unsafe {
            cursor.advance(ret);
        }
        Ok(())
    }

    pub fn read_vectored_at(&self, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
        let len = cmp::min(bufs.len(), max_iov());
        let vbufs: Vec<&mut [u8]> = bufs[..len].iter_mut().map(|msl| &mut **msl).collect();
        let ret = cvt_ocall(unsafe { libc::preadv64(self.as_raw_fd(), vbufs, offset as _) })?;
        Ok(ret)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = cvt_ocall(unsafe { libc::write(self.as_raw_fd(), buf) })?;
        Ok(ret)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let vbufs: Vec<&[u8]> = bufs[..cmp::min(bufs.len(), max_iov())]
            .iter()
            .map(|msl| &**msl)
            .collect();
        let ret = cvt_ocall(unsafe { libc::writev(self.as_raw_fd(), vbufs) })?;
        Ok(ret)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        true
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        cvt_ocall(unsafe { libc::pwrite64(self.as_raw_fd(), buf, offset as i64) })
    }

    pub fn write_vectored_at(&self, bufs: &[IoSlice<'_>], offset: u64) -> io::Result<usize> {
        let vbufs: Vec<&[u8]> = bufs[..cmp::min(bufs.len(), max_iov())]
            .iter()
            .map(|msl| &**msl)
            .collect();
        let ret = cvt_ocall(unsafe { libc::pwritev64(self.as_raw_fd(), vbufs, offset as _) })?;
        Ok(ret)
    }

    pub fn get_cloexec(&self) -> io::Result<bool> {
        unsafe { Ok((cvt_ocall(libc::fcntl_arg0(self.as_raw_fd(), libc::F_GETFD))? & libc::FD_CLOEXEC) != 0) }
    }

    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            let previous = cvt_ocall(libc::fcntl_arg0(self.as_raw_fd(), libc::F_GETFD))?;
            let new = previous | libc::FD_CLOEXEC;
            if new != previous {
                cvt_ocall(libc::fcntl_arg1(self.as_raw_fd(), libc::F_SETFD, new))?;
            }
            Ok(())
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        unsafe {
            let mut v = nonblocking as c_int;
            cvt_ocall(libc::ioctl_arg1(
                self.as_raw_fd(),
                libc::FIONBIO as u64,
                &mut v,
            ))?;
            Ok(())
        }
    }

    #[inline]
    pub fn duplicate(&self) -> io::Result<FileDesc> {
        Ok(Self(self.0.try_clone()?))
    }
}

impl<'a> Read for &'a FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }
}

impl AsInner<OwnedFd> for FileDesc {
    #[inline]
    fn as_inner(&self) -> &OwnedFd {
        &self.0
    }
}

impl IntoInner<OwnedFd> for FileDesc {
    fn into_inner(self) -> OwnedFd {
        self.0
    }
}

impl FromInner<OwnedFd> for FileDesc {
    fn from_inner(owned_fd: OwnedFd) -> Self {
        Self(owned_fd)
    }
}

impl AsFd for FileDesc {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for FileDesc {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for FileDesc {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(FromRawFd::from_raw_fd(raw_fd))
    }
}

mod libc {
    pub use sgx_oc::ocall::{
        fcntl_arg0, fcntl_arg1, ioctl_arg1, pread64, preadv64, pwrite64, pwritev64, read, readv, write, writev,
    };
    pub use sgx_oc::*;
}
