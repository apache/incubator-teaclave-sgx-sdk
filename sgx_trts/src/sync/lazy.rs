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

use core::cell::{Cell, UnsafeCell};
use core::fmt;
use core::mem;
use core::ops::Deref;

/// A cell which can be written to only once.
///
/// Unlike `RefCell`, a `OnceCell` only provides shared `&T` references to its value.
/// Unlike `Cell`, a `OnceCell` doesn't require copying or replacing the value to access it.
///
pub struct OnceCell<T> {
    // Invariant: written to at most once.
    inner: UnsafeCell<Option<T>>,
}

impl<T> Default for OnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for OnceCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get() {
            Some(v) => f.debug_tuple("OnceCell").field(v).finish(),
            None => f.write_str("OnceCell(Uninit)"),
        }
    }
}

impl<T: Clone> Clone for OnceCell<T> {
    fn clone(&self) -> OnceCell<T> {
        let res = OnceCell::new();
        if let Some(value) = self.get() {
            match res.set(value.clone()) {
                Ok(()) => (),
                Err(_) => unreachable!(),
            }
        }
        res
    }
}

impl<T: PartialEq> PartialEq for OnceCell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq> Eq for OnceCell<T> {}

impl<T> const From<T> for OnceCell<T> {
    /// Creates a new `OnceCell<T>` which already contains the given `value`.
    fn from(value: T) -> Self {
        OnceCell {
            inner: UnsafeCell::new(Some(value)),
        }
    }
}

impl<T> OnceCell<T> {
    /// Creates a new empty cell.
    #[must_use]
    pub const fn new() -> OnceCell<T> {
        OnceCell {
            inner: UnsafeCell::new(None),
        }
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty.
    pub fn get(&self) -> Option<&T> {
        // SAFETY: Safe due to `inner`'s invariant
        unsafe { &*self.inner.get() }.as_ref()
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.inner.get_mut().as_mut()
    }

    /// Sets the contents of the cell to `value`.
    ///
    /// # Errors
    ///
    /// This method returns `Ok(())` if the cell was empty and `Err(value)` if
    /// it was full.
    ///
    pub fn set(&self, value: T) -> Result<(), T> {
        // SAFETY: Safe because we cannot have overlapping mutable borrows
        let slot = unsafe { &*self.inner.get() };
        if slot.is_some() {
            return Err(value);
        }

        // SAFETY: This is the only place where we set the slot, no races
        // due to reentrancy/concurrency are possible, and we've
        // checked that slot is currently `None`, so this write
        // maintains the `inner`'s invariant.
        let slot = unsafe { &mut *self.inner.get() };
        *slot = Some(value);
        Ok(())
    }

    /// Gets the contents of the cell, initializing it with `f`
    /// if the cell was empty.
    ///
    /// # Panics
    ///
    /// If `f` panics, the panic is propagated to the caller, and the cell
    /// remains uninitialized.
    ///
    /// It is an error to reentrantly initialize the cell from `f`. Doing
    /// so results in a panic.
    ///
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
    /// If `f` panics, the panic is propagated to the caller, and the cell
    /// remains uninitialized.
    ///
    /// It is an error to reentrantly initialize the cell from `f`. Doing
    /// so results in a panic.
    ///
    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if let Some(val) = self.get() {
            return Ok(val);
        }
        /// Avoid inlining the initialization closure into the common path that fetches
        /// the already initialized value
        #[cold]
        fn outlined_call<F, T, E>(f: F) -> Result<T, E>
        where
            F: FnOnce() -> Result<T, E>,
        {
            f()
        }
        let val = outlined_call(f)?;
        // Note that *some* forms of reentrant initialization might lead to
        // UB (see `reentrant_init` test). I believe that just removing this
        // `assert`, while keeping `set/get` would be sound, but it seems
        // better to panic, rather than to silently use an old value.
        assert!(self.set(val).is_ok(), "reentrant init");
        Ok(self.get().unwrap())
    }

    /// Consumes the cell, returning the wrapped value.
    ///
    /// Returns `None` if the cell was empty.
    ///
    pub fn into_inner(self) -> Option<T> {
        // Because `into_inner` takes `self` by value, the compiler statically verifies
        // that it is not currently borrowed. So it is safe to move out `Option<T>`.
        self.inner.into_inner()
    }

    /// Takes the value out of this `OnceCell`, moving it back to an uninitialized state.
    ///
    /// Has no effect and returns `None` if the `OnceCell` hasn't been initialized.
    ///
    /// Safety is guaranteed by requiring a mutable reference.
    ///
    pub fn take(&mut self) -> Option<T> {
        mem::take(self).into_inner()
    }
}

/// A value which is initialized on the first access.
///
pub struct Lazy<T, F = fn() -> T> {
    cell: OnceCell<T>,
    init: Cell<Option<F>>,
}

impl<T: fmt::Debug, F> fmt::Debug for Lazy<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lazy")
            .field("cell", &self.cell)
            .field("init", &"..")
            .finish()
    }
}

impl<T, F> Lazy<T, F> {
    /// Creates a new lazy value with the given initializing function.
    ///
    pub const fn new(init: F) -> Lazy<T, F> {
        Lazy {
            cell: OnceCell::new(),
            init: Cell::new(Some(init)),
        }
    }
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    /// Forces the evaluation of this lazy value and returns a reference to
    /// the result.
    ///
    /// This is equivalent to the `Deref` impl, but is explicit.
    ///
    pub fn force(this: &Lazy<T, F>) -> &T {
        this.cell.get_or_init(|| match this.init.take() {
            Some(f) => f(),
            None => panic!("`Lazy` instance has previously been poisoned"),
        })
    }
}

impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        Lazy::force(self)
    }
}

impl<T: Default> Default for Lazy<T> {
    /// Creates a new lazy value using `Default` as the initializing function.
    fn default() -> Lazy<T> {
        Lazy::new(T::default)
    }
}
