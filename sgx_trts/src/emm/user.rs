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

use super::ema::{RegEmaAda, EMA};
use crate::emm::interior::Reserve;
use crate::enclave::MmLayout;
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::{Once, Mutex};
use core::alloc::Layout;
use core::ffi::c_void;
use core::ptr::NonNull;
use intrusive_collections::intrusive_adapter;
use intrusive_collections::{LinkedList, LinkedListLink};
use sgx_types::error::{SgxResult, SgxStatus};

#[derive(Clone, Copy)]
pub struct UserRange {
    pub start: usize,
    pub end: usize,
}

pub static USER_RANGE: Once<UserRange> = Once::new();

pub fn init_range(start: usize, end: usize) {
    // init 
    *USER_RANGE.call_once(|| {
        UserRange {
            start,
            end,
        }
    });
}

pub fn is_within_user_range(start: usize, len: usize) -> bool {
    let end = if len > 0 {
        if let Some(end) = start.checked_add(len - 1) {
            end
        } else {
            return false;
        }
    } else {
        start
    };
    let base = MmLayout::elrange_base();

    (start <= end) && (start >= base) && (end < base + MmLayout::elrange_size())
}

pub struct UserMem {
    emas: LinkedList<RegEmaAda>,

    // statistics
    allocated: usize,
    total: usize,
}

impl UserMem {
    pub fn new() -> Self {
        Self {
            emas: LinkedList::new(RegEmaAda::new()),
            allocated: 0,
            total: 0,
        }
    }
    // fn split(ema: Box<EMA>) -> SgxResult<()>{
    //     todo!()
    // }
    // fn merge(ema1: Box<EMA>, ema2: Box<EMA>)
    //     -> SgxResult<()> {
    //         todo!()
    //     }
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        todo!()
    }
    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        todo!()
    }
    pub fn commit(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        todo!()
    }
}
