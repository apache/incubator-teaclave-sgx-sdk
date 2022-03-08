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

use crate::arch::SE_PAGE_SHIFT;
use crate::call::{ocall, OCallIndex, OcAlloc};
use alloc::boxed::Box;
use sgx_types::error::{SgxResult, SgxStatus};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct TrimRangeOcall {
    from: usize,
    to: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct TrimRangeCommitOcall {
    addr: usize,
}

pub fn trim_range(addr: usize, count: usize) -> SgxResult {
    let mut trim = Box::try_new_in(
        TrimRangeOcall {
            from: addr,
            to: addr + (count << SE_PAGE_SHIFT),
        },
        OcAlloc,
    )
    .map_err(|_| SgxStatus::OutOfMemory)?;

    ocall(OCallIndex::Trim, Some(trim.as_mut()))
}

pub fn trim_range_commit(addr: usize, count: usize) -> SgxResult {
    for i in 0..count {
        let mut trim = Box::try_new_in(
            TrimRangeCommitOcall {
                addr: addr + i * SE_PAGE_SHIFT,
            },
            OcAlloc,
        )
        .map_err(|_| SgxStatus::OutOfMemory)?;

        ocall(OCallIndex::TrimCommit, Some(trim.as_mut()))?;
    }
    Ok(())
}
