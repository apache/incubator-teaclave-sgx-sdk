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

use super::Queue;
use crate::sync::mpsc::channel;
use crate::sync::Arc;
use crate::thread;

use sgx_test_utils::test_case;

#[test_case]
fn smoke() {
    unsafe {
        let queue = Queue::with_additions(0, (), ());
        queue.push(1);
        queue.push(2);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), None);
        queue.push(3);
        queue.push(4);
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), Some(4));
        assert_eq!(queue.pop(), None);
    }
}

#[test_case]
fn peek() {
    unsafe {
        let queue = Queue::with_additions(0, (), ());
        queue.push(vec![1]);

        // Ensure the borrowchecker works
        match queue.peek() {
            Some(vec) => {
                assert_eq!(&*vec, &[1]);
            }
            None => unreachable!(),
        }

        match queue.pop() {
            Some(vec) => {
                assert_eq!(&*vec, &[1]);
            }
            None => unreachable!(),
        }
    }
}

#[test_case]
fn drop_full() {
    unsafe {
        let q: Queue<Box<_>> = Queue::with_additions(0, (), ());
        q.push(box 1);
        q.push(box 2);
    }
}

#[test_case]
fn smoke_bound() {
    unsafe {
        let q = Queue::with_additions(0, (), ());
        q.push(1);
        q.push(2);
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), None);
        q.push(3);
        q.push(4);
        assert_eq!(q.pop(), Some(3));
        assert_eq!(q.pop(), Some(4));
        assert_eq!(q.pop(), None);
    }
}

#[test_case]
fn stress() {
    unsafe {
        stress_bound(0);
        stress_bound(1);
    }

    unsafe fn stress_bound(bound: usize) {
        let q = Arc::new(Queue::with_additions(bound, (), ()));

        let (tx, rx) = channel();
        let q2 = q.clone();
        let _t = thread::spawn(move || {
            for _ in 0..100000 {
                loop {
                    match q2.pop() {
                        Some(1) => break,
                        Some(_) => panic!(),
                        None => {}
                    }
                }
            }
            tx.send(()).unwrap();
        });
        for _ in 0..100000 {
            q.push(1);
        }
        rx.recv().unwrap();
    }
}
