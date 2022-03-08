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
    value: UnsafeCell<T>,
}

const READER: usize = 1 << 1;
const WRITER: usize = 1;

unsafe impl<T: ContiguousMemory> ContiguousMemory for SpinRwLock<T> {}

unsafe impl<T: ?Sized + Send> Send for SpinRwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for SpinRwLock<T> {}

pub struct SpinRwLockReadGuard<'a, T: ?Sized + 'a> {
    lock: &'a SpinRwLock<T>,
}

impl<T: ?Sized> !Send for SpinRwLockReadGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Sync for SpinRwLockReadGuard<'_, T> {}

pub struct SpinRwLockWriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a SpinRwLock<T>,
}

impl<T: ?Sized> !Send for SpinRwLockWriteGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Sync for SpinRwLockWriteGuard<'_, T> {}

impl<T> SpinRwLock<T> {
    #[inline]
    pub const fn new(user_data: T) -> SpinRwLock<T> {
        SpinRwLock {
            lock: AtomicUsize::new(0),
            value: UnsafeCell::new(user_data),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let SpinRwLock { value, .. } = self;
        value.into_inner()
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
    pub fn try_read(&self) -> Option<SpinRwLockReadGuard<T>> {
        let value = self.lock.fetch_add(READER, Ordering::Acquire);
        if value & WRITER != 0 {
            self.lock.fetch_sub(READER, Ordering::Release);
            None
        } else {
            Some(SpinRwLockReadGuard { lock: self })
        }
    }

    #[inline]
    pub fn write(&self) -> SpinRwLockWriteGuard<T> {
        loop {
            match self.try_write() {
                Some(guard) => return guard,
                None => spin_loop(),
            }
        }
    }

    #[inline]
    pub fn try_write(&self) -> Option<SpinRwLockWriteGuard<T>> {
        if self
            .lock
            .compare_exchange(0, WRITER, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinRwLockWriteGuard { lock: self })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value.get() }
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
            .field("lock", &self.lock)
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

impl<T: ?Sized> Deref for SpinRwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T: ?Sized> Deref for SpinRwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T: ?Sized> DerefMut for SpinRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T: ?Sized> Drop for SpinRwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        debug_assert!(self.lock.lock.load(Ordering::Relaxed) & !WRITER > 0);
        self.lock.lock.fetch_sub(READER, Ordering::Release);
    }
}

impl<T: ?Sized> Drop for SpinRwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        debug_assert_eq!(self.lock.lock.load(Ordering::Relaxed) & WRITER, WRITER);
        self.lock.lock.fetch_and(!WRITER, Ordering::Release);
    }
}

impl RawRwLock for SpinRwLock<()> {
    #[inline]
    fn read(&self) {
        mem::forget(self.read());
    }

    #[inline]
    fn try_read(&self) -> bool {
        self.try_read().map(mem::forget).is_some()
    }

    #[inline]
    unsafe fn read_unlock(&self) {
        drop(SpinRwLockReadGuard { lock: self });
    }

    #[inline]
    fn write(&self) {
        mem::forget(self.write());
    }

    #[inline]
    fn try_write(&self) -> bool {
        self.try_write().map(mem::forget).is_some()
    }

    unsafe fn write_unlock(&self) {
        drop(SpinRwLockWriteGuard { lock: self });
    }
}
