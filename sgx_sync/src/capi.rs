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

use crate::spin::SpinMutex;
use crate::sys::locks::generic::condvar::Condvar;
use crate::sys::locks::generic::mutex::{Mutex, MutexControl};
use crate::sys::locks::generic::rwlock::RwLock;
use alloc::boxed::Box;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr;
use core::time::Duration;
use sgx_trts::trts::is_within_enclave;
use sgx_types::error::errno::*;
use sgx_types::types::{c_int, c_void, timespec};

pub type sgx_thread_mutex_t = SgxThreadMutex;
pub type sgx_thread_cond_t = SgxThreadCond;
pub type sgx_thread_rwlock_t = SgxThreadRwlock;
pub type sgx_thread_mutexattr_t = SgxThreadMutexAttr;
pub type sgx_thread_condattr_t = SgxThreadCondAttr;
pub type sgx_thread_rwlockattr_t = SgxThreadRwlockAttr;

#[repr(C)]
pub struct SgxThreadMutex {
    pub control: u32,
    pub mutex: *mut c_void,
}

#[repr(C)]
pub struct SgxThreadCond {
    pub cond: *mut c_void,
}

#[repr(C)]
pub struct SgxThreadRwlock {
    pub rwlock: *mut c_void,
}

s! {
    #[derive(Default)]
    pub struct SgxThreadMutexAttr {
        pub dummy: u8,
    }

    #[derive(Default)]
    pub struct SgxThreadCondAttr {
        pub dummy: u8,
    }

    #[derive(Default)]
    pub struct SgxThreadRwlockAttr {
        pub dummy: u8,
    }
}

pub const SGX_THREAD_MUTEX_NONRECURSIVE: u32 = 0x01;
pub const SGX_THREAD_MUTEX_RECURSIVE: u32 = 0x02;

pub const SGX_THREAD_NONRECURSIVE_MUTEX_INITIALIZER: sgx_thread_mutex_t = SgxThreadMutex {
    control: SGX_THREAD_MUTEX_NONRECURSIVE,
    mutex: ptr::null_mut(),
};
pub const SGX_THREAD_RECURSIVE_MUTEX_INITIALIZER: sgx_thread_mutex_t = SgxThreadMutex {
    control: SGX_THREAD_MUTEX_RECURSIVE,
    mutex: ptr::null_mut(),
};
pub const SGX_THREAD_MUTEX_INITIALIZER: sgx_thread_mutex_t = SgxThreadMutex {
    control: SGX_THREAD_MUTEX_NONRECURSIVE,
    mutex: ptr::null_mut(),
};
pub const SGX_THREAD_COND_INITIALIZER: sgx_thread_cond_t = SgxThreadCond {
    cond: ptr::null_mut(),
};
pub const SGX_THREAD_LOCK_INITIALIZER: sgx_thread_rwlock_t = SgxThreadRwlock {
    rwlock: ptr::null_mut(),
};

static MUTEX_INIT_LOCK: SpinMutex<()> = SpinMutex::new(());
static CONDVAR_INIT_LOCK: SpinMutex<()> = SpinMutex::new(());
static RW_INIT_LOCK: SpinMutex<()> = SpinMutex::new(());

macro_rules! check_range {
    ($ptr:expr, $ty:ty) => {
        if !is_within_enclave($ptr as *const u8, mem::size_of::<$ty>()) {
            return EINVAL;
        }
    };
}

macro_rules! check_param {
    ($ptr:expr, $ty:ty) => {
        if $ptr.is_null() || !is_within_enclave($ptr as *const u8, mem::size_of::<$ty>()) {
            return EINVAL;
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_mutex_init(
    mutex: *mut sgx_thread_mutex_t,
    _attr: *const sgx_thread_mutexattr_t,
) -> c_int {
    check_param!(mutex, sgx_thread_mutex_t);

    let mutex = &mut *mutex;
    let control = if mutex.control == SGX_THREAD_MUTEX_NONRECURSIVE {
        MutexControl::NonRecursive
    } else if mutex.control == SGX_THREAD_MUTEX_RECURSIVE {
        MutexControl::Recursive
    } else {
        MutexControl::NonRecursive
    };

    let m = ManuallyDrop::new(Box::new(Mutex::new_with_control(control)));
    mutex.mutex = &**m as *const _ as *mut c_void;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_mutex_lock(mutex: *mut sgx_thread_mutex_t) -> c_int {
    check_param!(mutex, sgx_thread_mutex_t);

    let mutex = &mut *mutex;
    if mutex.mutex.is_null() {
        MUTEX_INIT_LOCK.lock();
        if sgx_thread_mutex_init(mutex as *mut sgx_thread_mutex_t, ptr::null_mut()) != 0 {
            return EINVAL;
        }
    }

    check_range!(mutex.mutex, Mutex);
    let m = ManuallyDrop::new(Box::from_raw(mutex.mutex as *mut Mutex));

    let result = m.lock();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_mutex_trylock(mutex: *mut sgx_thread_mutex_t) -> c_int {
    check_param!(mutex, sgx_thread_mutex_t);

    let mutex = &mut *mutex;
    if mutex.mutex.is_null() {
        MUTEX_INIT_LOCK.lock();
        if sgx_thread_mutex_init(mutex as *mut sgx_thread_mutex_t, ptr::null_mut()) != 0 {
            return EINVAL;
        }
    }

    check_range!(mutex.mutex, Mutex);
    let m = ManuallyDrop::new(Box::from_raw(mutex.mutex as *mut Mutex));

    let result = m.try_lock();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_mutex_unlock(mutex: *mut sgx_thread_mutex_t) -> c_int {
    check_param!(mutex, sgx_thread_mutex_t);

    let mutex = &mut *mutex;
    check_param!(mutex.mutex, Mutex);

    let m = ManuallyDrop::new(Box::from_raw(mutex.mutex as *mut Mutex));
    let result = m.unlock();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_mutex_destroy(mutex: *mut sgx_thread_mutex_t) -> c_int {
    check_param!(mutex, sgx_thread_mutex_t);

    let mutex = &mut *mutex;
    if !mutex.mutex.is_null() {
        check_range!(mutex.mutex, Mutex);
        let m = ManuallyDrop::new(Box::from_raw(mutex.mutex as *mut Mutex));
        let result = m.destroy();
        if result.is_ok() {
            mutex.mutex = ptr::null_mut();
            let _ = ManuallyDrop::into_inner(m);
        }
        result.err().unwrap_or(0)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_cond_init(
    cond: *mut sgx_thread_cond_t,
    _attr: *const sgx_thread_condattr_t,
) -> c_int {
    check_param!(cond, sgx_thread_cond_t);

    let c = ManuallyDrop::new(Box::new(Condvar::new()));
    (*cond).cond = &**c as *const _ as *mut c_void;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_cond_wait(
    cond: *mut sgx_thread_cond_t,
    mutex: *mut sgx_thread_mutex_t,
) -> c_int {
    check_param!(cond, sgx_thread_cond_t);
    check_param!(mutex, sgx_thread_mutex_t);

    let cond = &mut *cond;
    let mutex = &mut *mutex;
    check_param!(mutex.mutex, Mutex);

    if cond.cond.is_null() {
        CONDVAR_INIT_LOCK.lock();
        if sgx_thread_cond_init(cond as *mut sgx_thread_cond_t, ptr::null_mut()) != 0 {
            return EINVAL;
        }
    }

    check_range!(cond.cond, Condvar);
    let c = ManuallyDrop::new(Box::from_raw(cond.cond as *mut Condvar));
    let m = ManuallyDrop::new(Box::from_raw(mutex.mutex as *mut Mutex));

    let result = c.wait(m.as_ref());
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_cond_timedwait(
    cond: *mut sgx_thread_cond_t,
    mutex: *mut sgx_thread_mutex_t,
    timeout: *const timespec,
) -> c_int {
    check_param!(cond, sgx_thread_cond_t);
    check_param!(mutex, sgx_thread_mutex_t);
    if !timeout.is_null() {
        check_param!(timeout, timespec);
    }

    let cond = &mut *cond;
    let mutex = &mut *mutex;
    check_param!(mutex.mutex, Mutex);

    if cond.cond.is_null() {
        CONDVAR_INIT_LOCK.lock();
        if sgx_thread_cond_init(cond as *mut sgx_thread_cond_t, ptr::null_mut()) != 0 {
            return EINVAL;
        }
    }

    check_range!(cond.cond, Condvar);
    let c = ManuallyDrop::new(Box::from_raw(cond.cond as *mut Condvar));
    let m = ManuallyDrop::new(Box::from_raw(mutex.mutex as *mut Mutex));

    let result = if timeout.is_null() {
        c.wait(m.as_ref())
    } else {
        const NANOS_PER_SEC: i64 = 1_000_000_000;
        let timeout = &*timeout;
        let secs: u64 =
            match (timeout.tv_sec as u64).checked_add((timeout.tv_nsec / NANOS_PER_SEC) as u64) {
                Some(secs) => secs,
                None => return EINVAL,
            };
        let nanos = (timeout.tv_nsec % NANOS_PER_SEC) as u32;
        let duration = Duration::new(secs, nanos);
        c.wait_timeout(m.as_ref(), duration)
    };
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_cond_signal(cond: *mut sgx_thread_cond_t) -> c_int {
    check_param!(cond, sgx_thread_cond_t);

    let cond = &mut *cond;
    check_param!(cond.cond, Condvar);

    let c = ManuallyDrop::new(Box::from_raw(cond.cond as *mut Condvar));
    let result = c.notify_one();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_cond_broadcast(cond: *mut sgx_thread_cond_t) -> c_int {
    check_param!(cond, sgx_thread_cond_t);

    let cond = &mut *cond;
    check_param!(cond.cond, Condvar);

    let c = ManuallyDrop::new(Box::from_raw(cond.cond as *mut Condvar));
    let result = c.notify_all();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_cond_destroy(cond: *mut sgx_thread_cond_t) -> c_int {
    check_param!(cond, sgx_thread_cond_t);

    let cond = &mut *cond;
    if !cond.cond.is_null() {
        check_range!(cond.cond, Condvar);
        let c = ManuallyDrop::new(Box::from_raw(cond.cond as *mut Condvar));
        let result = c.destroy();
        if result.is_ok() {
            cond.cond = ptr::null_mut();
            let _ = ManuallyDrop::into_inner(c);
        }
        result.err().unwrap_or(0)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_rwlock_init(
    rwlock: *mut sgx_thread_rwlock_t,
    _attr: *const sgx_thread_rwlockattr_t,
) -> c_int {
    check_param!(rwlock, sgx_thread_rwlock_t);

    let rw = ManuallyDrop::new(Box::new(RwLock::new()));
    (*rwlock).rwlock = &**rw as *const _ as *mut c_void;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_rwlock_rdlock(rwlock: *mut sgx_thread_rwlock_t) -> c_int {
    check_param!(rwlock, sgx_thread_rwlock_t);

    let rwlock = &mut *rwlock;
    if rwlock.rwlock.is_null() {
        RW_INIT_LOCK.lock();
        if sgx_thread_rwlock_init(rwlock as *mut sgx_thread_rwlock_t, ptr::null_mut()) != 0 {
            return EINVAL;
        }
    }

    check_range!(rwlock.rwlock, RwLock);
    let rw = ManuallyDrop::new(Box::from_raw(rwlock.rwlock as *mut RwLock));

    let result = rw.read();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_rwlock_tryrdlock(rwlock: *mut sgx_thread_rwlock_t) -> c_int {
    check_param!(rwlock, sgx_thread_rwlock_t);

    let rwlock = &mut *rwlock;
    if rwlock.rwlock.is_null() {
        RW_INIT_LOCK.lock();
        if sgx_thread_rwlock_init(rwlock as *mut sgx_thread_rwlock_t, ptr::null_mut()) != 0 {
            return EINVAL;
        }
    }

    check_range!(rwlock.rwlock, RwLock);
    let rw = ManuallyDrop::new(Box::from_raw(rwlock.rwlock as *mut RwLock));

    let result = rw.try_read();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_rwlock_wrlock(rwlock: *mut sgx_thread_rwlock_t) -> c_int {
    check_param!(rwlock, sgx_thread_rwlock_t);

    let rwlock = &mut *rwlock;
    if rwlock.rwlock.is_null() {
        RW_INIT_LOCK.lock();
        if sgx_thread_rwlock_init(rwlock as *mut sgx_thread_rwlock_t, ptr::null_mut()) != 0 {
            return EINVAL;
        }
    }

    check_range!(rwlock.rwlock, RwLock);
    let rw = ManuallyDrop::new(Box::from_raw(rwlock.rwlock as *mut RwLock));

    let result = rw.write();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_rwlock_trywrlock(rwlock: *mut sgx_thread_rwlock_t) -> c_int {
    check_param!(rwlock, sgx_thread_rwlock_t);

    let rwlock = &mut *rwlock;
    if rwlock.rwlock.is_null() {
        RW_INIT_LOCK.lock();
        if sgx_thread_rwlock_init(rwlock as *mut sgx_thread_rwlock_t, ptr::null_mut()) != 0 {
            return EINVAL;
        }
    }

    check_range!(rwlock.rwlock, RwLock);
    let rw = ManuallyDrop::new(Box::from_raw(rwlock.rwlock as *mut RwLock));

    let result = rw.try_write();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_rwlock_unlock(rwlock: *mut sgx_thread_rwlock_t) -> c_int {
    check_param!(rwlock, sgx_thread_rwlock_t);

    let rwlock = &mut *rwlock;
    check_param!(rwlock.rwlock, RwLock);

    let rw = ManuallyDrop::new(Box::from_raw(rwlock.rwlock as *mut RwLock));
    let result = rw.unlock();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_rwlock_rdunlock(rwlock: *mut sgx_thread_rwlock_t) -> c_int {
    check_param!(rwlock, sgx_thread_rwlock_t);

    let rwlock = &mut *rwlock;
    check_param!(rwlock.rwlock, RwLock);

    let rw = ManuallyDrop::new(Box::from_raw(rwlock.rwlock as *mut RwLock));
    let result = rw.read_unlock();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_rwlock_wrunlock(rwlock: *mut sgx_thread_rwlock_t) -> c_int {
    check_param!(rwlock, sgx_thread_rwlock_t);

    let rwlock = &mut *rwlock;
    check_param!(rwlock.rwlock, RwLock);

    let rw = ManuallyDrop::new(Box::from_raw(rwlock.rwlock as *mut RwLock));
    let result = rw.write_unlock();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn sgx_thread_rwlock_destroy(rwlock: *mut sgx_thread_rwlock_t) -> c_int {
    check_param!(rwlock, sgx_thread_rwlock_t);

    let rwlock = &mut *rwlock;
    if !rwlock.rwlock.is_null() {
        check_range!(rwlock.rwlock, RwLock);
        let rw = ManuallyDrop::new(Box::from_raw(rwlock.rwlock as *mut RwLock));
        let result = rw.destroy();
        if result.is_ok() {
            rwlock.rwlock = ptr::null_mut();
            let _ = ManuallyDrop::into_inner(rw);
        }
        result.err().unwrap_or(0)
    } else {
        0
    }
}
