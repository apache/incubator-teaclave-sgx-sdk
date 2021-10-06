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

#![crate_name = "threadsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
extern crate sgx_tstd as std;

use std::sync::{SgxMutex, SgxCondvar};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::boxed::Box;

const BUFFER_SIZE: usize      = 50;
const LOOPS_PER_THREAD: usize = 500;

struct CondBuffer{
    buf: [usize; BUFFER_SIZE],
    occupied: i32,
    nextin: usize,
    nextout: usize,
}

impl Default for CondBuffer {
    fn default() -> CondBuffer {
        CondBuffer {
            buf: [0; BUFFER_SIZE],
            occupied: 0,
            nextin: 0,
            nextout: 0,
        }
    }
}

static GLOBAL_COND_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());

#[no_mangle]
pub extern "C" fn ecall_initialize() {
    let lock = Box::new((
        SgxMutex::<CondBuffer>::new(CondBuffer::default()),
        SgxCondvar::new(),
        SgxCondvar::new(),
    ));
    let ptr = Box::into_raw(lock);
    GLOBAL_COND_BUFFER.store(ptr as *mut (), Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn ecall_uninitialize() {
    let ptr = GLOBAL_COND_BUFFER.swap(0 as *mut (), Ordering::SeqCst)
        as *mut (SgxMutex<CondBuffer>, SgxCondvar, SgxCondvar);
    if ptr.is_null() {
        return;
    }
    let _ = unsafe { Box::from_raw(ptr) };
}

fn get_ref_cond_buffer() -> Option<&'static (SgxMutex<CondBuffer>, SgxCondvar, SgxCondvar)> {
    let ptr = GLOBAL_COND_BUFFER.load(Ordering::SeqCst)
        as *mut (SgxMutex<CondBuffer>, SgxCondvar, SgxCondvar);
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &*ptr })
    }
}

#[no_mangle]
pub extern "C" fn ecall_producer() {
    let max_index = 4 * LOOPS_PER_THREAD;
    let &(ref mutex, ref more, ref less) = get_ref_cond_buffer().unwrap();

    for _ in 0..max_index {
        let mut guard = mutex.lock().unwrap();

        while guard.occupied >= BUFFER_SIZE as i32 {
            guard = less.wait(guard).unwrap();
        }

        let index = guard.nextin;
        guard.buf[index] = guard.nextin;
        guard.nextin += 1;
        guard.nextin %= BUFFER_SIZE;
        guard.occupied += 1;

        let _ = more.signal();
    }
}

#[no_mangle]
pub extern "C" fn ecall_consumer() {
    let max_index = 4 * LOOPS_PER_THREAD;
    let &(ref mutex, ref more, ref less) = get_ref_cond_buffer().unwrap();

    for _ in 0..max_index {
        let mut guard = mutex.lock().unwrap();

        while guard.occupied <= 0 {
            guard = more.wait(guard).unwrap();
        }

        let index = guard.nextout;
        guard.buf[index] = 0;
        guard.nextout += 1;
        guard.nextout %= BUFFER_SIZE;
        guard.occupied -= 1;

        let _ = less.signal();
    }
}
