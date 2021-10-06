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

//! Runtime services

use crate::enclave;
use crate::slice;
use crate::str;
use crate::sync::SgxSpinlock;
use crate::thread;
use sgx_trts::enclave::rsgx_is_supported_EDMM;
use sgx_types::{sgx_enclave_id_t, sgx_thread_t, SGX_THREAD_T_NULL};

// Reexport some of our utilities which are expected by other crates.
pub use crate::panicking::{begin_panic, begin_panic_fmt, panic_count};
use crate::sync::Once;
pub use crate::sys_common::at_exit;
use crate::sys_common::rt::cleanup;

static INIT: Once = Once::new();
static EXIT: Once = Once::new();
static GLOBAL_INIT_LOCK: SgxSpinlock = SgxSpinlock::new();
static mut INIT_TCS: sgx_thread_t = SGX_THREAD_T_NULL;

#[no_mangle]
pub extern "C" fn t_global_exit_ecall() {
    extern "C" {
        fn uninit_global_object();
    }

    GLOBAL_INIT_LOCK.lock();
    EXIT.call_once(|| unsafe {
        if INIT_TCS == thread::rsgx_thread_self() && !rsgx_is_supported_EDMM() {
            uninit_global_object();
        }
    });
}

#[no_mangle]
pub extern "C" fn t_global_init_ecall(id: u64, path: *const u8, len: usize) {
    if path.is_null() {
        return;
    }

    GLOBAL_INIT_LOCK.lock();
    unsafe { INIT_TCS = thread::rsgx_thread_self() };

    INIT.call_once(|| {
        enclave::set_enclave_id(id as sgx_enclave_id_t);
        let s = unsafe {
            let str_slice = slice::from_raw_parts(path, len);
            str::from_utf8_unchecked(str_slice)
        };
        enclave::set_enclave_path(s);
    });
}

global_dtors_object! {
    GLOBAL_DTORS, global_exit = { cleanup(); }
}
