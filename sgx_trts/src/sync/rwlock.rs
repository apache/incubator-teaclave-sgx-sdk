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

use crate::sync::lock_api::RawRwLock;
use core::cell::UnsafeCell;
use core::convert::From;
use core::fmt;
use core::hint::spin_loop;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicUsize, Ordering};
use sgx_types::marker::ContiguousMemory;

pub struct SpinRwLock<T: ?Sized> {
    lock: AtomicUsize,
    data: UnsafeCell<T>,
}

const READER: usize = 1 << 1;
const WRITER: usize = 1;

unsafe impl<T: ContiguousMemory> ContiguousMemory for SpinRwLock<T> {}

unsafe impl<T: ?Sized + Send> Send for SpinRwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for SpinRwLock<T> {}

pub struct SpinRwLockReadGuard<'a, T: 'a + ?Sized> {
    lock: &'a AtomicUsize,
    data: &'a T,
}

impl<T: ?Sized> !Send for SpinRwLockReadGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Sync for SpinRwLockReadGuard<'_, T> {}

pub struct SpinRwLockWriteGuard<'a, T: ?Sized + 'a> {
    inner: &'a SpinRwLock<T>,
    data: &'a mut T,
}

impl<T: ?Sized> !Send for SpinRwLockWriteGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Sync for SpinRwLockWriteGuard<'_, T> {}

impl<T> SpinRwLock<T> {
    #[inline]
    pub const fn new(data: T) -> SpinRwLock<T> {
        SpinRwLock {
            lock: AtomicUsize::new(0),
            data: UnsafeCell::new(data),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let SpinRwLock { data, .. } = self;
        data.into_inner()
    }
}

impl<T: ?Sized> SpinRwLock<T> {
    #[inline]
    pub fn read(&self) -> SpinRwLockReadGuard<T> {
        loop {
            match self.try_read() {
                Some(guard) => return guard,
                None => spin_loop(),
            }
        }
    }

    #[inline]
    pub fn write(&self) -> SpinRwLockWriteGuard<T> {
        loop {
            match self.try_write_internal(false) {
                Some(guard) => return guard,
                None => spin_loop(),
            }
        }
    }

    #[inline]
    pub fn try_read(&self) -> Option<SpinRwLockReadGuard<T>> {
        let value = self.lock.fetch_add(READER, Ordering::Acquire);
        if value & WRITER != 0 {
            self.lock.fetch_sub(READER, Ordering::Release);
            None
        } else {
            Some(SpinRwLockReadGuard {
                lock: &self.lock,
                data: unsafe { &*self.data.get() },
            })
        }
    }

    #[inline]
    pub fn try_write(&self) -> Option<SpinRwLockWriteGuard<T>> {
        self.try_write_internal(true)
    }

    #[inline(always)]
    fn try_write_internal(&self, strong: bool) -> Option<SpinRwLockWriteGuard<T>> {
        if compare_exchange(
            &self.lock,
            0,
            WRITER,
            Ordering::Acquire,
            Ordering::Relaxed,
            strong,
        )
        .is_ok()
        {
            Some(SpinRwLockWriteGuard {
                inner: self,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    #[inline]
    unsafe fn force_read_unlock(&self) {
        debug_assert!(self.lock.load(Ordering::Relaxed) & !WRITER > 0);
        self.lock.fetch_sub(READER, Ordering::Release);
    }

    #[inline]
    unsafe fn force_write_unlock(&self) {
        debug_assert_eq!(self.lock.load(Ordering::Relaxed) & !WRITER, 0);
        self.lock.fetch_and(!WRITER, Ordering::Release);
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SpinRwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_read() {
            Some(guard) => write!(f, "SpinRwLock {{ value: ")
                .and_then(|()| (&*guard).fmt(f))
                .and_then(|()| write!(f, "}}")),
            None => write!(f, "SpinRwLock {{ <locked> }}"),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for SpinRwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpinRwLockReadGuard")
            .field("lock", &self.lock)
            .finish()
    }
}

impl<T: fmt::Debug> fmt::Debug for SpinRwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpinRwLockWriteGuard")
            .field("lock", &self.inner)
            .finish()
    }
}

impl<T: ?Sized + Default> Default for SpinRwLock<T> {
    fn default() -> SpinRwLock<T> {
        SpinRwLock::new(Default::default())
    }
}

impl<T> From<T> for SpinRwLock<T> {
    fn from(value: T) -> SpinRwLock<T> {
        SpinRwLock::new(value)
    }
}

impl<'rwlock, T: ?Sized> Deref for SpinRwLockReadGuard<'rwlock, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<'rwlock, T: ?Sized> Deref for SpinRwLockWriteGuard<'rwlock, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<'rwlock, T: ?Sized> DerefMut for SpinRwLockWriteGuard<'rwlock, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<T: ?Sized> Drop for SpinRwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        debug_assert!(self.lock.load(Ordering::Relaxed) & !WRITER > 0);
        self.lock.fetch_sub(READER, Ordering::Release);
    }
}

impl<T: ?Sized> Drop for SpinRwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        debug_assert_eq!(self.inner.lock.load(Ordering::Relaxed) & WRITER, WRITER);
        self.inner.lock.fetch_and(!WRITER, Ordering::Release);
    }
}

#[inline(always)]
fn compare_exchange(
    atomic: &AtomicUsize,
    current: usize,
    new: usize,
    success: Ordering,
    failure: Ordering,
    strong: bool,
) -> Result<usize, usize> {
    if strong {
        atomic.compare_exchange(current, new, success, failure)
    } else {
        atomic.compare_exchange_weak(current, new, success, failure)
    }
}

impl RawRwLock for SpinRwLock<()> {
    #[inline(always)]
    fn read(&self) {
        mem::forget(self.read());
    }

    #[inline(always)]
    fn try_read(&self) -> bool {
        self.try_read().map(mem::forget).is_some()
    }

    #[inline]
    unsafe fn read_unlock(&self) {
        drop(SpinRwLockReadGuard {
            lock: &self.lock,
            data: &(),
        });
    }

    #[inline(always)]
    fn write(&self) {
        core::mem::forget(self.write());
    }

    #[inline]
    fn try_write(&self) -> bool {
        self.try_write().map(mem::forget).is_some()
    }

    unsafe fn write_unlock(&self) {
        drop(SpinRwLockWriteGuard {
            inner: self,
            data: &mut (),
        });
    }
}
