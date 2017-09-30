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

use core::fmt;
use sync::{SgxMutex, SgxCondvar};

/// A barrier enables multiple threads to synchronize the beginning
/// of some computation.
///
pub struct Barrier {
    lock: SgxMutex<BarrierState>,
    cvar: SgxCondvar,
    num_threads: usize,
}

// The inner state of a double barrier
struct BarrierState {
    count: usize,
    generation_id: usize,
}

/// A `BarrierWaitResult` is returned by [`wait`] when all threads in the [`Barrier`]
/// have rendezvoused.
///
pub struct BarrierWaitResult(bool);

impl fmt::Debug for Barrier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("Barrier { .. }")
    }
}

impl Barrier {
    /// Creates a new barrier that can block a given number of threads.
    ///
    /// A barrier will block `n`-1 threads which call [`wait`] and then wake up
    /// all threads at once when the `n`th thread calls [`wait`].
    ///
    pub fn new(n: usize) -> Barrier {
        Barrier {
            lock: SgxMutex::new(BarrierState {
                count: 0,
                generation_id: 0,
            }),
            cvar: SgxCondvar::new(),
            num_threads: n,
        }
    }

    /// Blocks the current thread until all threads have rendezvoused here.
    ///
    /// Barriers are re-usable after all threads have rendezvoused once, and can
    /// be used continuously.
    ///
    /// A single (arbitrary) thread will receive a [`BarrierWaitResult`] that
    /// returns `true` from [`is_leader`] when returning from this function, and
    /// all other threads will receive a result that will return `false` from
    /// [`is_leader`].
    ///
    pub fn wait(&self) -> BarrierWaitResult {
        let mut lock = self.lock.lock().unwrap();
        let local_gen = lock.generation_id;
        lock.count += 1;
        if lock.count < self.num_threads {
            // We need a while loop to guard against spurious wakeups.
            while local_gen == lock.generation_id &&
                  lock.count < self.num_threads {
                lock = self.cvar.wait(lock).unwrap();
            }
            BarrierWaitResult(false)
        } else {
            lock.count = 0;
            lock.generation_id = lock.generation_id.wrapping_add(1);
            self.cvar.broadcast();
            BarrierWaitResult(true)
        }
    }
}

impl fmt::Debug for BarrierWaitResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BarrierWaitResult")
            .field("is_leader", &self.is_leader())
            .finish()
    }
}

impl BarrierWaitResult {
    /// Returns whether this thread from [`wait`] is the "leader thread".
    ///
    /// Only one thread will have `true` returned from their result, all other
    /// threads will have `false` returned.
    ///
    pub fn is_leader(&self) -> bool { self.0 }
}