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
use core::sync::atomic::{AtomicPtr, Ordering};
use core::mem;
use core::alloc::AllocErr;

static SGX_OOM_HANDLER: AtomicPtr<()> = AtomicPtr::new(default_oom_handler as *mut ());

#[allow(clippy::needless_pass_by_value)]
fn default_oom_handler( _err: AllocErr) -> ! {
    trts::rsgx_abort()
}

pub fn rsgx_oom(err: AllocErr) -> ! {
    let value = SGX_OOM_HANDLER.load(Ordering::SeqCst);
    let handler: fn(AllocErr) -> ! = unsafe { mem::transmute(value) };
    handler(err);
}

/// Set a custom handler for out-of-memory conditions
///
/// To avoid recursive OOM failures, it is critical that the OOM handler does
/// not allocate any memory itself.
pub fn set_panic_handler(handler: fn(AllocErr) -> !) {
    SGX_OOM_HANDLER.store(handler as *mut (), Ordering::SeqCst);
}
