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

use crate::sys::lazy_box::{LazyBox, LazyInit};
use crate::sys::ocall;
use alloc::boxed::Box;
use alloc::collections::LinkedList;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::mem;
use sgx_trts::sync::SpinMutex;
use sgx_trts::tcs::{self, TcsId};
use sgx_types::error::errno::{EBUSY, EDEADLK, EPERM};
use sgx_types::error::OsResult;

pub struct RwLock {
    inner: UnsafeCell<RwLockInner>,
}

pub type MovableRwLock = LazyBox<RwLock>;

unsafe impl Send for RwLock {}
unsafe impl Sync for RwLock {}

impl LazyInit for RwLock {
    fn init() -> Box<Self> {
        Box::new(Self::new())
    }

    fn destroy(rwlock: Box<Self>) {
        // We're not allowed to pthread_rwlock_destroy a locked rwlock,
        // so check first if it's unlocked.
        if unsafe { rwlock.is_locked() } {
            // The rwlock is locked. This happens if a RwLock{Read,Write}Guard is leaked.
            // In this case, we just leak the RwLock too.
            mem::forget(rwlock);
        }
    }

    fn cancel_init(_: Box<Self>) {
        // In this case, we can just drop it without any checks,
        // since it cannot have been locked yet.
    }
}

impl RwLock {
    pub const fn new() -> RwLock {
        RwLock {
            inner: UnsafeCell::new(RwLockInner::new()),
        }
    }

    #[inline]
    pub unsafe fn read(&self) -> OsResult {
        let rwlock = &mut *self.inner.get();
        rwlock.read()
    }

    #[inline]
    pub unsafe fn try_read(&self) -> OsResult {
        let rwlock = &mut *self.inner.get();
        rwlock.try_read()
    }

    #[inline]
    pub unsafe fn write(&self) -> OsResult {
        let rwlock = &mut *self.inner.get();
        rwlock.write()
    }

    #[inline]
    pub unsafe fn try_write(&self) -> OsResult {
        let rwlock = &mut *self.inner.get();
        rwlock.try_write()
    }

    #[inline]
    pub unsafe fn read_unlock(&self) -> OsResult {
        let rwlock = &mut *self.inner.get();
        rwlock.read_unlock()
    }

    #[inline]
    pub unsafe fn write_unlock(&self) -> OsResult {
        let rwlock = &mut *self.inner.get();
        rwlock.write_unlock()
    }

    #[allow(dead_code)]
    #[inline]
    pub unsafe fn unlock(&self) -> OsResult {
        let rwlock = &mut *self.inner.get();
        rwlock.unlock()
    }

    #[inline]
    pub unsafe fn destroy(&self) -> OsResult {
        let rwlock = &mut *self.inner.get();
        rwlock.destroy()
    }

    #[inline]
    unsafe fn is_locked(&self) -> bool {
        let rwlock = &*self.inner.get();
        rwlock.is_locked()
    }
}

impl Drop for RwLock {
    fn drop(&mut self) {
        let r = unsafe { self.destroy() };
        debug_assert_eq!(r, Ok(()));
    }
}

struct RwLockInner {
    inner: SpinMutex<Inner>,
}

struct Inner {
    reader_count: u32,
    writer_waiting: u32,
    owner: Option<TcsId>,
    reader_queue: LinkedList<TcsId>,
    writer_queue: LinkedList<TcsId>,
}

impl Inner {
    const fn new() -> Inner {
        Inner {
            reader_count: 0,
            writer_waiting: 0,
            owner: None,
            reader_queue: LinkedList::new(),
            writer_queue: LinkedList::new(),
        }
    }
}

impl RwLockInner {
    const fn new() -> RwLockInner {
        RwLockInner {
            inner: SpinMutex::new(Inner::new()),
        }
    }

    unsafe fn read(&mut self) -> OsResult {
        let current = tcs::current().id();

        let mut inner_guard = self.inner.lock();
        match inner_guard.owner {
            Some(ref id) if *id == current => Err(EDEADLK),
            None => {
                inner_guard.reader_count += 1;
                Ok(())
            }
            _ => {
                inner_guard.reader_queue.push_back(current);
                loop {
                    drop(inner_guard);
                    let _ = ocall::thread_wait_event(current, None);

                    inner_guard = self.inner.lock();
                    if inner_guard.owner.is_none() {
                        inner_guard.reader_count += 1;
                        if let Some(pos) = inner_guard
                            .reader_queue
                            .iter()
                            .position(|&waiter| waiter == current)
                        {
                            inner_guard.reader_queue.remove(pos);
                        }
                        break Ok(());
                    }
                }
            }
        }
    }

    unsafe fn try_read(&mut self) -> OsResult {
        let mut inner_guard = self.inner.lock();
        if inner_guard.owner.is_none() {
            inner_guard.reader_count += 1;
            Ok(())
        } else {
            Err(EBUSY)
        }
    }

    unsafe fn write(&mut self) -> OsResult {
        let current = tcs::current().id();

        let mut inner_guard = self.inner.lock();
        if inner_guard.owner.is_none() && inner_guard.reader_count == 0 {
            inner_guard.owner = Some(current);
        } else {
            if inner_guard.owner == Some(current) {
                return Err(EDEADLK);
            }

            inner_guard.writer_queue.push_back(current);
            loop {
                drop(inner_guard);
                let _ = ocall::thread_wait_event(current, None);

                inner_guard = self.inner.lock();
                if inner_guard.owner.is_none() && inner_guard.reader_count == 0 {
                    inner_guard.owner = Some(current);
                    if let Some(pos) = inner_guard
                        .writer_queue
                        .iter()
                        .position(|&waiter| waiter == current)
                    {
                        inner_guard.writer_queue.remove(pos);
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    unsafe fn try_write(&mut self) -> OsResult {
        let current = tcs::current().id();

        let mut inner_guard = self.inner.lock();
        if inner_guard.owner.is_none() && inner_guard.reader_count == 0 {
            inner_guard.owner = Some(current);
            Ok(())
        } else {
            Err(EBUSY)
        }
    }

    // remove a reader lock. If there are no more readers and there is a
    // writer waiting, then wake it up.
    unsafe fn read_unlock(&mut self) -> OsResult {
        let mut inner_guard = self.inner.lock();
        if inner_guard.reader_count == 0 {
            return Err(EPERM);
        }

        inner_guard.reader_count -= 1;
        if inner_guard.reader_count == 0 {
            let waiter = inner_guard.writer_queue.front();
            if let Some(waiter) = waiter.cloned() {
                drop(inner_guard);
                let _ = ocall::thread_set_event(waiter);
            }
        }
        Ok(())
    }

    // remove the writer lock. If there are reader threads waiting
    // then wake them, otherwise, if there are writer thread waiting,
    // then wake them.
    unsafe fn write_unlock(&mut self) -> OsResult {
        let current = tcs::current().id();

        let mut inner_guard = self.inner.lock();
        if inner_guard.owner != Some(current) {
            return Err(EPERM);
        }

        inner_guard.owner = None;
        if !inner_guard.reader_queue.is_empty() {
            let tcss: Vec<TcsId> = inner_guard.reader_queue.iter().copied().collect();
            drop(inner_guard);
            let _ = ocall::thread_set_multiple_events(&tcss);
        } else {
            let waiter = inner_guard.writer_queue.front();
            if let Some(waiter) = waiter.cloned() {
                drop(inner_guard);
                let _ = ocall::thread_set_event(waiter);
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    unsafe fn unlock(&mut self) -> OsResult {
        let result = self.write_unlock();
        match result {
            Err(e) if e == EPERM => self.read_unlock(),
            _ => result,
        }
    }

    unsafe fn destroy(&mut self) -> OsResult {
        if self.is_locked() {
            Err(EBUSY)
        } else {
            Ok(())
        }
    }

    unsafe fn is_locked(&self) -> bool {
        let inner_guard = self.inner.lock();
        inner_guard.owner.is_some()
            || inner_guard.reader_count != 0
            || inner_guard.writer_waiting != 0
            || !inner_guard.reader_queue.is_empty()
            || !inner_guard.writer_queue.is_empty()
    }
}
