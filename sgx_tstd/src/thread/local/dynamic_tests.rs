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

use crate::cell::RefCell;
use crate::collections::HashMap;
use crate::thread_local;

use sgx_test_utils::test_case;

#[test_case]
fn smoke() {
    fn square(i: i32) -> i32 {
        i * i
    }
    thread_local!(static FOO: i32 = square(3));

    FOO.with(|f| {
        assert_eq!(*f, 9);
    });
}

#[test_case]
fn hashmap() {
    fn map() -> RefCell<HashMap<i32, i32>> {
        let mut m = HashMap::new();
        m.insert(1, 2);
        RefCell::new(m)
    }
    thread_local!(static FOO: RefCell<HashMap<i32, i32>> = map());

    FOO.with(|map| {
        assert_eq!(map.borrow()[&1], 2);
    });
}

#[test_case]
fn refcell_vec() {
    thread_local!(static FOO: RefCell<Vec<u32>> = RefCell::new(vec![1, 2, 3]));

    FOO.with(|vec| {
        assert_eq!(vec.borrow().len(), 3);
        vec.borrow_mut().push(4);
        assert_eq!(vec.borrow()[3], 4);
    });
}
