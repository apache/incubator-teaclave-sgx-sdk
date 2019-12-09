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
