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

use libc::{self, c_int, c_long, c_ulong, cpu_set_t, pid_t, size_t};
use std::io::Error;

#[no_mangle]
pub extern "C" fn u_sysconf_ocall(error: *mut c_int, name: c_int) -> c_long {
    let mut errno = 0;
    let ret = unsafe { libc::sysconf(name) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_prctl_ocall(
    error: *mut c_int,
    option: c_int,
    arg2: c_ulong,
    arg3: c_ulong,
    arg4: c_ulong,
    arg5: c_ulong,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::prctl(option, arg2, arg3, arg4, arg5) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_sched_setaffinity_ocall(
    error: *mut c_int,
    pid: pid_t,
    cpusetsize: size_t,
    mask: *const cpu_set_t,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::sched_setaffinity(pid, cpusetsize, mask) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_sched_getaffinity_ocall(
    error: *mut c_int,
    pid: pid_t,
    cpusetsize: size_t,
    mask: *mut cpu_set_t,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::sched_getaffinity(pid, cpusetsize, mask) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe {
            *error = errno;
        }
    }
    ret
}
