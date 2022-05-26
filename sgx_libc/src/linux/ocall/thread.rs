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

use crate::linux::*;
use core::mem;
use sgx_oc::linux::ocall;
use sgx_oc::linux::ocall::OCallError;
use sgx_trts::trts::is_within_enclave;

#[no_mangle]
pub unsafe extern "C" fn sched_yield() -> c_int {
    if ocall::sched_yield().is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
    if rqtp.is_null() || !is_within_enclave(rqtp as *const u8, mem::size_of::<timespec>()) {
        set_errno(EINVAL);
        return -1;
    }
    if !rmtp.is_null() && !is_within_enclave(rmtp as *const u8, mem::size_of::<timespec>()) {
        set_errno(EINVAL);
        return -1;
    }

    let mut req = *rqtp;
    if let Err(err) = ocall::nanosleep(&mut req) {
        match err {
            OCallError::OsError(e) if e == EINTR => {
                if !rmtp.is_null() {
                    *rmtp = req;
                }
            }
            _ => (),
        }
        -1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn sleep(seconds: c_uint) -> c_uint {
    let mut req = timespec {
        tv_sec: seconds as time_t,
        tv_nsec: 0,
    };

    if ocall::nanosleep(&mut req).is_ok() {
        0
    } else {
        req.tv_sec as c_uint
    }
}

const MICROS_PER_SEC: c_uint = 1_000_000;
const NANOS_PER_MICRO: c_uint = 1_000;

#[no_mangle]
pub unsafe extern "C" fn usleep(useconds: c_uint) -> c_int {
    let mut req = timespec {
        tv_sec: (useconds / MICROS_PER_SEC) as time_t,
        tv_nsec: ((useconds % MICROS_PER_SEC) * NANOS_PER_MICRO) as c_long,
    };

    if ocall::nanosleep(&mut req).is_ok() {
        0
    } else {
        -1
    }
}
