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
use core::cmp;
use core::mem;
use sgx_trts::trts::is_within_enclave;

pub unsafe fn read(fd: c_int, buf: &mut [u8]) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let bufsz = buf.len();
    if bufsz == 0 {
        return Ok(0);
    }

    check_trusted_enclave_buffer(buf)?;
    let mut host_buf = HostBuffer::alloc_zeroed(bufsz)?;

    let status = u_read_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_mut_ptr() as _,
        host_buf.len(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nread = result as usize;
    ensure!(nread <= bufsz, ecust!("Malformed return size"));
    host_buf.to_enclave_slice(&mut buf[..nread])
}

pub unsafe fn pread64(fd: c_int, buf: &mut [u8], offset: off64_t) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let bufsz = buf.len();
    if bufsz == 0 {
        return Ok(0);
    }

    check_trusted_enclave_buffer(buf)?;
    let mut host_buf = HostBuffer::alloc_zeroed(bufsz)?;

    let status = u_pread64_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_mut_ptr() as _,
        host_buf.len(),
        offset,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nread = result as usize;
    ensure!(nread <= bufsz, ecust!("Malformed return size"));
    host_buf.to_enclave_slice(&mut buf[..nread])
}

pub unsafe fn readv(fd: c_int, mut iov: Vec<&mut [u8]>) -> OCallResult<size_t> {
    if iov.capacity() != 0 {
        ensure!(
            is_within_enclave(
                iov.as_ptr() as *const u8,
                iov.capacity() * mem::size_of::<&[u8]>()
            ),
            ecust!("Buffer is not strictly inside enclave")
        );
    }

    let mut total_size: usize = 0;
    for s in iov.iter() {
        check_enclave_buffer(*s)?;
        total_size = total_size
            .checked_add(s.len())
            .ok_or_else(|| OCallError::from_custom_error("Overflow"))?;
    }

    let bufsz = total_size;
    if total_size == 0 {
        return Ok(0);
    }

    let mut host_buf = HostBuffer::alloc_zeroed(bufsz)?;
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let status = u_read_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_mut_ptr() as *mut c_void,
        host_buf.len(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let u_read = result as usize;
    ensure!(u_read <= bufsz, ecust!("Malformed result"));

    if u_read == 0 {
        return Ok(0);
    }

    // Buffers are processed in array order. This means that readv() com‐
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

pub unsafe fn preadv64(fd: c_int, mut iov: Vec<&mut [u8]>, offset: off64_t) -> OCallResult<size_t> {
    if iov.capacity() != 0 {
        ensure!(
            is_within_enclave(
                iov.as_ptr() as *const u8,
                iov.capacity() * mem::size_of::<&[u8]>()
            ),
            ecust!("Buffer is not strictly inside enclave")
        );
    }

    let mut total_size: usize = 0;
    for s in iov.iter() {
        check_enclave_buffer(*s)?;
        total_size = total_size
            .checked_add(s.len())
            .ok_or_else(|| OCallError::from_custom_error("Overflow"))?;
    }

    let bufsz = total_size;
    if total_size == 0 {
        return Ok(0);
    }

    let mut host_buf = HostBuffer::alloc_zeroed(bufsz)?;
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let status = u_pread64_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_mut_ptr() as _,
        host_buf.len(),
        offset,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let u_read = result as usize;
    ensure!(u_read <= bufsz, ecust!("Malformed result"));

    if u_read == 0 {
        return Ok(0);
    }

    // Buffers are processed in array order. This means that preadv() com‐
    // pletely fills iov[0] before proceeding to iov[1], and so on.  (If
    // there is insufficient data, then not all buffers pointed to by iov
    // may be filled.)  Similarly, pwritev() writes out the entire contents
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

pub unsafe fn write(fd: c_int, buf: &[u8]) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let bufsz = buf.len();
    if bufsz == 0 {
        return Ok(0);
    }

    let host_buf = HostBuffer::from_enclave_slice(buf)?;

    let status = u_write_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_ptr() as _,
        host_buf.len(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nwritten = result as usize;
    ensure!(nwritten <= bufsz, ecust!("Malformed return size"));
    Ok(nwritten)
}

pub unsafe fn pwrite64(fd: c_int, buf: &[u8], offset: off64_t) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let bufsz = buf.len();
    if bufsz == 0 {
        return Ok(0);
    }

    let host_buf = HostBuffer::from_enclave_slice(buf)?;

    let status = u_pwrite64_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_ptr() as _,
        host_buf.len(),
        offset,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nwritten = result as usize;
    ensure!(nwritten <= bufsz, ecust!("Malformed return size"));
    Ok(nwritten)
}

pub unsafe fn writev(fd: c_int, iov: Vec<&[u8]>) -> OCallResult<size_t> {
    if iov.capacity() != 0 {
        ensure!(
            is_within_enclave(
                iov.as_ptr() as *const u8,
                iov.capacity() * mem::size_of::<&[u8]>()
            ),
            ecust!("Buffer is not strictly inside enclave")
        );
    }

    let mut total_size: usize = 0;
    for s in iov.iter() {
        check_enclave_buffer(*s)?;
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

    let host_buf = HostBuffer::from_enclave_slice(&buffer)?;
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let status = u_write_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_ptr() as _,
        host_buf.len(),
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nwritten = result as size_t;
    ensure!(nwritten <= bufsz, ecust!("Malformed result"));
    Ok(nwritten)
}

pub unsafe fn pwritev64(fd: c_int, iov: Vec<&[u8]>, offset: off64_t) -> OCallResult<size_t> {
    if iov.capacity() != 0 {
        ensure!(
            is_within_enclave(
                iov.as_ptr() as *const u8,
                iov.capacity() * mem::size_of::<&[u8]>()
            ),
            ecust!("Buffer is not strictly inside enclave")
        );
    }

    let mut total_size: usize = 0;
    for s in iov.iter() {
        check_enclave_buffer(*s)?;
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

    let host_buf = HostBuffer::from_enclave_slice(&buffer)?;
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let status = u_pwrite64_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd,
        host_buf.as_ptr() as _,
        host_buf.len(),
        offset,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nwritten = result as size_t;
    ensure!(nwritten <= bufsz, ecust!("Malformed result"));
    Ok(nwritten)
}

pub unsafe fn sendfile(
    out_fd: c_int,
    in_fd: c_int,
    offset: Option<&mut off_t>,
    count: size_t,
) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let offset = if let Some(off) = offset {
        off as *mut off_t
    } else {
        ptr::null_mut()
    };

    let status = u_sendfile_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        out_fd,
        in_fd,
        offset,
        count,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nsend = result as usize;
    ensure!(nsend <= count, ecust!("Malformed return size"));
    Ok(nsend)
}

pub unsafe fn copy_file_range(
    fd_in: c_int,
    off_in: Option<&mut off64_t>,
    fd_out: c_int,
    off_out: Option<&mut off64_t>,
    len: size_t,
    flags: c_uint,
) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let off_in = if let Some(off) = off_in {
        off as *mut off64_t
    } else {
        ptr::null_mut()
    };

    let off_out = if let Some(off) = off_out {
        off as *mut off64_t
    } else {
        ptr::null_mut()
    };

    let status = u_copy_file_range_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd_in,
        off_in,
        fd_out,
        off_out,
        len,
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let ncopied = result as usize;
    ensure!(ncopied <= len, ecust!("Malformed return size"));
    Ok(ncopied)
}

pub unsafe fn splice(
    fd_in: c_int,
    off_in: Option<&mut off64_t>,
    fd_out: c_int,
    off_out: Option<&mut off64_t>,
    len: size_t,
    flags: c_uint,
) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;

    let off_in = if let Some(off) = off_in {
        off as *mut off64_t
    } else {
        ptr::null_mut()
    };

    let off_out = if let Some(off) = off_out {
        off as *mut off64_t
    } else {
        ptr::null_mut()
    };

    let status = u_splice_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        fd_in,
        off_in,
        fd_out,
        off_out,
        len,
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nspliced = result as usize;
    ensure!(nspliced <= len, ecust!("Malformed return size"));
    Ok(nspliced)
}

pub unsafe fn fcntl(fd: c_int, cmd: c_int, arg: c_long) -> OCallResult<c_int> {
    match cmd {
        F_GETFD | F_GETFL | F_GETOWN | F_GETLEASE | F_GETPIPE_SZ | F_GET_SEALS => {
            fcntl_arg0(fd, cmd)
        }
        F_DUPFD | F_DUPFD_CLOEXEC | F_SETFD | F_SETFL | F_SETOWN | F_SETLEASE | F_NOTIFY
        | F_SETPIPE_SZ | F_ADD_SEALS => fcntl_arg1(fd, cmd, arg as c_int),
        _ => {
            bail!(eos!(EINVAL));
        }
    }
}

pub unsafe fn fcntl_arg0(fd: c_int, cmd: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_fcntl_arg0_ocall(&mut result as *mut c_int, &mut error as *mut c_int, fd, cmd);

    ensure!(status.is_success(), esgx!(status));
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

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn ioctl_arg0(fd: c_int, request: c_ulong) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_ioctl_arg0_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fd,
        request,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn ioctl_arg1(fd: c_int, request: c_ulong, arg: &mut c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_ioctl_arg1_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        fd,
        request,
        arg,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn close(fd: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_close_ocall(&mut result as *mut c_int, &mut error as *mut c_int, fd);

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn isatty(fd: c_int) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_isatty_ocall(&mut result as *mut c_int, &mut error as *mut c_int, fd);

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 1, eos!(error));
    Ok(())
}

pub unsafe fn dup(oldfd: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_dup_ocall(&mut result as *mut c_int, &mut error as *mut c_int, oldfd);

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    Ok(result)
}

pub unsafe fn eventfd(initval: c_uint, flags: c_int) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_eventfd_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        initval,
        flags,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    Ok(result)
}
