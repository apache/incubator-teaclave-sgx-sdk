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

use crate::linux::*;
use core::mem;
use core::ptr;
use sgx_sync::capi::*;
use sgx_trts::capi::*;
use sgx_trts::thread::tls::{Key, Tls};
use sgx_trts::thread::{self, Native, Thread};
use sgx_trts::trts::is_within_enclave;
use sgx_types::error::SgxStatus;

pub type pthread_t = *const c_void;
pub type pthread_key_t = size_t;

pub type pthread_mutex_t = sgx_thread_mutex_t;
pub type pthread_cond_t = sgx_thread_cond_t;
pub type pthread_rwlock_t = sgx_thread_rwlock_t;
pub type pthread_mutexattr_t = sgx_thread_mutexattr_t;
pub type pthread_condattr_t = sgx_thread_condattr_t;
pub type pthread_rwlockattr_t = sgx_thread_rwlockattr_t;
pub type pthread_attr_t = *mut pthread_attr;

#[repr(C)]
pub struct pthread_once_t {
    pub state: c_int,
    pub mutex: pthread_mutex_t,
}
s! {
    pub struct pthread_attr {
        pub reserved: c_char,
    }
}

macro_rules! check_param {
    ($ptr:expr, $ty:ty) => {
        if $ptr.is_null() || !is_within_enclave($ptr as *const u8, mem::size_of::<$ty>()) {
            return EINVAL;
        }
    };
}

pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t = SGX_THREAD_NONRECURSIVE_MUTEX_INITIALIZER;
pub const PTHREAD_COND_INITIALIZER: pthread_cond_t = SGX_THREAD_COND_INITIALIZER;
pub const PTHREAD_RWLOCK_INITIALIZER: pthread_rwlock_t = SGX_THREAD_LOCK_INITIALIZER;

const PTHREAD_NEEDS_INIT: c_int = 0;
const PTHREAD_DONE_INIT: c_int = 1;
pub const PTHREAD_ONCE_INIT: pthread_once_t = pthread_once_t {
    state: PTHREAD_NEEDS_INIT,
    mutex: PTHREAD_MUTEX_INITIALIZER,
};

#[allow(clippy::redundant_closure)]
#[no_mangle]
pub unsafe extern "C" fn pthread_create(
    thread: *mut pthread_t,
    _attr: *const pthread_attr_t,
    start_routine: extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    check_param!(thread, pthread_t);

    if !is_within_enclave(start_routine as *const u8, 0) {
        return EINVAL;
    }

    let f = move |arg| start_routine(arg);
    match Thread::new(f, arg) {
        Ok(t) => {
            *thread = Thread::into_raw(t) as pthread_t;
            0
        }
        Err(e) => match e {
            SgxStatus::InvalidParameter => EINVAL,
            SgxStatus::OutOfTcs => EAGAIN,
            _ => EINVAL,
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_join(thread: pthread_t, retval: *mut *mut c_void) -> c_int {
    check_param!(thread, Native);

    let t = Thread::from_raw(thread as *mut Native);
    match t.join() {
        Ok(val) => {
            *retval = val;
            0
        }
        Err(e) => match e {
            SgxStatus::InvalidParameter => EDEADLK,
            SgxStatus::InvalidState => EDEADLK,
            _ => EINVAL,
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_self() -> pthread_t {
    if let Some(t) = thread::current() {
        Thread::into_raw(t) as pthread_t
    } else {
        sgx_thread_self()
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_equal(t1: pthread_t, t2: pthread_t) -> c_int {
    i32::from(t1 == t2)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_key_create(
    key: *mut pthread_key_t,
    dtor: Option<unsafe extern "C" fn(*mut c_void)>,
) -> c_int {
    check_param!(key, pthread_key_t);

    match Tls::create(dtor.map(|f| mem::transmute(f))) {
        Ok(tls_key) => {
            *key = tls_key.as_usize();
            0
        }
        Err(e) => match e {
            SgxStatus::InvalidParameter => EINVAL,
            SgxStatus::Unexpected => EAGAIN,
            _ => EINVAL,
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_key_delete(key: pthread_key_t) -> c_int {
    if let Some(tls_key) = Key::from_usize(key) {
        Tls::destroy(tls_key);
        0
    } else {
        EINVAL
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getspecific(key: pthread_key_t) -> *mut c_void {
    if let Some(tls_key) = Key::from_usize(key) {
        Tls::get(tls_key).unwrap_or(None).unwrap_or(ptr::null_mut()) as *mut c_void
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_setspecific(key: pthread_key_t, value: *const c_void) -> c_int {
    if let Some(tls_key) = Key::from_usize(key) {
        if Tls::set(tls_key, value as *mut u8).is_ok() {
            0
        } else {
            EINVAL
        }
    } else {
        EINVAL
    }
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut pthread_mutex_t,
    _attr: *const pthread_mutexattr_t,
) -> c_int {
    sgx_thread_mutex_init(mutex, _attr)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    sgx_thread_mutex_lock(mutex)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    sgx_thread_mutex_trylock(mutex)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    sgx_thread_mutex_unlock(mutex)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int {
    sgx_thread_mutex_destroy(mutex)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_init(
    cond: *mut pthread_cond_t,
    _attr: *const pthread_condattr_t,
) -> c_int {
    sgx_thread_cond_init(cond, _attr)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_wait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
) -> c_int {
    sgx_thread_cond_wait(cond, mutex)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    timeout: *const timespec,
) -> c_int {
    sgx_thread_cond_timedwait(cond, mutex, timeout)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    sgx_thread_cond_signal(cond)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    sgx_thread_cond_broadcast(cond)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    sgx_thread_cond_destroy(cond)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_init(
    rwlock: *mut pthread_rwlock_t,
    _attr: *const pthread_rwlockattr_t,
) -> c_int {
    sgx_thread_rwlock_init(rwlock, _attr)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    sgx_thread_rwlock_rdlock(rwlock)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    sgx_thread_rwlock_tryrdlock(rwlock)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    sgx_thread_rwlock_wrlock(rwlock)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    sgx_thread_rwlock_trywrlock(rwlock)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    sgx_thread_rwlock_unlock(rwlock)
}

#[inline]
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_destroy(rwlock: *mut pthread_rwlock_t) -> c_int {
    sgx_thread_rwlock_destroy(rwlock)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_once(
    once_control: *mut pthread_once_t,
    init_routine: extern "C" fn(),
) -> c_int {
    check_param!(once_control, pthread_once_t);

    let once_control = &mut *once_control;
    pthread_mutex_lock(&mut once_control.mutex);
    if once_control.state == PTHREAD_NEEDS_INIT {
        init_routine();
        once_control.state = PTHREAD_DONE_INIT;
    }
    pthread_mutex_unlock(&mut once_control.mutex);
    0
}
