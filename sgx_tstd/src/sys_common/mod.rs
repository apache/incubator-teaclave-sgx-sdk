// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

pub mod at_exit_imp;
pub mod os_str_bytes;
#[cfg(feature = "backtrace")]
pub mod backtrace;
#[cfg(feature = "backtrace")]
pub mod gnu;
pub mod io;
pub mod memchr;
pub mod poison;
pub mod thread_info;
pub mod util;
pub mod wtf8;
#[cfg(feature = "net")]
pub mod net;
pub mod bytestring;
pub mod fs;

macro_rules! rtabort {
    ($($t:tt)*) => (crate::sys_common::util::abort(format_args!($($t)*)))
}

#[allow(unused_macros)]
macro_rules! rtassert {
    ($e:expr) => (if !$e {
        rtabort!(concat!("assertion failed: ", stringify!($e)));
    })
}

#[allow(unused_macros)] // not used on all platforms
macro_rules! rtunwrap {
    ($ok:ident, $e:expr) => (match $e {
        $ok(v) => v,
        ref err => {
            let err = err.as_ref().map(|_|()); // map Ok/Some which might not be Debug
            rtabort!(concat!("unwrap failed: ", stringify!($e), " = {:?}"), err)
        },
    })
}

/// A trait for viewing representations from std types
#[doc(hidden)]
pub trait AsInner<Inner: ?Sized> {
    fn as_inner(&self) -> &Inner;
}

/// A trait for viewing representations from std types
#[doc(hidden)]
pub trait AsInnerMut<Inner: ?Sized> {
    fn as_inner_mut(&mut self) -> &mut Inner;
}

/// A trait for extracting representations from std types
#[doc(hidden)]
pub trait IntoInner<Inner> {
    fn into_inner(self) -> Inner;
}

/// A trait for creating std types from internal representations
#[doc(hidden)]
pub trait FromInner<Inner> {
    fn from_inner(inner: Inner) -> Self;
}

/// Enqueues a procedure to run when the main thread exits.
///
/// Currently these closures are only run once the main *Rust* thread exits.
/// Once the `at_exit` handlers begin running, more may be enqueued, but not
/// infinitely so. Eventually a handler registration will be forced to fail.
///
/// Returns `Ok` if the handler was successfully registered, meaning that the
/// closure will be run once the main thread exits. Returns `Err` to indicate
/// that the closure could not be registered, meaning that it is not scheduled
/// to be run.
pub fn at_exit<F: FnOnce() + Send + 'static>(f: F) -> Result<(), ()> {
    if at_exit_imp::push(Box::new(f)) {Ok(())} else {Err(())}
}

/// One-time runtime cleanup.
//#[allow(dead_code)]
//pub fn cleanup() {
//
//    use crate::sync::Once;
//
//    static CLEANUP: Once = Once::new();
//    CLEANUP.call_once(||
//        at_exit_imp::cleanup()
//    );
//}

/// One-time runtime cleanup.
pub fn cleanup() {
    use crate::sync::SgxThreadSpinlock;

    static SPIN_LOCK: SgxThreadSpinlock = SgxThreadSpinlock::new();
    static mut IS_CLEAUP: bool = false;

    unsafe {
        SPIN_LOCK.lock();
        if IS_CLEAUP == false {
            at_exit_imp::cleanup();
            IS_CLEAUP = true;
        }
        SPIN_LOCK.unlock();
    }
}

// Computes (value*numer)/denom without overflow, as long as both
// (numer*denom) and the overall result fit into i64 (which is the case
// for our time conversions).
#[allow(dead_code)] // not used on all platforms
pub fn mul_div_u64(value: u64, numer: u64, denom: u64) -> u64 {
    let q = value / denom;
    let r = value % denom;
    // Decompose value as (value/denom*denom + value%denom),
    // substitute into (value*numer)/denom and simplify.
    // r < denom, so (denom*numer) is the upper bound of (r*numer)
    q * numer + r * numer / denom
}