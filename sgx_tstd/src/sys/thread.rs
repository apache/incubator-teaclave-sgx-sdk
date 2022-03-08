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

use crate::cmp;
use crate::ffi::CStr;
use crate::io;
use crate::num::NonZeroUsize;
use crate::ptr;
use crate::time::Duration;

use sgx_oc::ocall::HostBuffer;
use sgx_trts::thread::{Native, Thread as NativeThread};
use sgx_trts::trts;

pub struct Thread {
    native: NativeThread,
}
// Some platforms may have pthread_t as a pointer in which case we still want
// a thread to be Send/Sync
unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        let p = Box::into_raw(box p);
        let ret = NativeThread::new(thread_start, p as *mut _)
            .map(|t| Thread { native: t })
            .map_err(io::Error::from_sgx_error);

        if ret.is_err() {
            drop(Box::from_raw(p));
        }

        return ret;

        fn thread_start(main: *mut libc::c_void) -> *mut libc::c_void {
            unsafe {
                // Finally, let's run some code.
                Box::from_raw(main as *mut Box<dyn FnOnce()>)();
            }
            ptr::null_mut()
        }
    }

    pub fn yield_now() {
        let ret = unsafe { libc::sched_yield() };
        debug_assert!(ret.is_ok());
    }

    pub fn set_name(name: &CStr) {
        const PR_SET_NAME: libc::c_int = 15;

        let name = name.to_bytes_with_nul();
        if let Ok(host_buf) = HostBuffer::from_enclave_slice(name) {
            unsafe { libc::prctl(PR_SET_NAME, host_buf.as_ptr() as libc::c_ulong, 0, 0, 0); }
        }
    }

    pub fn sleep(dur: Duration) {
        let mut secs = dur.as_secs();
        let mut nsecs = dur.subsec_nanos() as _;

        // If we're awoken with a signal then the return value will be -1 and
        // nanosleep will fill in `ts` with the remaining time.
        unsafe {
            while secs > 0 || nsecs > 0 {
                let mut ts = libc::timespec {
                    tv_sec: cmp::min(libc::time_t::MAX as u64, secs) as libc::time_t,
                    tv_nsec: nsecs,
                };
                secs -= ts.tv_sec as u64;
                if let Err(e) = libc::nanosleep(&mut ts) {
                    assert!(e.equal_to_os_error(libc::EINTR));
                    secs += ts.tv_sec as u64;
                    nsecs = ts.tv_nsec;
                } else {
                    nsecs = 0;
                }
            }
        }
    }

    pub fn join(self) {
        let ret = self.native.join();
        assert!(ret.is_ok(), "failed to join thread: {}", io::Error::from_sgx_error(ret.unwrap_err()));
    }

    pub fn id(&self) -> *const Native {
        self.native.as_raw()
    }

    pub fn into_id(self) -> *const Native {
        self.native.into_raw()
    }
}

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    let cpus = trts::cpu_core_num();
    NonZeroUsize::new(cpus as usize).ok_or_else(|| io::const_io_error!(
        io::ErrorKind::NotFound,
        "The number of hardware threads is not known for the target platform",
    ))
}

mod libc {
    pub use sgx_oc::ocall::{nanosleep, prctl, sched_yield};
    pub use sgx_oc::*;
}
