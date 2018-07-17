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
use sgx_trts::libc::{self, c_void};
use io::{self, Error};
use core::cmp;

pub struct Stdin(());
pub struct Stdout(());
pub struct Stderr(());

extern "C" {
    pub fn u_stdin_ocall(result: * mut usize, buf: * mut c_void, nbytes: usize) -> sgx_status_t;
    pub fn u_stdout_ocall(result: * mut usize, buf: * const c_void, nbytes: usize) -> sgx_status_t;
    pub fn u_stderr_ocall(result: * mut usize, buf: * const c_void, nbytes: usize) -> sgx_status_t;
}

fn max_len() -> usize {
    u32::max_value() as usize
}

impl Stdin {
    pub fn new() -> io::Result<Stdin> { Ok(Stdin(())) }

    pub fn read(&self, data: &mut [u8]) -> io::Result<usize> {

        let mut result: isize = 0;
        let status = unsafe {
            u_stdin_ocall(&mut result as * mut isize as * mut usize,
                          data.as_mut_ptr() as * mut c_void,
                          cmp::min(data.len(), max_len()))
        };
        if status != sgx_status_t::SGX_SUCCESS {
            return Err(Error::from_sgx_error(status));
        } else if result == -1 {
            return Err(Error::from_raw_os_error(libc::EIO));
        }

        Ok(result as usize)
    }
}

impl Stdout {
    pub fn new() -> io::Result<Stdout> { Ok(Stdout(())) }

    pub fn write(&self, data: &[u8]) -> io::Result<usize> {

        let mut result: isize = 0;
        let status = unsafe {
            u_stdout_ocall(&mut result as * mut isize as * mut usize,
                           data.as_ptr() as * const c_void,
                           cmp::min(data.len(), max_len()))
        };
        if status != sgx_status_t::SGX_SUCCESS {
            return Err(Error::from_sgx_error(status));
        } else if result == -1 {
            return Err(Error::from_raw_os_error(libc::EIO));
        }

        Ok(result as usize)
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub fn new() -> io::Result<Stderr> { Ok(Stderr(())) }

    pub fn write(&self, data: &[u8]) -> io::Result<usize> {

        let mut result: isize = 0;
        let status = unsafe {
            u_stderr_ocall(&mut result as * mut isize as * mut usize,
                           data.as_ptr() as * const c_void,
                           cmp::min(data.len(), max_len()))
        };
        if status != sgx_status_t::SGX_SUCCESS {
            return Err(Error::from_sgx_error(status));
        } else if result == -1 {
            return Err(Error::from_raw_os_error(libc::EIO));
        }

        Ok(result as usize)
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }
}

// FIXME: right now this raw stderr handle is used in a few places because
//        std::io::stderr_raw isn't exposed, but once that's exposed this impl
//        should go away
impl io::Write for Stderr {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        Stderr::write(self, data)
    }

    fn flush(&mut self) -> io::Result<()> {
        Stderr::flush(self)
    }
}

pub fn is_ebadf(err: &io::Error) -> bool {
    err.raw_os_error() == Some(libc::EBADF as i32)
}

pub const STDIN_BUF_SIZE: usize = ::sys_common::io::DEFAULT_BUF_SIZE;

pub fn stderr_prints_nothing() -> bool {
    false
}
