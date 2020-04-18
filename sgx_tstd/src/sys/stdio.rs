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
use crate::io::{self, IoSlice, IoSliceMut};
use sgx_trts::libc;
use crate::sys::fd::FileDesc;
use crate::mem::ManuallyDrop;

pub struct Stdin(());
pub struct Stdout(());
pub struct Stderr(());

impl Stdin {
    pub fn new() -> io::Result<Stdin> {
        Ok(Stdin(()))
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDIN_FILENO)).read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDIN_FILENO)).read_vectored(bufs)
    }
}

impl Stdout {
    pub fn new() -> io::Result<Stdout> {
        Ok(Stdout(()))
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDOUT_FILENO)).write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDOUT_FILENO)).write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub fn new() -> io::Result<Stderr> {
        Ok(Stderr(()))
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDERR_FILENO)).write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        ManuallyDrop::new(FileDesc::new(libc::STDERR_FILENO)).write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn is_ebadf(err: &io::Error) -> bool {
    err.raw_os_error() == Some(libc::EBADF as i32)
}

pub const STDIN_BUF_SIZE: usize = crate::sys_common::io::DEFAULT_BUF_SIZE;

pub fn panic_output() -> Option<impl io::Write> {
    Stderr::new().ok()
}