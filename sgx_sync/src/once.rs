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

use core::cell::Cell;
use core::fmt;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Initialization value for static [`Once`] values.
pub const ONCE_INIT: Once = Once::new();

/// A synchronization primitive which can be used to run a one-time global
/// initialization. Useful for one-time initialization for FFI or related
/// functionality. This type can only be constructed with [`Once::new()`].
pub struct Once {
    state: AtomicUsize,
}

unsafe impl Sync for Once {}
unsafe impl Send for Once {}

const INCOMPLETE: usize = 0x0;
const POISONED: usize = 0x1;
const RUNNING: usize = 0x2;
const COMPLETE: usize = 0x3;

impl Once {
    /// Creates a new `Once` value.
    #[inline]
    pub const fn new() -> Once {
        Once {
            state: AtomicUsize::new(INCOMPLETE),
        }
    }

    /// Performs an initialization routine once and only once. The given closure
    /// will be executed if this is the first time `call_once` has been called,
    /// and otherwise the routine will *not* be invoked.
    ///
    /// This method will block the calling thread if another initialization
    /// routine is currently running.
    ///
    /// When this function returns, it is guaranteed that some initialization
    /// has run and completed (it may not be the closure specified). It is also
    /// guaranteed that any memory writes performed by the executed closure can
    /// be reliably observed by other threads at this point (there is a
    /// happens-before relation between the closure and code executing after the
    /// return).
    ///
    /// If the given closure recursively invokes `call_once` on the same [`Once`]
    /// instance the exact behavior is not specified, allowed outcomes are
    /// a panic or a deadlock.
    ///
    /// # Panics
    ///
    /// The closure `f` will only be executed once if this is called
    /// concurrently amongst many threads. If that closure panics, however, then
    /// it will *poison* this [`Once`] instance, causing all future invocations of
    /// `call_once` to also panic.
    pub fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        // Fast path check
        if self.is_completed() {
            return;
        }

        let mut f = Some(f);
        self.call_inner(false, &mut |_| f.take().unwrap()());
    }

    /// Performs the same function as [`call_once()`] except ignores poisoning.
    ///
    /// Unlike [`call_once()`], if this [`Once`] has been poisoned (i.e., a previous
    /// call to [`call_once()`] or [`call_once_force()`] caused a panic), calling
    /// [`call_once_force()`] will still invoke the closure `f` and will _not_
    /// result in an immediate panic. If `f` panics, the [`Once`] will remain
    /// in a poison state. If `f` does _not_ panic, the [`Once`] will no
    /// longer be in a poison state and all future calls to [`call_once()`] or
    /// [`call_once_force()`] will be no-ops.
    ///
    /// The closure `f` is yielded a [`OnceState`] structure which can be used
    /// to query the poison status of the [`Once`].
    ///
    pub fn call_once_force<F>(&self, f: F)
    where
        F: FnOnce(&OnceState),
    {
        // Fast path check
        if self.is_completed() {
            return;
        }

        let mut f = Some(f);
        self.call_inner(true, &mut |p| f.take().unwrap()(p));
    }

    /// Returns `true` if some [`call_once()`] call has completed
    /// successfully. Specifically, `is_completed` will return false in
    /// the following situations:
    ///   * [`call_once()`] was not called at all,
    ///   * [`call_once()`] was called, but has not yet completed,
    ///   * the [`Once`] instance is poisoned
    ///
    /// This function returning `false` does not mean that [`Once`] has not been
    /// executed. For example, it may have been executed in the time between
    /// when `is_completed` starts executing and when it returns, in which case
    /// the `false` return value would be stale (but still permissible).
    ///
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == COMPLETE
    }

    // This is a non-generic function to reduce the monomorphization cost of
    // using `call_once` (this isn't exactly a trivial or small implementation).
    //
    // Additionally, this is tagged with `#[cold]` as it should indeed be cold
    // and it helps let LLVM know that calls to this function should be off the
    // fast path. Essentially, this should help generate more straight line code
    // in LLVM.
    //
    // Finally, this takes an `FnMut` instead of a `FnOnce` because there's
    // currently no way to take an `FnOnce` and call it via virtual dispatch
    // without some allocation overhead.
    #[cold]
    fn call_inner(&self, ignore_poisoning: bool, init: &mut dyn FnMut(&OnceState)) {
        let mut state = self.state.load(Ordering::Acquire);
        loop {
            match state {
                COMPLETE => break,
                POISONED if !ignore_poisoning => {
                    // Panic to propagate the poison.
                    panic!("Once instance has previously been poisoned");
                }
                POISONED | INCOMPLETE => {
                    let exchange_result = self.state.compare_exchange(
                        state,
                        RUNNING,
                        Ordering::Acquire,
                        Ordering::Acquire,
                    );
                    if let Err(old) = exchange_result {
                        state = old;
                        continue;
                    }

                    let mut finish = Finish {
                        state: &self.state,
                        set_state_on_drop_to: POISONED,
                    };

                    // Run the initialization function, letting it know if we're
                    // poisoned or not.
                    let init_state = OnceState {
                        poisoned: state == POISONED,
                        set_state_on_drop_to: Cell::new(COMPLETE),
                    };
                    init(&init_state);
                    finish.set_state_on_drop_to = init_state.set_state_on_drop_to.get();
                    break;
                }
                _ => {
                    assert!(state == RUNNING);
                    self.wait();
                    state = self.state.load(Ordering::Acquire);
                }
            }
        }
    }

    fn wait(&self) {
        loop {
            match self.state.load(Ordering::Acquire) {
                RUNNING => spin_loop(),
                _ => return,
            }
        }
    }
}

impl fmt::Debug for Once {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Once { .. }")
    }
}

/// State yielded to [`Once::call_once_force()`]’s closure parameter. The state
/// can be used to query the poison status of the [`Once`].
#[derive(Debug)]
pub struct OnceState {
    poisoned: bool,
    set_state_on_drop_to: Cell<usize>,
}

impl OnceState {
    /// Returns `true` if the associated [`Once`] was poisoned prior to the
    /// invocation of the closure passed to [`Once::call_once_force()`].
    pub fn poisoned(&self) -> bool {
        self.poisoned
    }

    /// Poison the associated [`Once`] without explicitly panicking.
    // NOTE: This is currently only exposed for the `lazy` module
    pub(crate) fn poison(&self) {
        self.set_state_on_drop_to.set(POISONED);
    }
}

struct Finish<'a> {
    state: &'a AtomicUsize,
    set_state_on_drop_to: usize,
}

impl Drop for Finish<'_> {
    fn drop(&mut self) {
        // Swap out our state with however we finished.
        let state = self.state.swap(self.set_state_on_drop_to, Ordering::AcqRel);

        // We should only ever see an old state which was RUNNING.
        assert_eq!(state, RUNNING);
    }
}
