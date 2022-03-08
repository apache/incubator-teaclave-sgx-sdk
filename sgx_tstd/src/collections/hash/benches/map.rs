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

use crate::collections::HashMap;

use sgx_test_utils::{bench_case, Bencher};

#[bench_case]
fn new_drop(b: &mut Bencher) {
    b.iter(|| {
        let m: HashMap<i32, i32> = HashMap::new();
        assert_eq!(m.len(), 0);
    })
}

#[bench_case]
fn new_insert_drop(b: &mut Bencher) {
    b.iter(|| {
        let mut m = HashMap::new();
        m.insert(0, 0);
        assert_eq!(m.len(), 1);
    })
}

// #[bench_case]
// fn grow_by_insertion(b: &mut Bencher) {
//     let mut m = HashMap::new();

//     for i in 1..1001 {
//         m.insert(i, i);
//     }

//     let mut k = 1001;

//     b.iter(|| {
//         m.insert(k, k);
//         k += 1;
//     });
// }

#[bench_case]
fn find_existing(b: &mut Bencher) {
    let mut m = HashMap::new();

    for i in 1..1001 {
        m.insert(i, i);
    }

    let mut k = 1;
    b.iter(|| {
        for i in 1..1001 {
            m.contains_key(&i);
        }
        k += 1;
    });
}

#[bench_case]
fn find_nonexisting(b: &mut Bencher) {
    let mut m = HashMap::new();

    for i in 1..1001 {
        m.insert(i, i);
    }

    b.iter(|| {
        for i in 1001..2001 {
            m.contains_key(&i);
        }
    });
}

#[bench_case]
fn hashmap_as_queue(b: &mut Bencher) {
    let mut m = HashMap::new();

    for i in 1..1001 {
        m.insert(i, i);
    }

    let mut k = 1;

    b.iter(|| {
        m.remove(&k);
        m.insert(k + 1000, k + 1000);
        k += 1;
    });
}

#[bench_case]
fn get_remove_insert(b: &mut Bencher) {
    let mut m = HashMap::new();

    for i in 1..1001 {
        m.insert(i, i);
    }

    let mut k = 1;

    b.iter(|| {
        m.get(&(k + 400));
        m.get(&(k + 2000));
        m.remove(&k);
        m.insert(k + 1000, k + 1000);
        k += 1;
    })
}
