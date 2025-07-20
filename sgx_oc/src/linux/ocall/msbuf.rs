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

use super::*;
use core::cmp;

const MSBUF_OCALL_SIZE: usize = 0x40;

pub unsafe fn read_hostbuf(mut host_buf: &[u8], mut ecl_buf: &mut [u8]) -> OCallResult<size_t> {
    let bufsz = host_buf.len();
    let enclsz = ecl_buf.len();

    check_host_buffer(host_buf)?;

    ensure!(bufsz == enclsz, eos!(EINVAL));
    if bufsz == 0 {
        return Ok(0);
    }

    let remain_size = OcBuffer::remain_size();
    ensure!(remain_size > MSBUF_OCALL_SIZE, eos!(ENOMEM));
    let remain_size = remain_size - MSBUF_OCALL_SIZE;

    while !host_buf.is_empty() {
        let copy_len = cmp::min(host_buf.len(), remain_size);
        let n = read_hostbuf_each(&host_buf[..copy_len], &mut ecl_buf[..copy_len])?;

        host_buf = &host_buf[n..];
        ecl_buf = &mut ecl_buf[n..];
    }

    ensure!(host_buf.is_empty(), ecust!("failed to fill whole buffer"));
    Ok(bufsz)
}

pub unsafe fn write_hostbuf(mut host_buf: &mut [u8], mut encl_buf: &[u8]) -> OCallResult<size_t> {
    let bufsz = host_buf.len();
    let enclsz = encl_buf.len();

    check_host_buffer(host_buf)?;

    ensure!(bufsz == enclsz, eos!(EINVAL));
    if bufsz == 0 {
        return Ok(0);
    }

    let remain_size = OcBuffer::remain_size();
    ensure!(remain_size > MSBUF_OCALL_SIZE, eos!(ENOMEM));
    let remain_size = remain_size - MSBUF_OCALL_SIZE;

    while !host_buf.is_empty() {
        let copy_len = cmp::min(host_buf.len(), remain_size);
        let n = write_hostbuf_each(&mut host_buf[..copy_len], &encl_buf[..copy_len])?;

        host_buf = &mut host_buf[n..];
        encl_buf = &encl_buf[n..];
    }

    ensure!(host_buf.is_empty(), ecust!("failed to write whole buffer"));
    Ok(bufsz)
}

unsafe fn read_hostbuf_each(host_buf: &[u8], encl_buf: &mut [u8]) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let bufsz = host_buf.len();
    let enclsz = encl_buf.len();

    assert!(bufsz == enclsz);
    if bufsz == 0 {
        return Ok(0);
    }

    let status = u_read_hostbuf_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        host_buf.as_ptr() as _,
        encl_buf.as_mut_ptr() as _,
        bufsz,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nread = result as usize;
    ensure!(nread == bufsz, ecust!("Malformed return size"));
    Ok(nread)
}

unsafe fn write_hostbuf_each(host_buf: &mut [u8], encl_buf: &[u8]) -> OCallResult<size_t> {
    let mut result: ssize_t = 0;
    let mut error: c_int = 0;
    let bufsz = host_buf.len();
    let enclsz = encl_buf.len();

    assert!(bufsz == enclsz);
    if bufsz == 0 {
        return Ok(0);
    }

    let status = u_write_hostbuf_ocall(
        &mut result as *mut ssize_t,
        &mut error as *mut c_int,
        host_buf.as_mut_ptr() as _,
        encl_buf.as_ptr() as _,
        bufsz,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));

    let nwrite = result as usize;
    ensure!(nwrite == bufsz, ecust!("Malformed return size"));
    Ok(nwrite)
}
