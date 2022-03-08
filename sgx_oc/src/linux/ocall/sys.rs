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

use super::*;
use core::mem;

pub unsafe fn sysconf(name: c_int) -> OCallResult<c_long> {
    let mut result: c_long = 0;
    let mut error: c_int = 0;

    let status = u_sysconf_ocall(&mut result as *mut c_long, &mut error as *mut c_int, name);

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn prctl(
    option: c_int,
    arg2: c_ulong,
    arg3: c_ulong,
    arg4: c_ulong,
    arg5: c_ulong,
) -> OCallResult<c_int> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_prctl_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        option,
        arg2,
        arg3,
        arg4,
        arg5,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(result)
}

pub unsafe fn sched_getaffinity(pid: pid_t) -> OCallResult<cpu_set_t> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let mut mask: cpu_set_t = mem::zeroed();
    let status = u_sched_getaffinity_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        pid,
        mem::size_of::<cpu_set_t>(),
        &mut mask as *mut cpu_set_t,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(mask)
}

pub unsafe fn sched_setaffinity(pid: pid_t, mask: &cpu_set_t) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_sched_setaffinity_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        pid,
        mem::size_of::<cpu_set_t>(),
        mask as *const cpu_set_t,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(result >= 0, eos!(error));
    Ok(())
}
