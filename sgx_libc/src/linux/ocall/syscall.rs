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

use crate::linux::ocall::*;
use crate::linux::*;

#[allow(unused_variables, non_upper_case_globals)]
#[no_mangle]
pub unsafe extern "C" fn syscall(
    sysno: c_long,
    arg0: c_long,
    arg1: c_long,
    arg2: c_long,
    arg3: c_long,
    arg4: c_long,
    arg5: c_long,
    arg6: c_long,
) -> c_long {
    match sysno {
        SYS_getrandom => getrandom(arg0 as *mut c_void, arg1 as size_t, arg2 as c_uint) as c_long,
        SYS_futex => futex(
            arg0 as *const c_uint,
            arg1 as c_int,
            arg2 as c_uint,
            arg3 as *const timespec,
            arg4 as *const c_uint,
            arg5 as c_uint,
        ),
        _ => {
            set_errno(ENOSYS);
            #[cfg(feature = "panic_on_unsupported_syscall")]
            panic!("unsupported system call!");
            #[cfg(not(feature = "panic_on_unsupported_syscall"))]
            return -1;
        }
    }
}
