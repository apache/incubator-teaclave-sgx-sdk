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

use crate::linux::*;
use core::mem;
use sgx_oc::linux::ocall;
use sgx_trts::trts::is_within_enclave;

#[no_mangle]
pub unsafe extern "C" fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
    if tp.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::clock_gettime(clk_id, &mut *tp).is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn clock() -> clock_t {
    let mut ts: timespec = mem::zeroed();
    if clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &mut ts as *mut timespec) != 0 {
        return -1;
    }
    if ts.tv_sec > i64::MAX / 1000000 || ts.tv_nsec / 1000 > i64::MAX - 1000000 * ts.tv_sec {
        return -1;
    }
    ts.tv_sec * 1000000 + ts.tv_nsec / 1000
}

#[no_mangle]
pub unsafe extern "C" fn time(tloc: *mut time_t) -> time_t {
    let mut ts: timespec = mem::zeroed();
    clock_gettime(CLOCK_REALTIME, &mut ts as *mut timespec);
    if !tloc.is_null() && is_within_enclave(tloc as *const u8, mem::size_of::<time_t>()) {
        *tloc = ts.tv_sec;
    }
    ts.tv_sec
}
