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

use super::{Data, Empty, Inconsistent, Queue};
use crate::sync::mpsc::channel;
use crate::sync::Arc;
use crate::thread;

use sgx_test_utils::test_case;

#[test_case]
fn test_full() {
    let q: Queue<Box<_>> = Queue::new();
    q.push(box 1);
    q.push(box 2);
}

#[test_case]
fn test() {
    let nthreads = 8;
    let nmsgs = 1000;
    let q = Queue::new();
    match q.pop() {
        Empty => {}
        Inconsistent | Data(..) => panic!(),
    }
    let (tx, rx) = channel();
    let q = Arc::new(q);

    for _ in 0..nthreads {
        let tx = tx.clone();
        let q = q.clone();
        thread::spawn(move || {
            for i in 0..nmsgs {
                q.push(i);
            }
            tx.send(()).unwrap();
        });
    }

    let mut i = 0;
    while i < nthreads * nmsgs {
        match q.pop() {
            Empty | Inconsistent => {}
            Data(_) => i += 1,
        }
    }
    drop(tx);
    for _ in 0..nthreads {
        rx.recv().unwrap();
    }
}
