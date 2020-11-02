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

use crate::trts;
use core::alloc::AllocError;
use core::mem;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

static SGX_OOM_HANDLER: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

#[allow(clippy::needless_pass_by_value)]
fn default_oom_handler(_err: AllocError) -> ! {
    trts::rsgx_abort()
}

pub fn rsgx_oom(err: AllocError) -> ! {
    let hook = SGX_OOM_HANDLER.load(Ordering::SeqCst);
    let handler: fn(AllocError) -> ! = if hook.is_null() {
        default_oom_handler
    } else {
        unsafe { mem::transmute(hook) }
    };
    handler(err)
}

/// Set a custom handler for out-of-memory conditions
///
/// To avoid recursive OOM failures, it is critical that the OOM handler does
/// not allocate any memory itself.
pub fn set_oom_handler(handler: fn(AllocError) -> !) {
    SGX_OOM_HANDLER.store(handler as *mut (), Ordering::SeqCst);
}

/// Unregisters the current custom handler, returning it.
///
pub fn take_oom_handler() -> fn(AllocError) -> ! {
    let hook = SGX_OOM_HANDLER.swap(ptr::null_mut(), Ordering::SeqCst);
    if hook.is_null() {
        default_oom_handler
    } else {
        unsafe { mem::transmute(hook) }
    }
}
