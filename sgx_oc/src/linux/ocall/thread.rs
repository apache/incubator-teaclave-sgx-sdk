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

use super::*;

pub unsafe fn sched_yield() -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let status = u_sched_yield_ocall(&mut result as *mut c_int, &mut error as *mut c_int);

    ensure!(status.is_success(), esgx!(status));
    ensure!(result == 0, eos!(error));
    Ok(())
}

pub unsafe fn nanosleep(req: &mut timespec) -> OCallResult<()> {
    let mut result: c_int = 0;
    let mut error: c_int = 0;

    let mut rem = *req;
    let status = u_nanosleep_ocall(
        &mut result as *mut c_int,
        &mut error as *mut c_int,
        req as *const timespec,
        &mut rem as *mut timespec,
    );

    ensure!(status.is_success(), esgx!(status));
    ensure!(rem.tv_sec <= req.tv_sec, ecust!("Malformed remaining time"));
    if rem.tv_sec == req.tv_sec {
        ensure!(
            rem.tv_nsec < req.tv_nsec,
            ecust!("Malformed remaining time")
        );
    }

    req.tv_sec = rem.tv_sec;
    req.tv_nsec = rem.tv_nsec;

    ensure!(result == 0, eos!(error));

    req.tv_sec = 0;
    req.tv_nsec = 0;
    Ok(())
}
