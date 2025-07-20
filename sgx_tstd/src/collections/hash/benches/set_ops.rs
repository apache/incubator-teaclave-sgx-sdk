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

use crate::collections::HashSet;

use sgx_test_utils::{bench_case, Bencher};

#[bench_case]
fn set_difference(b: &mut Bencher) {
    let small: HashSet<_> = (0..10).collect();
    let large: HashSet<_> = (0..100).collect();

    b.iter(|| small.difference(&large).count());
}

#[bench_case]
fn set_is_subset(b: &mut Bencher) {
    let small: HashSet<_> = (0..10).collect();
    let large: HashSet<_> = (0..100).collect();

    b.iter(|| small.is_subset(&large));
}

#[bench_case]
fn set_intersection(b: &mut Bencher) {
    let small: HashSet<_> = (0..10).collect();
    let large: HashSet<_> = (0..100).collect();

    b.iter(|| small.intersection(&large).count());
}

#[bench_case]
fn set_symmetric_difference(b: &mut Bencher) {
    let small: HashSet<_> = (0..10).collect();
    let large: HashSet<_> = (0..100).collect();

    b.iter(|| small.symmetric_difference(&large).count());
}

#[bench_case]
fn set_union(b: &mut Bencher) {
    let small: HashSet<_> = (0..10).collect();
    let large: HashSet<_> = (0..100).collect();

    b.iter(|| small.union(&large).count());
}
