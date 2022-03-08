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

use super::Once;
use crate::panic;
use crate::sync::mpsc::channel;
use crate::thread;

use sgx_test_utils::test_case;

#[test_case]
fn smoke_once() {
    static O: Once = Once::new();
    let mut a = 0;
    O.call_once(|| a += 1);
    assert_eq!(a, 1);
    O.call_once(|| a += 1);
    assert_eq!(a, 1);
}

#[test_case]
fn stampede_once() {
    static O: Once = Once::new();
    static mut RUN: bool = false;

    let (tx, rx) = channel();
    for _ in 0..10 {
        let tx = tx.clone();
        thread::spawn(move || {
            for _ in 0..4 {
                thread::yield_now()
            }
            unsafe {
                O.call_once(|| {
                    assert!(!RUN);
                    RUN = true;
                });
                assert!(RUN);
            }
            tx.send(()).unwrap();
        });
    }

    unsafe {
        O.call_once(|| {
            assert!(!RUN);
            RUN = true;
        });
        assert!(RUN);
    }

    for _ in 0..10 {
        rx.recv().unwrap();
    }
}

#[test_case]
fn poison_bad() {
    static O: Once = Once::new();

    // poison the once
    let t = panic::catch_unwind(|| {
        O.call_once(|| panic!());
    });
    assert!(t.is_err());

    // poisoning propagates
    let t = panic::catch_unwind(|| {
        O.call_once(|| {});
    });
    assert!(t.is_err());

    // we can subvert poisoning, however
    let mut called = false;
    O.call_once_force(|p| {
        called = true;
        assert!(p.is_poisoned())
    });
    assert!(called);

    // once any success happens, we stop propagating the poison
    O.call_once(|| {});
}

#[test_case]
fn wait_for_force_to_finish() {
    static O: Once = Once::new();

    // poison the once
    let t = panic::catch_unwind(|| {
        O.call_once(|| panic!());
    });
    assert!(t.is_err());

    // make sure someone's waiting inside the once via a force
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();
    let t1 = thread::spawn(move || {
        O.call_once_force(|p| {
            assert!(p.is_poisoned());
            tx1.send(()).unwrap();
            rx2.recv().unwrap();
        });
    });

    rx1.recv().unwrap();

    // put another waiter on the once
    let t2 = thread::spawn(|| {
        let mut called = false;
        O.call_once(|| {
            called = true;
        });
        assert!(!called);
    });

    tx2.send(()).unwrap();

    assert!(t1.join().is_ok());
    assert!(t2.join().is_ok());
}
