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

//! Thread local storage

use crate::error::Error;
use crate::fmt;

/// A thread local storage key which owns its contents.
///
/// This key uses the fastest possible implementation available to it for the
/// target platform. It is instantiated with the [`thread_local!`] macro and the
/// primary method is the [`with`] method.
///
/// The [`with`] method yields a reference to the contained value which cannot be
/// sent across threads or escape the given closure.
///
/// # Initialization and Destruction
///
/// Initialization is dynamically performed on the first call to [`with`]
/// within a thread, and values that implement [`Drop`] get destructed when a
/// thread exits. Some caveats apply, which are explained below.
///
/// A `LocalKey`'s initializer cannot recursively depend on itself, and using
/// a `LocalKey` in this way will cause the initializer to infinitely recurse
/// on the first call to `with`.
///
/// # Examples
///
/// ```
/// use std::cell::RefCell;
/// use std::thread;
///
/// thread_local!(static FOO: RefCell<u32> = RefCell::new(1));
///
/// FOO.with(|f| {
///     assert_eq!(*f.borrow(), 1);
///     *f.borrow_mut() = 2;
/// });
///
/// // each thread starts out with the initial value of 1
/// let t = thread::spawn(move|| {
///     FOO.with(|f| {
///         assert_eq!(*f.borrow(), 1);
///         *f.borrow_mut() = 3;
///     });
/// });
///
/// // wait for the thread to complete and bail out on panic
/// t.join().unwrap();
///
/// // we retain our original value of 2 despite the child thread
/// FOO.with(|f| {
///     assert_eq!(*f.borrow(), 2);
/// });
/// ```
///
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
    inner: unsafe fn() -> Result<&'static T, AccessError>,
}

impl<T: 'static> fmt::Debug for LocalKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalKey").finish_non_exhaustive()
    }
}

/// Declare a new thread local storage key of type [`std::thread::LocalKey`].
///
/// # Syntax
///
/// The macro wraps any number of static declarations and makes them thread local.
/// Publicity and attributes for each static are allowed. Example:
///
/// ```
/// use std::cell::RefCell;
/// thread_local! {
///     pub static FOO: RefCell<u32> = RefCell::new(1);
///
///     #[allow(unused)]
///     static BAR: RefCell<f32> = RefCell::new(1.0);
/// }
/// # fn main() {}
/// ```
///
/// See [`LocalKey` documentation][`std::thread::LocalKey`] for more
/// information.
///
/// [`std::thread::LocalKey`]: crate::thread::LocalKey
#[macro_export]
#[allow_internal_unstable(thread_local_internals)]
macro_rules! thread_local {
    // empty (base case for the recursion)
    () => {};

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = const { $init:expr }; $($rest:tt)*) => (
        $crate::__thread_local_inner!($(#[$attr])* $vis $name, $t, const $init);
        $crate::thread_local!($($rest)*);
    );

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = const { $init:expr }) => (
        $crate::__thread_local_inner!($(#[$attr])* $vis $name, $t, const $init);
    );

    // process multiple declarations
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => (
        $crate::__thread_local_inner!($(#[$attr])* $vis $name, $t, $init);
        $crate::thread_local!($($rest)*);
    );

    // handle a single declaration
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr) => (
        $crate::__thread_local_inner!($(#[$attr])* $vis $name, $t, $init);
    );
}

#[macro_export]
#[allow_internal_unstable(thread_local_internals, cfg_target_thread_local, thread_local)]
#[allow_internal_unsafe]
macro_rules! __thread_local_inner {
    // used to generate the `LocalKey` value for const-initialized thread locals
    (@key $t:ty, const $init:expr) => {{
        #[cfg_attr(not(windows), inline)] // see comments below
        unsafe fn __getit() -> $crate::result::Result<&'static $t, $crate::thread::AccessError> {
            const _REQUIRE_UNSTABLE: () = $crate::thread::require_unstable_const_init_thread_local();

            if !$crate::mem::needs_drop::<$t>() || $crate::thread::thread_policy() == $crate::thread::SgxThreadPolicy::Bound {
                #[thread_local]
                static mut VAL: $t = $init;
                Ok(&VAL)
            } else {
                Err($crate::thread::AccessError::new(
                    "If TLS data needs to be destructed, TCS policy must be bound."
                ))
            }
        }

        unsafe {
            $crate::thread::LocalKey::new(__getit)
        }
    }};

    // used to generate the `LocalKey` value for `thread_local!`
    (@key $t:ty, $init:expr) => {
        {
            #[inline]
            fn __init() -> $t { $init }

            unsafe fn __getit() -> $crate::result::Result<&'static $t, $crate::thread::AccessError> {
                #[cfg(not(feature = "thread"))]
                #[thread_local]
                static __KEY: $crate::thread::__StaticLocalKeyInner<$t> =
                    $crate::thread::__StaticLocalKeyInner::new();

                #[cfg(feature = "thread")]
                #[thread_local]
                static __KEY: $crate::thread::__FastLocalKeyInner<$t> =
                    $crate::thread::__FastLocalKeyInner::new();

                __KEY.get(__init)
            }

            unsafe {
                $crate::thread::LocalKey::new(__getit)
            }
        }
    };
    ($(#[$attr:meta])* $vis:vis $name:ident, $t:ty, $($init:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::thread::LocalKey<$t> =
            $crate::__thread_local_inner!(@key $t, $($init)*);
    }
}

/// An error returned by [`LocalKey::try_with`](struct.LocalKey.html#method.try_with).
#[derive(Debug)]
pub struct AccessError {
    msg: &'static str,
}

impl AccessError {
    pub fn new(msg: &'static str) -> Self {
        Self { msg }
    }
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.msg, f)
    }
}

impl Error for AccessError {}

impl<T: 'static> LocalKey<T> {
    pub const unsafe fn new(inner: unsafe fn() -> Result<&'static T, AccessError>) -> LocalKey<T> {
        LocalKey { inner }
    }

    /// Acquires a reference to the value in this TLS key.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet.
    ///
    /// # Panics
    ///
    /// This function will `panic!()` if TLS data needs to be destructed,
    /// TCS policy must be bound.
    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.try_with(f).expect(
            "Cannot access a Thread Local Storage value."
        )
    }

    /// Acquires a reference to the value in this TLS key.
    ///
    /// This will lazily initialize the value if this thread has not referenced
    /// this key yet. If the key has been destroyed (which may happen if this is called
    /// in a destructor), this function will return an [`AccessError`].
    ///
    /// # Panics
    ///
    /// This function will still `panic!()` if the key is uninitialized and the
    /// key's initializer panics.
    #[inline]
    pub fn try_with<F, R>(&'static self, f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&T) -> R,
    {
        unsafe {
            let thread_local = (self.inner)()?;
            Ok(f(thread_local))
        }
    }
}

mod lazy {
    use crate::cell::UnsafeCell;
    use crate::hint;
    use crate::mem;

    pub struct LazyKeyInner<T> {
        inner: UnsafeCell<Option<T>>,
    }

    impl<T> LazyKeyInner<T> {
        pub const fn new() -> LazyKeyInner<T> {
            LazyKeyInner { inner: UnsafeCell::new(None) }
        }

        pub unsafe fn get(&self) -> Option<&'static T> {
            // SAFETY: The caller must ensure no reference is ever handed out to
            // the inner cell nor mutable reference to the Option<T> inside said
            // cell. This make it safe to hand a reference, though the lifetime
            // of 'static is itself unsafe, making the get method unsafe.
            (*self.inner.get()).as_ref()
        }

        /// The caller must ensure that no reference is active: this method
        /// needs unique access.
        pub unsafe fn initialize<F: FnOnce() -> T>(&self, init: F) -> &'static T {
            // Execute the initialization up front, *then* move it into our slot,
            // just in case initialization fails.
            let value = init();
            let ptr = self.inner.get();

            // SAFETY:
            //
            // note that this can in theory just be `*ptr = Some(value)`, but due to
            // the compiler will currently codegen that pattern with something like:
            //
            //      ptr::drop_in_place(ptr)
            //      ptr::write(ptr, Some(value))
            //
            // Due to this pattern it's possible for the destructor of the value in
            // `ptr` (e.g., if this is being recursively initialized) to re-access
            // TLS, in which case there will be a `&` and `&mut` pointer to the same
            // value (an aliasing violation). To avoid setting the "I'm running a
            // destructor" flag we just use `mem::replace` which should sequence the
            // operations a little differently and make this safe to call.
            //
            // The precondition also ensures that we are the only one accessing
            // `self` at the moment so replacing is fine.
            let _ = mem::replace(&mut *ptr, Some(value));

            // SAFETY: With the call to `mem::replace` it is guaranteed there is
            // a `Some` behind `ptr`, not a `None` so `unreachable_unchecked`
            // will never be reached.
            // After storing `Some` we want to get a reference to the contents of
            // what we just stored. While we could use `unwrap` here and it should
            // always work it empirically doesn't seem to always get optimized away,
            // which means that using something like `try_with` can pull in
            // panicking code and cause a large size bloat.
            match *ptr {
                Some(ref x) => x,
                None => hint::unreachable_unchecked(),
            }
        }

        /// The other methods hand out references while taking &self.
        /// As such, callers of this method must ensure no `&` and `&mut` are
        /// available and used at the same time.
        #[allow(unused)]
        pub unsafe fn take(&mut self) -> Option<T> {
            // SAFETY: See doc comment for this method.
            (*self.inner.get()).take()
        }
    }
}

pub mod statik {
    use super::lazy::LazyKeyInner;
    use super::AccessError;
    use crate::fmt;
    use crate::mem;
    use crate::thread::{self, SgxThreadPolicy};

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

        pub unsafe fn get(&self, init: fn() -> T) -> Result<&'static T, AccessError> {
            if !mem::needs_drop::<T>() || thread::thread_policy() == SgxThreadPolicy::Bound {
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
}

cfg_if! {
if #[cfg(feature = "thread")] {
    use sgx_libc::{c_void, c_long};
    use sgx_types::sgx_status_t;

    #[repr(C)]
    struct pthread_info {
        m_pthread: *mut c_void,       // struct _pthread
        m_local_storage: *mut c_void, // struct sgx_pthread_storage
        m_mark: [c_long; 8],          // jmpbuf
        m_state: sgx_status_t,
    }

    #[link(name = "sgx_pthread")]
    extern "C" {
        #[thread_local]
        static pthread_info_tls: pthread_info;
    }
}
} // cfg_if!

#[cfg(feature = "thread")]
pub mod fast {
    use super::lazy::LazyKeyInner;
    use super::AccessError;
    use crate::cell::Cell;
    use crate::fmt;
    use crate::mem;
    use crate::sys::thread_local_dtor::register_dtor;
    use crate::thread::{self, SgxThreadPolicy};

    #[derive(Copy, Clone)]
    enum DtorState {
        Unregistered,
        Registered,
        RunningOrHasRun,
    }

    // This data structure has been carefully constructed so that the fast path
    // only contains one branch on x86. That optimization is necessary to avoid
    // duplicated tls lookups on OSX.
    //
    // LLVM issue: https://bugs.llvm.org/show_bug.cgi?id=41722
    pub struct Key<T> {
        // If `LazyKeyInner::get` returns `None`, that indicates either:
        //   * The value has never been initialized
        //   * The value is being recursively initialized
        //   * The value has already been destroyed or is being destroyed
        // To determine which kind of `None`, check `dtor_state`.
        //
        // This is very optimizer friendly for the fast path - initialized but
        // not yet dropped.
        inner: LazyKeyInner<T>,

        // Metadata to keep track of the state of the destructor. Remember that
        // this variable is thread-local, not global.
        dtor_state: Cell<DtorState>,
    }

    impl<T> fmt::Debug for Key<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Key").finish_non_exhaustive()
        }
    }

    impl<T> Key<T> {
        pub const fn new() -> Key<T> {
            Key { inner: LazyKeyInner::new(), dtor_state: Cell::new(DtorState::Unregistered) }
        }

        pub unsafe fn get<F: FnOnce() -> T>(&self, init: F) -> Result<&'static T, AccessError> {
            // SAFETY: See the definitions of `LazyKeyInner::get` and
            // `try_initialize` for more informations.
            //
            // The caller must ensure no mutable references are ever active to
            // the inner cell or the inner T when this is called.
            // The `try_initialize` is dependant on the passed `init` function
            // for this.
            match self.inner.get() {
                Some(val) => Ok(val),
                None => self.try_initialize(init),
            }
        }

        // `try_initialize` is only called once per fast thread local variable,
        // except in corner cases where thread_local dtors reference other
        // thread_local's, or it is being recursively initialized.
        //
        // Macos: Inlining this function can cause two `tlv_get_addr` calls to
        // be performed for every call to `Key::get`.
        // LLVM issue: https://bugs.llvm.org/show_bug.cgi?id=41722
        #[inline(never)]
        unsafe fn try_initialize<F: FnOnce() -> T>(&self, init: F) -> Result<&'static T, AccessError> {
            if mem::needs_drop::<T>() && thread::thread_policy() == SgxThreadPolicy::Unbound {
                return Err(AccessError::new(
                    "If TLS data needs to be destructed, TCS policy must be bound."
                ));
            }

            if !super::pthread_info_tls.m_pthread.is_null() {
                // Note:
                // If the current thread was created by pthread_create, we should call
                // the try_register_dtor function. You can know whether the current thread has
                // been created by pthread_create() through the m_thread member of pthread_info
                // (thread local storage) of pthread library in intel sgx sdk.
                //
                // Destructor will only be called when a thread created by pthread_create exits,
                // because sys_common::thread_local::StaticKey does not call pthread_key_delete
                // to trigger the destructor.
                if !mem::needs_drop::<T>() || self.try_register_dtor() {
                // SAFETY: See comment above (his function doc).
                    Ok(self.inner.initialize(init))
                } else {
                    Err(AccessError::new("Failed to register destructor."))
                }
            } else {
                Ok(self.inner.initialize(init))
            }
        }

        // `try_register_dtor` is only called once per fast thread local
        // variable, except in corner cases where thread_local dtors reference
        // other thread_local's, or it is being recursively initialized.
        unsafe fn try_register_dtor(&self) -> bool {
            match self.dtor_state.get() {
                DtorState::Unregistered => {
                    // SAFETY: dtor registration happens before initialization.
                    // Passing `self` as a pointer while using `destroy_value<T>`
                    // is safe because the function will build a pointer to a
                    // Key<T>, which is the type of self and so find the correct
                    // size.
                    register_dtor(self as *const _ as *mut u8, destroy_value::<T>);
                    self.dtor_state.set(DtorState::Registered);
                    true
                }
                DtorState::Registered => {
                    // recursively initialized
                    true
                }
                DtorState::RunningOrHasRun => false,
            }
        }
    }

    unsafe extern "C" fn destroy_value<T>(ptr: *mut u8) {
        let ptr = ptr as *mut Key<T>;

        // SAFETY:
        //
        // The pointer `ptr` has been built just above and comes from
        // `try_register_dtor` where it is originally a Key<T> coming from `self`,
        // making it non-NUL and of the correct type.
        //
        // Right before we run the user destructor be sure to set the
        // `Option<T>` to `None`, and `dtor_state` to `RunningOrHasRun`. This
        // causes future calls to `get` to run `try_initialize_drop` again,
        // which will now fail, and return `None`.
        let value = (*ptr).inner.take();
        (*ptr).dtor_state.set(DtorState::RunningOrHasRun);
        drop(value);
    }
}

#[cfg(feature = "thread")]
pub mod os {
    use super::lazy::LazyKeyInner;
    use super::AccessError;
    use crate::cell::Cell;
    use crate::fmt;
    use crate::marker;
    use crate::ptr;
    use crate::sys_common::thread_local_key::StaticKey as OsStaticKey;

    pub struct Key<T> {
        // OS-TLS key that we'll use to key off.
        os: OsStaticKey,
        marker: marker::PhantomData<Cell<T>>,
    }

    impl<T> fmt::Debug for Key<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Key").finish_non_exhaustive()
        }
    }

    unsafe impl<T> Sync for Key<T> {}

    struct Value<T: 'static> {
        inner: LazyKeyInner<T>,
        key: &'static Key<T>,
    }

    impl<T: 'static> Key<T> {
        // Note:
        // 1. os::Key can be destructed normally when used for threads created by `pthread_create`.
        // 2. os::Key used in untrusted thread, the destructor will not be called.
        pub const fn new() -> Key<T> {
            Key { os: OsStaticKey::new(Some(destroy_value::<T>)), marker: marker::PhantomData }
        }

        /// It is a requirement for the caller to ensure that no mutable
        /// reference is active when this method is called.
        pub unsafe fn get(&'static self, init: fn() -> T) -> Result<&'static T, AccessError> {
            // SAFETY: See the documentation for this method.
            let ptr = self.os.get() as *mut Value<T>;
            if ptr as usize > 1 {
                // SAFETY: the check ensured the pointer is safe (its destructor
                // is not running) + it is coming from a trusted source (self).
                if let Some(value) = (*ptr).inner.get() {
                    return Ok(value);
                }
            }
            // SAFETY: At this point we are sure we have no value and so
            // initializing (or trying to) is safe.
            self.try_initialize(init)
        }

        // `try_initialize` is only called once per os thread local variable,
        // except in corner cases where thread_local dtors reference other
        // thread_local's, or it is being recursively initialized.
        unsafe fn try_initialize(&'static self, init: fn() -> T) -> Result<&'static T, AccessError> {
            // SAFETY: No mutable references are ever handed out meaning getting
            // the value is ok.
            let ptr = self.os.get() as *mut Value<T>;
            if ptr as usize == 1 {
                // destructor is running
                return Err(AccessError::new("Destructor is running."));
            }

            let ptr = if ptr.is_null() {
                // If the lookup returned null, we haven't initialized our own
                // local copy, so do that now.
                let ptr: Box<Value<T>> = box Value { inner: LazyKeyInner::new(), key: self };
                let ptr = Box::into_raw(ptr);
                // SAFETY: At this point we are sure there is no value inside
                // ptr so setting it will not affect anyone else.
                self.os.set(ptr as *mut u8);
                ptr
            } else {
                // recursive initialization
                ptr
            };

            // SAFETY: ptr has been ensured as non-NUL just above an so can be
            // dereferenced safely.
            Ok((*ptr).inner.initialize(init))
        }
    }

    unsafe extern "C" fn destroy_value<T: 'static>(ptr: *mut u8) {
        // SAFETY:
        //
        // The OS TLS ensures that this key contains a null value when this
        // destructor starts to run. We set it back to a sentinel value of 1 to
        // ensure that any future calls to `get` for this thread will return
        // `None`.
        //
        // Note that to prevent an infinite loop we reset it back to null right
        // before we return from the destructor ourselves.
        let ptr = Box::from_raw(ptr as *mut Value<T>);
        let key = ptr.key;
        key.os.set(1 as *mut u8);
        drop(ptr);
        key.os.set(ptr::null_mut());
    }
}
