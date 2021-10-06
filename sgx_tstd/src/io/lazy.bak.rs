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

use alloc_crate::sync::Arc;
use core::cell::Cell;
use core::cell::UnsafeCell;
use core::ptr;
use crate::sync::SgxThreadMutex as Mutex;
use crate::sys_common;

pub struct Lazy<T> {
    // We never call `lock.init()`, so it is UB to attempt to acquire this mutex reentrantly!
    lock: Mutex,
    ptr: Cell<*mut Arc<T>>,
}

#[inline]
const fn done<T>() -> *mut Arc<T> {
    1_usize as *mut _
}

unsafe impl<T> Sync for Lazy<T> {}

impl<T> Lazy<T> {
    pub const fn new() -> Lazy<T> {
        Lazy { lock: Mutex::new(), ptr: Cell::new(ptr::null_mut()) }
    }
}

impl<T: Send + Sync + 'static> Lazy<T> {
    /// Safety: `init` must not call `get` on the variable that is being
    /// initialized.
    pub unsafe fn get(&'static self, init: fn() -> Arc<T>) -> Option<Arc<T>> {
        let r = self.lock.lock();
        if r.is_err() {
            return None;
        }
        let ptr = self.ptr.get();
        let ret = if ptr.is_null() {
            Some(self.init(init))
        } else if ptr == done() {
            None
        } else {
            Some((*ptr).clone())
        };
        self.lock.unlock();
        ret
    }

    // Must only be called with `lock` held
    unsafe fn init(&'static self, init: fn() -> Arc<T>) -> Arc<T> {
        // If we successfully register an at exit handler, then we cache the
        // `Arc` allocation in our own internal box (it will get deallocated by
        // the at exit handler). Otherwise we just return the freshly allocated
        // `Arc`.
        let registered = sys_common::at_exit(move || {
            self.lock.lock();
            let ptr = self.ptr.replace(done());
            self.lock.unlock();
            drop(Box::from_raw(ptr))
        });
        // This could reentrantly call `init` again, which is a problem
        // because our `lock` allows reentrancy!
        // That's why `get` is unsafe and requires the caller to ensure no reentrancy happens.
        let ret = init();
        if registered.is_ok() {
            self.ptr.set(Box::into_raw(Box::new(ret.clone())));
        }
        ret
    }
}

#[allow(dead_code)]
pub struct LazyStatic<T> {
    lock: Mutex,
    opt: UnsafeCell<Option<Arc<T>>>,
}

unsafe impl<T> Sync for LazyStatic<T> {}

#[allow(dead_code)]
impl<T> LazyStatic<T> {
    pub const fn new() -> LazyStatic<T> {
        LazyStatic { lock: Mutex::new(), opt: UnsafeCell::new(None) }
    }
}

#[allow(dead_code)]
impl<T: Send + Sync + 'static> LazyStatic<T> {
    pub unsafe fn get(&'static self, init: fn() -> Arc<T>) -> Option<Arc<T>> {
        let r = self.lock.lock();
        if r.is_err() {
            return None;
        }
        let ret = match *self.opt.get() {
            Some(ref arc) => Some(arc.clone()),
            None => Some(self.init(init)),
        };
        self.lock.unlock();
        ret
    }

    unsafe fn init(&'static self, init: fn() -> Arc<T>) -> Arc<T> {
        let ret = init();
        *self.opt.get() = Some(ret.clone());
        ret
    }
}
