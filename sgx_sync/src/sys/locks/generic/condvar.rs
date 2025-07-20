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

use super::mutex::Mutex;
use crate::sys::lazy_box::{LazyBox, LazyInit};
use crate::sys::ocall;
use alloc::boxed::Box;
use alloc::collections::LinkedList;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::time::Duration;
use sgx_trts::error::errno;
use sgx_trts::sync::SpinMutex;
use sgx_trts::tcs::{self, TcsId};
use sgx_types::error::errno::{EBUSY, ETIMEDOUT};
use sgx_types::error::OsResult;

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
    pub unsafe fn wait(&self, mutex: &Mutex) -> OsResult {
        let condvar = &mut *self.inner.get();
        condvar.wait(mutex)
    }

    #[inline]
    pub unsafe fn wait_timeout(&self, mutex: &Mutex, dur: Duration) -> OsResult {
        let condvar = &mut *self.inner.get();
        condvar.wait_timeout(mutex, dur)
    }

    #[inline]
    pub unsafe fn notify_one(&self) -> OsResult {
        let condvar = &mut *self.inner.get();
        condvar.notify_one()
    }

    #[inline]
    pub unsafe fn notify_all(&self) -> OsResult {
        let condvar = &mut *self.inner.get();
        condvar.notify_all()
    }

    #[inline]
    pub unsafe fn destroy(&self) -> OsResult {
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
    inner: SpinMutex<Inner>,
}
struct Inner {
    queue: LinkedList<TcsId>,
}

impl Inner {
    const fn new() -> Inner {
        Inner {
            queue: LinkedList::new(),
        }
    }
}

impl CondvarInner {
    pub const fn new() -> CondvarInner {
        CondvarInner {
            inner: SpinMutex::new(Inner::new()),
        }
    }

    pub unsafe fn wait(&mut self, mutex: &Mutex) -> OsResult {
        let current = tcs::current().id();

        let mut inner_guard = self.inner.lock();
        inner_guard.queue.push_back(current);

        let mut waiter = mutex.unlock_lazy().map_err(|ret| {
            inner_guard.queue.pop_back();
            ret
        })?;

        loop {
            drop(inner_guard);
            let _ = if let Some(waiter) = waiter.take() {
                ocall::thread_setwait_events(waiter, current, None)
            } else {
                ocall::thread_wait_event(current, None)
            };

            inner_guard = self.inner.lock();
            if !inner_guard.queue.contains(&current) {
                break;
            }
        }
        drop(inner_guard);
        let _ = mutex.lock();
        Ok(())
    }

    pub unsafe fn wait_timeout(&mut self, mutex: &Mutex, dur: Duration) -> OsResult {
        let current = tcs::current().id();

        let mut inner_guard = self.inner.lock();
        inner_guard.queue.push_back(current);

        let mut waiter = mutex.unlock_lazy().map_err(|ret| {
            inner_guard.queue.pop_back();
            ret
        })?;

        let ret = loop {
            drop(inner_guard);
            let result = if let Some(waiter) = waiter.take() {
                ocall::thread_setwait_events(waiter, current, Some(dur))
            } else {
                ocall::thread_wait_event(current, Some(dur))
            };

            inner_guard = self.inner.lock();
            match inner_guard
                .queue
                .iter()
                .position(|&waiter| waiter == current)
            {
                Some(pos) => {
                    if result.is_err() && errno() == ETIMEDOUT {
                        inner_guard.queue.remove(pos);
                        break Err(ETIMEDOUT);
                    }
                }
                None => break Ok(()),
            }
        };
        drop(inner_guard);
        let _ = mutex.lock();
        ret
    }

    pub unsafe fn notify_one(&mut self) -> OsResult {
        let mut inner_guard = self.inner.lock();
        if inner_guard.queue.is_empty() {
            return Ok(());
        }

        let waiter = inner_guard.queue.front().cloned().unwrap();
        inner_guard.queue.pop_front();
        drop(inner_guard);
        let _ = ocall::thread_set_event(waiter);
        Ok(())
    }

    pub unsafe fn notify_all(&mut self) -> OsResult {
        let mut inner_guard = self.inner.lock();
        if inner_guard.queue.is_empty() {
            return Ok(());
        }

        let mut tcss: Vec<TcsId> = Vec::new();
        while let Some(waiter) = inner_guard.queue.pop_back() {
            tcss.push(waiter)
        }
        drop(inner_guard);
        let _ = ocall::thread_set_multiple_events(tcss.as_slice());
        Ok(())
    }

    pub unsafe fn destroy(&mut self) -> OsResult {
        let inner_guard = self.inner.lock();
        if inner_guard.queue.is_empty() {
            Ok(())
        } else {
            Err(EBUSY)
        }
    }
}
