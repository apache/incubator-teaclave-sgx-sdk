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

use super::ReadBuf;
use crate::mem::MaybeUninit;

use sgx_test_utils::test_case;

/// Test that ReadBuf has the correct numbers when created with new
#[test_case]
fn new() {
    let mut buf = [0; 16];
    let rbuf = ReadBuf::new(&mut buf);

    assert_eq!(rbuf.filled_len(), 0);
    assert_eq!(rbuf.initialized_len(), 16);
    assert_eq!(rbuf.capacity(), 16);
    assert_eq!(rbuf.remaining(), 16);
}

/// Test that ReadBuf has the correct numbers when created with uninit
#[test_case]
fn uninit() {
    let mut buf = [MaybeUninit::uninit(); 16];
    let rbuf = ReadBuf::uninit(&mut buf);

    assert_eq!(rbuf.filled_len(), 0);
    assert_eq!(rbuf.initialized_len(), 0);
    assert_eq!(rbuf.capacity(), 16);
    assert_eq!(rbuf.remaining(), 16);
}

#[test_case]
fn initialize_unfilled() {
    let mut buf = [MaybeUninit::uninit(); 16];
    let mut rbuf = ReadBuf::uninit(&mut buf);

    rbuf.initialize_unfilled();

    assert_eq!(rbuf.initialized_len(), 16);
}

#[test_case]
fn initialize_unfilled_to() {
    let mut buf = [MaybeUninit::uninit(); 16];
    let mut rbuf = ReadBuf::uninit(&mut buf);

    rbuf.initialize_unfilled_to(8);

    assert_eq!(rbuf.initialized_len(), 8);

    rbuf.initialize_unfilled_to(4);

    assert_eq!(rbuf.initialized_len(), 8);

    rbuf.set_filled(8);

    rbuf.initialize_unfilled_to(6);

    assert_eq!(rbuf.initialized_len(), 14);

    rbuf.initialize_unfilled_to(8);

    assert_eq!(rbuf.initialized_len(), 16);
}

#[test_case]
fn add_filled() {
    let mut buf = [0; 16];
    let mut rbuf = ReadBuf::new(&mut buf);

    rbuf.add_filled(1);

    assert_eq!(rbuf.filled_len(), 1);
    assert_eq!(rbuf.remaining(), 15);
}

#[test_case]
fn add_filled_panic() {
    let mut buf = [MaybeUninit::uninit(); 16];
    let mut rbuf = ReadBuf::uninit(&mut buf);

    should_panic!(rbuf.add_filled(1));
}

#[test_case]
fn set_filled() {
    let mut buf = [0; 16];
    let mut rbuf = ReadBuf::new(&mut buf);

    rbuf.set_filled(16);

    assert_eq!(rbuf.filled_len(), 16);
    assert_eq!(rbuf.remaining(), 0);

    rbuf.set_filled(6);

    assert_eq!(rbuf.filled_len(), 6);
    assert_eq!(rbuf.remaining(), 10);
}

#[test_case]
fn set_filled_panic() {
    let mut buf = [MaybeUninit::uninit(); 16];
    let mut rbuf = ReadBuf::uninit(&mut buf);

    should_panic!(rbuf.set_filled(16));
}

#[test_case]
fn clear() {
    let mut buf = [255; 16];
    let mut rbuf = ReadBuf::new(&mut buf);

    rbuf.set_filled(16);

    assert_eq!(rbuf.filled_len(), 16);
    assert_eq!(rbuf.remaining(), 0);

    rbuf.clear();

    assert_eq!(rbuf.filled_len(), 0);
    assert_eq!(rbuf.remaining(), 16);

    assert_eq!(rbuf.initialized(), [255; 16]);
}

#[test_case]
fn assume_init() {
    let mut buf = [MaybeUninit::uninit(); 16];
    let mut rbuf = ReadBuf::uninit(&mut buf);

    unsafe {
        rbuf.assume_init(8);
    }

    assert_eq!(rbuf.initialized_len(), 8);

    rbuf.add_filled(4);

    unsafe {
        rbuf.assume_init(2);
    }

    assert_eq!(rbuf.initialized_len(), 8);

    unsafe {
        rbuf.assume_init(8);
    }

    assert_eq!(rbuf.initialized_len(), 12);
}

#[test_case]
fn append() {
    let mut buf = [MaybeUninit::new(255); 16];
    let mut rbuf = ReadBuf::uninit(&mut buf);

    rbuf.append(&[0; 8]);

    assert_eq!(rbuf.initialized_len(), 8);
    assert_eq!(rbuf.filled_len(), 8);
    assert_eq!(rbuf.filled(), [0; 8]);

    rbuf.clear();

    rbuf.append(&[1; 16]);

    assert_eq!(rbuf.initialized_len(), 16);
    assert_eq!(rbuf.filled_len(), 16);
    assert_eq!(rbuf.filled(), [1; 16]);
}

#[test_case]
fn filled_mut() {
    let mut buf = [0; 16];
    let mut rbuf = ReadBuf::new(&mut buf);

    rbuf.add_filled(8);

    let filled = rbuf.filled().to_vec();

    assert_eq!(&*filled, &*rbuf.filled_mut());
}
