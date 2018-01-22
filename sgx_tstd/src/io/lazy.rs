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
use alloc::arc::Arc;
use core::cell::{Cell, UnsafeCell};
use core::ptr;

pub struct Lazy<T> {
    lock: SgxThreadMutex,
    ptr: Cell<*mut Arc<T>>,
    init: fn() -> Arc<T>,
}

unsafe impl<T> Sync for Lazy<T> {}

impl<T: Send + Sync + 'static> Lazy<T> {

    pub const fn new(init: fn() -> Arc<T>) -> Lazy<T> {
        Lazy {
            lock: SgxThreadMutex::new(),
            ptr: Cell::new(ptr::null_mut()),
            init: init
        }
    }

    pub fn get(&'static self) -> Option<Arc<T>> {  
        unsafe {
            let r = self.lock.lock();
            if r.is_err() {
                return None;
            }
            let ptr = self.ptr.get();
            let ret = if ptr.is_null() {
                Some(self.init())
            } else if ptr as usize == 1 {
                None
            } else {
                Some((*ptr).clone())
            };
            self.lock.unlock();
            return ret
        }
    }
    /*
    #[cfg(not(feature = "global_exit"))]
    pub unsafe fn destroy(&'static self) {

        self.lock.lock();
        let ptr = self.ptr.get();
        if ptr.is_null() || ptr as usize == 1 {
            self.lock.unlock();
            return;
        }
        self.ptr.set(1 as *mut _);
        self.lock.unlock();
        drop(Box::from_raw(ptr))
    }

    #[cfg(not(feature = "global_exit"))]
    unsafe fn init(&'static self) -> Arc<T> {

        let ret = (self.init)();
        self.ptr.set(Box::into_raw(Box::new(ret.clone())));
        ret
    }
    */

    unsafe fn init(&'static self) -> Arc<T> {
        // If we successfully register an at exit handler, then we cache the
        // `Arc` allocation in our own internal box (it will get deallocated by
        // the at exit handler). Otherwise we just return the freshly allocated
        // `Arc`.
        let registered = sys_common::at_exit(move || {
            self.lock.lock();
            let ptr = self.ptr.get();
            if ptr.is_null() || ptr as usize == 1 {
                self.lock.unlock();
                return;
            }
            self.ptr.set(1 as *mut _);
            self.lock.unlock();
            drop(Box::from_raw(ptr))
        });
        let ret = (self.init)();
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
    init: fn() -> Arc<T>,
}

unsafe impl<T> Sync for LazyStatic<T> {}

#[allow(dead_code)]
impl<T: Send + Sync + 'static> LazyStatic<T> {

    pub const fn new(init: fn() -> Arc<T>) -> LazyStatic<T> {
        LazyStatic {
            lock: SgxThreadMutex::new(),
            opt: UnsafeCell::new(None),
            init: init
        }
    }

    pub fn get(&'static self) -> Option<Arc<T>> {  
        unsafe {
            let r = self.lock.lock();
            if r.is_err() {
                return None;
            }
   
            let ret = match *self.opt.get() {
                Some(ref arc) => Some(arc.clone()),
                None => Some(self.init()),
            };
            self.lock.unlock();
            ret
        }
    }

    unsafe fn init(&'static self) -> Arc<T> {

        let ret = (self.init)();
        *self.opt.get() = Some(ret.clone());
        ret
    }
}