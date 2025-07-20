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

use crate::sys::futex::Futex;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::Relaxed;
use core::time::Duration;
use sgx_types::error::errno::{EINTR, ETIMEDOUT};

/// Wait for a futex_wake operation to wake us.
///
/// Returns directly if the futex doesn't hold the expected value.
///
/// Returns false on timeout, and true in all other cases.
pub fn futex_wait(futex: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    let futex_obj = Futex::new(futex as *const _ as usize);
    loop {
        // No need to wait if the value already changed.
        if futex.load(Relaxed) != expected {
            return true;
        }

        match futex_obj.wait(expected, timeout) {
            Err(ETIMEDOUT) => return false,
            Err(EINTR) => continue,
            _ => return true,
        }
    }
}

/// Wake up one thread that's blocked on futex_wait on this futex.
///
/// Returns true if this actually woke up such a thread,
/// or false if no thread was waiting on this futex.
///
/// On some platforms, this always returns false.
pub fn futex_wake(futex: &AtomicU32) -> bool {
    let futex = Futex::new(futex as *const _ as usize);
    match futex.wake(1) {
        Ok(0) => false,
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Wake up all threads that are waiting on futex_wait on this futex.
pub fn futex_wake_all(futex: &AtomicU32) {
    let futex = Futex::new(futex as *const _ as usize);
    let _ = futex.wake(i32::MAX as usize);
}
