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

use crate::linux::*;
use alloc::boxed::Box;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr;
use core::time::Duration;
use sgx_sync::sys::condvar::{Condvar, MovableCondvar};
use sgx_sync::sys::mutex::{MovableMutex, Mutex};
use sgx_sync::sys::rwlock::{MovableRwLock, RwLock};
use sgx_trts::sync::SpinMutex;
use sgx_trts::thread::tls::{Key, Tls};
use sgx_trts::thread::{self, Native, Thread};
use sgx_trts::trts::is_within_enclave;
use sgx_types::error::SgxStatus;

pub type pthread_t = *mut c_void;
pub type pthread_key_t = size_t;

pub type pthread_mutex_t = *mut c_void;
pub type pthread_cond_t = *mut c_void;
pub type pthread_rwlock_t = *mut c_void;
pub type pthread_attr_t = *mut pthread_attr;
pub type pthread_mutexattr_t = *mut pthread_mutexattr;
pub type pthread_condattr_t = *mut pthread_condattr;
pub type pthread_rwlockattr_t = *mut pthread_rwlockattr;

s! {
    pub struct pthread_attr {
        pub reserved: c_char,
    }

    pub struct pthread_mutexattr {
        pub m_dummy: c_uchar,
    }

    pub struct pthread_condattr {
        pub m_dummy: c_uchar,
    }

    pub struct pthread_rwlockattr {
        pub m_dummy: c_uchar,
    }

    pub struct pthread_once_t {
        pub state: c_int,
        pub mutex: pthread_mutex_t,
    }
}

macro_rules! check_null {
    ($ptr:expr) => {
        if $ptr.is_null() {
            return EINVAL;
        }
    };
}

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

pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t = ptr::null_mut();
pub const PTHREAD_COND_INITIALIZER: pthread_cond_t = ptr::null_mut();
pub const PTHREAD_RWLOCK_INITIALIZER: pthread_rwlock_t = ptr::null_mut();

const PTHREAD_NEEDS_INIT: c_int = 0;
const PTHREAD_DONE_INIT: c_int = 1;
pub const PTHREAD_ONCE_INIT: pthread_once_t = pthread_once_t {
    state: PTHREAD_NEEDS_INIT,
    mutex: PTHREAD_MUTEX_INITIALIZER,
};

static MUTEX_INIT_LOCK: SpinMutex<()> = SpinMutex::new(());
static CONDVAR_INIT_LOCK: SpinMutex<()> = SpinMutex::new(());
static RW_INIT_LOCK: SpinMutex<()> = SpinMutex::new(());

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
            *thread = Thread::into_raw(t) as *mut c_void;
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
        Thread::into_raw(t) as *mut c_void
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_equal(t1: pthread_t, t2: pthread_t) -> c_int {
    if t1.is_null() || t2.is_null() {
        return 0;
    }

    let t1 = ManuallyDrop::new(Thread::from_raw(t1 as *mut Native));
    let t2 = ManuallyDrop::new(Thread::from_raw(t2 as *mut Native));
    if t1 == t2 {
        1
    } else {
        0
    }
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

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut pthread_mutex_t,
    _attr: *const pthread_mutexattr_t,
) -> c_int {
    check_param!(mutex, pthread_mutex_t);

    let m = MovableMutex::from(Mutex::new());
    *mutex = Box::into_raw(m) as pthread_mutex_t;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    check_param!(mutex, pthread_mutex_t);

    let m = if (*mutex).is_null() {
        MUTEX_INIT_LOCK.lock();
        let m = ManuallyDrop::new(MovableMutex::from(Mutex::new()));
        *mutex = m.as_ref() as *const _ as pthread_mutex_t;
        m
    } else {
        check_range!(*mutex, Mutex);
        ManuallyDrop::new(Box::from_raw(*mutex as *mut Mutex))
    };

    let result = m.lock();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    check_param!(mutex, pthread_mutex_t);

    let m = if (*mutex).is_null() {
        MUTEX_INIT_LOCK.lock();
        let m = ManuallyDrop::new(MovableMutex::from(Mutex::new()));
        *mutex = m.as_ref() as *const _ as pthread_mutex_t;
        m
    } else {
        check_range!(*mutex, Mutex);
        ManuallyDrop::new(Box::from_raw(*mutex as *mut Mutex))
    };

    let result = m.try_lock();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    check_param!(mutex, pthread_mutex_t);
    check_param!(*mutex, Mutex);

    let m = ManuallyDrop::new(Box::from_raw(*mutex as *mut Mutex));
    let result = m.unlock();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int {
    check_param!(mutex, pthread_mutex_t);

    if !(*mutex).is_null() {
        check_range!(*mutex, Mutex);
        let m = ManuallyDrop::new(Box::from_raw(*mutex as *mut Mutex));
        let result = m.destroy();
        if result.is_ok() {
            *mutex = ptr::null_mut();
            let _ = ManuallyDrop::into_inner(m);
        }
        result.err().unwrap_or(0)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_init(
    cond: *mut pthread_cond_t,
    _attr: *const pthread_condattr_t,
) -> c_int {
    check_param!(cond, pthread_cond_t);

    let c = MovableCondvar::from(Condvar::new());
    *cond = Box::into_raw(c) as pthread_cond_t;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_wait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
) -> c_int {
    check_param!(cond, pthread_cond_t);
    check_param!(mutex, pthread_mutex_t);
    check_param!(*mutex, Mutex);

    let c = if (*cond).is_null() {
        CONDVAR_INIT_LOCK.lock();
        let c = ManuallyDrop::new(MovableCondvar::from(Condvar::new()));
        *cond = c.as_ref() as *const _ as pthread_cond_t;
        c
    } else {
        check_range!(*cond, Condvar);
        ManuallyDrop::new(Box::from_raw(*cond as *mut Condvar))
    };
    let m = ManuallyDrop::new(Box::from_raw(*mutex as *mut Mutex));

    let result = c.wait(m.as_ref());
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    timeout: *const timespec,
) -> c_int {
    check_param!(cond, pthread_cond_t);
    check_param!(mutex, pthread_mutex_t);
    check_param!(*mutex, Mutex);
    check_param!(timeout, timespec);

    let c = if (*cond).is_null() {
        CONDVAR_INIT_LOCK.lock();
        let c = ManuallyDrop::new(MovableCondvar::from(Condvar::new()));
        *cond = c.as_ref() as *const _ as pthread_cond_t;
        c
    } else {
        check_range!(*cond, Condvar);
        ManuallyDrop::new(Box::from_raw(*cond as *mut Condvar))
    };
    let m = ManuallyDrop::new(Box::from_raw(*mutex as *mut Mutex));

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
pub unsafe extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    check_param!(cond, pthread_cond_t);

    if !(*cond).is_null() {
        check_range!(*cond, Condvar);
        let c = ManuallyDrop::new(Box::from_raw(*cond as *mut Condvar));
        let result = c.notify_one();
        result.err().unwrap_or(0)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    check_param!(cond, pthread_cond_t);

    if !(*cond).is_null() {
        check_range!(*cond, Condvar);
        let c = ManuallyDrop::new(Box::from_raw(*cond as *mut Condvar));
        let result = c.notify_all();
        result.err().unwrap_or(0)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    check_param!(cond, pthread_cond_t);

    if !(*cond).is_null() {
        check_range!(*cond, Condvar);
        let c = ManuallyDrop::new(Box::from_raw(*cond as *mut Condvar));
        let result = c.destroy();
        if result.is_ok() {
            *cond = ptr::null_mut();
            let _ = ManuallyDrop::into_inner(c);
        }
        result.err().unwrap_or(0)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_init(
    rwlock: *mut pthread_rwlock_t,
    _attr: *const pthread_rwlockattr_t,
) -> c_int {
    check_param!(rwlock, pthread_rwlock_t);

    let rw = MovableRwLock::from(RwLock::new());
    *rwlock = Box::into_raw(rw) as pthread_rwlock_t;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    check_param!(rwlock, pthread_rwlock_t);

    let rw = if (*rwlock).is_null() {
        RW_INIT_LOCK.lock();
        let rw = ManuallyDrop::new(MovableRwLock::from(RwLock::new()));
        *rwlock = rw.as_ref() as *const _ as pthread_rwlock_t;
        rw
    } else {
        check_range!(*rwlock, RwLock);
        ManuallyDrop::new(Box::from_raw(*rwlock as *mut RwLock))
    };

    let result = rw.read();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    check_param!(rwlock, pthread_rwlock_t);

    let rw = if (*rwlock).is_null() {
        RW_INIT_LOCK.lock();
        let rw = ManuallyDrop::new(MovableRwLock::from(RwLock::new()));
        *rwlock = rw.as_ref() as *const _ as pthread_rwlock_t;
        rw
    } else {
        check_range!(*rwlock, RwLock);
        ManuallyDrop::new(Box::from_raw(*rwlock as *mut RwLock))
    };

    let result = rw.try_read();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    check_param!(rwlock, pthread_rwlock_t);

    let rw = if (*rwlock).is_null() {
        RW_INIT_LOCK.lock();
        let rw = ManuallyDrop::new(MovableRwLock::from(RwLock::new()));
        *rwlock = rw.as_ref() as *const _ as pthread_rwlock_t;
        rw
    } else {
        check_range!(*rwlock, RwLock);
        ManuallyDrop::new(Box::from_raw(*rwlock as *mut RwLock))
    };

    let result = rw.write();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    check_param!(rwlock, pthread_rwlock_t);

    let rw = if (*rwlock).is_null() {
        RW_INIT_LOCK.lock();
        let rw = ManuallyDrop::new(MovableRwLock::from(RwLock::new()));
        *rwlock = rw.as_ref() as *const _ as pthread_rwlock_t;
        rw
    } else {
        check_range!(*rwlock, RwLock);
        ManuallyDrop::new(Box::from_raw(*rwlock as *mut RwLock))
    };

    let result = rw.try_write();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    check_param!(rwlock, pthread_rwlock_t);
    check_param!(*rwlock, RwLock);

    let rw = ManuallyDrop::new(Box::from_raw(*rwlock as *mut RwLock));
    let result = rw.unlock();
    result.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_destroy(rwlock: *mut pthread_rwlock_t) -> c_int {
    check_param!(rwlock, pthread_rwlock_t);

    if !(*rwlock).is_null() {
        check_range!(*rwlock, RwLock);
        let rw = ManuallyDrop::new(Box::from_raw(*rwlock as *mut RwLock));
        let result = rw.destroy();
        if result.is_ok() {
            *rwlock = ptr::null_mut();
            let _ = ManuallyDrop::into_inner(rw);
        }
        result.err().unwrap_or(0)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_once(
    once_control: *mut pthread_once_t,
    init_routine: extern "C" fn(),
) -> c_int {
    check_param!(once_control, pthread_once_t);

    let mut once_control = &mut *once_control;
    pthread_mutex_lock(&mut once_control.mutex);
    if once_control.state == PTHREAD_NEEDS_INIT {
        init_routine();
        once_control.state = PTHREAD_DONE_INIT;
    }
    pthread_mutex_unlock(&mut once_control.mutex);
    0
}
