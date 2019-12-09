
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

use alloc_crate::collections::LinkedList;
use crate::sync::SgxThreadSpinlock;

static QUEUE_LOCK: SgxThreadSpinlock = SgxThreadSpinlock::new();
static mut THREAD_QUEUE: Option<LinkedList<*mut usize>> = None;

pub unsafe fn start_thread(main: *mut u8) {
    // Finally, let's run some code.
    Box::from_raw(main as *mut Box<dyn FnOnce()>)()
}

pub fn push_thread_queue(main: *mut usize) {
    unsafe {
        QUEUE_LOCK.lock();
        if THREAD_QUEUE.is_none() {
            THREAD_QUEUE = Some(LinkedList::<*mut usize>::new());
        }
        THREAD_QUEUE.as_mut().map(|q| q.push_back(main));
        QUEUE_LOCK.unlock();
    }
}

pub fn pop_thread_queue(main: *mut usize) -> Option<*mut usize> {
    unsafe {
        QUEUE_LOCK.lock();
        let p = THREAD_QUEUE.as_mut().map_or(None, |q| {
            q.drain_filter(|x| (*x == main) && (**x == *main)).next()
        });
        QUEUE_LOCK.unlock();
        p
    }
}