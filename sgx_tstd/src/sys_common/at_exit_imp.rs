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

//use sync::SgxThreadMutex;
use sync::SgxThreadSpinlock;
use alloc_crate::boxed::{Box, FnBox};
use core::ptr;

type Queue = Vec<Box<FnBox()>>;

// NB these are specifically not types from `std::sync` as they currently rely
// on poisoning and this module needs to operate at a lower level than requiring
// the thread infrastructure to be in place (useful on the borders of
// initialization/destruction).
//static LOCK: SgxThreadMutex = SgxThreadMutex::new();
static LOCK: SgxThreadSpinlock = SgxThreadSpinlock::new();
static mut QUEUE: *mut Queue = ptr::null_mut();

// The maximum number of times the cleanup routines will be run. While running
// the at_exit closures new ones may be registered, and this count is the number
// of times the new closures will be allowed to register successfully. After
// this number of iterations all new registrations will return `false`.
const ITERS: usize = 10;

unsafe fn init() -> bool {
    if QUEUE.is_null() {
        let state: Box<Queue> = Box::new(Vec::new());
        QUEUE = Box::into_raw(state);
    } else if QUEUE as usize == 1 {
        // can't re-init after a cleanup
        return false
    }

    true
}

pub fn cleanup() {
    for i in 0..ITERS {
        unsafe {
            LOCK.lock();
            let queue = QUEUE;
            QUEUE = if i == ITERS - 1 {1} else {0} as *mut _;
            LOCK.unlock();

            // make sure we're not recursively cleaning up
            assert!(queue as usize != 1);

            // If we never called init, not need to cleanup!
            if queue as usize != 0 {
                let queue: Box<Queue> = Box::from_raw(queue);
                for to_run in *queue {
                    to_run();
                }
            }
        }
    }
}

pub fn push(f: Box<FnBox()>) -> bool {
    let mut ret = true;
    unsafe {
        LOCK.lock();
        if init() {
            (*QUEUE).push(f);
        } else {
            ret = false;
        }
        LOCK.unlock();
    }
    ret
}