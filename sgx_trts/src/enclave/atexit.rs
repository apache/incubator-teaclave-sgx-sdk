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

use crate::rand::Rng;
use crate::sync::SpinMutex;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem;
use core::num::NonZeroUsize;
use core::ptr;

static LOCK: SpinMutex<()> = SpinMutex::new(());
static mut QUEUE: *mut Queue = ptr::null_mut();

const DONE: *mut Queue = 1_usize as *mut _;

const ITERS: usize = 10;

struct Queue {
    queue: Vec<[usize; 2]>,
    cookie: NonZeroUsize,
}

impl Queue {
    fn new() -> Queue {
        let cookie = loop {
            let r = Rng::new().next_usize();
            if r != 0 {
                break NonZeroUsize::new(r).unwrap();
            }
        };
        Queue {
            queue: Vec::new(),
            cookie,
        }
    }

    fn push(&mut self, f: Box<dyn FnOnce()>) {
        let mut raw: [usize; 2] = unsafe { mem::transmute(Box::into_raw(f)) };
        raw[0] ^= self.cookie.get();
        raw[1] ^= self.cookie.get();
        self.queue.push(raw);
    }

    fn pop(&mut self) -> Option<Box<dyn FnOnce()>> {
        self.queue.pop().map(|mut raw| {
            raw[0] ^= self.cookie.get();
            raw[1] ^= self.cookie.get();
            unsafe { Box::from_raw(mem::transmute(raw)) }
        })
    }
}

impl IntoIterator for Queue {
    type Item = Box<dyn FnOnce()>;
    type IntoIter = QueueIntoIter;

    #[inline]
    fn into_iter(self) -> QueueIntoIter {
        QueueIntoIter { queue: self }
    }
}

struct QueueIntoIter {
    queue: Queue,
}

impl Iterator for QueueIntoIter {
    type Item = Box<dyn FnOnce()>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.queue.queue.len(), Some(self.queue.queue.len()))
    }
}

unsafe fn init() -> bool {
    if QUEUE.is_null() {
        let state: Box<Queue> = Box::new(Queue::new());
        QUEUE = Box::into_raw(state);
    } else if QUEUE == DONE {
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
            assert!(queue != DONE);

            if !queue.is_null() {
                let queue: Box<Queue> = Box::from_raw(queue);
                for to_run in *queue {
                    to_run();
                }
            }
        }
    }
}

fn push(f: Box<dyn FnOnce()>) -> bool {
    unsafe {
        let _guard = LOCK.lock();
        if init() {
            (*QUEUE).push(f);
            true
        } else {
            false
        }
    }
}

pub fn at_exit<F: FnOnce() + Send + 'static>(f: F) -> bool {
    push(Box::new(f))
}
