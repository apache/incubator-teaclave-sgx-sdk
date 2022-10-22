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

use sgx_types::error::SgxStatus;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
#[cfg(feature = "ufs")]
use std::io::ErrorKind;

pub type FsResult<T = ()> = std::result::Result<T, FsError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FsError {
    SgxError(SgxStatus),
    OsError(i32),
}

impl FsError {
    #[inline]
    pub fn from_sgx_error(errno: SgxStatus) -> Self {
        FsError::SgxError(errno)
    }

    #[inline]
    pub fn from_os_error(errno: i32) -> Self {
        FsError::OsError(errno)
    }

    #[inline]
    pub fn equal_to_sgx_error(&self, other: SgxStatus) -> bool {
        matches!(self, FsError::SgxError(e) if *e == other)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn equal_to_os_error(&self, other: i32) -> bool {
        matches!(self, FsError::OsError(e) if *e == other)
    }

    #[inline]
    pub fn is_success(&self) -> bool {
        match self {
            Self::SgxError(status) => status.is_success(),
            Self::OsError(errno) => *errno == 0,
        }
    }

    #[cfg(feature = "tfs")]
    pub fn to_io_error(self) -> IoError {
        match self {
            Self::SgxError(status) => IoError::from_sgx_error(status),
            Self::OsError(errno) => IoError::from_raw_os_error(errno),
        }
    }

    #[cfg(feature = "ufs")]
    pub fn to_io_error(self) -> IoError {
        match self {
            Self::SgxError(status) => IoError::new(ErrorKind::Other, status.as_str()),
            Self::OsError(errno) => IoError::from_raw_os_error(errno),
        }
    }

    pub fn set_errno(&self) {
        extern "C" {
            #[cfg_attr(target_os = "linux", link_name = "__errno_location")]
            fn errno_location() -> *mut i32;
        }
        let e = match self {
            Self::SgxError(status) => *status as i32,
            Self::OsError(errno) => *errno,
        };
        unsafe { *errno_location() = e }
    }

    #[allow(dead_code)]
    pub fn to_errno(self) -> i32 {
        match self {
            Self::SgxError(status) => status as i32,
            Self::OsError(errno) => errno,
        }
    }
}

impl fmt::Display for FsError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SgxError(status) => write!(fmt, "sgx error {}", status.as_str()),
            Self::OsError(errno) => write!(fmt, "os error {}", errno),
        }
    }
}

impl From<SgxStatus> for FsError {
    fn from(errno: SgxStatus) -> FsError {
        FsError::from_sgx_error(errno)
    }
}

impl From<i32> for FsError {
    #[inline]
    fn from(code: i32) -> FsError {
        FsError::from_os_error(code)
    }
}

impl Error for FsError {
    #[cfg(feature = "tfs")]
    #[allow(deprecated)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SgxError(status) => Some(status),
            Self::OsError(..) => None,
        }
    }
}

#[macro_export]
macro_rules! esgx {
    ($status:expr) => {
        $crate::sys::error::FsError::from_sgx_error($status)
    };
}

#[macro_export]
macro_rules! eos {
    ($errno:expr) => {
        $crate::sys::error::FsError::from_os_error($errno)
    };
}
