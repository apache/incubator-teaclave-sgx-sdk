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

#![no_std]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#![feature(lang_items)]
#![feature(alloc_error_handler)]

extern crate alloc as alloc_crate;

#[cfg(target_env = "sgx")]
extern crate sgx_alloc;

use core::alloc::Layout;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::{mem, ptr};

pub use alloc_crate::alloc::*;
pub use sgx_alloc::System;

#[global_allocator]
static ALLOC: sgx_alloc::System = sgx_alloc::System;

#[panic_handler]
fn begin_panic_handler(_info: &PanicInfo<'_>) -> ! {
    sgx_abort();
}

#[lang = "eh_personality"]
#[no_mangle]
unsafe extern "C" fn rust_eh_personality() {}

static HOOK: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

/// Registers a custom allocation error hook, replacing any that was previously registered.
///
/// The allocation error hook is invoked when an infallible memory allocation fails, before
/// the runtime aborts. The default hook prints a message to standard error,
/// but this behavior can be customized with the [`set_alloc_error_hook`] and
/// [`take_alloc_error_hook`] functions.
///
/// The hook is provided with a `Layout` struct which contains information
/// about the allocation that failed.
///
/// The allocation error hook is a global resource.
pub fn set_alloc_error_hook(hook: fn(Layout)) {
    HOOK.store(hook as *mut (), Ordering::SeqCst);
}

/// Unregisters the current allocation error hook, returning it.
///
/// *See also the function [`set_alloc_error_hook`].*
///
/// If no custom hook is registered, the default hook will be returned.
pub fn take_alloc_error_hook() -> fn(Layout) {
    let hook = HOOK.swap(ptr::null_mut(), Ordering::SeqCst);
    if hook.is_null() {
        default_alloc_error_hook
    } else {
        unsafe { mem::transmute(hook) }
    }
}

fn default_alloc_error_hook(_layout: Layout) {}

#[alloc_error_handler]
pub fn rust_oom(layout: Layout) -> ! {
    let hook = HOOK.load(Ordering::SeqCst);
    let hook: fn(Layout) = if hook.is_null() {
        default_alloc_error_hook
    } else {
        unsafe { mem::transmute(hook) }
    };
    hook(layout);
    sgx_abort();
}

#[link(name = "sgx_trts")]
extern "C" {
    pub fn abort() -> !;
}

fn sgx_abort() -> ! {
    unsafe { abort() }
}
