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

macro_rules! rtabort {
    ($($t:tt)*) => (crate::sys_common::util::abort(format_args!($($t)*)))
}

#[allow(unused_macros)]
macro_rules! rtassert {
    ($e:expr) => {
        if !$e {
            rtabort!(concat!("assertion failed: ", stringify!($e)));
        }
    };
}

#[allow(unused_macros)] // not used on all platforms
macro_rules! rtunwrap {
    ($ok:ident, $e:expr) => {
        match $e {
            $ok(v) => v,
            ref err => {
                let err = err.as_ref().map(drop); // map Ok/Some which might not be Debug
                rtabort!(concat!("unwrap failed: ", stringify!($e), " = {:?}"), err)
            }
        }
    };
}

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
#[cfg(feature = "thread")]
pub mod thread;
#[cfg(feature = "thread")]
pub mod thread_local;
pub mod util;
pub mod wtf8;
#[cfg(feature = "net")]
pub mod net;
pub mod bytestring;
pub mod fs;

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
    if at_exit_imp::push(Box::new(f)) { Ok(()) } else { Err(()) }
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