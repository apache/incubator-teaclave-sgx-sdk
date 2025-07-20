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

use crate::arch::Tds;
use crate::tcs::tc;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TlsIndex {
    module: usize,
    offset: usize,
}

#[no_mangle]
pub unsafe extern "C" fn __tls_get_addr(ti: *const TlsIndex) -> *mut u8 {
    let ti = &*ti;
    let tds = Tds::from_raw(tc::get_tds());
    (tds.tls_addr + ti.offset) as *mut u8
}
