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

use crate::boxed::Box;
use crate::cell::UnsafeCell;
use crate::collections::LinkedList;
use crate::io::{self, Error};
use crate::sync::SgxThreadSpinlock;
use crate::sys_common::lazy_box::{LazyBox, LazyInit};
use crate::sys::locks::mutex::{self, Mutex};
use crate::thread::rsgx_thread_self;
use crate::time::Duration;
use crate::u64;

use sgx_libc as libc;
use sgx_trts::enclave::SgxThreadData;
use sgx_types::{sgx_thread_t, SysError, SGX_THREAD_T_NULL};

pub struct Condvar {
    inner: UnsafeCell<CondvarInner>,
}

pub type MovableCondvar = LazyBox<Condvar>;

unsafe impl Send for Condvar {}
unsafe impl Sync for Condvar {}

impl LazyInit for Condvar {
    fn init() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl Condvar {
    pub const fn new() -> Self {
        Condvar {
            inner: UnsafeCell::new(CondvarInner::new()),
        }
    }

    #[inline]
    pub unsafe fn wait(&self, mutex: &Mutex) -> SysError {
        let condvar = &mut *self.inner.get();
        condvar.wait(mutex)
    }

    #[inline]
    pub unsafe fn wait_timeout(&self, mutex: &Mutex, dur: Duration) -> SysError {
        let condvar = &mut *self.inner.get();
        condvar.wait_timeout(mutex, dur)
    }

    #[inline]
    pub unsafe fn notify_one(&self) -> SysError {
        let condvar = &mut *self.inner.get();
        condvar.notify_one()
    }

    #[inline]
    pub unsafe fn notify_all(&self) -> SysError {
        let condvar = &mut *self.inner.get();
        condvar.notify_all()
    }

    #[inline]
    pub unsafe fn destroy(&self) -> SysError {
        let condvar = &mut *self.inner.get();
        condvar.destroy()
    }
}

impl Drop for Condvar {
    #[inline]
    fn drop(&mut self) {
        let r = unsafe { self.destroy() };
        debug_assert_eq!(r, Ok(()));
    }
}

struct CondvarInner {
    lock: SgxThreadSpinlock,
    queue: LinkedList<sgx_thread_t>,
}

impl CondvarInner {
    pub const fn new() -> Self {
        CondvarInner {
            lock: SgxThreadSpinlock::new(),
            queue: LinkedList::new(),
        }
    }

    pub unsafe fn wait(&mut self, mutex: &Mutex) -> SysError {
        self.lock.lock();
        self.queue.push_back(rsgx_thread_self());
        let mut waiter: sgx_thread_t = SGX_THREAD_T_NULL;

        mutex.unlock_lazy(&mut waiter).map_err(|ret| {
            self.queue.pop_back();
            self.lock.unlock();
            ret
        })?;

        loop {
            self.lock.unlock();
            if waiter == SGX_THREAD_T_NULL {
                mutex::thread_wait_event(
                    SgxThreadData::current().get_tcs(),
                    Duration::new(u64::MAX, 1_000_000_000 - 1),
                );
            } else {
                mutex::thread_setwait_events(
                    SgxThreadData::from_raw(waiter).get_tcs(),
                    SgxThreadData::current().get_tcs(),
                    Duration::new(u64::MAX, 1_000_000_000 - 1),
                );
                waiter = SGX_THREAD_T_NULL;
            }
            self.lock.lock();

            if !self.queue.contains(&rsgx_thread_self()) {
                break;
            }
        }
        self.lock.unlock();
        mutex.lock();
        Ok(())
    }

    pub unsafe fn wait_timeout(&mut self, mutex: &Mutex, dur: Duration) -> SysError {
        self.lock.lock();
        self.queue.push_back(rsgx_thread_self());
        let mut waiter: sgx_thread_t = SGX_THREAD_T_NULL;

        mutex.unlock_lazy(&mut waiter).map_err(|ret| {
            self.queue.pop_back();
            self.lock.unlock();
            ret
        })?;
        let mut ret = Ok(());
        loop {
            self.lock.unlock();
            let mut result = 0;
            if waiter == SGX_THREAD_T_NULL {
                result = mutex::thread_wait_event(SgxThreadData::current().get_tcs(), dur);
            } else {
                result = mutex::thread_setwait_events(
                    SgxThreadData::from_raw(waiter).get_tcs(),
                    SgxThreadData::current().get_tcs(),
                    dur,
                );
                waiter = SGX_THREAD_T_NULL;
            }

            self.lock.lock();
            match self
                .queue
                .iter()
                .position(|&waiter| waiter == rsgx_thread_self())
            {
                Some(pos) => {
                    if result < 0 && Error::last_os_error().kind() == io::ErrorKind::TimedOut {
                        self.queue.remove(pos);
                        ret = Err(libc::ETIMEDOUT);
                        break;
                    }
                }
                None => break,
            }
        }
        self.lock.unlock();
        mutex.lock();
        ret
    }

    pub unsafe fn notify_one(&mut self) -> SysError {
        self.lock.lock();
        if self.queue.is_empty() {
            self.lock.unlock();
            return Ok(());
        }

        let waiter: sgx_thread_t = *self.queue.front().unwrap();
        self.queue.pop_front();
        self.lock.unlock();
        mutex::thread_set_event(SgxThreadData::from_raw(waiter).get_tcs());
        Ok(())
    }

    pub unsafe fn notify_all(&mut self) -> SysError {
        self.lock.lock();
        if self.queue.is_empty() {
            self.lock.unlock();
            return Ok(());
        }

        let mut tcs_vec: Vec<usize> = Vec::new();
        while let Some(waiter) = self.queue.pop_back() {
            tcs_vec.push(SgxThreadData::from_raw(waiter).get_tcs())
        }
        self.lock.unlock();
        mutex::thread_set_multiple_events(tcs_vec.as_slice());
        Ok(())
    }

    pub unsafe fn destroy(&mut self) -> SysError {
        self.lock.lock();
        let ret = if self.queue.is_empty() {
            Ok(())
        } else {
            Err(libc::EBUSY)
        };
        self.lock.unlock();
        ret
    }
}
