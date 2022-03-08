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

use crate::thread::Thread;
use core::cell::RefCell;

#[thread_local]
static THREAD_INFO: RefCell<ThreadInfo> = RefCell::new(ThreadInfo::new());

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    None,
    Exit,
}
struct ThreadInfo {
    thread: Option<Thread>,
    state: State,
}

impl ThreadInfo {
    const fn new() -> ThreadInfo {
        ThreadInfo {
            thread: None,
            state: State::None,
        }
    }
}

pub fn set(thread: Thread) {
    let mut info = THREAD_INFO.borrow_mut();
    info.thread.replace(thread);
    info.state = State::None;
}

pub fn get_state() -> State {
    THREAD_INFO.borrow().state
}

#[inline]
pub fn is_exit() -> bool {
    get_state() == State::Exit
}

pub fn clear() {
    let mut info = THREAD_INFO.borrow_mut();
    info.thread.take();
    info.state = State::Exit;
}

pub fn current_thread() -> Option<Thread> {
    THREAD_INFO.borrow().thread.clone()
}
