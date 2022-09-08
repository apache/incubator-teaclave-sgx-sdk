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

use crate::arch::Tcs;
use crate::enclave::state::State;
use crate::enclave::EnclaveRange;
use crate::enclave::{mem, parse, state};
use crate::feature::{SysFeatures, SystemFeatures, Version};
use crate::fence::lfence;
use crate::stackchk;
use crate::tcs::tc;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr;
use core::ptr::NonNull;
use core::slice;
use sgx_crypto_sys::sgx_init_crypto_lib;
use sgx_tlibc_sys::{sgx_heap_init, sgx_init_string_lib};
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::ContiguousMemory;
use sgx_types::types::EnclaveId;

#[allow(unused_variables)]
#[link_section = ".nipx"]
pub fn rtinit(tcs: &mut Tcs, ms: *mut SystemFeatures, tidx: usize) -> SgxResult {
    ensure!(state::lock_state().is_not_started(), SgxStatus::Unexpected);

    // TODO: pcl_entry
    parse::relocate()?;

    mem::Image::init();
    let features = SysFeatures::init(NonNull::new(ms).ok_or(SgxStatus::Unexpected)?)?;

    let heap = mem::Heap::get_or_init();
    let rsrvmem = mem::RsrvMem::get_or_init();
    ensure!(rsrvmem.check(), SgxStatus::Unexpected);

    unsafe {
        ensure!(
            sgx_heap_init(
                heap.base as *const c_void,
                heap.size,
                heap.min_size,
                features.is_edmm() as i32,
            ) == 0,
            SgxStatus::Unexpected
        );
        ensure!(
            sgx_init_string_lib(features.cpu_features()) == 0,
            SgxStatus::Unexpected
        );

        let cpuid_table = if features.version() > Version::Sdk2_0 {
            features.cpuinfo_table() as *const _ as *const u32
        } else {
            ptr::null()
        };

        ensure!(
            sgx_init_crypto_lib(features.cpu_features(), cpuid_table).is_success(),
            SgxStatus::Unexpected
        );

        stackchk::__intel_security_cookie = tc::get_stack_guard().get();
    }

    tc::ThreadControl::from_tcs(tcs).init(tidx, true)?;

    #[cfg(not(any(feature = "sim", feature = "hyper")))]
    {
        if features.is_edmm() {
            // EDMM:
            // need to accept the trimming of the POST_REMOVE pages
            crate::edmm::mem::accept_post_remove()?;
        }
    }

    heap.zero_memory();
    rsrvmem.zero_memory();

    state::set_state(State::InitDone);
    Ok(())
}

pub fn ctors() -> SgxResult {
    if let Some(init_array) = parse::init_array()? {
        let fn_array = init_array.get_array();
        for f in fn_array {
            f.get_fn()();
        }
    }
    Ok(())
}

pub fn global_init(tcs: &mut Tcs, raw: *mut InitInfoHeader, tidx: usize) -> SgxResult {
    let mut header = NonNull::new(raw).ok_or(SgxStatus::Unexpected)?;
    let header = unsafe { header.as_mut() };
    ensure!(header.is_host_range(), SgxStatus::Unexpected);
    lfence();

    ensure!(header.check(), SgxStatus::Unexpected);
    ensure!(header.as_ref().is_host_range(), SgxStatus::Unexpected);
    lfence();

    ensure!(state::get_state() == State::InitDone, SgxStatus::Unexpected);
    let mut tc = tc::ThreadControl::from_tcs(tcs);
    if !tc.is_initialized() {
        tc.init(tidx, false)?;
    }

    let eid = header.eid;
    let header_len = core::mem::size_of::<InitInfoHeader>();
    let path_len = header.path_len;
    let env_len = header.env_len;
    let args_len = header.args_len;

    let bytes: Vec<u8> = header.as_mut().into();

    unsafe {
        extern "C" {
            fn global_init_ecall(
                eid: EnclaveId,
                path: *const u8,
                path_len: usize,
                env: *const u8,
                env_len: usize,
                args: *const u8,
                args_len: usize,
            );
        }

        let header = bytes.as_ptr();
        let path = header.add(header_len);
        let env = header.add(header_len + path_len);
        let args = header.add(header_len + path_len + env_len);

        global_init_ecall(eid, path, path_len, env, env_len, args, args_len);
    }
    Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InitInfoHeader {
    eid: EnclaveId,
    info_size: usize,
    path_len: usize,
    env_len: usize,
    args_len: usize,
}

impl InitInfoHeader {
    fn check(&self) -> bool {
        if let Some(info_size) = self
            .path_len
            .checked_add(self.env_len)
            .and_then(|l| l.checked_add(self.args_len))
            .and_then(|l| l.checked_add(core::mem::size_of::<Self>()))
        {
            info_size == self.info_size
        } else {
            false
        }
    }
}

unsafe impl ContiguousMemory for InitInfoHeader {}

impl AsRef<[u8]> for InitInfoHeader {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, self.info_size) }
    }
}

impl AsMut<[u8]> for InitInfoHeader {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self as *mut _ as *mut u8, self.info_size) }
    }
}
