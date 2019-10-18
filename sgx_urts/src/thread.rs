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

use std::io::Error;
use libc::{self, c_int,  c_void, pthread_t, pthread_attr_t, timespec};
use crate::sgx_types::{sgx_enclave_id_t, sgx_status_t} ;
use std::{mem, ptr};

extern {
    fn t_thread_main(eid: sgx_enclave_id_t, retval: * mut * mut c_void, arg: * mut c_void, len: c_int) -> sgx_status_t;
}

const THREAD_PARAM_SEALED_SIZE: usize = 576;
#[repr(C)]
#[derive(Copy, Clone)]
struct ThreadParam {
    main: [u8; THREAD_PARAM_SEALED_SIZE],
    eid: sgx_enclave_id_t,
}

impl ThreadParam {
    pub fn get_eid(&self) -> sgx_enclave_id_t { self.eid }
    pub fn get_size(&self) -> usize { mem::size_of_val(self) }
}  

#[no_mangle]
pub extern "C" fn u_pthread_create_ocall(native: * mut pthread_t,
                                         attr: * const pthread_attr_t,
                                         _f: * mut c_void,
                                         value: * mut c_void,
                                         len: c_int) -> c_int {
    if value.is_null() || len as usize != mem::size_of::<ThreadParam>() {
        return libc::EINVAL;
    }
    let tp_ptr = unsafe {
        let tp: Box<ThreadParam> = Box::new(*(value as * mut ThreadParam));
        Box::into_raw(tp)
    };
    let ret = unsafe {
        libc::pthread_create(native, attr, thread_start, tp_ptr as * mut c_void)
    };
    if ret != 0 {
        unsafe { drop(Box::from_raw(tp_ptr)); }
    }

    extern fn thread_start(arg: * mut c_void) -> * mut c_void {        
        let mut retval: * mut c_void = ptr::null_mut();
        let result = unsafe {
            let tp = Box::from_raw(arg as * mut ThreadParam);
            t_thread_main(tp.get_eid(), &mut retval, arg, tp.get_size() as c_int)
        };
        if result != sgx_status_t::SGX_SUCCESS {
            retval = result as u32 as * mut c_void;
        }
        retval
    }
    ret              
}

#[no_mangle]
pub extern "C" fn u_pthread_join_ocall(native: pthread_t, value: * mut * mut c_void) -> c_int {
     unsafe { libc::pthread_join(native, value) }               
}

#[no_mangle]
pub extern "C" fn u_pthread_detach_ocall(native: pthread_t) -> c_int {
    unsafe { libc::pthread_detach(native) }                 
}

#[no_mangle]
pub extern "C" fn u_sched_yield_ocall(error: * mut c_int) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::sched_yield() };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_nanosleep_ocall(error: * mut c_int, rqtp: * const timespec, rmtp: * mut timespec) -> c_int {
    let mut errno = 0;
    let ret = unsafe { libc::nanosleep(rqtp, rmtp) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret                      
}
