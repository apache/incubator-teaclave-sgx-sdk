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

use crate::ocall::util::*;
use libc::{self, c_int, size_t, timespec};
use std::collections::VecDeque;
use std::io::Error;
use std::slice;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;
use std::sync::Once;

static mut TCS_EVENT_CACHE: Option<TcsEventCache> = None;
static INIT: Once = Once::new();

#[repr(transparent)]
pub struct SeEvent(AtomicI32);

impl SeEvent {
    pub fn new() -> SeEvent {
        SeEvent(AtomicI32::new(0))
    }

    pub fn wait_timeout(&self, timeout: &timespec, clockid: c_int, absolute_time: c_int) -> i32 {
        const FUTEX_BITSET_MATCH_ANY: u32 = 0xFFFF_FFFF;

        let (wait_op, clockid, bitset) = if absolute_time == 1 {
            (libc::FUTEX_WAIT_BITSET, clockid, FUTEX_BITSET_MATCH_ANY)
        } else {
            (libc::FUTEX_WAIT, 0, 0)
        };

        if self.0.fetch_add(-1, Ordering::SeqCst) == 0 {
            let ret = unsafe {
                libc::syscall(
                    libc::SYS_futex,
                    self,
                    wait_op | clockid | libc::FUTEX_PRIVATE_FLAG,
                    -1,
                    timeout as *const timespec,
                    0,
                    bitset,
                )
            };
            let _ = self
                .0
                .compare_exchange(-1, 0, Ordering::SeqCst, Ordering::SeqCst);
            if ret < 0 {
                return -1;
            }
        }
        0
    }

    pub fn wait(&self) -> i32 {
        if self.0.fetch_add(-1, Ordering::SeqCst) == 0 {
            let ret = unsafe {
                libc::syscall(
                    libc::SYS_futex,
                    self,
                    libc::FUTEX_WAIT | libc::FUTEX_PRIVATE_FLAG,
                    -1,
                    0,
                    0,
                    0,
                )
            };
            let _ = self
                .0
                .compare_exchange(-1, 0, Ordering::SeqCst, Ordering::SeqCst);
            if ret < 0 {
                return -1;
            }
        }
        0
    }

    pub fn wake(&self) -> i32 {
        if self.0.fetch_add(1, Ordering::SeqCst) != 0 {
            unsafe {
                libc::syscall(
                    libc::SYS_futex,
                    self,
                    libc::FUTEX_WAKE | libc::FUTEX_PRIVATE_FLAG,
                    1,
                    0,
                    0,
                    0,
                )
            };
        }
        0
    }
}

impl Default for SeEvent {
    fn default() -> SeEvent {
        SeEvent::new()
    }
}

struct TcsEvent<'a> {
    tcs: usize,
    event: &'a SeEvent,
}

struct TcsEventCache<'a> {
    cache: Mutex<VecDeque<TcsEvent<'a>>>,
}

impl<'a> TcsEventCache<'a> {
    fn new() -> TcsEventCache<'a> {
        TcsEventCache {
            cache: Mutex::new(VecDeque::with_capacity(16)),
        }
    }

    pub fn get_event(&self, tcs: usize) -> &SeEvent {
        let mut cahce_guard = self.cache.lock().unwrap();
        match cahce_guard.iter().find(|&e| e.tcs == tcs) {
            Some(e) => e.event,
            None => {
                let event = Box::leak(Box::new(SeEvent::new()));
                cahce_guard.push_back(TcsEvent { tcs, event });
                event
            }
        }
    }
}

fn get_tcs_event(tcs: usize) -> &'static SeEvent {
    unsafe {
        INIT.call_once(|| {
            TCS_EVENT_CACHE = Some(TcsEventCache::new());
        });
        TCS_EVENT_CACHE
            .as_ref()
            .expect("TCS_EVENT_CACHE is not initialized.")
            .get_event(tcs)
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_thread_set_event_ocall(error: *mut c_int, tcs: size_t) -> c_int {
    if tcs == 0 {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let mut errno = 0;
    let result = get_tcs_event(tcs).wake();
    if result != 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    if result == 0 {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_thread_wait_event_ocall(
    error: *mut c_int,
    tcs: size_t,
    timeout: *const timespec,
    clockid: c_int,
    absolute_time: c_int,
) -> c_int {
    if tcs == 0 {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let mut errno = 0;
    let result = if timeout.is_null() {
        get_tcs_event(tcs).wait()
    } else {
        get_tcs_event(tcs).wait_timeout(&*timeout, clockid, absolute_time)
    };
    if result != 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    if result == 0 {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_thread_set_multiple_events_ocall(
    error: *mut c_int,
    tcss: *const size_t,
    total: size_t,
) -> c_int {
    if tcss.is_null() || total == 0 {
        set_error(error, libc::EINVAL);
        return -1;
    }

    let tcss_slice = slice::from_raw_parts(tcss, total);
    let mut errno = 0;
    let mut result = 0;
    for tcs in tcss_slice.iter().filter(|&&tcs| tcs != 0) {
        result = get_tcs_event(*tcs).wake();
        if result != 0 {
            errno = Error::last_os_error().raw_os_error().unwrap_or(0);
            break;
        }
    }
    set_error(error, errno);
    if result == 0 {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn u_thread_setwait_events_ocall(
    error: *mut c_int,
    waite_tcs: size_t,
    self_tcs: size_t,
    timeout: *const timespec,
    clockid: c_int,
    absolute_time: c_int,
) -> c_int {
    let result = u_thread_set_event_ocall(error, waite_tcs);
    if result < 0 {
        result
    } else {
        u_thread_wait_event_ocall(error, self_tcs, timeout, clockid, absolute_time)
    }
}
