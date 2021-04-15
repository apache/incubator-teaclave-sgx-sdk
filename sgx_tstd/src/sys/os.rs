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

use crate::error::Error as StdError;
use crate::ffi::{CStr, CString, OsStr, OsString};
use crate::io;
use crate::memchr;
use crate::os::unix::prelude::*;
use crate::path::{self, PathBuf};
use crate::sync::SgxThreadMutex;
use crate::sys::cvt_ocall;
use alloc_crate::slice;
use alloc_crate::str;
use alloc_crate::string::String;
use alloc_crate::vec::{self, Vec};
use core::fmt;
use core::iter;
use core::marker::PhantomData;
use sgx_trts::error as trts_error;
use sgx_types::metadata;

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

        let p = buf.as_ptr() as *const _;
        str::from_utf8(CStr::from_ptr(p).to_bytes())
            .unwrap()
            .to_owned()
    }
}

pub struct SplitPaths<'a> {
    iter: iter::Map<slice::Split<'a, u8, fn(&u8) -> bool>, fn(&'a [u8]) -> PathBuf>,
}

pub fn split_paths(unparsed: &OsStr) -> SplitPaths<'_> {
    fn bytes_to_path(b: &[u8]) -> PathBuf {
        PathBuf::from(<OsStr as OsStrExt>::from_bytes(b))
    }
    fn is_colon(b: &u8) -> bool {
        *b == b':'
    }
    let unparsed = unparsed.as_bytes();
    SplitPaths {
        iter: unparsed
            .split(is_colon as fn(&u8) -> bool)
            .map(bytes_to_path as fn(&[u8]) -> PathBuf),
    }
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Debug)]
pub struct JoinPathsError;

pub fn join_paths<I, T>(paths: I) -> Result<OsString, JoinPathsError>
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    let mut joined = Vec::new();
    let sep = b':';

    for (i, path) in paths.enumerate() {
        let path = path.as_ref().as_bytes();
        if i > 0 {
            joined.push(sep)
        }
        if path.contains(&sep) {
            return Err(JoinPathsError);
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
    fn description(&self) -> &str {
        "failed to join paths"
    }
}

pub fn current_exe() -> io::Result<PathBuf> {
    match crate::fs::read_link("/proc/self/exe") {
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Err(io::Error::new(
            io::ErrorKind::Other,
            "no /proc/self/exe available. Is /proc mounted?",
        )),
        other => other,
    }
}

pub struct Env {
    iter: vec::IntoIter<(OsString, OsString)>,
    _dont_send_or_sync_me: PhantomData<*mut ()>,
}

impl Iterator for Env {
    type Item = (OsString, OsString);
    fn next(&mut self) -> Option<(OsString, OsString)> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/*
pub unsafe fn environ() -> *const *const libc::c_char {
    libc::environ()
}
*/

/// Returns a vector of (variable, value) byte-vector pairs for all the
/// environment variables of the current process.
pub fn env() -> Env {
    unsafe {
        ENV_LOCK.lock();
        /*
        let mut environ = environ();
        let mut result = Vec::new();
        if !environ.is_null() {
            while !(*environ).is_null() {
                if let Some(key_value) = parse(CStr::from_ptr(*environ).to_bytes()) {
                    result.push(key_value);
                }
                environ = environ.add(1);
            }
        }
        */
        let result = Vec::new();
        let ret = Env {
            iter: result.into_iter(),
            _dont_send_or_sync_me: PhantomData,
        };
        ENV_LOCK.unlock();
        return ret;
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
        pos.map(|p| {
            (
                OsStringExt::from_vec(input[..p].to_vec()),
                OsStringExt::from_vec(input[p + 1..].to_vec()),
            )
        })
    }
}

pub fn getenv(k: &OsStr) -> io::Result<Option<OsString>> {
    // environment variables with a nul byte can't be set, so their value is
    // always None as well
    let k = CString::new(k.as_bytes())?;

    unsafe {
        ENV_LOCK.lock();
        let result_opt = cvt_ocall(libc::getenv(&k));
        let ret = result_opt.map(|opt| opt.map(|v| OsString::from_vec(v)));
        ENV_LOCK.unlock();
        ret
    }
}

pub fn setenv(_k: &OsStr, _v: &OsStr) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "To be implemented completely within trusted part",
    ))
    /*
    let k = CString::new(k.as_bytes())?;
    let v = CString::new(v.as_bytes())?;

    unsafe {
        ENV_LOCK.lock();
        let ret = cvt_ocall(libc::setenv(k.as_ptr(), v.as_ptr(), 1)).map(drop);
        ENV_LOCK.unlock();
        ret
    }
    */
}

pub fn unsetenv(_n: &OsStr) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "To be implemented completely within trusted part",
    ))
    /*
    let nbuf = CString::new(n.as_bytes())?;

    unsafe {
        ENV_LOCK.lock();
        let ret = cvt_ocall(libc::unsetenv(nbuf.as_ptr())).map(drop);
        ENV_LOCK.unlock();
        ret
    }
    */
}

pub fn page_size() -> usize {
    metadata::SE_PAGE_SIZE
}

pub fn temp_dir() -> PathBuf {
    crate::env::var_os("TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

pub fn getcwd() -> io::Result<PathBuf> {
    let v = cvt_ocall(unsafe { libc::getcwd() })?;
    return Ok(PathBuf::from(OsString::from_vec(v)));
}

pub fn chdir(p: &path::Path) -> io::Result<()> {
    let p: &OsStr = p.as_ref();
    let p = CString::new(p.as_bytes())?;
    unsafe { cvt_ocall(libc::chdir(&p)) }
}

// No fallback logic here for security reasons.
pub fn home_dir() -> Option<PathBuf> {
    return crate::env::var_os("HOME").map(PathBuf::from);
}

mod libc {
    pub use sgx_trts::libc::ocall::{chdir, getcwd, getenv, getuid, sysconf};
    pub use sgx_trts::libc::*;
}
