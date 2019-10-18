// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

use sgx_trts::error as trts_error;
use sgx_types::metadata;
use crate::os::unix::prelude::*;
use crate::error::Error as StdError;
use crate::ffi::{CString, CStr, OsString, OsStr};
use crate::path::{self, PathBuf};
use crate::sync::SgxThreadMutex;
use crate::sys::{cvt, fd};
use crate::memchr;
use crate::io;
use core::marker::PhantomData;
use core::fmt;
use core::iter;
use core::ptr;
use alloc_crate::slice;
use alloc_crate::string::String;
use alloc_crate::str;
use alloc_crate::vec::{self, Vec};
use crate::mem;
use libc::{c_int, c_char, c_void};

const TMPBUF_SZ: usize = 128;
static ENV_LOCK: SgxThreadMutex = SgxThreadMutex::new();

pub fn errno() -> i32 {
    trts_error::errno()
}

pub fn set_errno(e: i32) {
    trts_error::set_errno(e)
}

pub fn error_string(error: i32) -> String {
    let mut buf = [0_i8; TMPBUF_SZ];
    unsafe {
        if trts_error::error_string(error, &mut buf) < 0 {
            panic!("strerror_r failure");
        }

        let p = buf.as_ptr() as * const _;
        str::from_utf8(CStr::from_ptr(p).to_bytes()).unwrap().to_owned()
    }
}

pub struct SplitPaths<'a> {
    iter: iter::Map<slice::Split<'a, u8, fn(&u8) -> bool>,
                    fn(&'a [u8]) -> PathBuf>,
}

pub fn split_paths(unparsed: &OsStr) -> SplitPaths<'_> {
    fn bytes_to_path(b: &[u8]) -> PathBuf {
        PathBuf::from(<OsStr as OsStrExt>::from_bytes(b))
    }
    fn is_colon(b: &u8) -> bool { *b == b':' }
    let unparsed = unparsed.as_bytes();
    SplitPaths {
        iter: unparsed.split(is_colon as fn(&u8) -> bool)
                      .map(bytes_to_path as fn(&[u8]) -> PathBuf)
    }
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

#[derive(Debug)]
pub struct JoinPathsError;

pub fn join_paths<I, T>(paths: I) -> Result<OsString, JoinPathsError>
    where I: Iterator<Item=T>, T: AsRef<OsStr>
{
    let mut joined = Vec::new();
    let sep = b':';

    for (i, path) in paths.enumerate() {
        let path = path.as_ref().as_bytes();
        if i > 0 { joined.push(sep) }
        if path.contains(&sep) {
            return Err(JoinPathsError)
        }
        joined.extend_from_slice(path);
    }
    Ok(OsStringExt::from_vec(joined))
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "path segment contains separator `:`".fmt(f)
    }
}

impl StdError for JoinPathsError {
    fn description(&self) -> &str { "failed to join paths" }
}

pub fn current_exe() -> io::Result<PathBuf> {
    match crate::fs::read_link("/proc/self/exe") {
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "no /proc/self/exe available. Is /proc mounted?"
            ))
        },
        other => other,
    }
}

pub struct Env {
    iter: vec::IntoIter<(OsString, OsString)>,
    _dont_send_or_sync_me: PhantomData<*mut ()>,
}

impl Iterator for Env {
    type Item = (OsString, OsString);
    fn next(&mut self) -> Option<(OsString, OsString)> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

pub unsafe fn environ() -> *const *const libc::c_char {
    libc::environ()
}

/// Returns a vector of (variable, value) byte-vector pairs for all the
/// environment variables of the current process.
pub fn env() -> Env {
    unsafe {
        ENV_LOCK.lock();
        let mut environ = environ();
        let mut result = Vec::new();
        while environ != ptr::null() && *environ != ptr::null() {
            if let Some(key_value) = parse(CStr::from_ptr(*environ).to_bytes()) {
                result.push(key_value);
            }
            environ = environ.offset(1);
        }
        let ret = Env {
            iter: result.into_iter(),
            _dont_send_or_sync_me: PhantomData,
        };
        ENV_LOCK.unlock();
        return ret
    }

    fn parse(input: &[u8]) -> Option<(OsString, OsString)> {
        // Strategy (copied from glibc): Variable name and value are separated
        // by an ASCII equals sign '='. Since a variable name must not be
        // empty, allow variable names starting with an equals sign. Skip all
        // malformed lines.
        if input.is_empty() {
            return None;
        }
        let pos = memchr::memchr(b'=', &input[1..]).map(|p| p + 1);
        pos.map(|p| (
            OsStringExt::from_vec(input[..p].to_vec()),
            OsStringExt::from_vec(input[p+1..].to_vec()),
        ))
    }
}

pub fn getenv(k: &OsStr) -> io::Result<Option<OsString>> {
    // environment variables with a nul byte can't be set, so their value is
    // always None as well
    let k = CString::new(k.as_bytes())?;
    unsafe {
        ENV_LOCK.lock();
        let s = libc::getenv(k.as_ptr()) as *const libc::c_char;
        let ret = if s.is_null() {
            None
        } else {
            Some(OsStringExt::from_vec(CStr::from_ptr(s).to_bytes().to_vec()))
        };
        ENV_LOCK.unlock();
        Ok(ret)
    }
}

pub fn setenv(k: &OsStr, v: &OsStr) -> io::Result<()> {
    let k = CString::new(k.as_bytes())?;
    let v = CString::new(v.as_bytes())?;

    unsafe {
        ENV_LOCK.lock();
        let ret = cvt(libc::setenv(k.as_ptr(), v.as_ptr(), 1)).map(|_| ());
        ENV_LOCK.unlock();
        ret
    }
}

pub fn unsetenv(n: &OsStr) -> io::Result<()> {
    let nbuf = CString::new(n.as_bytes())?;

    unsafe {
        ENV_LOCK.lock();
        let ret = cvt(libc::unsetenv(nbuf.as_ptr())).map(|_| ());
        ENV_LOCK.unlock();
        ret
    }
}

pub fn page_size() -> usize {
    metadata::SE_PAGE_SIZE
}

pub fn temp_dir() -> PathBuf {
    crate::env::var_os("TMPDIR").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("/tmp")
    })
}

pub fn getcwd() -> io::Result<PathBuf> {
    let mut buf = Vec::with_capacity(512);
    loop {
        unsafe {
            let ptr = buf.as_mut_ptr() as *mut libc::c_char;
            if !libc::getcwd(ptr, buf.capacity()).is_null() {
                let len = CStr::from_ptr(buf.as_ptr() as *const libc::c_char).to_bytes().len();
                buf.set_len(len);
                buf.shrink_to_fit();
                return Ok(PathBuf::from(OsString::from_vec(buf)));
            } else {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ERANGE) {
                    return Err(error);
                }
            }

            // Trigger the internal buffer resizing logic of `Vec` by requiring
            // more space than the current capacity.
            let cap = buf.capacity();
            buf.set_len(cap);
            buf.reserve(1);
        }
    }
}

pub fn chdir(p: &path::Path) -> io::Result<()> {
    let p: &OsStr = p.as_ref();
    let p = CString::new(p.as_bytes())?;
    unsafe {
        match libc::chdir(p.as_ptr()) == (0 as c_int) {
            true => Ok(()),
            false => Err(io::Error::last_os_error()),
        }
    }
}

pub fn home_dir() -> Option<PathBuf> {
    return crate::env::var_os("HOME").or_else(|| unsafe {
        fallback()
    }).map(PathBuf::from);
    unsafe fn fallback() -> Option<OsString> {
        let amt = match libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) {
            n if n < 0 => 512 as usize,
            n => n as usize,
        };
        let mut buf = Vec::with_capacity(amt);
        let mut passwd: libc::passwd = mem::zeroed();
        let mut result = ptr::null_mut();
        match libc::getpwuid_r(libc::getuid(), &mut passwd, buf.as_mut_ptr(),
                               buf.capacity(), &mut result) {
            0 if !result.is_null() => {
                let ptr = passwd.pw_dir as *const _;
                let bytes = CStr::from_ptr(ptr).to_bytes().to_vec();
                Some(OsStringExt::from_vec(bytes))
            },
            _ => None,
        }
    }
}

mod libc {
    pub use sgx_trts::libc::*;
    pub use sgx_trts::libc::ocall::{environ, getenv, setenv, unsetenv, getcwd, chdir, sysconf, getuid, getpwuid_r};
}