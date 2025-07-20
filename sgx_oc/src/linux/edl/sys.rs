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

use super::sgx_status_t;
use crate::linux::x86_64::*;

extern "C" {
    pub fn u_sysconf_ocall(result: *mut c_long, error: *mut c_int, name: c_int) -> sgx_status_t;
    pub fn u_prctl_ocall(
        result: *mut c_int,
        error: *mut c_int,
        option: c_int,
        arg2: c_ulong,
        arg3: c_ulong,
        arg4: c_ulong,
        arg5: c_ulong,
    ) -> sgx_status_t;
    pub fn u_sched_getaffinity_ocall(
        result: *mut c_int,
        error: *mut c_int,
        pid: pid_t,
        cpusetsize: size_t,
        mask: *mut cpu_set_t,
    ) -> sgx_status_t;
    pub fn u_sched_setaffinity_ocall(
        result: *mut c_int,
        error: *mut c_int,
        pid: pid_t,
        cpusetsize: size_t,
        mask: *const cpu_set_t,
    ) -> sgx_status_t;
}
