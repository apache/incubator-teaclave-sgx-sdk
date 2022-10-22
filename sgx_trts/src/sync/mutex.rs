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

use crate::sync::lock_api::RawMutex;
use core::cell::UnsafeCell;
use core::convert::From;
use core::fmt;
use core::hint::spin_loop;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
use sgx_types::marker::ContiguousMemory;

pub struct SpinMutex<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: ContiguousMemory> ContiguousMemory for SpinMutex<T> {}

unsafe impl<T: ?Sized + Send> Sync for SpinMutex<T> {}
unsafe impl<T: ?Sized + Send> Send for SpinMutex<T> {}

pub struct SpinMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}

impl<T: ?Sized> !Send for SpinMutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for SpinMutexGuard<'_, T> {}

impl<T> SpinMutex<T> {
    pub const fn new(data: T) -> Self {
        SpinMutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let SpinMutex { data, .. } = self;
        data.into_inner()
    }
}

impl<T: ?Sized> SpinMutex<T> {
    #[inline]
    pub fn lock(&self) -> SpinMutexGuard<'_, T> {
        while self
            .lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.is_locked() {
                spin_loop();
            }
        }

        SpinMutexGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn unlock(guard: SpinMutexGuard<'_, T>) {
        drop(guard);
    }

    #[inline]
    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }

    #[inline]
    pub fn try_lock(&self) -> Option<SpinMutexGuard<T>> {
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinMutexGuard {
                lock: &self.lock,
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
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SpinMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "SpinMutex {{ value: ")
                .and_then(|()| (*guard).fmt(f))
                .and_then(|()| write!(f, "}}")),
            None => write!(f, "SpinMutex {{ <locked> }}"),
        }
    }
}

impl<T: ?Sized + Default> Default for SpinMutex<T> {
    fn default() -> SpinMutex<T> {
        SpinMutex::new(Default::default())
    }
}

impl<T> From<T> for SpinMutex<T> {
    fn from(value: T) -> SpinMutex<T> {
        SpinMutex::new(value)
    }
}

impl<T: ?Sized> Deref for SpinMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<T: ?Sized> DerefMut for SpinMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T: ?Sized> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SpinMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl RawMutex for SpinMutex<()> {
    #[inline]
    fn lock(&self) {
        mem::forget(SpinMutex::lock(self));
    }

    #[inline]
    fn try_lock(&self) -> bool {
        SpinMutex::try_lock(self).map(mem::forget).is_some()
    }

    #[inline]
    unsafe fn unlock(&self) {
        self.force_unlock();
    }
}
