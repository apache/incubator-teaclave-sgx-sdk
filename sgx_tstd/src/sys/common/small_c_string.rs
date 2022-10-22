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

use crate::ffi::{CStr, CString};
use crate::mem::MaybeUninit;
use crate::path::Path;
use crate::slice;
use crate::{io, ptr};

const MAX_STACK_ALLOCATION: usize = 128;

const NUL_ERR: io::Error =
    io::const_io_error!(io::ErrorKind::InvalidInput, "file name contained an unexpected NUL byte");

#[inline]
pub fn run_path_with_cstr<T, F>(path: &Path, f: F) -> io::Result<T>
where
    F: FnOnce(&CStr) -> io::Result<T>,
{
    run_with_cstr(path.as_os_str().bytes(), f)
}

#[inline]
pub fn run_with_cstr<T, F>(bytes: &[u8], f: F) -> io::Result<T>
where
    F: FnOnce(&CStr) -> io::Result<T>,
{
    if bytes.len() >= MAX_STACK_ALLOCATION {
        return run_with_cstr_allocating(bytes, f);
    }

    let mut buf = MaybeUninit::<[u8; MAX_STACK_ALLOCATION]>::uninit();
    let buf_ptr = buf.as_mut_ptr() as *mut u8;

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf_ptr, bytes.len());
        buf_ptr.add(bytes.len()).write(0);
    }

    match CStr::from_bytes_with_nul(unsafe { slice::from_raw_parts(buf_ptr, bytes.len() + 1) }) {
        Ok(s) => f(s),
        Err(_) => Err(NUL_ERR),
    }
}

#[cold]
#[inline(never)]
fn run_with_cstr_allocating<T, F>(bytes: &[u8], f: F) -> io::Result<T>
where
    F: FnOnce(&CStr) -> io::Result<T>,
{
    match CString::new(bytes) {
        Ok(s) => f(&s),
        Err(_) => Err(NUL_ERR),
    }
}
