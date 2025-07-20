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

use crate::cell::Cell;
use crate::ops::{Deref, DerefMut};

/// Pads and aligns a value to the length of a cache line.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq)]
// Starting from Intel's Sandy Bridge, spatial prefetcher is now pulling pairs of 64-byte cache
// lines at a time, so we have to align to 128 bytes rather than 64.
//
// Sources:
// - https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-optimization-manual.pdf
// - https://github.com/facebook/folly/blob/1b5288e6eea6df074758f877c849b6e73bbb9fbb/folly/lang/Align.h#L107
//
// ARM's big.LITTLE architecture has asymmetric cores and "big" cores have 128-byte cache line size.
//
// Sources:
// - https://www.mono-project.com/news/2016/09/12/arm64-icache/
//
// powerpc64 has 128-byte cache line size.
//
// Sources:
// - https://github.com/golang/go/blob/3dd58676054223962cd915bb0934d1f9f489d4d2/src/internal/cpu/cpu_ppc64x.go#L9
#[repr(align(128))]
pub struct CachePadded<T> {
    value: T,
}

impl<T> CachePadded<T> {
    /// Pads and aligns a value to the length of a cache line.
    pub fn new(value: T) -> CachePadded<T> {
        CachePadded::<T> { value }
    }
}

impl<T> Deref for CachePadded<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for CachePadded<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

const SPIN_LIMIT: u32 = 6;

/// Performs quadratic backoff in spin loops.
pub struct Backoff {
    step: Cell<u32>,
}

impl Backoff {
    /// Creates a new `Backoff`.
    pub fn new() -> Self {
        Backoff { step: Cell::new(0) }
    }

    /// Backs off using lightweight spinning.
    ///
    /// This method should be used for retrying an operation because another thread made
    /// progress. i.e. on CAS failure.
    #[inline]
    pub fn spin_light(&self) {
        let step = self.step.get().min(SPIN_LIMIT);
        for _ in 0..step.pow(2) {
            crate::hint::spin_loop();
        }

        self.step.set(self.step.get() + 1);
    }

    /// Backs off using heavyweight spinning.
    ///
    /// This method should be used in blocking loops where parking the thread is not an option.
    #[inline]
    pub fn spin_heavy(&self) {
        if self.step.get() <= SPIN_LIMIT {
            for _ in 0..self.step.get().pow(2) {
                crate::hint::spin_loop()
            }
        } else {
            crate::thread::yield_now();
        }

        self.step.set(self.step.get() + 1);
    }
}
