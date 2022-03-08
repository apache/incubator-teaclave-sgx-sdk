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

use super::{FileDesc, IoSlice};
use crate::os::unix::io::FromRawFd;
use core::mem::ManuallyDrop;

use sgx_test_utils::test_case;

#[test_case]
fn limit_vector_count() {
    let stdout = ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(1) });
    let bufs = (0..1500).map(|_| IoSlice::new(&[])).collect::<Vec<_>>();
    assert!(stdout.write_vectored(&bufs).is_ok());
}
