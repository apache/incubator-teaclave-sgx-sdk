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

//! Thread-local destructor
//!
//! Besides thread-local "keys" (pointer-sized non-addressable thread-local store
//! with an associated destructor), many platforms also provide thread-local
//! destructors that are not associated with any particular data. These are
//! often more efficient.
//!
//! This module provides a fallback implementation for that interface, based
//! on the less efficient thread-local "keys". Each platform provides
//! a `thread_local_dtor` module which will either re-export the fallback,
//! or implement something more efficient.

#![allow(dead_code)]

use crate::cell::RefCell;
use crate::ptr;
use crate::sys_common::thread_local_key::StaticKey;

pub unsafe fn register_dtor_fallback(t: *mut u8, dtor: unsafe extern "C" fn(*mut u8)) {
    // The fallback implementation uses a vanilla OS-based TLS key to track
    // the list of destructors that need to be run for this thread. The key
    // then has its own destructor which runs all the other destructors.
    //
    // The destructor for DTORS is a little special in that it has a `while`
    // loop to continuously drain the list of registered destructors. It
    // *should* be the case that this loop always terminates because we
    // provide the guarantee that a TLS key cannot be set after it is
    // flagged for destruction.

    static DTORS: StaticKey = StaticKey::new(Some(run_dtors));
    // FIXME(joboet): integrate RefCell into pointer to avoid infinite recursion
    // when the global allocator tries to register a destructor and just panic
    // instead.
    type List = RefCell<Vec<(*mut u8, unsafe extern "C" fn(*mut u8))>>;
    if DTORS.get().is_null() {
        let v: Box<List> = Box::new(RefCell::new(Vec::new()));
        DTORS.set(Box::into_raw(v) as *mut u8);
    }
    let list = &*(DTORS.get() as *const List);
    match list.try_borrow_mut() {
        Ok(mut dtors) => dtors.push((t, dtor)),
        Err(_) => rtabort!("global allocator may not use TLS"),
    }

    unsafe extern "C" fn run_dtors(mut ptr: *mut u8) {
        while !ptr.is_null() {
            let list = Box::from_raw(ptr as *mut List).into_inner();
            for (ptr, dtor) in list.into_iter() {
                dtor(ptr);
            }
            ptr = DTORS.get();
            DTORS.set(ptr::null_mut());
        }
    }
}
