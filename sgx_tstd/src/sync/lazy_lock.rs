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

use crate::cell::Cell;
use crate::fmt;
use crate::ops::Deref;
use crate::panic::{RefUnwindSafe, UnwindSafe};
use crate::sync::OnceLock;

/// A value which is initialized on the first access.
///
/// This type is a thread-safe `Lazy`, and can be used in statics.
///
/// # Examples
///
/// ```
/// #![feature(once_cell)]
///
/// use std::collections::HashMap;
///
/// use std::sync::LazyLock;
///
/// static HASHMAP: LazyLock<HashMap<i32, String>> = LazyLock::new(|| {
///     println!("initializing");
///     let mut m = HashMap::new();
///     m.insert(13, "Spica".to_string());
///     m.insert(74, "Hoyten".to_string());
///     m
/// });
///
/// fn main() {
///     println!("ready");
///     std::thread::spawn(|| {
///         println!("{:?}", HASHMAP.get(&13));
///     }).join().unwrap();
///     println!("{:?}", HASHMAP.get(&74));
///
///     // Prints:
///     //   ready
///     //   initializing
///     //   Some("Spica")
///     //   Some("Hoyten")
/// }
/// ```
pub struct LazyLock<T, F = fn() -> T> {
    cell: OnceLock<T>,
    init: Cell<Option<F>>,
}

impl<T, F> LazyLock<T, F> {
    /// Creates a new lazy value with the given initializing
    /// function.
    pub const fn new(f: F) -> LazyLock<T, F> {
        LazyLock { cell: OnceLock::new(), init: Cell::new(Some(f)) }
    }
}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    /// Forces the evaluation of this lazy value and
    /// returns a reference to result. This is equivalent
    /// to the `Deref` impl, but is explicit.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(once_cell)]
    ///
    /// use std::sync::LazyLock;
    ///
    /// let lazy = LazyLock::new(|| 92);
    ///
    /// assert_eq!(LazyLock::force(&lazy), &92);
    /// assert_eq!(&*lazy, &92);
    /// ```
    pub fn force(this: &LazyLock<T, F>) -> &T {
        this.cell.get_or_init(|| match this.init.take() {
            Some(f) => f(),
            None => panic!("Lazy instance has previously been poisoned"),
        })
    }
}

impl<T, F: FnOnce() -> T> Deref for LazyLock<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        LazyLock::force(self)
    }
}

impl<T: Default> Default for LazyLock<T> {
    /// Creates a new lazy value using `Default` as the initializing function.
    fn default() -> LazyLock<T> {
        LazyLock::new(T::default)
    }
}

impl<T: fmt::Debug, F> fmt::Debug for LazyLock<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lazy").field("cell", &self.cell).finish_non_exhaustive()
    }
}

// We never create a `&F` from a `&LazyLock<T, F>` so it is fine
// to not impl `Sync` for `F`
// we do create a `&mut Option<F>` in `force`, but this is
// properly synchronized, so it only happens once
// so it also does not contribute to this impl.
unsafe impl<T, F: Send> Sync for LazyLock<T, F> where OnceLock<T>: Sync {}
// auto-derived `Send` impl is OK.

impl<T, F: UnwindSafe> RefUnwindSafe for LazyLock<T, F> where OnceLock<T>: RefUnwindSafe {}
impl<T, F: UnwindSafe> UnwindSafe for LazyLock<T, F> where OnceLock<T>: UnwindSafe {}

#[cfg(feature = "unit_test")]
mod tests;
