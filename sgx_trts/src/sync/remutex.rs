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

use crate::sync::lock_api::RawMutex;
use crate::tcs;
use core::cell::UnsafeCell;
use core::convert::From;
use core::fmt;
use core::hint;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use sgx_types::marker::ContiguousMemory;

pub struct SpinReentrantMutex<T: ?Sized> {
    lock: AtomicBool,
    owner: AtomicUsize, // tcs id
    count: UnsafeCell<u32>,
    data: UnsafeCell<T>,
}

unsafe impl<T: ContiguousMemory> ContiguousMemory for SpinReentrantMutex<T> {}

unsafe impl<T: ?Sized + Sync> Sync for SpinReentrantMutex<T> {}
unsafe impl<T: ?Sized + Send> Send for SpinReentrantMutex<T> {}

impl<T> SpinReentrantMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            owner: AtomicUsize::new(0),
            count: UnsafeCell::new(0),
            data: UnsafeCell::new(data),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let SpinReentrantMutex { data, .. } = self;
        data.into_inner()
    }
}

impl<T: ?Sized> SpinReentrantMutex<T> {
    #[inline]
    pub fn lock(&self) -> SpinReentrantMutexGuard<'_, T> {
        let current_thread = tcs::current().id().as_usize();
        if self.owner.load(Ordering::Relaxed) == current_thread {
            self.increment_count()
        } else {
            self.acquire_lock();
            self.owner.store(current_thread, Ordering::Relaxed);
            unsafe {
                assert_eq!(*self.count.get(), 0);
                *self.count.get() = 1;
            }
        }

        SpinReentrantMutexGuard { lock: self }
    }

    #[inline]
    pub fn try_lock(&self) -> Option<SpinReentrantMutexGuard<'_, T>> {
        if self.try_acquire_lock() {
            Some(SpinReentrantMutexGuard { lock: self })
        } else {
            None
        }
    }

    #[inline]
    pub fn unlock(guard: SpinReentrantMutexGuard<'_, T>) {
        drop(guard);
    }

    #[inline]
    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    #[inline]
    fn increment_count(&self) {
        unsafe {
            *self.count.get() = (*self.count.get())
                .checked_add(1)
                .expect("lock count overflow in reentrant mutex");
        }
    }

    #[inline]
    fn acquire_lock(&self) {
        while self
            .lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.is_locked() {
                hint::spin_loop();
            }
        }
    }

    #[inline]
    fn try_acquire_lock(&self) -> bool {
        self.lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SpinReentrantMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "SpinReentrantMutex {{ value: ")
                .and_then(|()| (*guard).fmt(f))
                .and_then(|()| write!(f, "}}")),
            None => write!(f, "SpinReentrantMutex {{ <locked> }}"),
        }
    }
}

impl<T: ?Sized + Default> Default for SpinReentrantMutex<T> {
    fn default() -> SpinReentrantMutex<T> {
        SpinReentrantMutex::new(Default::default())
    }
}

impl<T> From<T> for SpinReentrantMutex<T> {
    fn from(value: T) -> SpinReentrantMutex<T> {
        SpinReentrantMutex::new(value)
    }
}

impl<T> RawMutex for SpinReentrantMutex<T> {
    #[inline]
    fn lock(&self) {
        mem::forget(SpinReentrantMutex::lock(self));
    }

    #[inline]
    fn try_lock(&self) -> bool {
        SpinReentrantMutex::try_lock(self)
            .map(mem::forget)
            .is_some()
    }

    #[inline]
    unsafe fn unlock(&self) {
        self.force_unlock();
    }
}

pub struct SpinReentrantMutexGuard<'a, T: 'a + ?Sized> {
    lock: &'a SpinReentrantMutex<T>,
}

impl<T: ?Sized> !Send for SpinReentrantMutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for SpinReentrantMutexGuard<'_, T> {}

impl<T: ?Sized> Drop for SpinReentrantMutexGuard<'_, T> {
    fn drop(&mut self) {
        let remutx = self.lock;
        unsafe {
            *remutx.count.get() -= 1;
            if *remutx.count.get() == 0 {
                remutx.owner.store(0, Ordering::Relaxed);
                remutx.lock.store(false, Ordering::Release);
            }
        }
    }
}

impl<T: ?Sized> Deref for SpinReentrantMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for SpinReentrantMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SpinReentrantMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}
