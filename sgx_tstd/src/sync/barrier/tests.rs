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

use crate::sync::mpsc::{channel, TryRecvError};
use crate::sync::{Arc, Barrier};
use crate::thread;

use sgx_test_utils::test_case;

#[test_case]
fn test_barrier() {
    const N: usize = 10;

    let barrier = Arc::new(Barrier::new(N));
    let (tx, rx) = channel();

    for _ in 0..N - 1 {
        let c = barrier.clone();
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(c.wait().is_leader()).unwrap();
        });
    }

    // At this point, all spawned threads should be blocked,
    // so we shouldn't get anything from the port
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

    let mut leader_found = barrier.wait().is_leader();

    // Now, the barrier is cleared and we should get data.
    for _ in 0..N - 1 {
        if rx.recv().unwrap() {
            assert!(!leader_found);
            leader_found = true;
        }
    }
    assert!(leader_found);
}
