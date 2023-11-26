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

use sgx_test_utils::test_case;

#[test_case]
fn slice_debug_output() {
    let input = unsafe { Slice::from_encoded_bytes_unchecked(b"\xF0hello,\tworld") };
    let expected = r#""\xF0hello,\tworld""#;
    let output = format!("{input:?}");

    assert_eq!(output, expected);
}

#[test_case]
fn display() {
    assert_eq!("Hello\u{FFFD}\u{FFFD} There\u{FFFD} Goodbye", unsafe {
        Slice::from_encoded_bytes_unchecked(b"Hello\xC0\x80 There\xE6\x83 Goodbye").to_string()
    },);
}
