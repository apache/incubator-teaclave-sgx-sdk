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

//! Thread local storage

use sgx_trts::enclave::{SgxGlobalData, SgxThreadPolicy};
use core::cell::UnsafeCell;
use core::mem;
use core::fmt;
use core::intrinsics;

pub struct LocalKey<T: 'static> {
    // This outer `LocalKey<T>` type is what's going to be stored in statics,
    // but actual data inside will sometimes be tagged with #[thread_local].
    // It's not valid for a true static to reference a #[thread_local] static,
    // so we get around that by exposing an accessor through a layer of function
    // indirection (this thunk).
    //
    // Note that the thunk is itself unsafe because the returned lifetime of the
    // slot where data lives, `'static`, is not actually valid. The lifetime
    // here is actually slightly shorter than the currently running thread!
    //
    // Although this is an extra layer of indirection, it should in theory be
    // trivially devirtualizable by LLVM because the value of `inner` never
    // changes and the constant should be readonly within a crate. This mainly
    // only runs into problems when TLS statics are exported across crates.
    inner: unsafe fn() -> Option<&'static UnsafeCell<Option<T>>>,

    // initialization routine to invoke to create a value
    init: fn() -> T,
}

impl<T: 'static> fmt::Debug for LocalKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("LocalKey { .. }")
    }
}

/// Declare a new thread local storage key of type [`sgx_trts::LocalKey`].
///
/// # Syntax
///
/// The macro wraps any number of static declarations and makes them thread local.
/// Publicity and attributes for each static are allowed. Example:
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
#[allow_internal_unstable(thread_local_internals)]
macro_rules! thread_local {
    // empty (base case for the recursion)
    () => {};

    // process multiple declarations
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => (
        __thread_local_inner!($(#[$attr])* $vis $name, $t, $init);
        thread_local!($($rest)*);
    );

    // handle a single declaration
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr) => (
        __thread_local_inner!($(#[$attr])* $vis $name, $t, $init);
    );
}

#[macro_export]
#[allow_internal_unstable(thread_local_internals, cfg_target_thread_local, thread_local)]
macro_rules! __thread_local_inner {
    (@key $(#[$attr:meta])* $vis:vis $name:ident, $t:ty, $init:expr) => {
        {
            #[inline]
            fn __init() -> $t { $init }

            unsafe fn __getit() -> $crate::option::Option<
                &'static $crate::cell::UnsafeCell<
                    $crate::option::Option<$t>>>
            {
                #[thread_local]
                static __KEY: $crate::thread::LocalKeyInner<$t> =
                    $crate::thread::LocalKeyInner::new();

                __KEY.get()
            }

            unsafe {
                $crate::thread::LocalKey::new(__getit, __init)
            }
        }
    };
    ($(#[$attr:meta])* $vis:vis $name:ident, $t:ty, $init:expr) => {
        $(#[$attr])* $vis static $name: $crate::thread::LocalKey<$t> =
            __thread_local_inner!(@key $(#[$attr])* $vis $name, $t, $init);
    }
}

pub struct AccessError {
    _private: (),
}

impl fmt::Debug for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AccessError").finish()
    }
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt("already destroyed", f)
    }
}

impl<T: 'static> LocalKey<T> {

    pub const unsafe fn new(inner: unsafe fn() -> Option<&'static UnsafeCell<Option<T>>>,
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
    pub fn with<F, R>(&'static self, f: F) -> R
                      where F: FnOnce(&T) -> R {
        self.try_with(f).expect("if TLS data needs to be destructed, TCS policy must be Bound.")
    }

    unsafe fn init(&self, slot: &UnsafeCell<Option<T>>) -> &T {

        let value = (self.init)();
        let ptr = slot.get();

        mem::replace(&mut *ptr, Some(value));

        (*ptr).as_ref().unwrap()
    }

    /// Acquires a reference to the value in this TLS key.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet. If the key has been destroyed (which may happen if this is called
    /// in a destructor), this function will return a ThreadLocalError.
    ///
    /// # Panics
    ///
    /// This function will still `panic!()` if the key is uninitialized and the
    /// key's initializer panics.
    pub fn try_with<F, R>(&'static self, f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&T) -> R,
    {
        unsafe {
            let slot = (self.inner)().ok_or(AccessError {
                _private: (),
            })?;
            Ok(f(match *slot.get() {
                Some(ref inner) => inner,
                None => self.init(slot),
            }))
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

    pub unsafe fn get(&self) -> Option<&'static UnsafeCell<Option<T>>> {

        if intrinsics::needs_drop::<T>() {
            match SgxGlobalData::new().thread_policy() {
                SgxThreadPolicy::Unbound => {
                    return None;
                },
                SgxThreadPolicy::Bound => (),
            }
        }
        Some(&*(&self.inner as * const _))
    }
}
