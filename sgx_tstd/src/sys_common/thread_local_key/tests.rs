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

use super::{Key, StaticKey};

use sgx_test_utils::test_case;

fn assert_sync<T: Sync>() {}
fn assert_send<T: Send>() {}

#[test_case]
fn smoke() {
    assert_sync::<Key>();
    assert_send::<Key>();

    let k1 = Key::new(None);
    let k2 = Key::new(None);
    assert!(k1.get().is_null());
    assert!(k2.get().is_null());
    k1.set(1 as *mut _);
    k2.set(2 as *mut _);
    assert_eq!(k1.get() as usize, 1);
    assert_eq!(k2.get() as usize, 2);
}

#[test_case]
fn statik() {
    static K1: StaticKey = StaticKey::new(None);
    static K2: StaticKey = StaticKey::new(None);

    unsafe {
        assert!(K1.get().is_null());
        assert!(K2.get().is_null());
        K1.set(1 as *mut _);
        K2.set(2 as *mut _);
        assert_eq!(K1.get() as usize, 1);
        assert_eq!(K2.get() as usize, 2);
    }
}
