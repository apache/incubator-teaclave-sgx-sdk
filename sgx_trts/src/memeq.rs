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

//! Comparing buffer contents in constant time.
//!
//! This crate provides constant time memory comparison functions. These functions
//! are useful in cyptographic functions, defending against timing based side
//! channel attacks

use alloc::slice;
use core::mem;
use sgx_types::marker::BytewiseEquality;

pub trait ConsttimeMemEq<T: BytewiseEquality + ?Sized = Self> {
    fn consttime_memeq(&self, other: &T) -> bool;
    fn consttime_memne(&self, other: &T) -> bool {
        !self.consttime_memeq(other)
    }
}

impl<T> ConsttimeMemEq<[T]> for [T]
where
    T: Eq + BytewiseEquality,
{
    fn consttime_memeq(&self, other: &[T]) -> bool {
        if self.len() != other.len() {
            return false;
        }
        if self.as_ptr() == other.as_ptr() {
            return true;
        }
        let size = mem::size_of_val(self);
        unsafe {
            consttime_memequal(
                self.as_ptr() as *const u8,
                other.as_ptr() as *const u8,
                size,
            ) != 0
        }
    }
}

impl<T> ConsttimeMemEq<T> for T
where
    T: Eq + BytewiseEquality,
{
    fn consttime_memeq(&self, other: &T) -> bool {
        let size = mem::size_of_val(self);
        if size == 0 {
            return true;
        }
        unsafe {
            consttime_memequal(
                self as *const T as *const u8,
                other as *const T as *const u8,
                size,
            ) != 0
        }
    }
}

unsafe fn consttime_memequal(b1: *const u8, b2: *const u8, l: usize) -> i32 {
    let mut res: i32 = 0;
    let mut len = l;
    let p1 = slice::from_raw_parts(b1, l);
    let p2 = slice::from_raw_parts(b2, l);

    while len > 0 {
        len -= 1;
        res |= (p1[len] ^ p2[len]) as i32;
    }
    /*
     * Map 0 to 1 and [1, 256) to 0 using only constant-time
     * arithmetic.
     *
     * This is not simply `!res' because although many CPUs support
     * branchless conditional moves and many compilers will take
     * advantage of them, certain compilers generate branches on
     * certain CPUs for `!res'.
     */
    1 & ((res - 1) >> 8)
}
