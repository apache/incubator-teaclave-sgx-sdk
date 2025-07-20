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

use super::lazy::LazyKeyInner;
use crate::fmt;
use crate::mem;
use crate::thread::AccessError;

use sgx_trts::tcs::{self, TcsPolicy};

#[allow_internal_unstable(thread_local_internals, cfg_target_thread_local, thread_local)]
#[allow_internal_unsafe]
#[rustc_macro_transparency = "semitransparent"]
pub macro thread_local_inner {
    // used to generate the `LocalKey` value for const-initialized thread locals
    (@key $t:ty, const $init:expr) => {{
        #[inline] // see comments below
        unsafe fn __getit(
            _init: $crate::option::Option<&mut $crate::option::Option<$t>>,
        ) -> $crate::result::Result<&'static $t, $crate::thread::AccessError> {
            const INIT_EXPR: $t = $init;

            if !$crate::mem::needs_drop::<$t>() || $crate::thread::tcs_policy() == $crate::thread::TcsPolicy::Bind {
                #[thread_local]
                static mut VAL: $t = INIT_EXPR;
                Ok(&VAL)
            } else {
                $crate::result::Result::Err($crate::thread::AccessError::new(
                    "If TLS data needs to be destructed, TCS policy must be bound."
                ))
            }
        }

        unsafe {
            $crate::thread::LocalKey::new(__getit)
        }
    }},

    // used to generate the `LocalKey` value for `thread_local!`
    (@key $t:ty, $init:expr) => {
        {
            #[inline]
            fn __init() -> $t { $init }
            #[inline]
            unsafe fn __getit(
                init: $crate::option::Option<&mut $crate::option::Option<$t>>,
            ) -> $crate::result::Result<&'static $t, $crate::thread::AccessError> {
                #[thread_local]
                static __KEY: $crate::thread::local_impl::Key<$t> =
                    $crate::thread::local_impl::Key::new();

                #[allow(unused_unsafe)]
                unsafe {
                    __KEY.get(move || {
                        if let $crate::option::Option::Some(init) = init {
                            if let $crate::option::Option::Some(value) = init.take() {
                                return value;
                            } else if $crate::cfg!(debug_assertions) {
                                $crate::unreachable!("missing default value");
                            }
                        }
                        __init()
                    })
                }
            }

            unsafe {
                $crate::thread::LocalKey::new(__getit)
            }
        }
    },
    ($(#[$attr:meta])* $vis:vis $name:ident, $t:ty, $($init:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::thread::LocalKey<$t> =
            $crate::thread::local_impl::thread_local_inner!(@key $t, $($init)*);
    },
}

pub struct Key<T> {
    inner: LazyKeyInner<T>,
}

unsafe impl<T> Sync for Key<T> {}

impl<T> fmt::Debug for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Key").finish_non_exhaustive()
    }
}

impl<T> Key<T> {
    pub const fn new() -> Key<T> {
        Key { inner: LazyKeyInner::new() }
    }

    pub unsafe fn get(&self, init: impl FnOnce() -> T) -> Result<&'static T, AccessError> {
        if !mem::needs_drop::<T>() || tcs::tcs_policy() == TcsPolicy::Bind {
            // SAFETY: The caller must ensure no reference is ever handed out to
            // the inner cell nor mutable reference to the Option<T> inside said
            // cell. This make it safe to hand a reference, though the lifetime
            // of 'static is itself unsafe, making the get method unsafe.
            let value = match self.inner.get() {
                Some(value) => value,
                None => self.inner.initialize(init),
            };
            Ok(value)
        } else {
            Err(AccessError::new(
                "If TLS data needs to be destructed, TCS policy must be bound."
            ))
        }
    }
}
