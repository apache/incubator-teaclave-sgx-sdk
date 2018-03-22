// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

use trts;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::mem;
use alloc::heap::AllocErr;

static SGX_OOM_HANDLER: AtomicPtr<()> = AtomicPtr::new(default_oom_handler as * mut ());

#[allow(unused_variables)]
fn default_oom_handler(err: AllocErr) -> ! {

    //panic!("enclave allocate memory failed.");
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

    SGX_OOM_HANDLER.store(handler as * mut (), Ordering::SeqCst);
}