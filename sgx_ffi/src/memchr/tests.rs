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

// test the implementations for the current platform
use super::{memchr, memrchr};

use sgx_test_utils::test_case;

#[test_case]
fn matches_one() {
    assert_eq!(Some(0), memchr(b'a', b"a"));
}

#[test_case]
fn matches_begin() {
    assert_eq!(Some(0), memchr(b'a', b"aaaa"));
}

#[test_case]
fn matches_end() {
    assert_eq!(Some(4), memchr(b'z', b"aaaaz"));
}

#[test_case]
fn matches_nul() {
    assert_eq!(Some(4), memchr(b'\x00', b"aaaa\x00"));
}

#[test_case]
fn matches_past_nul() {
    assert_eq!(Some(5), memchr(b'z', b"aaaa\x00z"));
}

#[test_case]
fn no_match_empty() {
    assert_eq!(None, memchr(b'a', b""));
}

#[test_case]
fn no_match() {
    assert_eq!(None, memchr(b'a', b"xyz"));
}

#[test_case]
fn matches_one_reversed() {
    assert_eq!(Some(0), memrchr(b'a', b"a"));
}

#[test_case]
fn matches_begin_reversed() {
    assert_eq!(Some(3), memrchr(b'a', b"aaaa"));
}

#[test_case]
fn matches_end_reversed() {
    assert_eq!(Some(0), memrchr(b'z', b"zaaaa"));
}

#[test_case]
fn matches_nul_reversed() {
    assert_eq!(Some(4), memrchr(b'\x00', b"aaaa\x00"));
}

#[test_case]
fn matches_past_nul_reversed() {
    assert_eq!(Some(0), memrchr(b'z', b"z\x00aaaa"));
}

#[test_case]
fn no_match_empty_reversed() {
    assert_eq!(None, memrchr(b'a', b""));
}

#[test_case]
fn no_match_reversed() {
    assert_eq!(None, memrchr(b'a', b"xyz"));
}

#[test_case]
fn each_alignment() {
    let mut data = [1u8; 64];
    let needle = 2;
    let pos = 40;
    data[pos] = needle;
    for start in 0..16 {
        assert_eq!(Some(pos - start), memchr(needle, &data[start..]));
    }
}
