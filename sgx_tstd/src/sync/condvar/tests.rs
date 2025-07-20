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

#![allow(clippy::mutex_atomic)]

use crate::sync::atomic::{AtomicBool, Ordering};
use crate::sync::mpsc::channel;
use crate::sync::{Arc, Condvar, Mutex};
use crate::thread;
use crate::time::Duration;

use sgx_test_utils::test_case;

#[test_case]
fn smoke() {
    let c = Condvar::new();
    c.notify_one();
    c.notify_all();
}

#[test_case]
fn notify_one() {
    let m = Arc::new(Mutex::new(()));
    let m2 = m.clone();
    let c = Arc::new(Condvar::new());
    let c2 = c.clone();

    let g = m.lock().unwrap();
    let _t = thread::spawn(move || {
        let _g = m2.lock().unwrap();
        c2.notify_one();
    });
    let g = c.wait(g).unwrap();
    drop(g);
}

#[test_case]
fn notify_all() {
    const N: usize = 10;

    let data = Arc::new((Mutex::new(0), Condvar::new()));
    let (tx, rx) = channel();
    for _ in 0..N {
        let data = data.clone();
        let tx = tx.clone();
        thread::spawn(move || {
            let (lock, cond) = &*data;
            let mut cnt = lock.lock().unwrap();
            *cnt += 1;
            if *cnt == N {
                tx.send(()).unwrap();
            }
            while *cnt != 0 {
                cnt = cond.wait(cnt).unwrap();
            }
            tx.send(()).unwrap();
        });
    }
    drop(tx);

    let (lock, cond) = &*data;
    rx.recv().unwrap();
    let mut cnt = lock.lock().unwrap();
    *cnt = 0;
    cond.notify_all();
    drop(cnt);

    for _ in 0..N {
        rx.recv().unwrap();
    }
}

#[test_case]
fn wait_while() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();

    // Inside of our lock, spawn a new thread, and then wait for it to start.
    thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        let mut started = lock.lock().unwrap();
        *started = true;
        // We notify the condvar that the value has changed.
        cvar.notify_one();
    });

    // Wait for the thread to start up.
    let (lock, cvar) = &*pair;
    let guard = cvar.wait_while(lock.lock().unwrap(), |started| !*started);
    assert!(*guard.unwrap());
}

#[test_case]
fn wait_timeout_wait() {
    let m = Arc::new(Mutex::new(()));
    let c = Arc::new(Condvar::new());

    loop {
        let g = m.lock().unwrap();
        let (_g, no_timeout) = c.wait_timeout(g, Duration::from_millis(1)).unwrap();
        // spurious wakeups mean this isn't necessarily true
        // so execute test again, if not timeout
        if !no_timeout.timed_out() {
            continue;
        }

        break;
    }
}

#[test_case]
fn wait_timeout_while_wait() {
    let m = Arc::new(Mutex::new(()));
    let c = Arc::new(Condvar::new());

    let g = m.lock().unwrap();
    let (_g, wait) = c.wait_timeout_while(g, Duration::from_millis(1), |_| true).unwrap();
    // no spurious wakeups. ensure it timed-out
    assert!(wait.timed_out());
}

#[test_case]
fn wait_timeout_while_instant_satisfy() {
    let m = Arc::new(Mutex::new(()));
    let c = Arc::new(Condvar::new());

    let g = m.lock().unwrap();
    let (_g, wait) = c.wait_timeout_while(g, Duration::from_millis(0), |_| false).unwrap();
    // ensure it didn't time-out even if we were not given any time.
    assert!(!wait.timed_out());
}

#[test_case]
fn wait_timeout_while_wake() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair_copy = pair.clone();

    let (m, c) = &*pair;
    let g = m.lock().unwrap();
    let _t = thread::spawn(move || {
        let (lock, cvar) = &*pair_copy;
        let mut started = lock.lock().unwrap();
        thread::sleep(Duration::from_millis(1));
        *started = true;
        cvar.notify_one();
    });
    let (g2, wait) = c
        .wait_timeout_while(g, Duration::from_millis(u64::MAX), |&mut notified| !notified)
        .unwrap();
    // ensure it didn't time-out even if we were not given any time.
    assert!(!wait.timed_out());
    assert!(*g2);
}

#[test_case]
fn wait_timeout_wake() {
    let m = Arc::new(Mutex::new(()));
    let c = Arc::new(Condvar::new());

    loop {
        let g = m.lock().unwrap();

        let c2 = c.clone();
        let m2 = m.clone();

        let notified = Arc::new(AtomicBool::new(false));
        let notified_copy = notified.clone();

        let t = thread::spawn(move || {
            let _g = m2.lock().unwrap();
            thread::sleep(Duration::from_millis(1));
            notified_copy.store(true, Ordering::SeqCst);
            c2.notify_one();
        });
        let (g, timeout_res) = c.wait_timeout(g, Duration::from_millis(u64::MAX)).unwrap();
        assert!(!timeout_res.timed_out());
        // spurious wakeups mean this isn't necessarily true
        // so execute test again, if not notified
        if !notified.load(Ordering::SeqCst) {
            t.join().unwrap();
            continue;
        }
        drop(g);

        t.join().unwrap();

        break;
    }
}
