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

use crate::cell::RefCell;
use crate::thread::SgxThread;

struct SgxThreadInfo {
    thread: SgxThread,
}

thread_local! { static THREAD_INFO: RefCell<Option<SgxThreadInfo>> = const { RefCell::new(None) } }

impl SgxThreadInfo {
    fn with<R, F>(f: F) -> Option<R>
    where
        F: FnOnce(&mut SgxThreadInfo) -> R,
    {
        THREAD_INFO
            .try_with(move |thread_info| {
                let mut thread_info = thread_info.borrow_mut();
                let thread_info = thread_info.get_or_insert_with(|| SgxThreadInfo {
                    thread: SgxThread::new(None),
                });
                f(thread_info)
            })
            .ok()
    }
}

pub fn current_thread() -> Option<SgxThread> {
    SgxThreadInfo::with(|info| info.thread.clone())
}

pub fn set(thread: SgxThread) {
    THREAD_INFO.with(move |thread_info| {
        let mut thread_info = thread_info.borrow_mut();
        //rtassert!(thread_info.is_none());
        *thread_info = Some(SgxThreadInfo { thread });
    });
}
