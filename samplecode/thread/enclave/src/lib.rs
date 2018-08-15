// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

    let lock = Box::new((SgxMutex::<CondBuffer>::new(CondBuffer::default()), SgxCondvar::new(), SgxCondvar::new()));
    let ptr = Box::into_raw(lock);
    GLOBAL_COND_BUFFER.store(ptr as *mut (), Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn ecall_uninitialize() {

    let ptr = GLOBAL_COND_BUFFER.swap(0 as * mut (), Ordering::SeqCst) as * mut (SgxMutex<CondBuffer>, SgxCondvar, SgxCondvar);
    if ptr.is_null() {
       return;
    }
    let _ = unsafe { Box::from_raw(ptr) };
}

fn get_ref_cond_buffer() -> Option<&'static (SgxMutex<CondBuffer>, SgxCondvar, SgxCondvar)>
{
    let ptr = GLOBAL_COND_BUFFER.load(Ordering::SeqCst) as * mut (SgxMutex<CondBuffer>, SgxCondvar, SgxCondvar);
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
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
