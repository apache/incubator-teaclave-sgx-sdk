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

use alloc_crate::slice;
use core::mem;

pub fn hashmap_random_keys() -> (u64, u64) {
    let mut v = (0, 0);
    unsafe {
        let view = slice::from_raw_parts_mut(&mut v as *mut _ as *mut u8, mem::size_of_val(&v));
        imp::fill_bytes(view);
    }
    v
}

mod imp {
    use sgx_types::SgxError;
    use sgx_trts::trts;

    fn getrandom(buf: &mut [u8]) -> SgxError {
        trts::rsgx_read_rand(buf)
    }

    fn getrandom_fill_bytes(v: &mut [u8]) {
        getrandom(v).expect("unexpected getrandom error");
    }

    pub fn fill_bytes(v: &mut [u8]) {
        getrandom_fill_bytes(v)
    }
}
