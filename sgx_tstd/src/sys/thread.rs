// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

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
use crate::sgx_tseal::{SgxSealedData, SgxUnsealedData};
use crate::sgx_types::{sgx_enclave_id_t, sgx_sealed_data_t, sgx_status_t, c_void, c_int} ;

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
        let aad: [u8; 8] = unsafe{ mem::transmute(eid) };
        let main = p as usize;
        let result = SgxSealedData::<usize>::seal_data(&aad, &main);
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
        if eid != self.eid {
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
    unsafe { start_thread(main as *mut u8); }
    ptr::null_mut()
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
        let tp = ThreadParam::new(eid, &*p as *const _ as *mut _)?;
        let ret = libc::pthread_create(&mut native, ptr::null(), ptr::null_mut(), &tp as *const _ as *mut _, tp.get_size() as c_int);                        
        return if ret != 0 {
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