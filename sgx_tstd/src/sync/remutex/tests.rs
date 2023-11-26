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

use super::{ReentrantMutex, ReentrantMutexGuard};
use crate::cell::RefCell;
use crate::sync::Arc;
use crate::thread;

use sgx_test_utils::test_case;

#[test_case]
#[allow(clippy::unit_cmp)]
fn smoke() {
    let m = ReentrantMutex::new(());
    {
        let a = m.lock();
        {
            let b = m.lock();
            {
                let c = m.lock();
                assert_eq!(*c, ());
            }
            assert_eq!(*b, ());
        }
        assert_eq!(*a, ());
    }
}

#[test_case]
fn is_mutex() {
    let m = Arc::new(ReentrantMutex::new(RefCell::new(0)));
    let m2 = m.clone();
    let lock = m.lock();
    let child = thread::spawn(move || {
        let lock = m2.lock();
        assert_eq!(*lock.borrow(), 4950);
    });
    for i in 0..100 {
        let lock = m.lock();
        *lock.borrow_mut() += i;
    }
    drop(lock);
    child.join().unwrap();
}

#[test_case]
fn trylock_works() {
    let m = Arc::new(ReentrantMutex::new(()));
    let m2 = m.clone();
    let _lock = m.try_lock();
    let _lock2 = m.try_lock();
    thread::spawn(move || {
        let lock = m2.try_lock();
        assert!(lock.is_none());
    })
    .join()
    .unwrap();
    let _lock3 = m.try_lock();
}

pub struct Answer<'a>(pub ReentrantMutexGuard<'a, RefCell<u32>>);
impl Drop for Answer<'_> {
    fn drop(&mut self) {
        *self.0.borrow_mut() = 42;
    }
}
