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

use crate::sys::ocall;
use alloc::boxed::Box;
use alloc::collections::LinkedList;
use core::cell::UnsafeCell;
use sgx_trts::sync::SpinMutex;
use sgx_trts::tcs::{self, TcsId};
use sgx_types::error::errno::{EBUSY, EPERM};
use sgx_types::error::OsResult;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MutexControl {
    NonRecursive,
    Recursive,
}

struct Inner {
    refcount: usize,
    control: MutexControl,
    owner: Option<TcsId>,
    queue: LinkedList<TcsId>,
}

impl Inner {
    const fn new(control: MutexControl) -> Inner {
        Inner {
            refcount: 0,
            control,
            owner: None,
            queue: LinkedList::new(),
        }
    }
}

struct MutexInner {
    inner: SpinMutex<Inner>,
}

impl MutexInner {
    const fn new(control: MutexControl) -> MutexInner {
        MutexInner {
            inner: SpinMutex::new(Inner::new(control)),
        }
    }

    unsafe fn lock(&mut self) -> OsResult {
        let current = tcs::current().id();

        loop {
            let mut inner_guard = self.inner.lock();
            if inner_guard.control == MutexControl::Recursive && inner_guard.owner == Some(current)
            {
                inner_guard.refcount += 1;
                return Ok(());
            }

            if inner_guard.owner.is_none()
                && (inner_guard.queue.front() == Some(&current)
                    || inner_guard.queue.front().is_none())
            {
                if inner_guard.queue.front() == Some(&current) {
                    inner_guard.queue.pop_front();
                }

                inner_guard.owner = Some(current);
                inner_guard.refcount += 1;
                return Ok(());
            }

            if !inner_guard.queue.contains(&current) {
                inner_guard.queue.push_back(current);
            }

            drop(inner_guard);
            let _ = ocall::thread_wait_event(current, None);
        }
    }

    unsafe fn try_lock(&mut self) -> OsResult {
        let current = tcs::current().id();

        let mut inner_guard = self.inner.lock();
        if inner_guard.control == MutexControl::Recursive && inner_guard.owner == Some(current) {
            inner_guard.refcount += 1;
            return Ok(());
        }

        if inner_guard.owner.is_none()
            && (inner_guard.queue.front() == Some(&current) || inner_guard.queue.front().is_none())
        {
            if inner_guard.queue.front() == Some(&current) {
                inner_guard.queue.pop_front();
            }

            inner_guard.owner = Some(current);
            inner_guard.refcount += 1;
            return Ok(());
        }
        Err(EBUSY)
    }

    unsafe fn unlock(&mut self) -> OsResult {
        if let Some(waiter) = self.unlock_lazy()? {
            let _ = ocall::thread_set_event(waiter);
        }
        Ok(())
    }

    unsafe fn unlock_lazy(&mut self) -> OsResult<Option<TcsId>> {
        let mut inner_guard = self.inner.lock();

        if inner_guard.owner == Some(tcs::current().id()) {
            // the mutex is locked by current thread

            inner_guard.refcount -= 1;
            if inner_guard.refcount == 0 {
                inner_guard.owner = None;
            } else {
                return Ok(None);
            }
            // Before releasing the mutex, get the first thread,
            // the thread should be waked up by the caller.
            let waiter = if inner_guard.queue.is_empty() {
                None
            } else {
                inner_guard.queue.front().cloned()
            };
            Ok(waiter)
        } else {
            // mutux is not locked by anyone
            // the mutex is locked by another thread
            Err(EPERM)
        }
    }

    unsafe fn destroy(&mut self) -> OsResult {
        let mut inner_guard = self.inner.lock();

        if inner_guard.owner.is_none() && inner_guard.queue.is_empty() {
            inner_guard.control = MutexControl::NonRecursive;
            inner_guard.refcount = 0;
            Ok(())
        } else {
            Err(EBUSY)
        }
    }
}

pub type MovableMutex = Box<Mutex>;

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

pub struct Mutex {
    inner: UnsafeCell<MutexInner>,
}

impl Mutex {
    pub const fn new() -> Mutex {
        Mutex {
            inner: UnsafeCell::new(MutexInner::new(MutexControl::NonRecursive)),
        }
    }

    #[inline]
    pub unsafe fn lock(&self) -> OsResult {
        let mutex = &mut *self.inner.get();
        mutex.lock()
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> OsResult {
        let mutex = &mut *self.inner.get();
        mutex.try_lock()
    }

    #[inline]
    pub unsafe fn unlock(&self) -> OsResult {
        let mutex = &mut *self.inner.get();
        mutex.unlock()
    }

    #[inline]
    pub unsafe fn unlock_lazy(&self) -> OsResult<Option<TcsId>> {
        let mutex = &mut *self.inner.get();
        mutex.unlock_lazy()
    }

    #[inline]
    pub unsafe fn destroy(&self) -> OsResult {
        let mutex = &mut *self.inner.get();
        mutex.destroy()
    }
}

pub struct ReentrantMutex {
    inner: UnsafeCell<MutexInner>,
}

impl ReentrantMutex {
    pub const fn new() -> ReentrantMutex {
        ReentrantMutex {
            inner: UnsafeCell::new(MutexInner::new(MutexControl::Recursive)),
        }
    }

    #[inline]
    pub unsafe fn lock(&self) -> OsResult {
        let mutex = &mut *self.inner.get();
        mutex.lock()
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> OsResult {
        let mutex = &mut *self.inner.get();
        mutex.try_lock()
    }

    #[inline]
    pub unsafe fn unlock(&self) -> OsResult {
        let mutex = &mut *self.inner.get();
        mutex.unlock()
    }

    #[inline]
    pub unsafe fn destroy(&self) -> OsResult {
        let mutex = &mut *self.inner.get();
        mutex.destroy()
    }
}
