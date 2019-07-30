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

use sgx_types::*;
use sgx_trts::enclave;
use crate::sync::SgxThreadSpinlock;
use crate::path::{Path, PathBuf};
use crate::io;
use core::sync::atomic::{AtomicU64, Ordering};

pub use sgx_trts::enclave::SgxThreadPolicy;

static LOCK: SgxThreadSpinlock = SgxThreadSpinlock::new();
static mut ENCLAVE_PATH: Option<PathBuf> = None;
static ENCLAVE_ID: AtomicU64 = AtomicU64::new(0);

///
/// get_enclave_base is to get enclave map base address.
///
#[inline]
pub fn get_enclave_base() -> * const u8 {
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
pub fn get_heap_base() -> * const u8 {
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
    ENCLAVE_ID.store(eid as u64, Ordering::SeqCst);
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
    unsafe {
        LOCK.lock();
        if ENCLAVE_PATH.is_none() {
            ENCLAVE_PATH = Some(path.as_ref().to_owned());
        }
        LOCK.unlock();
        Ok(())
    }
}