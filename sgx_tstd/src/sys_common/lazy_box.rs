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

#![allow(dead_code)] // Only used on some platforms.

// This is used to wrap pthread {Mutex, Condvar, RwLock} in.

use crate::marker::PhantomData;
use crate::ops::{Deref, DerefMut};
use crate::ptr::null_mut;
use crate::sync::atomic::{
    AtomicPtr,
    Ordering::{AcqRel, Acquire},
};

pub struct LazyBox<T: LazyInit> {
    ptr: AtomicPtr<T>,
    _phantom: PhantomData<T>,
}

pub trait LazyInit {
    /// This is called before the box is allocated, to provide the value to
    /// move into the new box.
    ///
    /// It might be called more than once per LazyBox, as multiple threads
    /// might race to initialize it concurrently, each constructing and initializing
    /// their own box. All but one of them will be passed to `cancel_init` right after.
    fn init() -> Box<Self>;

    /// Any surplus boxes from `init()` that lost the initialization race
    /// are passed to this function for disposal.
    ///
    /// The default implementation calls destroy().
    fn cancel_init(x: Box<Self>) {
        Self::destroy(x);
    }

    /// This is called to destroy a used box.
    ///
    /// The default implementation just drops it.
    fn destroy(_: Box<Self>) {}
}

impl<T: LazyInit> LazyBox<T> {
    #[inline]
    pub const fn new() -> Self {
        Self { ptr: AtomicPtr::new(null_mut()), _phantom: PhantomData }
    }

    #[inline]
    fn get_pointer(&self) -> *mut T {
        let ptr = self.ptr.load(Acquire);
        if ptr.is_null() { self.initialize() } else { ptr }
    }

    #[cold]
    fn initialize(&self) -> *mut T {
        let new_ptr = Box::into_raw(T::init());
        match self.ptr.compare_exchange(null_mut(), new_ptr, AcqRel, Acquire) {
            Ok(_) => new_ptr,
            Err(ptr) => {
                // Lost the race to another thread.
                // Drop the box we created, and use the one from the other thread instead.
                T::cancel_init(unsafe { Box::from_raw(new_ptr) });
                ptr
            }
        }
    }
}

impl<T: LazyInit> Deref for LazyBox<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.get_pointer() }
    }
}

impl<T: LazyInit> DerefMut for LazyBox<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.get_pointer() }
    }
}

impl<T: LazyInit> Drop for LazyBox<T> {
    fn drop(&mut self) {
        let ptr = *self.ptr.get_mut();
        if !ptr.is_null() {
            T::destroy(unsafe { Box::from_raw(ptr) });
        }
    }
}
