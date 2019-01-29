// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use sync::SgxThreadMutex;
use sys_common;
use alloc_crate::sync::Arc;
use core::cell::{Cell, UnsafeCell};
use core::ptr;

pub struct Lazy<T> {
    // We never call `lock.init()`, so it is UB to attempt to acquire this mutex reentrantly!
    lock: SgxThreadMutex,
    ptr: Cell<*mut Arc<T>>,
}

#[inline]
const fn done<T>() -> *mut Arc<T> { 1_usize as *mut _ }

unsafe impl<T> Sync for Lazy<T> {}

impl<T> Lazy<T> {
    pub const fn new() -> Lazy<T> {
        Lazy {
            lock: SgxThreadMutex::new(),
            ptr: Cell::new(ptr::null_mut()),
        }
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
    lock: SgxThreadMutex,
    opt: UnsafeCell<Option<Arc<T>>>,
}

unsafe impl<T> Sync for LazyStatic<T> {}

#[allow(dead_code)]
impl<T> LazyStatic<T> {
    pub const fn new() -> LazyStatic<T> {
        LazyStatic {
            lock: SgxThreadMutex::new(),
            opt: UnsafeCell::new(None),
        }
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