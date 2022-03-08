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

use super::*;
use crate::collections::hash_map::DefaultHasher;
use crate::hash::{Hash, Hasher};
use sgx_types::types::c_char;

use sgx_test_utils::test_case;

#[test_case]
fn equal_hash() {
    let data = b"123\xE2\xFA\xA6\0";
    let ptr = data.as_ptr() as *const c_char;
    let cstr: &'static CStr = unsafe { CStr::from_ptr(ptr) };

    let mut s = DefaultHasher::new();
    cstr.hash(&mut s);
    let cstr_hash = s.finish();
    let mut s = DefaultHasher::new();
    CString::new(&data[..data.len() - 1]).unwrap().hash(&mut s);
    let cstring_hash = s.finish();

    assert_eq!(cstr_hash, cstring_hash);
}

#[test_case]
fn cstr_index_from_empty() {
    let original = b"Hello, world!\0";
    let cstr = CStr::from_bytes_with_nul(original).unwrap();
    should_panic!(&cstr[original.len()..]);
}
