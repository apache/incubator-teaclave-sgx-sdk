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

use super::BorrowedBuf;
use crate::mem::MaybeUninit;

use sgx_test_utils::test_case;

/// Test that BorrowedBuf has the correct numbers when created with new
#[test_case]
fn new() {
    let buf: &mut [_] = &mut [0; 16];
    let mut rbuf: BorrowedBuf<'_> = buf.into();

    assert_eq!(rbuf.filled().len(), 0);
    assert_eq!(rbuf.init_len(), 16);
    assert_eq!(rbuf.capacity(), 16);
    assert_eq!(rbuf.unfilled().capacity(), 16);
}

/// Test that BorrowedBuf has the correct numbers when created with uninit
#[test_case]
fn uninit() {
    let buf: &mut [_] = &mut [MaybeUninit::uninit(); 16];
    let mut rbuf: BorrowedBuf<'_> = buf.into();

    assert_eq!(rbuf.filled().len(), 0);
    assert_eq!(rbuf.init_len(), 0);
    assert_eq!(rbuf.capacity(), 16);
    assert_eq!(rbuf.unfilled().capacity(), 16);
}

#[test_case]
fn initialize_unfilled() {
    let buf: &mut [_] = &mut [MaybeUninit::uninit(); 16];
    let mut rbuf: BorrowedBuf<'_> = buf.into();

    rbuf.unfilled().ensure_init();

    assert_eq!(rbuf.init_len(), 16);
}

#[test_case]
fn addvance_filled() {
    let buf: &mut [_] = &mut [0; 16];
    let mut rbuf: BorrowedBuf<'_> = buf.into();

    unsafe {
        rbuf.unfilled().advance(1);
    }

    assert_eq!(rbuf.filled().len(), 1);
    assert_eq!(rbuf.unfilled().capacity(), 15);
}

#[test_case]
fn clear() {
    let buf: &mut [_] = &mut [255; 16];
    let mut rbuf: BorrowedBuf<'_> = buf.into();

    unsafe {
        rbuf.unfilled().advance(16);
    }

    assert_eq!(rbuf.filled().len(), 16);
    assert_eq!(rbuf.unfilled().capacity(), 0);

    rbuf.clear();

    assert_eq!(rbuf.filled().len(), 0);
    assert_eq!(rbuf.unfilled().capacity(), 16);

    assert_eq!(rbuf.unfilled().init_ref(), [255; 16]);
}

#[test_case]
fn set_init() {
    let buf: &mut [_] = &mut [MaybeUninit::uninit(); 16];
    let mut rbuf: BorrowedBuf<'_> = buf.into();

    unsafe {
        rbuf.set_init(8);
    }

    assert_eq!(rbuf.init_len(), 8);

    unsafe {
        rbuf.unfilled().advance(4);
    }

    unsafe {
        rbuf.set_init(2);
    }

    assert_eq!(rbuf.init_len(), 8);

    unsafe {
        rbuf.set_init(8);
    }

    assert_eq!(rbuf.init_len(), 8);
}

#[test_case]
fn append() {
    let buf: &mut [_] = &mut [MaybeUninit::new(255); 16];
    let mut rbuf: BorrowedBuf<'_> = buf.into();

    rbuf.unfilled().append(&[0; 8]);

    assert_eq!(rbuf.init_len(), 8);
    assert_eq!(rbuf.filled().len(), 8);
    assert_eq!(rbuf.filled(), [0; 8]);

    rbuf.clear();

    rbuf.unfilled().append(&[1; 16]);

    assert_eq!(rbuf.init_len(), 16);
    assert_eq!(rbuf.filled().len(), 16);
    assert_eq!(rbuf.filled(), [1; 16]);
}

#[test_case]
fn reborrow_written() {
    let buf: &mut [_] = &mut [MaybeUninit::new(0); 32];
    let mut buf: BorrowedBuf<'_> = buf.into();

    let mut cursor = buf.unfilled();
    cursor.append(&[1; 16]);

    let mut cursor2 = cursor.reborrow();
    cursor2.append(&[2; 16]);

    assert_eq!(cursor2.written(), 32);
    assert_eq!(cursor.written(), 32);

    assert_eq!(buf.unfilled().written(), 0);
    assert_eq!(buf.init_len(), 32);
    assert_eq!(buf.filled().len(), 32);
    let filled = buf.filled();
    assert_eq!(&filled[..16], [1; 16]);
    assert_eq!(&filled[16..], [2; 16]);
}

#[test_case]
fn cursor_set_init() {
    let buf: &mut [_] = &mut [MaybeUninit::uninit(); 16];
    let mut rbuf: BorrowedBuf<'_> = buf.into();

    unsafe {
        rbuf.unfilled().set_init(8);
    }

    assert_eq!(rbuf.init_len(), 8);
    assert_eq!(rbuf.unfilled().init_ref().len(), 8);
    assert_eq!(rbuf.unfilled().init_mut().len(), 8);
    assert_eq!(rbuf.unfilled().uninit_mut().len(), 8);
    assert_eq!(unsafe { rbuf.unfilled().as_mut() }.len(), 16);

    unsafe {
        rbuf.unfilled().advance(4);
    }

    unsafe {
        rbuf.unfilled().set_init(2);
    }

    assert_eq!(rbuf.init_len(), 8);

    unsafe {
        rbuf.unfilled().set_init(8);
    }

    assert_eq!(rbuf.init_len(), 12);
    assert_eq!(rbuf.unfilled().init_ref().len(), 8);
    assert_eq!(rbuf.unfilled().init_mut().len(), 8);
    assert_eq!(rbuf.unfilled().uninit_mut().len(), 4);
    assert_eq!(unsafe { rbuf.unfilled().as_mut() }.len(), 12);
}
