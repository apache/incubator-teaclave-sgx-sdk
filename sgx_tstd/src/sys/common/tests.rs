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

use crate::ffi::CString;
use crate::hint::black_box;
use crate::path::Path;
use crate::sys::common::small_c_string::run_path_with_cstr;
use core::iter::repeat;

use sgx_test_utils::test_case;
use sgx_test_utils::{bench_case, Bencher};

#[test_case]
fn stack_allocation_works() {
    let path = Path::new("abc");
    let result = run_path_with_cstr(path, |p| {
        assert_eq!(p, &*CString::new(path.as_os_str().as_encoded_bytes()).unwrap());
        Ok(42)
    });
    assert_eq!(result.unwrap(), 42);
}

#[test_case]
fn stack_allocation_fails() {
    let path = Path::new("ab\0");
    assert!(run_path_with_cstr::<(), _>(path, |_| unreachable!()).is_err());
}

#[test_case]
fn heap_allocation_works() {
    let path = repeat("a").take(128).collect::<String>();
    let path = Path::new(&path);
    let result = run_path_with_cstr(path, |p| {
        assert_eq!(p, &*CString::new(path.as_os_str().as_encoded_bytes()).unwrap());
        Ok(42)
    });
    assert_eq!(result.unwrap(), 42);
}

#[test_case]
fn heap_allocation_fails() {
    let mut path = repeat("a").take(128).collect::<String>();
    path.push('\0');
    let path = Path::new(&path);
    assert!(run_path_with_cstr::<(), _>(path, |_| unreachable!()).is_err());
}

#[bench_case]
fn bench_stack_path_alloc(b: &mut Bencher) {
    let path = repeat("a").take(127).collect::<String>();
    let p = Path::new(&path);
    b.iter(|| {
        run_path_with_cstr(p, |cstr| {
            black_box(cstr);
            Ok(())
        })
        .unwrap();
    });
}

#[bench_case]
fn bench_heap_path_alloc(b: &mut Bencher) {
    let path = repeat("a").take(128).collect::<String>();
    let p = Path::new(&path);
    b.iter(|| {
        run_path_with_cstr(p, |cstr| {
            black_box(cstr);
            Ok(())
        })
        .unwrap();
    });
}
