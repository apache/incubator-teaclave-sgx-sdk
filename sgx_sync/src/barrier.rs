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

use crate::spin::SpinMutex;
use core::fmt;
use core::hint::spin_loop;

/// A barrier enables multiple threads to synchronize the beginning
/// of some computation.
pub struct Barrier {
    lock: SpinMutex<BarrierState>,
    num_threads: usize,
}

// The inner state of a double barrier
struct BarrierState {
    count: usize,
    generation_id: usize,
}

/// A `BarrierWaitResult` is returned by `Barrier::wait()` when all threads
/// in the `Barrier` have rendezvoused.
///
pub struct BarrierWaitResult(bool);

impl fmt::Debug for Barrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Barrier { .. }")
    }
}

impl Barrier {
    /// Creates a new barrier that can block a given number of threads.
    ///
    /// A barrier will block `n`-1 threads which call `wait()` and then wake
    /// up all threads at once when the `n`th thread calls `wait()`.
    ///
    pub const fn new(n: usize) -> Barrier {
        Barrier {
            lock: SpinMutex::new(BarrierState {
                count: 0,
                generation_id: 0,
            }),
            num_threads: n,
        }
    }

    /// Blocks the current thread until all threads have rendezvoused here.
    ///
    /// Barriers are re-usable after all threads have rendezvoused once, and can
    /// be used continuously.
    ///
    /// A single (arbitrary) thread will receive a `BarrierWaitResult` that
    /// returns `true` from `BarrierWaitResult::is_leader()` when returning
    /// from this function, and all other threads will receive a result that
    /// will return `false` from `BarrierWaitResult::is_leader()`.
    ///
    pub fn wait(&self) -> BarrierWaitResult {
        let mut lock = self.lock.lock();
        lock.count += 1;

        if lock.count < self.num_threads {
            // not the leader
            let local_gen = lock.generation_id;

            while local_gen == lock.generation_id && lock.count < self.num_threads {
                drop(lock);
                spin_loop();
                lock = self.lock.lock();
            }
            BarrierWaitResult(false)
        } else {
            // this thread is the leader,
            // and is responsible for incrementing the generation
            lock.count = 0;
            lock.generation_id = lock.generation_id.wrapping_add(1);
            BarrierWaitResult(true)
        }
    }
}

impl fmt::Debug for BarrierWaitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BarrierWaitResult")
            .field("is_leader", &self.is_leader())
            .finish()
    }
}

impl BarrierWaitResult {
    /// Returns `true` if this thread is the "leader thread" for the call to
    /// `Barrier::wait()`.
    ///
    /// Only one thread will have `true` returned from their result, all other
    /// threads will have `false` returned.
    ///
    pub fn is_leader(&self) -> bool {
        self.0
    }
}
