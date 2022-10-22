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

use crate::ptr;

use sgx_trts::thread::tls::{Key as NativeKey, Tls};

pub type Key = usize;

#[inline]
pub unsafe fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    let key = Tls::create(dtor).unwrap();
    key.as_usize()
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    let key = NativeKey::from_usize(key).unwrap();
    let r = Tls::set(key, value);
    assert!(r.is_ok());
}

#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    let key = NativeKey::from_usize(key).unwrap();
    Tls::get(key).unwrap_or(None).unwrap_or(ptr::null_mut())
}

#[inline]
pub unsafe fn destroy(key: Key) {
    let key = NativeKey::from_usize(key).unwrap();
    Tls::destroy(key)
}
