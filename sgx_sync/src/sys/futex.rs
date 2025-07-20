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

#![allow(deprecated)]

use crate::futex::FutexClockId;
use crate::lazy_lock::LazyLock;
use crate::sys::ocall::{self, Timeout, Timespec};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher, SipHasher13};
use core::intrinsics;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use sgx_trts::sync::SpinMutex;
use sgx_trts::tcs::{self, TcsId};
use sgx_trts::trts;
use sgx_types::error::errno::{EAGAIN, EINTR, EINVAL, ETIMEDOUT};
use sgx_types::error::OsResult;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Futex(usize);

impl Futex {
    const FUTEX_BITSET_MATCH_ANY: u32 = 0xFFFF_FFFF;

    #[inline]
    pub fn new(addr: usize) -> Futex {
        Futex(addr)
    }

    #[inline]
    pub fn wait(&self, expected: u32, timeout: Option<Duration>) -> OsResult {
        let timeout =
            timeout.map(|dur| Timeout::new(dur.into(), FutexClockId::Monotonic as u32, false));
        self.wait_with_timeout(expected, timeout, Self::FUTEX_BITSET_MATCH_ANY)
    }

    pub fn wait_bitset(
        &self,
        expected: u32,
        timeout: Option<(Timespec, FutexClockId)>,
        bitset: u32,
    ) -> OsResult {
        let timeout = timeout.map(|(ts, clockid)| Timeout::new(ts, clockid.into(), true));
        self.wait_with_timeout(expected, timeout, bitset)
    }

    fn wait_with_timeout(&self, expected: u32, timeout: Option<Timeout>, bitset: u32) -> OsResult {
        let (_, bucket) = FUTEX_BUCKETS.get_bucket(self);
        let mut futex_bucket = bucket.lock();

        // Check the futex value
        if self.load_val() != expected {
            bail!(EAGAIN);
        }

        let item = Item::new(*self, bitset);
        futex_bucket.save_item(item.clone());

        // Must make sure that no locks are holded by this thread before wait
        drop(futex_bucket);
        item.wait(timeout)
    }

    #[inline]
    pub fn wake(self, max_count: usize) -> OsResult<usize> {
        self.wake_bitset(max_count, Self::FUTEX_BITSET_MATCH_ANY)
    }

    pub fn wake_bitset(&self, max_count: usize, bitset: u32) -> OsResult<usize> {
        if max_count > i32::MAX as usize {
            bail!(EINVAL);
        }
        // Get and lock the futex bucket
        let (_, bucket) = FUTEX_BUCKETS.get_bucket(self);
        let mut futex_bucket = bucket.lock();

        // Dequeue and wake up the items in the bucket
        let count = futex_bucket.wake_items(self, max_count, bitset);
        Ok(count)
    }

    #[inline]
    pub fn cmp_requeue(
        &self,
        max_nwakes: usize,
        new_futex: &Futex,
        max_nrequeues: usize,
        expected: u32,
    ) -> OsResult<usize> {
        self.requeue_internal(max_nwakes, new_futex, max_nrequeues, Some(expected))
            .map(|(nwakes, nrequeues)| nwakes + nrequeues)
    }

    #[inline]
    pub fn requeue(
        &self,
        max_nwakes: usize,
        new_futex: &Futex,
        max_nrequeues: usize,
    ) -> OsResult<usize> {
        self.requeue_internal(max_nwakes, new_futex, max_nrequeues, None)
            .map(|(nwakes, _)| nwakes)
    }

    fn requeue_internal(
        &self,
        max_nwakes: usize,
        new_futex: &Futex,
        max_nrequeues: usize,
        expected: Option<u32>,
    ) -> OsResult<(usize, usize)> {
        if max_nwakes > i32::MAX as usize || max_nrequeues > i32::MAX as usize {
            bail!(EINVAL);
        }

        if self == new_futex || max_nwakes == i32::MAX as usize {
            return self.wake(max_nwakes).map(|n| (n, 0));
        }

        let (bucket_idx, bucket) = FUTEX_BUCKETS.get_bucket(self);
        let (new_bucket_idx, new_bucket) = FUTEX_BUCKETS.get_bucket(new_futex);
        let total_number = {
            if bucket_idx != new_bucket_idx {
                let (mut futex_bucket, mut futex_new_bucket) = {
                    if bucket_idx < new_bucket_idx {
                        let futex_bucket = bucket.lock();
                        let futex_new_bucket = new_bucket.lock();
                        (futex_bucket, futex_new_bucket)
                    } else {
                        // bucket_idx > new_bucket_idx
                        let futex_new_bucket = new_bucket.lock();
                        let futex_bucket = bucket.lock();
                        (futex_bucket, futex_new_bucket)
                    }
                };

                if let Some(expected) = expected {
                    if self.load_val() != expected {
                        bail!(EAGAIN);
                    }
                }

                let nwakes =
                    futex_bucket.wake_items(self, max_nwakes, Self::FUTEX_BITSET_MATCH_ANY);
                let nrequeues = futex_bucket.requeue_items_to_another_bucket(
                    self,
                    &mut futex_new_bucket,
                    new_futex,
                    max_nrequeues,
                );
                (nwakes, nrequeues)
            } else {
                // bucket_idx == new_bucket_idx
                let mut futex_bucket = bucket.lock();
                let nwakes =
                    futex_bucket.wake_items(self, max_nwakes, Self::FUTEX_BITSET_MATCH_ANY);
                let nrequeues = futex_bucket.update_item_keys(self, new_futex, max_nrequeues);
                (nwakes, nrequeues)
            }
        };
        Ok(total_number)
    }

    fn load_val(&self) -> u32 {
        unsafe { intrinsics::atomic_load_seqcst(self.0 as *const u32) }
    }

    fn addr(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
struct Bucket {
    queue: VecDeque<Item>,
}

type BucketRef = Arc<SpinMutex<Bucket>>;

impl Bucket {
    pub fn new() -> Bucket {
        Bucket {
            queue: VecDeque::new(),
        }
    }

    fn save_item(&mut self, futex_item: Item) {
        self.queue.push_back(futex_item);
    }

    fn del_item(&mut self, futex_item: &Item) -> Option<Item> {
        self.queue
            .iter()
            .position(|item| item == futex_item)
            .and_then(|index| self.queue.remove(index))
    }

    fn wake_items(&mut self, futex: &Futex, max_count: usize, bitset: u32) -> usize {
        let mut count = 0;
        let mut items_to_wake = Vec::new();

        self.queue.retain(|item| {
            if count >= max_count || *futex != item.futex || (bitset & item.bitset) == 0 {
                true
            } else {
                items_to_wake.push(item.clone());
                count += 1;
                false
            }
        });

        if !items_to_wake.is_empty() {
            Item::batch_wake(&items_to_wake);
        }
        count
    }

    pub fn update_item_keys(
        &mut self,
        futex: &Futex,
        new_futex: &Futex,
        max_count: usize,
    ) -> usize {
        let mut count = 0;
        for item in self
            .queue
            .iter_mut()
            .filter(|item| item.futex == *futex)
            .take(max_count)
        {
            item.futex = *new_futex;
            count += 1;
        }
        count
    }

    pub fn requeue_items_to_another_bucket(
        &mut self,
        futex: &Futex,
        another: &mut Self,
        new_futex: &Futex,
        max_nrequeues: usize,
    ) -> usize {
        let mut count = 0;
        self.queue.retain(|item| {
            if count >= max_nrequeues || *futex != item.futex {
                true
            } else {
                let mut new_item = item.clone();
                new_item.futex = *new_futex;
                another.save_item(new_item);
                count += 1;
                false
            }
        });
        count
    }
}

static BUCKET_COUNT: LazyLock<usize> =
    LazyLock::new(|| ((1 << 8) * (trts::cpu_core_num())).next_power_of_two() as usize);
static BUCKET_MASK: LazyLock<usize> = LazyLock::new(|| *BUCKET_COUNT - 1);
static FUTEX_BUCKETS: LazyLock<FutexBuckets> = LazyLock::new(|| FutexBuckets::new(*BUCKET_COUNT));

#[derive(Debug)]
struct FutexBuckets {
    vec: Vec<BucketRef>,
}

impl FutexBuckets {
    fn new(size: usize) -> FutexBuckets {
        let mut buckets = FutexBuckets {
            vec: Vec::with_capacity(size),
        };
        for _ in 0..size {
            let bucket = Arc::new(SpinMutex::new(Bucket::new()));
            buckets.vec.push(bucket);
        }
        buckets
    }

    fn get_bucket(&self, futex: &Futex) -> (usize, BucketRef) {
        let idx = *BUCKET_MASK & {
            // The addr is the multiples of 4, so we ignore the last 2 bits
            let addr = futex.addr() >> 2;
            let mut hasher = SipHasher13::new();
            addr.hash(&mut hasher);
            hasher.finish() as usize
        };
        (idx, self.vec[idx].clone())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Item {
    futex: Futex,
    bitset: u32,
    waiter: WaiterRef,
}

impl Item {
    fn new(futex: Futex, bitset: u32) -> Item {
        Item {
            futex,
            bitset,
            waiter: Arc::new(Waiter::new()),
        }
    }

    fn wait(&self, timeout: Option<Timeout>) -> OsResult {
        self.waiter.wait(timeout).map_err(|e| {
            let (_, bucket) = FUTEX_BUCKETS.get_bucket(&self.futex);
            let mut futex_bucket = bucket.lock();
            futex_bucket.del_item(self);
            e
        })
    }

    #[allow(dead_code)]
    fn wake(&self) {
        self.waiter.wake()
    }

    fn batch_wake(items: &[Item]) {
        let waiters: Vec<WaiterRef> = items.iter().map(|item| item.waiter.clone()).collect();
        Waiter::batch_wake(&waiters);
    }
}

#[derive(Debug)]
struct Waiter {
    tcs: TcsId,
    is_woken: AtomicBool,
}

type WaiterRef = Arc<Waiter>;

impl Waiter {
    fn new() -> Waiter {
        Waiter {
            tcs: tcs::current().id(),
            is_woken: AtomicBool::new(false),
        }
    }

    fn wait(&self, timeout: Option<Timeout>) -> OsResult {
        let current = tcs::current().id();
        if current != self.tcs {
            return Ok(());
        }
        while !self.is_woken() {
            ocall::thread_wait_event_ex(current, timeout).map_err(|e| {
                if (timeout.is_some() && e == ETIMEDOUT) || e == EINTR || e == EAGAIN {
                    self.set_woken();
                }
                e
            })?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn wake(&self) {
        if !self.fetch_and_set_woken() {
            let _ = ocall::thread_set_event(self.tcs);
        }
    }

    fn batch_wake(waiters: &[WaiterRef]) {
        let tcss: Vec<TcsId> = waiters
            .iter()
            .filter_map(|waiter| {
                // Only wake up items that are not woken.
                // Set the item to be woken if it is not woken.
                if !waiter.fetch_and_set_woken() {
                    Some(waiter.tcs)
                } else {
                    None
                }
            })
            .collect();
        let _ = ocall::thread_set_multiple_events(&tcss);
    }

    #[inline]
    fn is_woken(&self) -> bool {
        self.is_woken.load(Ordering::SeqCst)
    }

    #[inline]
    fn set_woken(&self) {
        self.is_woken.store(true, Ordering::SeqCst);
    }

    #[inline]
    fn fetch_and_set_woken(&self) -> bool {
        self.is_woken.fetch_or(true, Ordering::SeqCst)
    }
}

impl PartialEq for Waiter {
    fn eq(&self, other: &Self) -> bool {
        self.tcs == other.tcs
    }
}

impl Eq for Waiter {}
