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

//! consttime memcmp

use sgx_types::marker::{BytewiseEquality};
use core::mem;
use core::slice;

pub trait ConsttimeMemEq<T: BytewiseEquality + ?Sized = Self> {
    fn consttime_memeq(&self, other: &T) -> bool;

    fn consttime_memne(&self, other: &T) -> bool { !self.consttime_memeq(other) }
}

impl<T> ConsttimeMemEq<[T]> for [T]
    where T: Eq + BytewiseEquality
{
    fn consttime_memeq(&self, other: &[T]) -> bool {

        if self.len() != other.len() {
            return false;
        }
        if self.as_ptr() == other.as_ptr() {
            return true;
        }
        let size = mem::size_of_val(self);
        consttime_memequal(self.as_ptr() as * const u8,
                           other.as_ptr() as * const u8,
                           size) != 0
    }
}

impl<T> ConsttimeMemEq<T> for T
    where T: Eq + BytewiseEquality
{
    fn consttime_memeq(&self, other: &T) -> bool {

        let size = mem::size_of_val(self);
        if size == 0 {
            return true;
        }
        consttime_memequal(self as * const T as * const u8,
                           other as * const T as * const u8,
                           size) != 0
    }
}

fn consttime_memequal(b1: * const u8, b2: * const u8, l: usize) -> i32
{
    let mut res: i32 = 0;
    let mut len = l;
    let p1 = unsafe { slice::from_raw_parts(b1, l) };
    let p2 = unsafe { slice::from_raw_parts(b2, l) };

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
    (1 & ((res - 1) >> 8))
}
