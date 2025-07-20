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

use crate::io as std_io;

use sgx_trts::error::abort;

pub mod memchr {
    #[allow(unused_imports)]
    pub use core::slice::memchr::{memchr, memrchr};
}

// This is not necessarily correct. May want to consider making it part of the
// spec definition?
use crate::os::raw::c_char;

// SAFETY: must be called only once during runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub unsafe fn cleanup() {}

pub fn unsupported<T>() -> std_io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> std_io::Error {
    std_io::const_io_error!(
        std_io::ErrorKind::Unsupported,
        "operation not supported on this platform",
    )
}

pub fn decode_error_kind(_code: i32) -> crate::io::ErrorKind {
    crate::io::ErrorKind::Uncategorized
}

pub fn abort_internal() -> ! {
    abort();
}

pub fn hashmap_random_keys() -> (u64, u64) {
    (1, 2)
}

pub unsafe fn strlen(mut s: *const c_char) -> usize {
    // SAFETY: The caller must guarantee `s` points to a valid 0-terminated string.
    let mut n = 0;
    while *s != 0 {
        n += 1;
        s = s.offset(1);
    }
    n
}
