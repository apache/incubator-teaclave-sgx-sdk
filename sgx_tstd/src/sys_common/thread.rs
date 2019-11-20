
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