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

use crate::sync::{SpinMutex, SpinMutexGuard};
use core::sync::atomic::{AtomicUsize, Ordering};
use sgx_types::error::SgxResult;

pub struct Once {
    lock: SpinMutex<()>,
    state: AtomicUsize,
}

unsafe impl Sync for Once {}
unsafe impl Send for Once {}

const INCOMPLETE: usize = 0x0;
const COMPLETE: usize = 0x1;

impl Once {
    pub const fn new() -> Once {
        Once {
            lock: SpinMutex::new(()),
            state: AtomicUsize::new(INCOMPLETE),
        }
    }

    pub fn lock(&self) -> SpinMutexGuard<()> {
        self.lock.lock()
    }

    pub fn call_once<F>(&self, init: F) -> SgxResult
    where
        F: FnOnce() -> SgxResult,
    {
        if self.is_completed() {
            return Ok(());
        }

        let _guard = self.lock.lock();
        if !self.is_completed() {
            init()?;
            self.state.store(COMPLETE, Ordering::Release);
        }
        Ok(())
    }

    #[inline]
    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == COMPLETE
    }
}
