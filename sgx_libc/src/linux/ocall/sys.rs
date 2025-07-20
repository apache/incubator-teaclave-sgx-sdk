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

#[no_mangle]
pub unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    if let Ok(ret) = ocall::sysconf(name) {
        ret
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn prctl(
    option: c_int,
    arg2: c_ulong,
    arg3: c_ulong,
    arg4: c_ulong,
    arg5: c_ulong,
) -> c_int {
    if let Ok(ret) = ocall::prctl(option, arg2, arg3, arg4, arg5) {
        ret
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn sched_getaffinity(
    pid: pid_t,
    cpusetsize: size_t,
    mask: *mut cpu_set_t,
) -> c_int {
    if cpusetsize < mem::size_of::<cpu_set_t>() || mask.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if let Ok(cpuset) = ocall::sched_getaffinity(pid) {
        *mask = cpuset;
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn sched_setaffinity(
    pid: pid_t,
    cpusetsize: size_t,
    mask: *const cpu_set_t,
) -> c_int {
    if cpusetsize < mem::size_of::<cpu_set_t>() || mask.is_null() {
        set_errno(EINVAL);
        return -1;
    }

    if ocall::sched_setaffinity(pid, &*mask).is_ok() {
        0
    } else {
        -1
    }
}
