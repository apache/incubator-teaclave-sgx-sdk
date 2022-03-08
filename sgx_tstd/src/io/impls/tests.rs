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

use crate::io::prelude::*;

use sgx_test_utils::{bench_case, black_box, Bencher};

#[bench_case]
fn bench_read_slice(b: &mut Bencher) {
    let buf = [5; 1024];
    let mut dst = [0; 128];

    b.iter(|| {
        let mut rd = &buf[..];
        for _ in 0..8 {
            let _ = rd.read(&mut dst);
            black_box(&dst);
        }
    })
}

#[bench_case]
fn bench_write_slice(b: &mut Bencher) {
    let mut buf = [0; 1024];
    let src = [5; 128];

    b.iter(|| {
        let mut wr = &mut buf[..];
        for _ in 0..8 {
            let _ = wr.write_all(&src);
            black_box(&wr);
        }
    })
}

#[bench_case]
fn bench_read_vec(b: &mut Bencher) {
    let buf = vec![5; 1024];
    let mut dst = [0; 128];

    b.iter(|| {
        let mut rd = &buf[..];
        for _ in 0..8 {
            let _ = rd.read(&mut dst);
            black_box(&dst);
        }
    })
}

#[bench_case]
fn bench_write_vec(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(1024);
    let src = [5; 128];

    b.iter(|| {
        let mut wr = &mut buf[..];
        for _ in 0..8 {
            let _ = wr.write_all(&src);
            black_box(&wr);
        }
    })
}
