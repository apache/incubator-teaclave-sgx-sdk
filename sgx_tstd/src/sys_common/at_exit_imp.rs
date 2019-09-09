// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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
use crate::sync::SgxThreadSpinlock;

use core::ptr;
use core::mem;

type Queue = Vec<Box<dyn FnOnce()>>;

// NB these are specifically not types from `std::sync` as they currently rely
// on poisoning and this module needs to operate at a lower level than requiring
// the thread infrastructure to be in place (useful on the borders of
// initialization/destruction).
//static LOCK: SgxThreadMutex = SgxThreadMutex::new();
static LOCK: SgxThreadSpinlock = SgxThreadSpinlock::new();
static mut QUEUE: *mut Queue = ptr::null_mut();
const DONE: *mut Queue = 1_usize as *mut _;

// The maximum number of times the cleanup routines will be run. While running
// the at_exit closures new ones may be registered, and this count is the number
// of times the new closures will be allowed to register successfully. After
// this number of iterations all new registrations will return `false`.
const ITERS: usize = 10;

unsafe fn init() -> bool {
    if QUEUE.is_null() {
        let state: Box<Queue> = Box::new(Vec::new());
        QUEUE = Box::into_raw(state);
    } else if QUEUE == DONE {
        // can't re-init after a cleanup
        return false
    }

    true
}

pub fn cleanup() {
    for i in 1..=ITERS {
        unsafe {
            LOCK.lock();
            let queue = QUEUE;
            QUEUE = mem::replace(&mut QUEUE, if i == ITERS { DONE } else { ptr::null_mut() });
            LOCK.unlock();

            // make sure we're not recursively cleaning up
            assert!(queue != DONE);

            // If we never called init, not need to cleanup!
            if !queue.is_null() {
                let queue: Box<Queue> = Box::from_raw(queue);
                for to_run in *queue {
                    // We are not holding any lock, so reentrancy is fine.
                    to_run();
                }
            }
        }
    }
}

pub fn push(f: Box<dyn FnOnce()>) -> bool {
    unsafe {
        LOCK.lock();
        let ret = if init() {
            // We are just moving `f` around, not calling it.
            // There is no possibility of reentrancy here.
            (*QUEUE).push(f);
            true
        } else {
            false
        };
        LOCK.unlock();
        ret
    }
}