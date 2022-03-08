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

use sgx_types::function::{
    sgx_ecall_ms_buffer_alloc_aligned, sgx_ecall_ms_buffer_free, sgx_ecall_ms_buffer_remain_size,
};
use sgx_types::types::EnclaveId;
use std::alloc::{AllocError, Allocator, Layout};
use std::ptr::NonNull;

pub struct MsBufAlloc {
    eid: EnclaveId,
}

impl MsBufAlloc {
    pub fn new(eid: EnclaveId) -> MsBufAlloc {
        MsBufAlloc { eid }
    }

    #[inline]
    pub fn remain_size(&self) -> usize {
        unsafe { sgx_ecall_ms_buffer_remain_size(self.eid) }
    }
}

unsafe impl Allocator for MsBufAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr =
            unsafe { sgx_ecall_ms_buffer_alloc_aligned(self.eid, layout.align(), layout.size()) };
        NonNull::new(ptr.cast())
            .map(|ptr| NonNull::slice_from_raw_parts(ptr, layout.size()))
            .ok_or(AllocError)
    }

    #[inline]
    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        sgx_ecall_ms_buffer_free(self.eid);
    }
}
