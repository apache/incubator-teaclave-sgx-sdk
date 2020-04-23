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

use sgx_types::{self, SysError, sgx_status_t, sgx_thread_t, SGX_THREAD_T_NULL};
use sgx_trts::libc;
use sgx_trts::error::set_errno;
use sgx_trts::enclave::SgxThreadData;
use sgx_libc::{c_void, c_int, c_long, time_t, timespec};
use core::ptr;
use core::cmp;
use core::cell::UnsafeCell;
use crate::sync::SgxThreadSpinlock;
use crate::thread::{self, rsgx_thread_self};
use crate::time::Duration;
use crate::u64;

extern "C" {
    pub fn u_thread_wait_event_ocall(
        result: *mut c_int,
        error: *mut c_int,
        tcs: *const c_void,
        timeout: *const timespec,
    ) -> sgx_status_t;

    pub fn u_thread_set_event_ocall(
        result: *mut c_int,
        error: *mut c_int,
        tcs: *const c_void,
    ) -> sgx_status_t;

    pub fn u_thread_set_multiple_events_ocall(
        result: *mut c_int,
        error: *mut c_int,
        tcss: *const *const c_void,
        total: c_int,
    ) -> sgx_status_t;

    pub fn u_thread_setwait_events_ocall(
        result: *mut c_int,
        error: *mut c_int,
        wait_tcs: *const c_void,
        self_tcs: *const c_void,
        timeout: *const timespec,
    ) -> sgx_status_t;
}

pub unsafe fn thread_wait_event(tcs: usize, dur: Duration) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let mut timeout = timespec { tv_sec: 0, tv_nsec: 0 };
    let timeout_ptr: *const timespec = if dur != Duration::new(u64::MAX, 1_000_000_000 - 1) {
        timeout.tv_sec = cmp::min(dur.as_secs(), time_t::max_value() as u64) as time_t;
        timeout.tv_nsec = dur.subsec_nanos() as c_long;
        &timeout as *const timespec
    } else {
        ptr::null()
    };

    let status = u_thread_wait_event_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        tcs as *const c_void,
        timeout_ptr,
    );
    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 { set_errno(error); }
    } else {
        set_errno(libc::ESGX);
        result = -1;
    }
    result
}

pub unsafe fn thread_set_event(tcs: usize) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let status = u_thread_set_event_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        tcs as *const c_void,
    );
    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 { set_errno(error); }
    } else {
        set_errno(libc::ESGX);
        result = -1;
    }
    result
}

pub unsafe fn thread_set_multiple_events(tcss: &[usize]) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_thread_set_multiple_events_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        tcss.as_ptr() as *const *const c_void,
        tcss.len() as c_int,
    );
    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 {
            set_errno(error);
        }
    } else {
        set_errno(libc::ESGX);
        result = -1;
    }
    result
}

pub unsafe fn thread_setwait_events(wait_tcs: usize, self_tcs: usize, dur: Duration) -> c_int {
    let mut result: c_int = 0;
    let mut error: c_int = 0;
    let mut timeout = timespec { tv_sec: 0, tv_nsec: 0 };
    let timeout_ptr: *const timespec = if dur != Duration::new(u64::MAX, 1_000_000_000 - 1) {
        timeout.tv_sec = cmp::min(dur.as_secs(), time_t::max_value() as u64) as time_t;
        timeout.tv_nsec = dur.subsec_nanos() as c_long;
        &timeout as *const timespec
    } else {
        ptr::null()
    };

    let status = u_thread_setwait_events_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        wait_tcs as *const c_void,
        self_tcs as *const c_void,
        timeout_ptr,
    );
    if status == sgx_status_t::SGX_SUCCESS {
        if result == -1 { set_errno(error); }
    } else {
        set_errno(libc::ESGX);
        result = -1;
    }
    result
}

#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SgxThreadMutexControl {
    SGX_THREAD_MUTEX_NONRECURSIVE = 1,
    SGX_THREAD_MUTEX_RECURSIVE = 2,
}

struct SgxThreadMutexInner {
    refcount: usize,
    control: SgxThreadMutexControl,
    spinlock: SgxThreadSpinlock,
    thread_owner: sgx_thread_t,
    thread_vec: Vec<sgx_thread_t>,
}

impl SgxThreadMutexInner {

    const fn new(control: SgxThreadMutexControl) -> Self {
        SgxThreadMutexInner {
            refcount: 0,
            control: control,
            spinlock: SgxThreadSpinlock::new(),
            thread_owner: SGX_THREAD_T_NULL,
            thread_vec: Vec::new(),
        }
    }

    unsafe fn lock(&mut self) -> SysError {
        loop {
            self.spinlock.lock();
            if self.control == SgxThreadMutexControl::SGX_THREAD_MUTEX_RECURSIVE &&
                self.thread_owner == rsgx_thread_self() {
                self.refcount += 1;
                self.spinlock.unlock();
                return Ok(());
            }

            if self.thread_owner == SGX_THREAD_T_NULL &&
                (self.thread_vec.first() == Some(&rsgx_thread_self()) ||
                self.thread_vec.first() == None) {

                if self.thread_vec.first() == Some(&rsgx_thread_self()) {
                    self.thread_vec.remove(0);
                }

                self.thread_owner = rsgx_thread_self();
                self.refcount += 1;
                self.spinlock.unlock();

                return Ok(());
            }

            let mut thread_waiter: sgx_thread_t = SGX_THREAD_T_NULL;
            for waiter in &self.thread_vec {
                if thread::rsgx_thread_equal(*waiter, rsgx_thread_self()) {
                    thread_waiter = *waiter;
                    break;
                }
            }

            if thread_waiter == SGX_THREAD_T_NULL {
                self.thread_vec.push(rsgx_thread_self());
            }
            self.spinlock.unlock();
            thread_wait_event(SgxThreadData::current().get_tcs(), Duration::new(u64::MAX, 1_000_000_000 - 1));
        }
    }

    unsafe fn try_lock(&mut self) -> SysError {
        self.spinlock.lock();
        if self.control == SgxThreadMutexControl::SGX_THREAD_MUTEX_RECURSIVE &&
            self.thread_owner == rsgx_thread_self() {

            self.refcount += 1;
            self.spinlock.unlock();
            return Ok(());
        }

        if self.thread_owner == SGX_THREAD_T_NULL &&
            (self.thread_vec.first() == Some(&rsgx_thread_self()) ||
            self.thread_vec.first() == None) {

            if self.thread_vec.first() == Some(&rsgx_thread_self()) {
                self.thread_vec.remove(0);
            }

            self.thread_owner = rsgx_thread_self();
            self.refcount += 1;
            self.spinlock.unlock();
            return Ok(());
        }
        self.spinlock.unlock();
        Err(libc::EBUSY)
    }

    unsafe fn unlock(&mut self) -> SysError {
        let mut thread_waiter = SGX_THREAD_T_NULL;
        self.unlock_lazy(&mut thread_waiter)?;

        if thread_waiter != SGX_THREAD_T_NULL /* wake the waiter up*/ {
            thread_set_event(SgxThreadData::from_raw(thread_waiter).get_tcs());
        }
        Ok(())
    }

    unsafe fn unlock_lazy(&mut self, waiter: &mut sgx_thread_t) -> SysError {
        self.spinlock.lock();
        //if the mutux is not locked by anyone
        if self.thread_owner == SGX_THREAD_T_NULL {
            self.spinlock.unlock();
            return Err(libc::EPERM);
        }

        //if the mutex is locked by another thread
        if self.thread_owner != rsgx_thread_self() {
            self.spinlock.unlock();
            return Err(libc::EPERM);
        }
        //the mutex is locked by current thread
        self.refcount -= 1;
        if self.refcount == 0 {
            self.thread_owner = SGX_THREAD_T_NULL;
        } else {
            self.spinlock.unlock();
            return Ok(());
        }
        //Before releasing the mutex, get the first thread,
        //the thread should be waked up by the caller.
        if self.thread_vec.is_empty() {
            *waiter = SGX_THREAD_T_NULL;
        } else {
            *waiter = *self.thread_vec.first().unwrap();
        }

        self.spinlock.unlock();
        Ok(())
    }

    unsafe fn destroy(&mut self) -> SysError {
        self.spinlock.lock();
        if self.thread_owner != SGX_THREAD_T_NULL || !self.thread_vec.is_empty() {
            self.spinlock.unlock();
            Err(libc::EBUSY)
        } else {
            self.control = SgxThreadMutexControl::SGX_THREAD_MUTEX_NONRECURSIVE;
            self.refcount = 0;
            self.spinlock.unlock();
            Ok(())
        }
    }
}

pub struct SgxThreadMutex {
    lock: UnsafeCell<SgxThreadMutexInner>,
}

impl SgxThreadMutex {
    pub const fn new(control: SgxThreadMutexControl) -> Self {
        SgxThreadMutex { 
            lock: UnsafeCell::new(SgxThreadMutexInner::new(control))
        }
    }

    #[inline]
    pub unsafe fn lock(&self) -> SysError {
        let mutex: &mut SgxThreadMutexInner = &mut *self.lock.get();
        mutex.lock()
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> SysError {
        let mutex: &mut SgxThreadMutexInner = &mut *self.lock.get();
        mutex.try_lock()
    }

    #[inline]
    pub unsafe fn unlock(&self) -> SysError {
        let mutex: &mut SgxThreadMutexInner = &mut *self.lock.get();
        mutex.unlock()
    }

    #[inline]
    pub unsafe fn unlock_lazy(&self, waiter: &mut sgx_thread_t) -> SysError {
        let mutex: &mut SgxThreadMutexInner = &mut *self.lock.get();
        mutex.unlock_lazy(waiter)
    }

    #[inline]
    pub unsafe fn destroy(&self) -> SysError {
        let mutex: &mut SgxThreadMutexInner = &mut *self.lock.get();
        mutex.destroy()
    }
}
