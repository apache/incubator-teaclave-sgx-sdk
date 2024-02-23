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

use crate::sync::{SpinMutex, SpinMutexGuard};
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};
use sgx_types::error::SgxResult;

pub struct Once<T = ()> {
    lock: SpinMutex<()>,
    state: AtomicUsize,
    data: UnsafeCell<MaybeUninit<T>>,
}

impl<T> Default for Once<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T: Send + Sync> Sync for Once<T> {}
unsafe impl<T: Send> Send for Once<T> {}

const INCOMPLETE: usize = 0x0;
const COMPLETE: usize = 0x1;

impl<T> Once<T> {
    /// Initialization constant of [`Once`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const INIT: Self = Self {
        lock: SpinMutex::new(()),
        state: AtomicUsize::new(INCOMPLETE),
        data: UnsafeCell::new(MaybeUninit::uninit()),
    };

    /// Creates a new [`Once`].
    pub const fn new() -> Self {
        Self::INIT
    }

    pub fn lock(&self) -> SpinMutexGuard<()> {
        self.lock.lock()
    }

    pub fn call_once<F>(&self, init: F) -> SgxResult<&T>
    where
        F: FnOnce() -> SgxResult<T>,
    {
        if self.is_completed() {
            return Ok(unsafe {
                // SAFETY: The status is Complete
                self.force_get()
            });
        }

        let _guard = self.lock.lock();
        if !self.is_completed() {
            let val = init()?;
            unsafe {
                (*self.data.get()).as_mut_ptr().write(val);
            }
            self.state.store(COMPLETE, Ordering::Release);
        }
        unsafe { Ok(self.force_get()) }
    }

    /// Returns a reference to the inner value if the [`Once`] has been initialized.
    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) == COMPLETE {
            Some(unsafe { self.force_get() })
        } else {
            None
        }
    }

    /// Get a reference to the initialized instance. Must only be called once COMPLETE.
    unsafe fn force_get(&self) -> &T {
        // SAFETY:
        // * `UnsafeCell`/inner deref: data never changes again
        // * `MaybeUninit`/outer deref: data was initialized
        &*(*self.data.get()).as_ptr()
    }

    /// Get a reference to the initialized instance. Must only be called once COMPLETE.
    unsafe fn force_get_mut(&mut self) -> &mut T {
        // SAFETY:
        // * `UnsafeCell`/inner deref: data never changes again
        // * `MaybeUninit`/outer deref: data was initialized
        &mut *(*self.data.get()).as_mut_ptr()
    }

    #[inline]
    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == COMPLETE
    }
}
