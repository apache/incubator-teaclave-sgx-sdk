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

use crate::alloc::borrow::ToOwned;
use crate::linux::x86_64::*;
use alloc::str;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::From;
use core::error::Error;
use core::fmt;
use core::num::NonZeroUsize;
use core::ptr;
use core::slice;
use core::slice::SliceIndex;
use sgx_ffi::c_str::{CStr, CString};
use sgx_ffi::memchr;
use sgx_trts::trts::enclave_mode;
use sgx_trts::trts::{EnclaveRange, OcBuffer};
use sgx_types::error::SgxStatus;
use sgx_types::types::EnclaveMode;

pub use crate::linux::edl::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OCallError {
    SgxError(SgxStatus),
    OsError(i32),
    GaiError(i32),
    CustomError(&'static str),
}

impl OCallError {
    pub fn from_sgx_error(errno: SgxStatus) -> Self {
        set_errno(ESGX);
        OCallError::SgxError(errno)
    }

    pub fn from_os_error(errno: i32) -> Self {
        set_errno(errno);
        OCallError::OsError(errno)
    }

    pub fn from_gai_error(err: i32) -> Self {
        OCallError::GaiError(err)
    }

    pub fn from_custom_error(err: &'static str) -> Self {
        set_errno(EOCALL);
        OCallError::CustomError(err)
    }

    pub fn equal_to_sgx_error(&self, other: SgxStatus) -> bool {
        matches!(self, OCallError::SgxError(e) if *e == other)
    }

    pub fn equal_to_os_error(&self, other: i32) -> bool {
        matches!(self, OCallError::OsError(e) if *e == other)
    }

    pub fn equal_to_gai_error(&self, other: i32) -> bool {
        matches!(self, OCallError::GaiError(e) if *e == other)
    }
}

impl OCallError {
    pub fn error_description(&self) -> String {
        match self {
            Self::SgxError(status) => sgx_error_string(*status),
            Self::OsError(errno) => os_error_string(*errno),
            Self::GaiError(errno) => gai_error_string(*errno),
            Self::CustomError(s) => (*s).to_owned(),
        }
    }
}

impl fmt::Display for OCallError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SgxError(status) => write!(fmt, "sgx error {}", status.as_str()),
            Self::OsError(errno) => write!(fmt, "os error {}", errno),
            Self::GaiError(errno) => write!(fmt, "gai error {}", errno),
            Self::CustomError(s) => write!(fmt, "custom error {}", *s),
        }
    }
}

impl From<SgxStatus> for OCallError {
    fn from(errno: SgxStatus) -> OCallError {
        OCallError::from_sgx_error(errno)
    }
}

impl From<&'static str> for OCallError {
    fn from(err: &'static str) -> OCallError {
        OCallError::from_custom_error(err)
    }
}

impl Error for OCallError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self {
            OCallError::SgxError(s) => s.description(),
            OCallError::OsError(code) => os_error_str(*code),
            OCallError::GaiError(code) => gai_error_str(*code),
            OCallError::CustomError(s) => s,
        }
    }

    #[allow(deprecated)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            OCallError::SgxError(ref s) => Some(s),
            OCallError::OsError(..) => None,
            OCallError::GaiError(..) => None,
            OCallError::CustomError(..) => None,
        }
    }
}

pub type OCallResult<T> = Result<T, OCallError>;

pub fn gai_error_cstr(errno: i32) -> &'static CStr {
    let msg = match errno {
        EAI_BADFLAGS => &b"Bad value for ai_flags\0"[..],
        EAI_NONAME => &b"Name or service not known\0"[..],
        EAI_AGAIN => &b"Temporary failure in name resolution\0"[..],
        EAI_FAIL => &b"Non-recoverable failure in name resolution\0"[..],
        EAI_FAMILY => &b"ai_family not supported\0"[..],
        EAI_SOCKTYPE => &b"ai_socktype not supported\0"[..],
        EAI_SERVICE => &b"Servname not supported for ai_socktype\0"[..],
        EAI_MEMORY => &b"Memory allocation failure\0"[..],
        EAI_SYSTEM => &b"System error\0"[..],
        EAI_OVERFLOW => &b"Argument buffer overflow\0"[..],
        EAI_NODATA => &b"No address associated with hostname\0"[..],
        EAI_ADDRFAMILY => &b"Address family for hostname not supported\0"[..],
        EAI_INPROGRESS => &b"Processing request in progress\0"[..],
        EAI_CANCELED => &b"Request canceled\0"[..],
        EAI_NOTCANCELED => &b"Request not canceled\0"[..],
        EAI_ALLDONE => &b"All requests done\0"[..],
        EAI_INTR => &b"Interrupted by a signal\0"[..],
        EAI_IDN_ENCODE => &b"Parameter string not correctly encoded\0"[..],
        _ => &b"Unknown gai_error_code\0"[..],
    };
    CStr::from_bytes_with_nul(msg).unwrap()
}

pub fn gai_error_str(errno: i32) -> &'static str {
    match errno {
        EAI_BADFLAGS => "Bad value for ai_flags",
        EAI_NONAME => "Name or service not known",
        EAI_AGAIN => "Temporary failure in name resolution",
        EAI_FAIL => "Non-recoverable failure in name resolution",
        EAI_FAMILY => "ai_family not supported",
        EAI_SOCKTYPE => "ai_socktype not supported",
        EAI_SERVICE => "Servname not supported for ai_socktype",
        EAI_MEMORY => "Memory allocation failure",
        EAI_SYSTEM => "System error",
        EAI_OVERFLOW => "Argument buffer overflow",
        EAI_NODATA => "No address associated with hostname",
        EAI_ADDRFAMILY => "Address family for hostname not supported",
        EAI_INPROGRESS => "Processing request in progress",
        EAI_CANCELED => "Request canceled",
        EAI_NOTCANCELED => "Request not canceled",
        EAI_ALLDONE => "All requests done",
        EAI_INTR => "Interrupted by a signal",
        EAI_IDN_ENCODE => "Parameter string not correctly encoded",
        _ => "Unknown gai_error_code",
    }
}

pub fn os_error_str(errno: i32) -> &'static str {
    extern "C" {
        pub fn strerror(errnum: c_int) -> *const c_char;
    }

    unsafe {
        let p = strerror(errno);
        assert!(!p.is_null(), "strerror failure");
        str::from_utf8(CStr::from_ptr(p).to_bytes()).unwrap()
    }
}

pub fn os_error_string(errno: i32) -> String {
    const TMPBUF_SZ: usize = 128;
    extern "C" {
        pub fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: size_t) -> c_int;
    }

    let mut buf = [0_i8; TMPBUF_SZ];
    let p = buf.as_mut_ptr();
    unsafe {
        assert!(
            strerror_r(errno as c_int, p, buf.len()) >= 0,
            "strerror_r failure"
        );

        let p = p as *const _;
        str::from_utf8(CStr::from_ptr(p).to_bytes())
            .unwrap()
            .to_owned()
    }
}

#[inline]
pub fn gai_error_string(errno: i32) -> String {
    gai_error_str(errno).to_owned()
}

#[inline]
pub fn sgx_error_string(status: SgxStatus) -> String {
    status.__description().to_owned()
}

#[inline]
pub fn set_errno(e: i32) {
    extern "C" {
        #[cfg_attr(target_os = "linux", link_name = "__errno_location")]
        fn errno_location() -> *mut i32;
    }
    unsafe { *errno_location() = e }
}

macro_rules! bail {
    ($e:expr) => {
        return Err($e);
    };
}

macro_rules! ensure {
    ($cond:expr) => {
        if !($cond) {
            bail!(crate::ocall::OCallError::from_custom_error(stringify!(
                $cond
            )));
        }
    };
    ($cond:expr, $e:expr) => {
        if !($cond) {
            bail!($e);
        }
    };
}

macro_rules! esgx {
    ($status:expr) => {
        crate::ocall::OCallError::from_sgx_error($status)
    };
}

macro_rules! eos {
    ($errno:expr) => {
        crate::ocall::OCallError::from_os_error($errno)
    };
}

macro_rules! ecust {
    ($lt:literal) => {
        crate::ocall::OCallError::from_custom_error($lt)
    };
}

macro_rules! egai {
    ($errno:expr) => {
        crate::ocall::OCallError::from_gai_error($errno)
    };
}

#[inline]
fn check_enclave_buffer(buf: &[u8]) -> OCallResult<()> {
    ensure!(
        buf.is_enclave_range(),
        ecust!("Buffer is not strictly inside enclave")
    );
    Ok(())
}

#[inline]
fn check_host_buffer(buf: &[u8]) -> OCallResult<()> {
    ensure!(
        buf.is_host_range(),
        ecust!("Buffer is not strictly outside enclave")
    );
    Ok(())
}

#[inline]
fn check_trusted_enclave_buffer(buf: &[u8]) -> OCallResult<()> {
    ensure!(!buf.as_ptr().is_null(), ecust!("Invalid null buffer."));
    ensure!(!buf.is_empty(), ecust!("Invalid buffer checking length."));
    ensure!(
        buf.is_enclave_range(),
        ecust!("Buffer is not strictly inside enclave")
    );
    Ok(())
}

pub unsafe fn shrink_to_fit_cstring<T: Into<Vec<u8>>>(buf: T) -> OCallResult<CString> {
    let mut buf = buf.into();
    if let Some(i) = memchr::memchr(0, &buf) {
        buf.set_len(i + 1);
        buf.shrink_to_fit();
        Ok(CString::from_vec_with_nul_unchecked(buf))
    } else {
        Err(ecust!("Malformed CString: not null terminated"))
    }
}

trait AsPtrAndLen<T> {
    fn as_ptr_and_len(&self) -> (*const T, usize);
}

impl<T> AsPtrAndLen<T> for Option<&[T]> {
    fn as_ptr_and_len(&self) -> (*const T, usize) {
        match self {
            Some(slice) => slice.as_ptr_and_len(),
            None => (ptr::null(), 0),
        }
    }
}

impl<T> AsPtrAndLen<T> for Option<&mut [T]> {
    fn as_ptr_and_len(&self) -> (*const T, usize) {
        match self {
            Some(slice) => slice.as_ptr_and_len(),
            None => (ptr::null(), 0),
        }
    }
}

impl<T> AsPtrAndLen<T> for &[T] {
    fn as_ptr_and_len(&self) -> (*const T, usize) {
        if !self.is_empty() {
            (self.as_ptr(), self.len())
        } else {
            (ptr::null(), 0)
        }
    }
}

impl<T> AsPtrAndLen<T> for &mut [T] {
    fn as_ptr_and_len(&self) -> (*const T, usize) {
        if !self.is_empty() {
            (self.as_ptr(), self.len())
        } else {
            (ptr::null(), 0)
        }
    }
}

pub trait AsMutPtrAndLen<T> {
    fn as_mut_ptr_and_len(&mut self) -> (*mut T, usize);
}

impl<T> AsMutPtrAndLen<T> for Option<&mut [T]> {
    fn as_mut_ptr_and_len(&mut self) -> (*mut T, usize) {
        match self {
            Some(slice) => slice.as_mut_ptr_and_len(),
            None => (ptr::null_mut(), 0),
        }
    }
}

impl<T> AsMutPtrAndLen<T> for &mut [T] {
    fn as_mut_ptr_and_len(&mut self) -> (*mut T, usize) {
        if !self.is_empty() {
            (self.as_mut_ptr(), self.len())
        } else {
            (ptr::null_mut(), 0)
        }
    }
}

mod asyncio;
mod cpuid;
mod env;
mod fd;
mod file;
mod hostbuf;
mod mem;
mod msbuf;
mod net;
mod pipe;
mod process;
mod socket;
mod socket_msg;
mod sys;
mod thread;
mod time;

pub use asyncio::*;
pub use cpuid::*;
pub use env::*;
pub use fd::*;
pub use file::*;
pub use hostbuf::*;
pub use mem::*;
pub use net::*;
pub use pipe::*;
pub use process::*;
pub use socket::*;
pub use socket_msg::*;
pub use sys::*;
pub use thread::*;
pub use time::*;

pub(crate) use msbuf::*;
