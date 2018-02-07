// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

#[cfg(target_env = "sgx")]
use libc::c_void;
#[cfg(not(target_env = "sgx"))]
use sgx_trts::libc::c_void;
use error::Error;
use io;
use sys::backtrace::BacktraceContext;
use sys_common::backtrace::Frame;

use unwind as uw;

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

impl ::fmt::Display for UnwindError {
    fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
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
        frames: frames,
    };
    let result_unwind = unsafe {
        uw::_Unwind_Backtrace(trace_fn,
                              &mut cx as *mut Context
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
    let cx = unsafe { &mut *(arg as *mut Context) };
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

    if cx.idx < cx.frames.len() {
        cx.frames[cx.idx] = Frame {
            symbol_addr: symaddr as *mut u8,
            exact_position: ip as *mut u8,
            inline_context: 0,
        };
        cx.idx += 1;
    }

    uw::_URC_NO_REASON
}