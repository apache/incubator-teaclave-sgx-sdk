// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

#[cfg(not(feature = "use_std"))]
use core::sync::atomic::{AtomicPtr, Ordering};
#[cfg(not(feature = "use_std"))]
use core::{mem, fmt};
use super::function;

#[cfg(not(feature = "use_std"))]
static PANIC_HANDLER: AtomicPtr<()> = AtomicPtr::new(default_panic_handler as * mut ());

#[cfg(not(feature = "use_std"))]
fn default_panic_handler(msg: fmt::Arguments) {

}

#[cfg(not(feature = "use_std"))]
pub fn set_panic_handler(handler: fn(fmt::Arguments)) {
    PANIC_HANDLER.store(handler as * mut (), Ordering::SeqCst);
}

pub fn rsgx_abort() -> ! {
    unsafe { function::abort() }
}

#[cfg(not(feature = "use_std"))]
#[lang = "panic_fmt"] 
#[no_mangle]
pub extern fn rust_begin_panic(msg: fmt::Arguments, file: &'static str, line: u32) -> ! { 

    let value = PANIC_HANDLER.load(Ordering::SeqCst);
    let handler: fn(fmt::Arguments) = unsafe { mem::transmute(value) };
    handler(msg);

    rsgx_abort()
}

#[cfg(not(feature = "use_std"))]
#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {}

#[cfg(not(feature = "use_std"))]
#[lang = "eh_unwind_resume"]
#[no_mangle]
pub extern fn rust_eh_unwind_resume() {}