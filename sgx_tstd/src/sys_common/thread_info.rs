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

use crate::cell::OnceCell;
use crate::thread::Thread;
use guard::Guard;

struct ThreadInfo {
    stack_guard: OnceCell<Guard>,
    thread: OnceCell<Thread>,
}

thread_local! {
   static THREAD_INFO: ThreadInfo = const { ThreadInfo {
       stack_guard: OnceCell::new(),
       thread: OnceCell::new()
   } };
}

impl ThreadInfo {
    fn with<R, F>(f: F) -> Option<R>
    where
        F: FnOnce(&Thread, &OnceCell<Guard>) -> R,
    {
        THREAD_INFO
            .try_with(move |thread_info| {
                let thread = thread_info.thread.get_or_init(|| Thread::new(None));
                f(thread, &thread_info.stack_guard)
            })
            .ok()
    }
}

pub fn current_thread() -> Option<Thread> {
    ThreadInfo::with(|thread, _| thread.clone())
}

pub fn stack_guard() -> Option<Guard> {
    ThreadInfo::with(|_, guard| guard.get().cloned()).flatten()
}

/// Set new thread info, panicking if it has already been initialized
#[allow(unreachable_code, unreachable_patterns)] // some platforms don't use stack_guard
pub fn set(stack_guard: Option<Guard>, thread: Thread) {
    THREAD_INFO.with(move |thread_info| {
        rtassert!(thread_info.stack_guard.get().is_none() && thread_info.thread.get().is_none());
        if let Some(guard) = stack_guard {
            thread_info.stack_guard.set(guard).unwrap();
        }
        thread_info.thread.set(thread).unwrap();
    });
}

pub mod guard {
    pub type Guard = !;
    pub unsafe fn current() -> Option<Guard> {
        None
    }
    pub unsafe fn init() -> Option<Guard> {
        None
    }
}
