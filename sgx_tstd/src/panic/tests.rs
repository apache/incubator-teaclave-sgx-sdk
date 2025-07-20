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

#![allow(dead_code)]

use crate::cell::RefCell;
use crate::panic::{AssertUnwindSafe, UnwindSafe};
use crate::rc::Rc;
use crate::sync::{Arc, Mutex, RwLock};

use sgx_test_utils::test_case;

struct Foo {
    a: i32,
}

fn assert<T: UnwindSafe + ?Sized>() {}

#[test_case]
fn panic_safety_traits() {
    assert::<i32>();
    assert::<&i32>();
    assert::<*mut i32>();
    assert::<*const i32>();
    assert::<usize>();
    assert::<str>();
    assert::<&str>();
    assert::<Foo>();
    assert::<&Foo>();
    assert::<Vec<i32>>();
    assert::<String>();
    assert::<RefCell<i32>>();
    assert::<Box<i32>>();
    assert::<Mutex<i32>>();
    assert::<RwLock<i32>>();
    assert::<&Mutex<i32>>();
    assert::<&RwLock<i32>>();
    assert::<Rc<i32>>();
    assert::<Arc<i32>>();
    assert::<Box<[u8]>>();

    {
        trait Trait: UnwindSafe {}
        assert::<Box<dyn Trait>>();
    }

    fn bar<T>() {
        assert::<Mutex<T>>();
        assert::<RwLock<T>>();
    }

    fn baz<T: UnwindSafe>() {
        assert::<Box<T>>();
        assert::<Vec<T>>();
        assert::<RefCell<T>>();
        assert::<AssertUnwindSafe<T>>();
        assert::<&AssertUnwindSafe<T>>();
        assert::<Rc<AssertUnwindSafe<T>>>();
        assert::<Arc<AssertUnwindSafe<T>>>();
    }
}
