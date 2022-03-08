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
use core::alloc::Layout;
use core::num::NonZeroUsize;
use core::ptr;
use core::ptr::NonNull;
use sgx_oc::linux::ocall::HostAlloc;

#[no_mangle]
pub unsafe extern "C" fn host_malloc(size: size_t) -> *mut c_void {
    if let Some(size) = NonZeroUsize::new(size) {
        let layout = Layout::from_size_align_unchecked(size.get(), 1);
        HostAlloc
            .host_malloc(layout, false)
            .map(|p| p.as_ptr() as _)
            .unwrap_or(ptr::null_mut())
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn host_malloc_zeored(size: size_t) -> *mut c_void {
    if let Some(size) = NonZeroUsize::new(size) {
        let layout = Layout::from_size_align_unchecked(size.get(), 1);
        HostAlloc
            .host_malloc(layout, true)
            .map(|p| p.as_ptr() as _)
            .unwrap_or(ptr::null_mut())
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn host_free(p: *mut c_void) {
    if let Some(p) = NonNull::new(p as *mut u8) {
        HostAlloc.host_free(p);
    }
}
