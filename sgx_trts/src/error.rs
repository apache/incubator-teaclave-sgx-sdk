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

use crate::arch::Tds;
use crate::enclave::state::{self, State};
use crate::tcs::tc;

#[no_mangle]
pub extern "C" fn get_errno_addr() -> *mut i32 {
    let tds = unsafe { Tds::from_raw_mut(tc::get_tds()) };
    &mut tds.last_error as *mut usize as *mut i32
}

pub fn errno() -> i32 {
    unsafe { *get_errno_addr() }
}

pub fn set_errno(e: i32) {
    unsafe { *get_errno_addr() = e }
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
    state::set_state(State::Crashed);
    core::intrinsics::abort()
}
