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

use crate::enclave::{is_within_enclave, EnclaveRange, UNINIT_FLAG};
use crate::error;
use crate::sync::SpinMutex;
use crate::tcs;
use crate::tcs::tc::{TcsId, TdFlags};
use crate::thread::{info, task, tls};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::ffi::c_void;
use core::mem;
use core::ptr;
use core::sync::atomic::Ordering;
use sgx_types::error::{SgxResult, SgxStatus};

const WAIT_TIMEOUT_SECONDS: u64 = 5;

#[derive(Clone)]
pub struct Thread {
    native: Arc<Native>,
}

pub struct Native(SpinMutex<Inner>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Preparing,
    Execution,
    WakedUp,
    OutOfTcs,
    Unexpected,
}

struct Inner {
    state: State,
    creater: TcsId,
    joiner: Option<TcsId>,
    retval: *mut c_void,
}

impl Thread {
    pub fn new<F>(p: F, arg: *mut c_void) -> SgxResult<Thread>
    where
        F: FnOnce(*mut c_void) -> *mut c_void + Send + 'static,
    {
        extern "C" {
            fn pthread_create_ocall(ret: *mut SgxStatus, tcs: usize) -> SgxStatus;
        }

        if !arg.is_null() && !is_within_enclave(arg as *const u8, mem::size_of::<*mut c_void>()) {
            bail!(SgxStatus::InvalidParameter);
        }

        let tcs_id = tcs::current().id();
        let thread = Thread {
            native: Arc::new(Native(SpinMutex::new(Inner {
                state: State::Preparing,
                creater: tcs_id,
                joiner: None,
                retval: ptr::null_mut(),
            }))),
        };

        let task = task::Task::new(Box::new(p), arg, thread.clone());
        task::push(task);

        let mut ret = SgxStatus::Unexpected;
        unsafe {
            pthread_create_ocall(&mut ret as *mut _, tcs_id.as_usize());
        }
        if ret != SgxStatus::Success {
            if let Some(task) = task::pop() {
                if task.thread().eq(&thread) {
                    bail!(ret);
                } else {
                    let state = if ret == SgxStatus::OutOfTcs {
                        State::OutOfTcs
                    } else {
                        State::Unexpected
                    };
                    let creater = {
                        let mut guard = task.thread().native.0.lock();
                        guard.state = state;
                        guard.creater
                    };
                    let _ = creater.wakeup();
                }
            }
        }

        loop {
            tcs_id.wait_timeout(WAIT_TIMEOUT_SECONDS)?;

            let guard = thread.native.0.lock();
            let state = guard.state;
            match state {
                State::Preparing => (),
                State::OutOfTcs => bail!(SgxStatus::OutOfTcs),
                State::Unexpected => bail!(SgxStatus::Unexpected),
                _ => break,
            }
        }

        Ok(thread)
    }

    pub fn join(self) -> SgxResult<*mut c_void> {
        if let Some(thread) = current() {
            if self.eq(&thread) {
                bail!(SgxStatus::InvalidParameter);
            }
        }

        if UNINIT_FLAG.load(Ordering::SeqCst) {
            return Ok(ptr::null_mut());
        }

        {
            let guard = self.native.0.lock();
            if guard.joiner.is_some() {
                bail!(SgxStatus::InvalidState);
            }
        }

        let tcs_id = tcs::current().id();
        let result = loop {
            let mut guard = self.native.0.lock();
            if guard.state == State::WakedUp {
                break Ok(guard.retval);
            }

            if guard.joiner.is_none() {
                guard.joiner = Some(tcs_id);
            }
            drop(guard);

            tcs_id.wait_timeout(WAIT_TIMEOUT_SECONDS)?;
        };
        result
    }

    pub fn as_raw(&self) -> *const Native {
        Arc::as_ptr(&self.native)
    }

    pub fn into_raw(self) -> *const Native {
        Arc::into_raw(self.native)
    }

    pub unsafe fn from_raw(native: *const Native) -> Thread {
        Thread {
            native: Arc::from_raw(native),
        }
    }

    pub fn id(&self) -> TcsId {
        tcs::current().id()
    }

    pub(crate) fn set_retval(&self, retval: *mut c_void) {
        let mut guard = self.native.0.lock();
        guard.retval = retval;
    }
}

pub extern "C" fn thread_run(ms: *mut c_void) -> SgxStatus {
    if let Some(task) = task::pop() {
        let mut tc = tcs::current();
        tc.tds_mut().flags = (tc.td_flags() | TdFlags::PTHREAD_CREATE).bits();

        info::set(task.thread().clone());
        tls::Tls::init();

        let creater = {
            let mut guard = task.thread().native.0.lock();
            guard.state = State::Execution;
            guard.creater
        };
        let _ = creater.wakeup();

        task.run();

        let active_tls = tls::Tls::activate();
        drop(active_tls);

        wakeup_join(ms);
        info::clear();
        tc.tds_mut().flags = (tc.td_flags() & (!TdFlags::PTHREAD_CREATE)).bits();
    }
    SgxStatus::Success
}

fn wakeup_join(ms: *mut c_void) {
    if ms.is_null() {
        error::abort();
    }

    let waiter = unsafe { &mut *(ms as *mut usize) };
    if !waiter.is_host_range() {
        error::abort();
    }

    if let Some(thread) = current() {
        let mut guard = thread.native.0.lock();
        if let Some(joiner) = guard.joiner {
            *waiter = joiner.as_usize();
        }
        guard.state = State::WakedUp;
    }
}

pub fn current() -> Option<Thread> {
    info::current_thread()
}

impl PartialEq for Thread {
    fn eq(&self, other: &Thread) -> bool {
        Arc::ptr_eq(&self.native, &other.native)
    }
}

impl Eq for Thread {}

impl TcsId {
    pub fn wait_timeout(self, timeout: u64) -> SgxResult {
        extern "C" {
            fn pthread_wait_timeout_ocall(
                ret: *mut SgxStatus,
                tcs: usize,
                timeout: u64,
            ) -> SgxStatus;
        }

        let mut ret = SgxStatus::Unexpected;
        unsafe {
            pthread_wait_timeout_ocall(&mut ret as *mut _, self.as_usize(), timeout);
        }
        if ret == SgxStatus::Success {
            Ok(())
        } else {
            Err(ret)
        }
    }

    pub fn wakeup(self) -> SgxResult {
        extern "C" {
            fn pthread_wakeup_ocall(ret: *mut SgxStatus, tcs: usize) -> SgxStatus;
        }

        let mut ret = SgxStatus::Unexpected;
        unsafe {
            pthread_wakeup_ocall(&mut ret as *mut _, self.as_usize());
        }
        if ret == SgxStatus::Success {
            Ok(())
        } else {
            Err(ret)
        }
    }
}
