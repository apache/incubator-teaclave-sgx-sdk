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

use core::u64;
use crate::sys_common::thread::*;
use crate::time::Duration;
use crate::ptr;
use crate::io;
use crate::mem;
use crate::cmp;
use crate::sys::os;
use crate::ffi::CStr;
use crate::enclave::get_enclave_id;
use crate::sgx_tseal::SgxSealedData;
use crate::sgx_types::{self, sgx_attributes_t, sgx_enclave_id_t, sgx_sealed_data_t, sgx_status_t, c_void, c_int};

pub struct Thread {
    id: libc::pthread_t,
}
// Some platforms may have pthread_t as a pointer in which case we still want
// a thread to be Send/Sync
unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

const THREAD_PARAM_SEALED_SIZE: usize = 576;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ThreadParam {
    main: [u8; THREAD_PARAM_SEALED_SIZE],
    eid: sgx_enclave_id_t,
}

impl ThreadParam {
    pub fn new(eid: sgx_enclave_id_t, p: *mut c_void) -> io::Result<ThreadParam> {
        let key_policy = sgx_types::SGX_KEYPOLICY_MRENCLAVE;
        let attr_mask = sgx_attributes_t {
                        flags: sgx_types::TSEAL_DEFAULT_FLAGSMASK,
                        xfrm: 0};
        let misc_mask = sgx_types::TSEAL_DEFAULT_MISCMASK;

        let aad: [u8; 8] = unsafe{ mem::transmute(eid) };
        let main = p as usize;
        let result = SgxSealedData::<usize>::seal_data_ex(key_policy, attr_mask, misc_mask, &aad, &main);
        let sealed_data = match result {
            Ok(t) => t,
            Err(ret) => { return Err(io::Error::from_sgx_error(ret)); },
        };

        let mut main = [0_u8; THREAD_PARAM_SEALED_SIZE];
        let result = unsafe {
            sealed_data.to_raw_sealed_data_t(main.as_mut_ptr() as *mut sgx_sealed_data_t, THREAD_PARAM_SEALED_SIZE as u32)
        };
        if result.is_none() {
            return Err(io::Error::from_sgx_error(sgx_status_t::SGX_ERROR_UNEXPECTED));
        }

        Ok(ThreadParam {
            main: main.clone(),
            eid: eid,
        })
    }

    pub fn get_main(&self) -> io::Result<*mut c_void> {
        let result = unsafe {
            let p = self.main.as_ptr() as * const _ as *mut sgx_sealed_data_t;
            SgxSealedData::<usize>::from_raw_sealed_data_t(p, THREAD_PARAM_SEALED_SIZE as u32)
        };
        let sealed_data = match result {
            Some(t) => t,
            None => { return Err(io::Error::from_sgx_error(sgx_status_t::SGX_ERROR_UNEXPECTED)); },
        };

        let result = sealed_data.unseal_data();
        let unsealed_data = match result {
            Ok(t) => t,
            Err(ret) => { return Err(io::Error::from_sgx_error(ret)); },
        };

        let mut addr = [0_u8; 8];
        &mut addr[..].copy_from_slice(unsealed_data.get_additional_txt());
        let eid = u64::from_ne_bytes(addr);
        if eid != self.eid || eid != get_enclave_id() {
            return Err(io::Error::from_sgx_error(sgx_status_t::SGX_ERROR_INVALID_ENCLAVE_ID));
        }
        Ok(*unsealed_data.get_decrypt_txt() as *mut c_void)
    }

    pub fn get_eid(&self) -> sgx_enclave_id_t { self.eid }
    pub fn get_size(&self) -> usize { mem::size_of_val(self) }
}  

#[no_mangle]
pub extern "C" fn t_thread_main(arg: *mut c_void, len: c_int) -> *mut c_void {
    if arg.is_null() || len as usize != mem::size_of::<ThreadParam>() {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER as u32 as *mut c_void;
    }

    let param: &ThreadParam = unsafe { &*(arg as *mut ThreadParam) };
    let result = param.get_main();
    let main = match result {
        Ok(p) => p,
        Err(ret) => { return ret.raw_sgx_error().unwrap() as u32 as *mut c_void; }
    };

    if pop_thread_queue(main as *mut usize).is_some() {
        unsafe { start_thread(main as *mut u8); }
        ptr::null_mut()
    } else {
        sgx_status_t::SGX_ERROR_INVALID_PARAMETER as u32 as *mut c_void
    }
}

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        let p = box p;
        let mut native: libc::pthread_t = mem::zeroed();
      
        let eid = get_enclave_id();
        if eid == 0 {
            return Err(io::Error::from_sgx_error(sgx_status_t::SGX_ERROR_INVALID_ENCLAVE_ID));
        }
        let main = &*p as *const _ as *mut c_void;
        let tp = ThreadParam::new(eid, main)?;
        push_thread_queue(main as *mut usize);
        let ret = libc::pthread_create(&mut native, ptr::null(), ptr::null_mut(), &tp as *const _ as *mut _, tp.get_size() as c_int);                        
        return if ret != 0 {
            let _ = pop_thread_queue(main as *mut usize);
            Err(io::Error::from_raw_os_error(ret))
        } else {
            mem::forget(p); // ownership passed to pthread_create
            Ok(Thread { id: native })
        };
    }

    pub fn set_name(name: &CStr) {
        const PR_SET_NAME: libc::c_int = 15;
        // pthread wrapper only appeared in glibc 2.12, so we use syscall
        // directly.
        unsafe {
            libc::prctl(PR_SET_NAME, name.as_ptr() as libc::c_ulong, 0, 0, 0);
        }
    }

    pub fn yield_now() {
        let ret = unsafe { libc::sched_yield() };
        debug_assert_eq!(ret, 0);
    }

    pub fn sleep(dur: Duration) {
        let mut secs = dur.as_secs();
        let mut nsecs = dur.subsec_nanos() as _;

        // If we're awoken with a signal then the return value will be -1 and
        // nanosleep will fill in `ts` with the remaining time.
        unsafe {
            while secs > 0 || nsecs > 0 {
                let mut ts = libc::timespec {
                    tv_sec: cmp::min(libc::time_t::max_value() as u64, secs) as libc::time_t,
                    tv_nsec: nsecs,
                };
                secs -= ts.tv_sec as u64;
                if libc::nanosleep(&ts, &mut ts) == -1 {
                    assert_eq!(os::errno(), libc::EINTR);
                    secs += ts.tv_sec as u64;
                    nsecs = ts.tv_nsec;
                } else {
                    nsecs = 0;
                }
            }
        }
    }

    pub fn join(self) -> sgx_status_t {
        unsafe {
            let mut reval: * mut c_void = ptr::null_mut();
            let ret = libc::pthread_join(self.id, &mut reval);
            mem::forget(self);
            assert!(ret == 0,
                    "failed to join thread: {}", io::Error::from_raw_os_error(ret));
            if reval.is_null() {
                sgx_status_t::SGX_SUCCESS
            } else {
                sgx_status_t::from_repr(reval as u32).unwrap()
            }
        }
    }

    pub fn id(&self) -> libc::pthread_t { self.id }

    pub fn into_id(self) -> libc::pthread_t {
        let id = self.id;
        mem::forget(self);
        id
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        let ret = unsafe { libc::pthread_detach(self.id) };
        debug_assert_eq!(ret, 0);
    }
}

mod libc {
    pub use sgx_trts::libc::*;
    pub use sgx_trts::libc::ocall::{pthread_create, sched_yield, nanosleep, pthread_join, pthread_detach, prctl};
}