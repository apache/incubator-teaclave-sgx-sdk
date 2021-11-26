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

use crate::sync::atomic::{AtomicUsize, Ordering};
use crate::sys::mutex as mutex_imp;
use crate::sys_common::mutex::SgxMovableThreadMutex;

pub trait CondvarCheck {
    type Check;
}

/// For boxed mutexes, a `Condvar` will check it's only ever used with the same
/// mutex, based on its (stable) address.
impl CondvarCheck for Box<mutex_imp::SgxThreadMutex> {
    type Check = SameMutexCheck;
}

pub struct SameMutexCheck {
    addr: AtomicUsize,
}

#[allow(dead_code)]
impl SameMutexCheck {
    pub const fn new() -> Self {
        Self { addr: AtomicUsize::new(0) }
    }
    pub fn verify(&self, mutex: &SgxMovableThreadMutex) {
        let addr = mutex.raw() as *const mutex_imp::SgxThreadMutex as usize;
        match self.addr.compare_exchange(0, addr, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => {}               // Stored the address
            Err(n) if n == addr => {} // Lost a race to store the same address
            _ => panic!("attempted to use a condition variable with two mutexes"),
        }
    }
}

/// Unboxed mutexes may move, so `Condvar` can not require its address to stay
/// constant.
impl CondvarCheck for mutex_imp::SgxThreadMutex {
    type Check = NoCheck;
}

pub struct NoCheck;

#[allow(dead_code)]
impl NoCheck {
    pub const fn new() -> Self {
        Self
    }
    pub fn verify(&self, _: &SgxMovableThreadMutex) {}
}
