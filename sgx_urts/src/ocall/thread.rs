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

use crate::ocall::util::*;
use libc::{self, c_int, timespec};
use std::io::Error;

#[no_mangle]
pub unsafe extern "C" fn u_sched_yield_ocall(error: *mut c_int) -> c_int {
    let mut errno = 0;
    let ret = libc::sched_yield();
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn u_nanosleep_ocall(
    error: *mut c_int,
    rqtp: *const timespec,
    rmtp: *mut timespec,
) -> c_int {
    let mut errno = 0;
    let ret = libc::nanosleep(rqtp, rmtp);
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    set_error(error, errno);
    ret
}
