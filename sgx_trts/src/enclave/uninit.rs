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

use crate::emm::tcs::trim_tcs;
use crate::enclave::state::{self, State};
use crate::enclave::{atexit, parse};
use crate::tcs::ThreadControl;
use core::sync::atomic::AtomicBool;
use sgx_types::error::SgxResult;

pub static UNINIT_FLAG: AtomicBool = AtomicBool::new(false);

pub fn dtors() -> SgxResult {
    if let Some(uninit_array) = parse::uninit_array()? {
        let fn_array = uninit_array.get_array();
        for f in fn_array {
            f.get_fn()();
        }
    }
    Ok(())
}

#[inline]
pub fn at_exit_cleaup() {
    atexit::cleanup();
}

pub fn rtuninit(tc: ThreadControl) -> SgxResult {
    use crate::call;
    use crate::tcs;
    use core::ptr;
    use core::sync::atomic::Ordering;
    use sgx_types::error::SgxStatus;

    if UNINIT_FLAG.load(Ordering::SeqCst) {
        state::set_state(State::Crashed);
        return Ok(());
    }

    let current = tcs::current();
    let tcs = tc.tcs();
    assert!(ptr::eq(current.tcs(), tcs));

    cfg_if! {
        if #[cfg(feature = "sim")] {
            let is_legal = tc.is_init();
        } else if #[cfg(feature = "hyper")] {
            let is_legal = tc.is_init();
        } else {
            use crate::feature::SysFeatures;
            use crate::emm::layout::LayoutTable;

            let is_legal = if SysFeatures::get().is_edmm() {
                tc.is_utility() || !LayoutTable::new().is_dyn_tcs_exist()
            } else {
                tc.is_init()
            };
        }
    }
    if !is_legal {
        state::set_state(State::Crashed);
        bail!(SgxStatus::Unexpected);
    }

    UNINIT_FLAG.store(true, Ordering::SeqCst);

    #[cfg(not(any(feature = "sim", feature = "hyper")))]
    {
        if SysFeatures::get().is_edmm() {
            trim_tcs(tcs)?;
        }
    }

    let guard = call::FIRST_ECALL.lock();
    if call::FIRST_ECALL.is_completed() {
        at_exit_cleaup();
        let _ = dtors();
    }
    drop(guard);

    state::set_state(State::Crashed);
    Ok(())
}

#[allow(unused_variables)]
pub fn global_exit(tc: ThreadControl) -> SgxResult {
    extern "C" {
        fn global_exit_ecall();
    }
    unsafe { global_exit_ecall() }

    cfg_if! {
        if #[cfg(feature = "sim")] {
            rtuninit(tc)
        } else if #[cfg(feature = "hyper")] {
            rtuninit(tc)
        } else {
            use crate::feature::SysFeatures;
            if !SysFeatures::get().is_edmm() {
                rtuninit(tc)
            } else {
                Ok(())
            }
        }
    }
}
