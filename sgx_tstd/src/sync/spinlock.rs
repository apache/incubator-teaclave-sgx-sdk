// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use sgx_types::*;
use core::marker;
use core::cell::UnsafeCell;

unsafe fn raw_lock(lock: &mut sgx_spinlock_t) -> * mut sgx_spinlock_t {
    lock as * mut _
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

    sgx_spin_lock(raw_lock(lock));
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

    sgx_spin_unlock(raw_lock(lock));
}

pub struct SgxThreadSpinlock {
    lock: UnsafeCell<sgx_spinlock_t>,
}

unsafe impl Send for SgxThreadSpinlock {}
unsafe impl Sync for SgxThreadSpinlock {}

impl SgxThreadSpinlock {

    pub const fn new() -> Self {
        SgxThreadSpinlock{ lock: UnsafeCell::new(SGX_SPINLOCK_INITIALIZER) }
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

    pub fn new() -> Self {
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
        SgxSpinlockGuard{lock: lock}
    }
}

impl<'a> Drop for SgxSpinlockGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            self.lock.inner.unlock();
        }
    }
}