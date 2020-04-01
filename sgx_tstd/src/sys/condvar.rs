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

use sgx_types::{SysError, sgx_thread_t, SGX_THREAD_T_NULL};
use sgx_trts::enclave::SgxThreadData;
use sgx_trts::libc;
use sgx_libc::{c_void, c_int, c_long};
use core::cell::UnsafeCell;
use crate::io::{self, Error};
use crate::sys::mutex::{self, SgxThreadMutex};
use crate::sync::SgxThreadSpinlock;
use crate::time::Duration;
use crate::thread::{self, rsgx_thread_self};
use crate::u64;

struct SgxThreadCondvarInner {
    spinlock: SgxThreadSpinlock,
    thread_vec: Vec<sgx_thread_t>,
}

impl SgxThreadCondvarInner {

    pub const fn new() -> Self {
        SgxThreadCondvarInner {
            spinlock: SgxThreadSpinlock::new(),
            thread_vec: Vec::new(),
        }
    }

    pub unsafe fn wait(&mut self, mutex: &SgxThreadMutex) -> SysError {
        self.spinlock.lock();
        self.thread_vec.push(rsgx_thread_self());
        let mut waiter: sgx_thread_t = SGX_THREAD_T_NULL;

        mutex.unlock_lazy(&mut waiter).map_err(|ret| {
            self.thread_vec.pop();
            self.spinlock.unlock();
            ret
        })?;

        loop {
            self.spinlock.unlock();
            if waiter == SGX_THREAD_T_NULL {
                mutex::thread_wait_event(SgxThreadData::current().get_tcs(), Duration::new(u64::MAX, 1_000_000_000 - 1));
            } else {
                mutex::thread_setwait_events(SgxThreadData::from_raw(waiter).get_tcs(),
                                             SgxThreadData::current().get_tcs(),
                                             Duration::new(u64::MAX, 1_000_000_000 - 1));
                waiter = SGX_THREAD_T_NULL;
            }
            self.spinlock.lock();
            let mut thread_waiter: sgx_thread_t = SGX_THREAD_T_NULL;
            for tmp in &self.thread_vec {
                if thread::rsgx_thread_equal(*tmp, rsgx_thread_self()) {
                    thread_waiter = *tmp;
                    break;
                }
            }
            if thread_waiter == SGX_THREAD_T_NULL {
                break;
            }
        }
        self.spinlock.unlock();
        mutex.lock();
        Ok(())
    }

    pub unsafe fn wait_timeout(&mut self, mutex: &SgxThreadMutex, dur: Duration) -> SysError {
        self.spinlock.lock();
        self.thread_vec.push(rsgx_thread_self());
        let mut waiter: sgx_thread_t = SGX_THREAD_T_NULL;

        mutex.unlock_lazy(&mut waiter).map_err(|ret| {
            self.thread_vec.pop();
            self.spinlock.unlock();
            ret
        })?;
        let mut ret = Ok(());
        loop {
            self.spinlock.unlock();
            let mut result = 0;
            if waiter == SGX_THREAD_T_NULL {
                result = mutex::thread_wait_event(SgxThreadData::current().get_tcs(), dur);
            } else {
                result = mutex::thread_setwait_events(SgxThreadData::from_raw(waiter).get_tcs(),
                                                      SgxThreadData::current().get_tcs(),
                                                      dur);
                waiter = SGX_THREAD_T_NULL;
            }

            self.spinlock.lock();
            let mut thread_waiter: sgx_thread_t = SGX_THREAD_T_NULL;
            for tmp in &self.thread_vec {
                if thread::rsgx_thread_equal(*tmp, rsgx_thread_self()) {
                    thread_waiter = *tmp;
                    break;
                }
            }

            if thread_waiter != SGX_THREAD_T_NULL && result < 0 {
                if Error::last_os_error().kind() == io::ErrorKind::TimedOut {
                    self.thread_vec.remove_item(&thread_waiter);
                    ret = Err(libc::ETIMEDOUT);
                    break;
                }
            }

            if thread_waiter == SGX_THREAD_T_NULL {
                break;
            }
        }
        self.spinlock.unlock();
        mutex.lock();
        ret
    }

    pub unsafe fn signal(&mut self) -> SysError {
        self.spinlock.lock();
        if self.thread_vec.is_empty() {
            self.spinlock.unlock();
            return Ok(());
        }

        let waiter: sgx_thread_t = *self.thread_vec.first().unwrap();
        self.thread_vec.remove(0);
        self.spinlock.unlock();
        mutex::thread_set_event(SgxThreadData::from_raw(waiter).get_tcs());
        Ok(())
    }

    pub unsafe fn broadcast(&mut self) -> SysError {
        self.spinlock.lock();
        if self.thread_vec.is_empty() {
            self.spinlock.unlock();
            return Ok(());
        }

        let mut tcs_vec: Vec<usize> = Vec::new();
        while let Some(waiter) = self.thread_vec.pop() {
           tcs_vec.push(SgxThreadData::from_raw(waiter).get_tcs())
        }
        self.spinlock.unlock();
        mutex::thread_set_multiple_events(tcs_vec.as_slice());
        Ok(())
    }

    pub unsafe fn notify_one(&mut self) -> SysError {
        self.signal()
    }

    pub unsafe fn notify_all(&mut self) -> SysError {
        self.broadcast()
    }

    pub unsafe fn destroy(&mut self) -> SysError {
        self.spinlock.lock();
        let ret = if self.thread_vec.is_empty() {
            Ok(())
        } else {
            Err(libc::EBUSY)
        };
        self.spinlock.unlock();
        ret
    }
}

pub struct SgxThreadCondvar {
    inner: UnsafeCell<SgxThreadCondvarInner>,
}

impl SgxThreadCondvar {

    pub const fn new() -> Self {
        SgxThreadCondvar { inner: UnsafeCell::new(SgxThreadCondvarInner::new()) }
    }

    #[inline]
    pub unsafe fn wait(&self, mutex: &SgxThreadMutex) -> SysError {
        let condvar: &mut SgxThreadCondvarInner = &mut *self.inner.get();
        condvar.wait(mutex)
        
    }

    #[inline]
    pub unsafe fn wait_timeout(&self, mutex: &SgxThreadMutex, dur: Duration) -> SysError {
        let condvar: &mut SgxThreadCondvarInner = &mut *self.inner.get();
        condvar.wait_timeout(mutex, dur)
    }

    #[inline]
    pub unsafe fn signal(&self) -> SysError {
        let condvar: &mut SgxThreadCondvarInner = &mut *self.inner.get();
        condvar.signal()
    }

    #[inline]
    pub unsafe fn broadcast(&self) -> SysError {
        let condvar: &mut SgxThreadCondvarInner = &mut *self.inner.get();
        condvar.broadcast()
    }

    #[inline]
    pub unsafe fn notify_one(&self) -> SysError {
        self.signal()
    }

    #[inline]
    pub unsafe fn notify_all(&self) -> SysError {
        self.broadcast()
    }

    #[inline]
    pub unsafe fn destroy(&self) -> SysError {
        let condvar: &mut SgxThreadCondvarInner = &mut *self.inner.get();
        condvar.destroy()
    }
}