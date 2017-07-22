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

//! Thread local storage

use super::enclave::{SgxGlobalData, SgxThreadPolicy};
use core::cell::UnsafeCell;
use core::mem;
use core::intrinsics;

pub struct LocalKey<T: 'static> {
    inner: fn() -> Option<&'static UnsafeCell<Option<T>>>,
    init: fn() -> T,
}

/// Declare a new thread local storage key of type [`sgx_trts::LocalKey`].
///
/// # Syntax
///
/// The macro wraps any number of static declarations and makes them thread local.
/// Each static may be public or private, and attributes are allowed. Example:
///
/// ```
/// use core::cell::RefCell;
/// thread_local! {
///     pub static FOO: RefCell<u32> = RefCell::new(1);
///
///     #[allow(unused)]
///     static BAR: RefCell<f32> = RefCell::new(1.0);
/// }
/// # fn main() {}
/// ```
///
#[macro_export]
#[allow_internal_unstable]
macro_rules! thread_local {
    // rule 0: empty (base case for the recursion)
    () => {};

    // rule 1: process multiple declarations where the first one is private
    ($(#[$attr:meta])* static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => (
        thread_local!($(#[$attr])* static $name: $t = $init); // go to rule 2
        thread_local!($($rest)*);
    );

    // rule 2: handle a single private declaration
    ($(#[$attr:meta])* static $name:ident: $t:ty = $init:expr) => (
        $(#[$attr])* static $name: $crate::LocalKey<$t> =
            __thread_local_inner!($t, $init);
    );

    // rule 3: handle multiple declarations where the first one is public
    ($(#[$attr:meta])* pub static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => (
        thread_local!($(#[$attr])* pub static $name: $t = $init); // go to rule 4
        thread_local!($($rest)*);
    );

    // rule 4: handle a single public declaration
    ($(#[$attr:meta])* pub static $name:ident: $t:ty = $init:expr) => (
        $(#[$attr])* pub static $name: $crate::LocalKey<$t> =
            __thread_local_inner!($t, $init);
    );
}

#[macro_export]
#[allow_internal_unstable]
macro_rules! __thread_local_inner {
    ($t:ty, $init:expr) => {{

        use core::cell::UnsafeCell;

        fn __init() -> $t { $init }

        fn __getit() -> Option<&'static UnsafeCell<Option<$t>>>
        {
            #[thread_local]
            static __KEY: $crate::local::LocalKeyInner<$t> =
                $crate::local::LocalKeyInner::new();

            __KEY.get()
        }

        $crate::LocalKey::new(__getit, __init)
    }}
}

/// Indicator of the state of a thread local storage key.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum LocalKeyState {
    /// All keys are in this state whenever a thread starts. Keys will
    /// transition to the `Valid` state once the first call to [`with`] happens
    /// and the initialization expression succeeds.
    ///
    /// Keys in the `Uninitialized` state will yield a reference to the closure
    /// passed to [`with`] so long as the initialization routine does not panic.
    ///
    Uninitialized,
    /// Once a key has been accessed successfully, it will enter the `Valid`
    /// state. Keys in the `Valid` state will remain so until the thread exits,
    /// at which point the destructor will be run and the key will enter the
    /// `Destroyed` state.
    ///
    /// Keys in the `Valid` state will be guaranteed to yield a reference to the
    /// closure passed to [`with`].
    ///
    Valid,
    /// if TLS data needs to be destructed, TCS policy must be Bound, The key will
    /// enter the 'Error' state.
    ///
    Error,
}

impl<T: 'static> LocalKey<T> {

    pub const fn new(inner: fn() -> Option<&'static UnsafeCell<Option<T>>>,
                     init: fn() -> T) -> LocalKey<T> {
        LocalKey {
            inner: inner,
            init: init,
        }
    }

    /// Acquires a reference to the value in this TLS key.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet.
    ///
    /// # Panics
    ///
    /// This function will `panic!()` if TLS data needs to be destructed,
    /// TCS policy must be Bound.
    pub fn with<F, R>(&'static self, f: F) -> R where F: FnOnce(&T) -> R {

        unsafe {
            let slot = (self.inner)();
            let slot = slot.expect("if TLS data needs to be destructed, TCS policy must be Bound.");
            f(match *slot.get() {
                Some(ref inner) => inner,
                None => self.init(slot),
            })
        }
    }

    unsafe fn init(&self, slot: &UnsafeCell<Option<T>>) -> &T {

        let value = (self.init)();
        let ptr = slot.get();

        mem::replace(&mut *ptr, Some(value));

        (*ptr).as_ref().unwrap()
    }

    /// Query the current state of this key.
    ///
    pub fn state(&'static self) -> LocalKeyState {
        unsafe {
            match (self.inner)() {
                Some(cell) => {
                    match *cell.get() {
                        Some(..) => LocalKeyState::Valid,
                        None => LocalKeyState::Uninitialized,
                    }
                }
                None => LocalKeyState::Error,
            }
        }
    }
}

pub struct LocalKeyInner<T> {
    inner: UnsafeCell<Option<T>>,
}

unsafe impl<T> Sync for LocalKeyInner<T> { }

impl<T> LocalKeyInner<T> {

    pub const fn new() -> LocalKeyInner<T> {
        LocalKeyInner {
            inner: UnsafeCell::new(None),
        }
    }

    pub fn get(&'static self) -> Option<&'static UnsafeCell<Option<T>>> {
        unsafe {
            if intrinsics::needs_drop::<T>() {
                match SgxGlobalData::new().thread_policy() {
                    SgxThreadPolicy::Unbound => {
                        return None;
                    },
                    SgxThreadPolicy::Bound => (),
                }
            }
        }
        Some(&self.inner)
    }
}
