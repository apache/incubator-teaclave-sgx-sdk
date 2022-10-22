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

use core::sync::atomic::{AtomicU64, Ordering};
use crate::io;
use crate::path::{Path, PathBuf};
use crate::sync::SgxThreadSpinlock;
use crate::untrusted::fs;
use sgx_trts::enclave;
use sgx_types::*;

pub use sgx_trts::enclave::SgxThreadPolicy;

static LOCK: SgxThreadSpinlock = SgxThreadSpinlock::new();
static mut ENCLAVE_PATH: Option<PathBuf> = None;
static ENCLAVE_ID: AtomicU64 = AtomicU64::new(0);

///
/// get_enclave_base is to get enclave map base address.
///
#[inline]
pub fn get_enclave_base() -> *const u8 {
    enclave::rsgx_get_enclave_base()
}

///
/// get_enclave_size is to get enclave map size.
///
#[inline]
pub fn get_enclave_size() -> usize {
    enclave::rsgx_get_enclave_size()
}

///
/// get_heap_base is to get heap base address.
///
#[inline]
pub fn get_heap_base() -> *const u8 {
    enclave::rsgx_get_heap_base()
}

///
/// get_heap_size is to get heap size.
///
#[inline]
pub fn get_heap_size() -> usize {
    enclave::rsgx_get_heap_size()
}

///
/// get_rsrv_base is to get reserved memory base address.
///
#[inline]
pub fn get_rsrv_base() -> *const u8 {
    enclave::rsgx_get_rsrv_base()
}

///
/// get_rsrv_size is to get reserved memory size.
///
#[inline]
pub fn get_rsrv_size() -> usize {
    enclave::rsgx_get_rsrv_size()
}

///
/// get_tcs_max_num is to get max tcs number.
///
#[inline]
pub fn get_tcs_max_num() -> u32 {
    enclave::rsgx_get_tcs_max_num()
}
///
/// get_thread_policy is to get TCS policy.
///
#[inline]
pub fn get_thread_policy() -> SgxThreadPolicy {
    enclave::rsgx_get_thread_policy()
}

///
/// get_enclave_id is to get enclave ID.
///
pub fn get_enclave_id() -> sgx_enclave_id_t {
    ENCLAVE_ID.load(Ordering::SeqCst) as sgx_enclave_id_t
}

///
/// set_enclave_id is to set enclave ID.
///
pub fn set_enclave_id(eid: sgx_enclave_id_t) {
    ENCLAVE_ID.store(eid, Ordering::SeqCst);
}

///
/// get_enclave_path is to get the path or name of the enclave.
///
pub fn get_enclave_path() -> Option<PathBuf> {
    unsafe {
        LOCK.lock();
        let path = ENCLAVE_PATH.as_ref().map(|p| p.to_owned());
        LOCK.unlock();
        path
    }
}

///
/// set_enclave_path is to set the path or name of the enclave.
///
pub fn set_enclave_path<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let _ = fs::metadata(&path)?;
    unsafe {
        LOCK.lock();
        if ENCLAVE_PATH.is_none() {
            ENCLAVE_PATH = Some(path.as_ref().to_owned());
        }
        LOCK.unlock();
        Ok(())
    }
}