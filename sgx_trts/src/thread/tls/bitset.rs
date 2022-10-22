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

use crate::thread::tls::{TLS_KEYS_BITSET_SIZE, USIZE_BITS};
use core::iter::{Enumerate, Peekable};
use core::slice::Iter;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct Bitset([AtomicUsize; TLS_KEYS_BITSET_SIZE]);

impl Bitset {
    pub const fn new() -> Bitset {
        Bitset([
            AtomicUsize::new(0),
            AtomicUsize::new(0),
            AtomicUsize::new(0),
            AtomicUsize::new(0),
        ])
    }

    pub fn get(&self, index: usize) -> bool {
        let (hi, lo) = Self::split(index);
        (self.0[hi].load(Ordering::Relaxed) & lo) != 0
    }

    // Not atomic.
    pub fn iter(&self) -> BitsetIter<'_> {
        BitsetIter {
            iter: self.0.iter().enumerate().peekable(),
            elem_idx: 0,
        }
    }

    pub fn clear(&self, index: usize) {
        let (hi, lo) = Self::split(index);
        self.0[hi].fetch_and(!lo, Ordering::Relaxed);
    }

    /// Sets any unset bit. Not atomic. Returns `None` if all bits were
    /// observed to be set.
    pub fn set(&self) -> Option<usize> {
        'elems: for (idx, elem) in self.0.iter().enumerate() {
            let mut current = elem.load(Ordering::Relaxed);
            loop {
                if 0 == !current {
                    continue 'elems;
                }
                let trailing_ones = (!current).trailing_zeros() as usize;
                match elem.compare_exchange(
                    current,
                    current | (1 << trailing_ones),
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return Some(idx * USIZE_BITS + trailing_ones),
                    Err(previous) => current = previous,
                }
            }
        }
        None
    }

    fn split(index: usize) -> (usize, usize) {
        (index / USIZE_BITS, 1 << (index % USIZE_BITS))
    }
}

pub struct BitsetIter<'a> {
    iter: Peekable<Enumerate<Iter<'a, AtomicUsize>>>,
    elem_idx: usize,
}

impl<'a> Iterator for BitsetIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.iter.peek().cloned().and_then(|(idx, elem)| {
            let elem = elem.load(Ordering::Relaxed);
            let low_mask = (1 << self.elem_idx) - 1;
            let next = elem & !low_mask;
            let next_idx = next.trailing_zeros() as usize;
            self.elem_idx = next_idx + 1;
            if self.elem_idx >= 64 {
                self.elem_idx = 0;
                self.iter.next();
            }
            match next_idx {
                64 => self.next(),
                _ => Some(idx * USIZE_BITS + next_idx),
            }
        })
    }
}
