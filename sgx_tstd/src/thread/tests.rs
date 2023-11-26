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

#![allow(clippy::redundant_clone)]
#![allow(clippy::redundant_closure)]

use super::Builder;
use crate::mem;
use crate::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Sender},
    Arc, Barrier,
};
use crate::thread::{self, Scope, ThreadId};
use crate::time::Duration;
use crate::time::Instant;

use sgx_test_utils::test_case;

// !!! These tests are dangerous. If something is buggy, they will hang, !!!
// !!! instead of exiting cleanly. This might wedge the buildbots.       !!!

#[allow(clippy::ok_expect)]
#[test_case]
fn test_unnamed_thread() {
    thread::spawn(move || {
        assert!(thread::current().name().is_none());
    })
    .join()
    .ok()
    .expect("thread panicked");
}

#[test_case]
fn test_named_thread() {
    Builder::new()
        .name("ada lovelace".to_string())
        .spawn(move || {
            assert!(thread::current().name().unwrap() == "ada lovelace");
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test_case]
fn test_invalid_named_thread() {
    should_panic!(Builder::new().name("ada l\0velace".to_string()).spawn(|| {}));
}

#[test_case]
fn test_run_basic() {
    let (tx, rx) = channel();
    thread::spawn(move || {
        tx.send(()).unwrap();
    });
    rx.recv().unwrap();
}

#[test_case]
fn test_is_finished() {
    let b = Arc::new(Barrier::new(2));
    let t = thread::spawn({
        let b = b.clone();
        move || {
            b.wait();
            1234
        }
    });

    // Thread is definitely running here, since it's still waiting for the barrier.
    assert!(!t.is_finished());

    // Unblock the barrier.
    b.wait();

    // Now check that t.is_finished() becomes true within a reasonable time.
    let start = Instant::now();
    while !t.is_finished() {
        assert!(start.elapsed() < Duration::from_secs(2));
        thread::sleep(Duration::from_millis(15));
    }

    // Joining the thread should not block for a significant time now.
    let join_time = Instant::now();
    assert_eq!(t.join().unwrap(), 1234);
    assert!(join_time.elapsed() < Duration::from_secs(2));
}

#[test_case]
fn test_spawn_sched() {
    let (tx, rx) = channel();

    fn f(i: i32, tx: Sender<()>) {
        let tx = tx.clone();
        thread::spawn(move || {
            if i == 0 {
                tx.send(()).unwrap();
            } else {
                f(i - 1, tx);
            }
        });
    }
    f(10, tx);
    rx.recv().unwrap();
}

#[test_case]
fn test_spawn_sched_childs_on_default_sched() {
    let (tx, rx) = channel();

    thread::spawn(move || {
        thread::spawn(move || {
            tx.send(()).unwrap();
        });
    });

    rx.recv().unwrap();
}

fn avoid_copying_the_body<F>(spawnfn: F)
where
    F: FnOnce(Box<dyn Fn() + Send>),
{
    let (tx, rx) = channel();

    let x: Box<_> = Box::new(1);
    let x_in_parent = (&*x) as *const i32 as usize;

    spawnfn(Box::new(move || {
        let x_in_child = (&*x) as *const i32 as usize;
        tx.send(x_in_child).unwrap();
    }));

    let x_in_child = rx.recv().unwrap();
    assert_eq!(x_in_parent, x_in_child);
}

#[test_case]
fn test_avoid_copying_the_body_spawn() {
    avoid_copying_the_body(|v| {
        thread::spawn(move || v());
    });
}

#[test_case]
fn test_avoid_copying_the_body_thread_spawn() {
    avoid_copying_the_body(|f| {
        thread::spawn(move || {
            f();
        });
    })
}

#[test_case]
fn test_avoid_copying_the_body_join() {
    avoid_copying_the_body(|f| {
        let _ = thread::spawn(move || f()).join();
    })
}

#[test_case]
#[allow(clippy::needless_return)]
fn test_child_doesnt_ref_parent() {
    // If the child refcounts the parent thread, this will stack overflow when
    // climbing the thread tree to dereference each ancestor. (See #1789)
    // (well, it would if the constant were 8000+ - I lowered it to be more
    // valgrind-friendly. try this at home, instead..!)
    const GENERATIONS: u32 = 16;
    fn child_no(x: u32) -> Box<dyn Fn() + Send> {
        return Box::new(move || {
            if x < GENERATIONS {
                thread::spawn(move || child_no(x + 1)());
            }
        });
    }
    thread::spawn(|| child_no(0)());
}

#[test_case]
fn test_simple_newsched_spawn() {
    thread::spawn(move || {});
}

#[test_case]
fn test_park_unpark_before() {
    for _ in 0..10 {
        thread::current().unpark();
        thread::park();
    }
}

#[test_case]
fn test_park_unpark_called_other_thread() {
    for _ in 0..10 {
        let th = thread::current();

        let _guard = thread::spawn(move || {
            super::sleep(Duration::from_millis(50));
            th.unpark();
        });

        thread::park();
    }
}

#[test_case]
fn test_park_timeout_unpark_before() {
    for _ in 0..10 {
        thread::current().unpark();
        thread::park_timeout(Duration::from_millis(u32::MAX as u64));
    }
}

#[test_case]
fn test_park_timeout_unpark_not_called() {
    for _ in 0..10 {
        thread::park_timeout(Duration::from_millis(10));
    }
}

#[test_case]
fn test_park_timeout_unpark_called_other_thread() {
    for _ in 0..10 {
        let th = thread::current();

        let _guard = thread::spawn(move || {
            super::sleep(Duration::from_millis(50));
            th.unpark();
        });

        thread::park_timeout(Duration::from_millis(u32::MAX as u64));
    }
}

#[test_case]
fn sleep_ms_smoke() {
    thread::sleep(Duration::from_millis(2));
}

#[test_case]
fn test_size_of_option_thread_id() {
    assert_eq!(mem::size_of::<Option<ThreadId>>(), mem::size_of::<ThreadId>());
}

#[test_case]
fn test_thread_id_equal() {
    assert!(thread::current().id() == thread::current().id());
}

#[test_case]
fn test_thread_id_not_equal() {
    let spawned_id = thread::spawn(|| thread::current().id()).join().unwrap();
    assert!(thread::current().id() != spawned_id);
}

#[test_case]
fn test_scoped_threads_drop_result_before_join() {
    let actually_finished = &AtomicBool::new(false);
    struct X<'scope, 'env>(&'scope Scope<'scope, 'env>, &'env AtomicBool);
    impl Drop for X<'_, '_> {
        fn drop(&mut self) {
            thread::sleep(Duration::from_millis(20));
            let actually_finished = self.1;
            self.0.spawn(move || {
                thread::sleep(Duration::from_millis(20));
                actually_finished.store(true, Ordering::Relaxed);
            });
        }
    }
    thread::scope(|s| {
        s.spawn(move || {
            thread::sleep(Duration::from_millis(20));
            X(s, actually_finished)
        });
    });
    assert!(actually_finished.load(Ordering::Relaxed));
}

#[test_case]
#[allow(dropping_references)]
#[allow(clippy::match_single_binding)]
fn test_scoped_threads_nll() {
    // this is mostly a *compilation test* for this exact function:
    fn foo(x: &u8) {
        thread::scope(|s| {
            s.spawn(|| match x {
                _ => (),
            });
        });
    }
    // let's also run it for good measure
    let x = 42_u8;
    foo(&x);
}
