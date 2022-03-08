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
    value: UnsafeCell<T>,
}

unsafe impl<T: ContiguousMemory> ContiguousMemory for SpinMutex<T> {}

unsafe impl<T: ?Sized + Send> Sync for SpinMutex<T> {}
unsafe impl<T: ?Sized + Send> Send for SpinMutex<T> {}

pub struct SpinMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a SpinMutex<T>,
}

impl<T: ?Sized> !Send for SpinMutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for SpinMutexGuard<'_, T> {}

impl<T> SpinMutex<T> {
    pub const fn new(value: T) -> Self {
        SpinMutex {
            value: UnsafeCell::new(value),
            lock: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let SpinMutex { value, .. } = self;
        value.into_inner()
    }
}

impl<T: ?Sized> SpinMutex<T> {
    #[inline]
    pub fn lock(&self) -> SpinMutexGuard<'_, T> {
        loop {
            match self.try_lock() {
                None => {
                    while self.lock.load(Ordering::Relaxed) {
                        spin_loop()
                    }
                }
                Some(guard) => return guard,
            }
        }
    }

    #[inline]
    pub fn try_lock(&self) -> Option<SpinMutexGuard<'_, T>> {
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_ok()
        {
            Some(SpinMutexGuard { lock: self })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SpinMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "SpinMutex {{ value: ")
                .and_then(|()| (&*guard).fmt(f))
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
        unsafe { &*self.lock.value.get() }
    }
}

impl<T: ?Sized> DerefMut for SpinMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<'a, T: ?Sized> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.lock.store(false, Ordering::Release)
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
        drop(SpinMutexGuard { lock: self });
    }
}
