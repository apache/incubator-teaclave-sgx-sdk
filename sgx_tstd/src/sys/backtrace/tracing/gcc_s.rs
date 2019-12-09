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

use sgx_trts::libc::c_void;
use core::fmt;
use crate::error::Error;
use crate::io;
use crate::sys::backtrace::BacktraceContext;
use crate::sys_common::backtrace::Frame;

use sgx_unwind as uw;

struct Context<'a> {
    idx: usize,
    frames: &'a mut [Frame],
}

#[derive(Debug)]
struct UnwindError(uw::_Unwind_Reason_Code);

impl Error for UnwindError {
    fn description(&self) -> &'static str {
        "unexpected return value while unwinding"
    }
}

impl fmt::Display for UnwindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.description(), self.0)
    }
}

#[inline(never)] // if we know this is a function call, we can skip it when
                 // tracing
pub fn unwind_backtrace(frames: &mut [Frame])
    -> io::Result<(usize, BacktraceContext)>
{
    let mut cx = Context {
        idx: 0,
        frames,
    };
    let result_unwind = unsafe {
        uw::_Unwind_Backtrace(trace_fn,
                              &mut cx as *mut Context<'_>
                              as *mut c_void)
    };
    // See libunwind:src/unwind/Backtrace.c for the return values.
    // No, there is no doc.
    match result_unwind {
        // These return codes seem to be benign and need to be ignored for backtraces
        // to show up properly on all tested platforms.
        uw::_URC_END_OF_STACK | uw::_URC_FATAL_PHASE1_ERROR | uw::_URC_FAILURE => {
            Ok((cx.idx, BacktraceContext))
        }
        _ => {
            Err(io::Error::new(io::ErrorKind::Other,
                               UnwindError(result_unwind)))
        }
    }
}

extern fn trace_fn(ctx: *mut uw::_Unwind_Context,
                   arg: *mut c_void) -> uw::_Unwind_Reason_Code {
    let cx = unsafe { &mut *(arg as *mut Context<'_>) };
    if cx.idx >= cx.frames.len() {
        return uw::_URC_NORMAL_STOP;
    }

    let mut ip_before_insn = 0;
    let mut ip = unsafe {
        uw::_Unwind_GetIPInfo(ctx, &mut ip_before_insn) as *mut c_void
    };
    if !ip.is_null() && ip_before_insn == 0 {
        // this is a non-signaling frame, so `ip` refers to the address
        // after the calling instruction. account for that.
        ip = (ip as usize - 1) as *mut _;
    }

    // dladdr() on osx gets whiny when we use FindEnclosingFunction, and
    // it appears to work fine without it, so we only use
    // FindEnclosingFunction on non-osx platforms. In doing so, we get a
    // slightly more accurate stack trace in the process.
    //
    // This is often because panic involves the last instruction of a
    // function being "call std::rt::begin_unwind", with no ret
    // instructions after it. This means that the return instruction
    // pointer points *outside* of the calling function, and by
    // unwinding it we go back to the original function.
    let symaddr = if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
        ip
    } else {
        unsafe { uw::_Unwind_FindEnclosingFunction(ip) }
    };

    cx.frames[cx.idx] = Frame {
        symbol_addr: symaddr as *mut u8,
        exact_position: ip as *mut u8,
        inline_context: 0,
    };
    cx.idx += 1;

    uw::_URC_NO_REASON
}
