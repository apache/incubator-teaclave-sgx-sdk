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
use core::convert::TryFrom;
use core::mem;
use core::sync::atomic::AtomicI32;
use core::time::Duration;
use sgx_sync::{Futex, FutexFlags, FutexOp, Timespec};
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
    match futex_inernal(uaddr, futex_op, val, timeout, uaddr2, val3) {
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
    timeout: *const timespec,
    uaddr2: *const u32,
    val3: u32,
) -> OsResult<isize> {
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

    let get_timeout = |timeout: *const timespec| -> OsResult<Option<Timespec>> {
        if timeout.is_null() {
            Ok(None)
        } else {
            Timespec::try_from(*timeout).map(Some)
        }
    };

    let futex = Futex::new(get_addr(uaddr)?);
    let (op, flags) = futex_op_and_flags_from_bits(futex_op as u32)?;

    match op {
        FutexOp::Wait => {
            let timeout = get_timeout(timeout).and_then(|ts| match ts {
                Some(t) => Duration::try_from(t).map(Some),
                None => Ok(None),
            })?;
            futex.wait(val as i32, timeout).map(|_| 0)
        }
        FutexOp::WaitBitset => {
            let timeout = get_timeout(timeout)?.map(|ts| (ts, flags.into()));
            futex.wait_bitset(val as i32, timeout, val3).map(|_| 0)
        }
        FutexOp::Wake => {
            let count = get_val(val)?;
            futex.wake(count).map(|count| count as isize)
        }
        FutexOp::WakeBitset => {
            let count = get_val(val)?;
            futex.wake_bitset(count, val3).map(|count| count as isize)
        }
        FutexOp::Requeue => {
            let new_futex = Futex::new(get_addr(uaddr2)?);
            let nwakes = get_val(val)?;
            let nrequeues = get_val(timeout as u32)?;
            futex
                .requeue(nwakes, &new_futex, nrequeues)
                .map(|nwakes| nwakes as isize)
        }
        FutexOp::CmpRequeue => {
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

fn futex_op_and_flags_from_bits(bits: u32) -> OsResult<(FutexOp, FutexFlags)> {
    const FUTEX_OP_MASK: u32 = 0x0000_000F;
    const FUTEX_FLAGS_MASK: u32 = 0xFFFF_FFF0;

    let op = {
        let op_bits = bits & FUTEX_OP_MASK;
        FutexOp::try_from(op_bits).map_err(|_| EINVAL)?
    };
    let flags = {
        let flags_bits = bits & FUTEX_FLAGS_MASK;
        FutexFlags::from_bits(flags_bits).ok_or(EINVAL)?
    };
    Ok((op, flags))
}
