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
use core::cmp;
use core::convert::{From, TryFrom};
use core::mem;
use core::sync::atomic::AtomicI32;
use core::time::Duration;
use sgx_sync::Futex;
use sgx_trts::trts::is_within_enclave;
use sgx_types::error::OsResult;

#[no_mangle]
pub unsafe extern "C" fn futex(
    uaddr: *const c_uint,
    futex_op: c_int,
    val: c_uint,
    timeout: *const timespec,
    uaddr2: *const c_uint,
    val3: c_uint,
) -> c_long {
    match futex_inernal(
        uaddr,
        futex_op,
        val,
        timeout as *const TimeSpec,
        uaddr2,
        val3,
    ) {
        Ok(ret) => ret as c_long,
        Err(e) => {
            set_errno(e);
            -1
        }
    }
}

unsafe fn futex_inernal(
    uaddr: *const u32,
    futex_op: i32,
    val: u32,
    timeout: *const TimeSpec,
    uaddr2: *const u32,
    val3: u32,
) -> OsResult<isize> {
    const FUTEX_OP_MASK: c_int = 0x0000_000F;

    let get_addr = |addr: *const u32| -> OsResult<&AtomicI32> {
        if addr.is_null() || !is_within_enclave(addr as *const u8, mem::size_of::<c_int>()) {
            Err(EINVAL)
        } else {
            Ok(&*(addr as *const AtomicI32))
        }
    };

    let get_val = |val: u32| -> OsResult<i32> {
        if val > i32::MAX as u32 {
            Err(EINVAL)
        } else {
            Ok(val as i32)
        }
    };

    let get_timeout = |timeout: *const TimeSpec| -> OsResult<Option<Duration>> {
        if timeout.is_null() {
            Ok(None)
        } else {
            Duration::try_from(*timeout).map(Some)
        }
    };

    let futex = Futex::new(get_addr(uaddr)?);
    match futex_op & FUTEX_OP_MASK {
        FUTEX_WAIT => {
            let timeout = get_timeout(timeout)?;
            futex.wait(val as i32, timeout).map(|_| 0)
        }
        FUTEX_WAIT_BITSET => {
            let timeout = get_timeout(timeout)?;
            futex.wait_bitset(val as i32, timeout, val3).map(|_| 0)
        }
        FUTEX_WAKE => {
            let count = get_val(val)?;
            futex.wake(count).map(|count| count as isize)
        }
        FUTEX_WAKE_BITSET => {
            let count = get_val(val)?;
            futex.wake_bitset(count, val3).map(|count| count as isize)
        }
        FUTEX_REQUEUE => {
            let new_futex = Futex::new(get_addr(uaddr2)?);
            let nwakes = get_val(val)?;
            let nrequeues = get_val(timeout as u32)?;
            futex
                .requeue(nwakes, &new_futex, nrequeues)
                .map(|nwakes| nwakes as isize)
        }
        FUTEX_CMP_REQUEUE => {
            let new_futex = Futex::new(get_addr(uaddr2)?);
            let nwakes = get_val(val)?;
            let nrequeues = get_val(timeout as u32)?;
            futex
                .cmp_requeue(nwakes, &new_futex, nrequeues, val3 as i32)
                .map(|total| total as isize)
        }
        _ => {
            #[cfg(feature = "panic_on_unsupported_syscall")]
            panic!("unsupported futex operation!");
            #[cfg(not(feature = "panic_on_unsupported_syscall"))]
            return Err(ENOSYS);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
struct TimeSpec {
    pub sec: i64,
    pub nsec: i64,
}

impl TimeSpec {
    fn validate(&self) -> bool {
        self.sec >= 0 && self.nsec >= 0 && self.nsec < 1_000_000_000
    }
}

impl From<Duration> for TimeSpec {
    fn from(dur: Duration) -> TimeSpec {
        TimeSpec {
            sec: cmp::min(dur.as_secs(), i64::MAX as u64) as i64,
            nsec: dur.subsec_nanos() as i64,
        }
    }
}

impl TryFrom<TimeSpec> for Duration {
    type Error = i32;

    fn try_from(ts: TimeSpec) -> OsResult<Duration> {
        if ts.validate() {
            Ok(Duration::new(ts.sec as u64, ts.nsec as u32))
        } else {
            Err(EINVAL)
        }
    }
}
