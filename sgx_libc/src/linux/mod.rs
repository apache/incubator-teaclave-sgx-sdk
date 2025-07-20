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

use core::mem;
use core::ptr;
use core::slice;
use sgx_oc::ocall::set_errno;
use sgx_trts::rand::rand;

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x86_64;
        pub use x86_64::*;
    } else {

    }
}

mod edl;
pub mod ocall;
#[cfg(feature = "pthread")]
pub mod pthread;

pub use sgx_oc::linux::{major, makedev, minor};
pub use sgx_oc::linux::{sigaddset, sigdelset, sigemptyset, sigfillset, sigismember};
pub use sgx_oc::linux::{CMSG_ALIGN, CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_NXTHDR, CMSG_SPACE};
pub use sgx_oc::linux::{
    CPU_ALLOC_SIZE, CPU_CLR, CPU_COUNT, CPU_COUNT_S, CPU_EQUAL, CPU_ISSET, CPU_SET, CPU_ZERO,
    FD_CLR, FD_ISSET, FD_SET, FD_ZERO,
};
pub use sgx_tlibc_sys::*;

extern "C" {
    pub fn abort() -> !;
    pub fn atexit(func: extern "C" fn()) -> c_int;
}

/// Get the last error number.
#[no_mangle]
pub extern "C" fn errno() -> c_int {
    unsafe { *errno_location() }
}

#[no_mangle]
pub extern "C" fn gai_strerror(errcode: c_int) -> *const c_char {
    use sgx_oc::linux::ocall::gai_error_cstr;

    gai_error_cstr(errcode).as_ptr()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn memrchr(cx: *const c_void, c: c_int, n: size_t) -> *mut c_void {
    if n == 0 {
        return ptr::null_mut();
    }
    let mut ret = ptr::null();
    let mut p: *const u8 = (cx as usize + (n - 1)) as *const u8;
    for _ in 0..n {
        if *p == c as u8 {
            ret = p;
            break;
        }
        p = p.offset(-1);
    }
    ret as *mut c_void
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: size_t, _flags: c_uint) -> ssize_t {
    if buf.is_null() || buflen == 0 {
        set_errno(EINVAL);
        return -1;
    }

    let random_buf = slice::from_raw_parts_mut(buf as *mut u8, buflen);
    if rand(random_buf).is_ok() {
        buflen as ssize_t
    } else {
        set_errno(EFAULT);
        -1
    }
}
