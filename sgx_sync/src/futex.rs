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

use crate::sys::futex as imp;
use core::marker::PhantomData;
use core::sync::atomic::AtomicI32;
use core::time::Duration;
use sgx_types::error::OsResult;

pub struct Futex<'a> {
    futex: imp::Futex,
    marker: PhantomData<&'a AtomicI32>,
}

impl<'a> Futex<'a> {
    pub fn new(futex: &AtomicI32) -> Futex<'a> {
        Futex {
            futex: imp::Futex::new(futex as *const _ as usize),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn wait(&self, expected: i32, timeout: Option<Duration>) -> OsResult {
        self.futex.wait(expected, timeout)
    }

    #[inline]
    pub fn wait_bitset(&self, expected: i32, timeout: Option<Duration>, bitset: u32) -> OsResult {
        self.futex.wait_bitset(expected, timeout, bitset)
    }

    #[inline]
    pub fn wake(self, count: i32) -> OsResult<usize> {
        self.futex.wake(count as usize)
    }

    #[inline]
    pub fn wake_bitset(self, count: i32, bitset: u32) -> OsResult<usize> {
        self.futex.wake_bitset(count as usize, bitset)
    }

    #[inline]
    pub fn cmp_requeue(
        &self,
        nwakes: i32,
        new_futex: &Futex,
        nrequeues: i32,
        expected: i32,
    ) -> OsResult<usize> {
        self.futex.cmp_requeue(
            nwakes as usize,
            &new_futex.futex,
            nrequeues as usize,
            expected,
        )
    }

    #[inline]
    pub fn requeue(&self, nwakes: i32, new_futex: &Futex, nrequeues: i32) -> OsResult<usize> {
        self.futex
            .requeue(nwakes as usize, &new_futex.futex, nrequeues as usize)
    }
}
