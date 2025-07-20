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

#[cfg(feature = "unit_test")]
mod tests;

/// A safe interface to `memchr`.
///
pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    let p = unsafe { c::memchr(haystack.as_ptr(), needle, haystack.len()) };
    if p.is_null() {
        None
    } else {
        Some(p as usize - (haystack.as_ptr() as usize))
    }
}

pub fn memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    fn memrchr_specific(needle: u8, haystack: &[u8]) -> Option<usize> {
        if haystack.is_empty() {
            return None;
        }
        let p = unsafe { c::memrchr(haystack.as_ptr(), needle, haystack.len()) };
        if p.is_null() {
            None
        } else {
            Some(p as usize - (haystack.as_ptr() as usize))
        }
    }

    memrchr_specific(needle, haystack)
}

mod c {
    use core::ptr;
    /// # Safety
    pub unsafe fn memchr(s: *const u8, c: u8, n: usize) -> *const u8 {
        let mut ret = ptr::null();
        let mut p = s;
        for _ in 0..n {
            if *p == c {
                ret = p;
                break;
            }
            p = p.offset(1);
        }
        ret
    }

    /// # Safety
    pub unsafe fn memrchr(s: *const u8, c: u8, n: usize) -> *const u8 {
        if n == 0 {
            return ptr::null();
        }
        let mut ret = ptr::null();
        let mut p: *const u8 = (s as usize + (n - 1)) as *const u8;
        for _ in 0..n {
            if *p == c {
                ret = p;
                break;
            }
            p = p.offset(-1);
        }
        ret
    }
}
