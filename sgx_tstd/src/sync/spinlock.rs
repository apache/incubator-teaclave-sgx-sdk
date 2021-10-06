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

use core::marker;
use core::cell::UnsafeCell;
use sgx_types::{self, sgx_spinlock_t};

unsafe fn raw_lock(lock: &mut sgx_spinlock_t) -> *mut sgx_spinlock_t {
    lock as *mut _
}

///
/// The rsgx_spin_lock function acquires a spin lock within the enclave.
///
/// # Description
///
/// rsgx_spin_lock modifies the value of the spin lock by using compiler atomic
/// operations. If the lock is not available to be acquired, the thread will always
/// wait on the lock until it can be acquired successfully.
///
/// # Parameters
///
/// **lock**
///
/// The trusted spin lock object to be acquired.
///
/// # Requirements
///
/// Library: libsgx_tstdc.a
///
unsafe fn rsgx_spin_lock(lock: &mut sgx_spinlock_t) {
    sgx_types::sgx_spin_lock(raw_lock(lock));
}

///
/// The rsgx_spin_unlock function releases a spin lock within the enclave.
///
/// # Description
///
/// rsgx_spin_unlock resets the value of the spin lock, regardless of its current
/// state. This function simply assigns a value of zero to the lock, which indicates
/// the lock is released.
///
/// # Parameters
///
/// **lock**
///
/// The trusted spin lock object to be released.
///
/// # Requirements
///
/// Library: libsgx_tstdc.a
///
unsafe fn rsgx_spin_unlock(lock: &mut sgx_spinlock_t) {
    sgx_types::sgx_spin_unlock(raw_lock(lock));
}

pub struct SgxThreadSpinlock {
    lock: UnsafeCell<sgx_spinlock_t>,
}

unsafe impl Send for SgxThreadSpinlock {}
unsafe impl Sync for SgxThreadSpinlock {}

impl SgxThreadSpinlock {
    pub const fn new() -> SgxThreadSpinlock {
        SgxThreadSpinlock{ lock: UnsafeCell::new(sgx_types::SGX_SPINLOCK_INITIALIZER) }
    }

    pub unsafe fn lock(&self) {
        rsgx_spin_lock(&mut *self.lock.get());
    }

    pub unsafe fn unlock(&self) {
        rsgx_spin_unlock(&mut *self.lock.get());
    }
}


pub struct SgxSpinlock {
    inner: SgxThreadSpinlock,
}

unsafe impl Send for SgxSpinlock {}
unsafe impl Sync for SgxSpinlock {}

impl SgxSpinlock {
    pub const fn new() -> SgxSpinlock {
        SgxSpinlock{inner: SgxThreadSpinlock::new()}
    }

    pub fn lock(&self) -> SgxSpinlockGuard {
        unsafe {
            self.inner.lock();
            SgxSpinlockGuard::new(self)
        }
    }
}

impl Default for SgxSpinlock {
    fn default() -> SgxSpinlock {
        SgxSpinlock::new()
    }
}

pub struct SgxSpinlockGuard<'a> {
    lock: &'a SgxSpinlock,
}

impl<'a> !marker::Send for SgxSpinlockGuard<'a> { }

impl<'spinlock> SgxSpinlockGuard<'spinlock> {
    unsafe fn new(lock: &'spinlock SgxSpinlock) -> Self {
        SgxSpinlockGuard{ lock }
    }
}

impl<'a> Drop for SgxSpinlockGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            self.lock.inner.unlock();
        }
    }
}
