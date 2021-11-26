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

use crate::ffi::c_void;
use crate::sys::backtrace::Bomb;

use sgx_unwind as uw;

pub enum Frame {
    Raw(*mut uw::_Unwind_Context),
    Cloned {
        ip: *mut c_void,
        sp: *mut c_void,
        symbol_address: *mut c_void,
    },
}

// With a raw libunwind pointer it should only ever be access in a readonly
// threadsafe fashion, so it's `Sync`. When sending to other threads via `Clone`
// we always switch to a version which doesn't retain interior pointers, so we
// should be `Send` as well.
unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Frame {
    pub fn ip(&self) -> *mut c_void {
        let ctx = match *self {
            Frame::Raw(ctx) => ctx,
            Frame::Cloned { ip, .. } => return ip,
        };
        unsafe { uw::_Unwind_GetIP(ctx) as *mut c_void }
    }

    pub fn sp(&self) -> *mut c_void {
        match *self {
            Frame::Raw(ctx) => unsafe { uw::_Unwind_GetCFA(ctx) as *mut c_void },
            Frame::Cloned { sp, .. } => sp,
        }
    }

    pub fn symbol_address(&self) -> *mut c_void {
        if let Frame::Cloned { symbol_address, .. } = *self {
            return symbol_address;
        }

        // It seems that on OSX `_Unwind_FindEnclosingFunction` returns a
        // pointer to... something that's unclear. It's definitely not always
        // the enclosing function for whatever reason. It's not entirely clear
        // to me what's going on here, so pessimize this for now and just always
        // return the ip.
        //
        // Note the `skip_inner_frames.rs` test is skipped on OSX due to this
        // clause, and if this is fixed that test in theory can be run on OSX!
        if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
            self.ip()
        } else {
            unsafe { uw::_Unwind_FindEnclosingFunction(self.ip()) }
        }
    }

    pub fn module_base_address(&self) -> Option<*mut c_void> {
        None
    }
}

impl Clone for Frame {
    fn clone(&self) -> Frame {
        Frame::Cloned {
            ip: self.ip(),
            sp: self.sp(),
            symbol_address: self.symbol_address(),
        }
    }
}

#[inline(always)]
pub unsafe fn trace(mut cb: &mut dyn FnMut(&super::Frame) -> bool) {
    uw::_Unwind_Backtrace(trace_fn, &mut cb as *mut _ as *mut _);

    extern "C" fn trace_fn(
        ctx: *mut uw::_Unwind_Context,
        arg: *mut c_void,
    ) -> uw::_Unwind_Reason_Code {
        let cb = unsafe { &mut *(arg as *mut &mut dyn FnMut(&super::Frame) -> bool) };
        let cx = super::Frame {
            inner: Frame::Raw(ctx),
        };

        let mut bomb = Bomb::new(true);
        let keep_going = cb(&cx);
        bomb.set(false);

        if keep_going {
            uw::_URC_NO_REASON
        } else {
            uw::_URC_FAILURE
        }
    }
}
