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

//! Implementation of running at_exit routines
//!
//! Documentation can be found on the `rt::at_exit` function.

use crate::mem;
use crate::ptr;
use crate::sys_common::mutex::StaticMutex;

type Queue = Vec<Box<dyn FnOnce()>>;

// NB these are specifically not types from `std::sync` as they currently rely
// on poisoning and this module needs to operate at a lower level than requiring
// the thread infrastructure to be in place (useful on the borders of
// initialization/destruction).
// It is UB to attempt to acquire this mutex reentrantly!
static LOCK: StaticMutex = StaticMutex::new();
static mut QUEUE: *mut Queue = ptr::null_mut();

const DONE: *mut Queue = 1_usize as *mut _;

// The maximum number of times the cleanup routines will be run. While running
// the at_exit closures new ones may be registered, and this count is the number
// of times the new closures will be allowed to register successfully. After
// this number of iterations all new registrations will return `false`.
const ITERS: usize = 10;

unsafe fn init() -> bool {
    if QUEUE.is_null() {
        let state: Box<Queue> = Box::default();
        QUEUE = Box::into_raw(state);
    } else if QUEUE == DONE {
        // can't re-init after a cleanup
        return false;
    }

    true
}

pub fn cleanup() {
    for i in 1..=ITERS {
        unsafe {
            let queue = {
                let _guard = LOCK.lock();
                mem::replace(&mut QUEUE, if i == ITERS { DONE } else { ptr::null_mut() })
            };

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
        let _guard = LOCK.lock();
        if init() {
            // We are just moving `f` around, not calling it.
            // There is no possibility of reentrancy here.
            (*QUEUE).push(f);
            true
        } else {
            false
        }
    }
}
