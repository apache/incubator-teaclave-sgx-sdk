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

#![allow(clippy::unnecessary_mut_passed)]

use crate::cell::UnsafeCell;
use crate::fmt;
use crate::marker::PhantomData;
use crate::mem::MaybeUninit;
use crate::panic::{RefUnwindSafe, UnwindSafe};
use crate::sync::Once;

/// A synchronization primitive which can be written to only once.
///
/// This type is a thread-safe [`OnceCell`], and can be used in statics.
///
/// [`OnceCell`]: crate::cell::OnceCell
///
/// # Examples
///
/// ```
/// use std::sync::OnceLock;
///
/// static CELL: OnceLock<String> = OnceLock::new();
/// assert!(CELL.get().is_none());
///
/// std::thread::spawn(|| {
///     let value: &String = CELL.get_or_init(|| {
///         "Hello, World!".to_string()
///     });
///     assert_eq!(value, "Hello, World!");
/// }).join().unwrap();
///
/// let value: Option<&String> = CELL.get();
/// assert!(value.is_some());
/// assert_eq!(value.unwrap().as_str(), "Hello, World!");
/// ```
pub struct OnceLock<T> {
    once: Once,
    // Whether or not the value is initialized is tracked by `once.is_completed()`.
    value: UnsafeCell<MaybeUninit<T>>,
    /// `PhantomData` to make sure dropck understands we're dropping T in our Drop impl.
    ///
    /// ```compile_fail,E0597
    /// use std::sync::OnceLock;
    ///
    /// struct A<'a>(&'a str);
    ///
    /// impl<'a> Drop for A<'a> {
    ///     fn drop(&mut self) {}
    /// }
    ///
    /// let cell = OnceLock::new();
    /// {
    ///     let s = String::new();
    ///     let _ = cell.set(A(&s));
    /// }
    /// ```
    _marker: PhantomData<T>,
}

impl<T> OnceLock<T> {
    /// Creates a new empty cell.
    #[inline]
    #[must_use]
    pub const fn new() -> OnceLock<T> {
        OnceLock {
            once: Once::new(),
            value: UnsafeCell::new(MaybeUninit::uninit()),
            _marker: PhantomData,
        }
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty, or being initialized. This
    /// method never blocks.
    #[inline]
    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            // Safe b/c checked is_initialized
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty. This method never blocks.
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_initialized() {
            // Safe b/c checked is_initialized and we have a unique access
            Some(unsafe { self.get_unchecked_mut() })
        } else {
            None
        }
    }

    /// Sets the contents of this cell to `value`.
    ///
    /// May block if another thread is currently attempting to initialize the cell. The cell is
    /// guaranteed to contain a value when set returns, though not necessarily the one provided.
    ///
    /// Returns `Ok(())` if the cell's value was set by this call.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// static CELL: OnceLock<i32> = OnceLock::new();
    ///
    /// fn needless_main() {
    ///     assert!(CELL.get().is_none());
    ///
    ///     std::thread::spawn(|| {
    ///         assert_eq!(CELL.set(92), Ok(()));
    ///     }).join().unwrap();
    ///
    ///     assert_eq!(CELL.set(62), Err(62));
    ///     assert_eq!(CELL.get(), Some(&92));
    /// }
    /// ```
    #[inline]
    pub fn set(&self, value: T) -> Result<(), T> {
        match self.try_insert(value) {
            Ok(_) => Ok(()),
            Err((_, value)) => Err(value),
        }
    }

    /// Sets the contents of this cell to `value` if the cell was empty, then
    /// returns a reference to it.
    ///
    /// May block if another thread is currently attempting to initialize the cell. The cell is
    /// guaranteed to contain a value when set returns, though not necessarily the one provided.
    ///
    /// Returns `Ok(&value)` if the cell was empty and `Err(&current_value, value)` if it was full.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(once_cell_try_insert)]
    ///
    /// use std::sync::OnceLock;
    ///
    /// static CELL: OnceLock<i32> = OnceLock::new();
    ///
    /// fn needless_main() {
    ///     assert!(CELL.get().is_none());
    ///
    ///     std::thread::spawn(|| {
    ///         assert_eq!(CELL.try_insert(92), Ok(&92));
    ///     }).join().unwrap();
    ///
    ///     assert_eq!(CELL.try_insert(62), Err((&92, 62)));
    ///     assert_eq!(CELL.get(), Some(&92));
    /// }
    /// ```
    #[inline]
    pub fn try_insert(&self, value: T) -> Result<&T, (&T, T)> {
        let mut value = Some(value);
        let res = self.get_or_init(|| value.take().unwrap());
        match value {
            None => Ok(res),
            Some(value) => Err((res, value)),
        }
    }

    /// Gets the contents of the cell, initializing it with `f` if the cell
    /// was empty.
    ///
    /// Many threads may call `get_or_init` concurrently with different
    /// initializing functions, but it is guaranteed that only one function
    /// will be executed.
    ///
    /// # Panics
    ///
    /// If `f` panics, the panic is propagated to the caller, and the cell
    /// remains uninitialized.
    ///
    /// It is an error to reentrantly initialize the cell from `f`. The
    /// exact outcome is unspecified. Current implementation deadlocks, but
    /// this may be changed to a panic in the future.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// let cell = OnceLock::new();
    /// let value = cell.get_or_init(|| 92);
    /// assert_eq!(value, &92);
    /// let value = cell.get_or_init(|| unreachable!());
    /// assert_eq!(value, &92);
    /// ```
    #[inline]
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        match self.get_or_try_init(|| Ok::<T, !>(f())) {
            Ok(val) => val,
            Err(_) => unreachable!(),
        }
    }

    /// Gets the contents of the cell, initializing it with `f` if
    /// the cell was empty. If the cell was empty and `f` failed, an
    /// error is returned.
    ///
    /// # Panics
    ///
    /// If `f` panics, the panic is propagated to the caller, and
    /// the cell remains uninitialized.
    ///
    /// It is an error to reentrantly initialize the cell from `f`.
    /// The exact outcome is unspecified. Current implementation
    /// deadlocks, but this may be changed to a panic in the future.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(once_cell_try)]
    ///
    /// use std::sync::OnceLock;
    ///
    /// let cell = OnceLock::new();
    /// assert_eq!(cell.get_or_try_init(|| Err(())), Err(()));
    /// assert!(cell.get().is_none());
    /// let value = cell.get_or_try_init(|| -> Result<i32, ()> {
    ///     Ok(92)
    /// });
    /// assert_eq!(value, Ok(&92));
    /// assert_eq!(cell.get(), Some(&92))
    /// ```
    #[inline]
    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // Fast path check
        // NOTE: We need to perform an acquire on the state in this method
        // in order to correctly synchronize `LazyLock::force`. This is
        // currently done by calling `self.get()`, which in turn calls
        // `self.is_initialized()`, which in turn performs the acquire.
        if let Some(value) = self.get() {
            return Ok(value);
        }
        self.initialize(f)?;

        debug_assert!(self.is_initialized());

        // SAFETY: The inner value has been initialized
        Ok(unsafe { self.get_unchecked() })
    }

    /// Consumes the `OnceLock`, returning the wrapped value. Returns
    /// `None` if the cell was empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// let cell: OnceLock<String> = OnceLock::new();
    /// assert_eq!(cell.into_inner(), None);
    ///
    /// let cell = OnceLock::new();
    /// cell.set("hello".to_string()).unwrap();
    /// assert_eq!(cell.into_inner(), Some("hello".to_string()));
    /// ```
    #[inline]
    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }

    /// Takes the value out of this `OnceLock`, moving it back to an uninitialized state.
    ///
    /// Has no effect and returns `None` if the `OnceLock` hasn't been initialized.
    ///
    /// Safety is guaranteed by requiring a mutable reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// let mut cell: OnceLock<String> = OnceLock::new();
    /// assert_eq!(cell.take(), None);
    ///
    /// let mut cell = OnceLock::new();
    /// cell.set("hello".to_string()).unwrap();
    /// assert_eq!(cell.take(), Some("hello".to_string()));
    /// assert_eq!(cell.get(), None);
    /// ```
    #[inline]
    pub fn take(&mut self) -> Option<T> {
        if self.is_initialized() {
            self.once = Once::new();
            // SAFETY: `self.value` is initialized and contains a valid `T`.
            // `self.once` is reset, so `is_initialized()` will be false again
            // which prevents the value from being read twice.
            unsafe { Some((&mut *self.value.get()).assume_init_read()) }
        } else {
            None
        }
    }

    #[inline]
    fn is_initialized(&self) -> bool {
        self.once.is_completed()
    }

    #[cold]
    fn initialize<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let mut res: Result<(), E> = Ok(());
        let slot = &self.value;

        // Ignore poisoning from other threads
        // If another thread panics, then we'll be able to run our closure
        self.once.call_once_force(|p| {
            match f() {
                Ok(value) => {
                    unsafe { (&mut *slot.get()).write(value) };
                }
                Err(e) => {
                    res = Err(e);

                    // Treat the underlying `Once` as poisoned since we
                    // failed to initialize our value. Calls
                    p.poison();
                }
            }
        });
        res
    }

    /// # Safety
    ///
    /// The value must be initialized
    #[inline]
    unsafe fn get_unchecked(&self) -> &T {
        debug_assert!(self.is_initialized());
        (&*self.value.get()).assume_init_ref()
    }

    /// # Safety
    ///
    /// The value must be initialized
    #[inline]
    unsafe fn get_unchecked_mut(&mut self) -> &mut T {
        debug_assert!(self.is_initialized());
        (&mut *self.value.get()).assume_init_mut()
    }
}

// Why do we need `T: Send`?
// Thread A creates a `OnceLock` and shares it with
// scoped thread B, which fills the cell, which is
// then destroyed by A. That is, destructor observes
// a sent value.
unsafe impl<T: Sync + Send> Sync for OnceLock<T> {}
unsafe impl<T: Send> Send for OnceLock<T> {}

impl<T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for OnceLock<T> {}
impl<T: UnwindSafe> UnwindSafe for OnceLock<T> {}

impl<T> Default for OnceLock<T> {
    /// Creates a new empty cell.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// fn needless_main() {
    ///     assert_eq!(OnceLock::<()>::new(), OnceLock::default());
    /// }
    /// ```
    #[inline]
    fn default() -> OnceLock<T> {
        OnceLock::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for OnceLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("OnceLock");
        match self.get() {
            Some(v) => d.field(v),
            None => d.field(&format_args!("<uninit>")),
        };
        d.finish()
    }
}

impl<T: Clone> Clone for OnceLock<T> {
    #[inline]
    fn clone(&self) -> OnceLock<T> {
        let cell = Self::new();
        if let Some(value) = self.get() {
            match cell.set(value.clone()) {
                Ok(()) => (),
                Err(_) => unreachable!(),
            }
        }
        cell
    }
}

impl<T> From<T> for OnceLock<T> {
    /// Create a new cell with its contents set to `value`.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// # fn needless_main() -> Result<(), i32> {
    /// let a = OnceLock::from(3);
    /// let b = OnceLock::new();
    /// b.set(3)?;
    /// assert_eq!(a, b);
    /// Ok(())
    /// # }
    /// ```
    #[inline]
    fn from(value: T) -> Self {
        let cell = Self::new();
        match cell.set(value) {
            Ok(()) => cell,
            Err(_) => unreachable!(),
        }
    }
}

impl<T: PartialEq> PartialEq for OnceLock<T> {
    #[inline]
    fn eq(&self, other: &OnceLock<T>) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq> Eq for OnceLock<T> {}

unsafe impl<#[may_dangle] T> Drop for OnceLock<T> {
    #[inline]
    fn drop(&mut self) {
        if self.is_initialized() {
            // SAFETY: The cell is initialized and being dropped, so it can't
            // be accessed again. We also don't touch the `T` other than
            // dropping it, which validates our usage of #[may_dangle].
            unsafe { (&mut *self.value.get()).assume_init_drop() };
        }
    }
}

#[cfg(feature = "unit_test")]
mod tests;
