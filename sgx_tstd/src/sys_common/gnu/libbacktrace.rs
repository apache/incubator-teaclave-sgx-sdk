// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

use libc;
use ffi::CStr;
use io;
use sys::backtrace::BacktraceContext;
use sys_common::backtrace::Frame;
use core::mem;
use core::ptr;

pub fn foreach_symbol_fileline<F>(frame: Frame,
                                  mut f: F,
                                  _: &BacktraceContext) -> io::Result<bool>
where F: FnMut(&[u8], libc::c_int) -> io::Result<()>
{
    // pcinfo may return an arbitrary number of file:line pairs,
    // in the order of stack trace (i.e. inlined calls first).
    // in order to avoid allocation, we stack-allocate a fixed size of entries.
    const FILELINE_SIZE: usize = 32;
    let mut fileline_buf = [(ptr::null(), -1); FILELINE_SIZE];
    let ret;
    let fileline_count = {
        let state = unsafe { try!(__init_state()) };
        if state.is_null() {
            ret = -1;
            0
        } else {
            let mut fileline_win: &mut [FileLine] = &mut fileline_buf;
            let fileline_addr = &mut fileline_win as *mut &mut [FileLine];
            ret = unsafe {
                backtrace_pcinfo(state,
                                frame.exact_position as libc::uintptr_t,
                                pcinfo_cb,
                                error_cb,
                                fileline_addr as *mut libc::c_void)
            };
            FILELINE_SIZE - fileline_win.len()
        }
    };
    if ret == 0 {
        for &(file, line) in &fileline_buf[..fileline_count] {
            if file.is_null() { continue; } // just to be sure
            let file = unsafe { CStr::from_ptr(file).to_bytes() };
            f(file, line)?;
        }
        Ok(fileline_count == FILELINE_SIZE)
    } else {
        Ok(false)
    }
}

/// Converts a pointer to symbol to its string value.
pub fn resolve_symname<F>(frame: Frame,
                          callback: F,
                          _: &BacktraceContext) -> io::Result<()>
    where F: FnOnce(Option<&str>) -> io::Result<()>
{
    let symname = {
        let state = unsafe { try!(__init_state()) };
        if state.is_null() {
            None
        } else {
            let mut data = ptr::null();
            let data_addr = &mut data as *mut *const libc::c_char;
            let ret = unsafe {
                backtrace_syminfo(state,
                                  frame.symbol_addr as libc::uintptr_t,
                                  syminfo_cb,
                                  error_cb,
                                  data_addr as *mut libc::c_void)
            };
            if ret == 0 || data.is_null() {
                None
            } else {
                unsafe {
                    CStr::from_ptr(data).to_str().ok()
                }
            }
        }
    };
    callback(symname)
}

pub fn init_state() -> io::Result<()> {
    unsafe { __init_state().map(|_|()) }
}

////////////////////////////////////////////////////////////////////////
// libbacktrace.h API
////////////////////////////////////////////////////////////////////////
type backtrace_syminfo_callback =
extern "C" fn(data: *mut libc::c_void,
              pc: libc::uintptr_t,
              symname: *const libc::c_char,
              symval: libc::uintptr_t,
              symsize: libc::uintptr_t);
type backtrace_full_callback =
extern "C" fn(data: *mut libc::c_void,
              pc: libc::uintptr_t,
              filename: *const libc::c_char,
              lineno: libc::c_int,
              function: *const libc::c_char) -> libc::c_int;
type backtrace_error_callback =
extern "C" fn(data: *mut libc::c_void,
              msg: *const libc::c_char,
              errnum: libc::c_int);
enum backtrace_state {}

extern {
    fn backtrace_create_state(filename: *const libc::c_char,
                              threaded: libc::c_int,
                              error: backtrace_error_callback,
                              data: *mut libc::c_void)
        -> *mut backtrace_state;
    fn backtrace_syminfo(state: *mut backtrace_state,
                         addr: libc::uintptr_t,
                         cb: backtrace_syminfo_callback,
                         error: backtrace_error_callback,
                         data: *mut libc::c_void) -> libc::c_int;
    fn backtrace_pcinfo(state: *mut backtrace_state,
                        addr: libc::uintptr_t,
                        cb: backtrace_full_callback,
                        error: backtrace_error_callback,
                        data: *mut libc::c_void) -> libc::c_int;
}

////////////////////////////////////////////////////////////////////////
// helper callbacks
////////////////////////////////////////////////////////////////////////

type FileLine = (*const libc::c_char, libc::c_int);

extern fn error_cb(_data: *mut libc::c_void, _msg: *const libc::c_char,
                   _errnum: libc::c_int) {
    // do nothing for now
}
extern fn syminfo_cb(data: *mut libc::c_void,
                     _pc: libc::uintptr_t,
                     symname: *const libc::c_char,
                     _symval: libc::uintptr_t,
                     _symsize: libc::uintptr_t) {

    let slot = data as *mut *const libc::c_char;
    unsafe { *slot = symname; }
}
extern fn pcinfo_cb(data: *mut libc::c_void,
                    _pc: libc::uintptr_t,
                    filename: *const libc::c_char,
                    lineno: libc::c_int,
                    _function: *const libc::c_char) -> libc::c_int {
    if !filename.is_null() {
        let slot = data as *mut &mut [FileLine];
        let buffer = unsafe {ptr::read(slot)};

        // if the buffer is not full, add file:line to the buffer
        // and adjust the buffer for next possible calls to pcinfo_cb.
        if !buffer.is_empty() {
            buffer[0] = (filename, lineno);
            unsafe { ptr::write(slot, &mut buffer[1..]); }
        }
    }

    0
}

// The libbacktrace API supports creating a state, but it does not
// support destroying a state. I personally take this to mean that a
// state is meant to be created and then live forever.
//
// I would love to register an at_exit() handler which cleans up this
// state, but libbacktrace provides no way to do so.
//
// With these constraints, this function has a statically cached state
// that is calculated the first time this is requested. Remember that
// backtracing all happens serially (one global lock).
//
// Things don't work so well on not-Linux since libbacktrace can't track
// down that executable this is. We at one point used env::current_exe but
// it turns out that there are some serious security issues with that
// approach.
//
// Specifically, on certain platforms like BSDs, a malicious actor can cause
// an arbitrary file to be placed at the path returned by current_exe.
// libbacktrace does not behave defensively in the presence of ill-formed
// DWARF information, and has been demonstrated to segfault in at least one
// case. There is no evidence at the moment to suggest that a more carefully
// constructed file can't cause arbitrary code execution. As a result of all
// of this, we don't hint libbacktrace with the path to the current process.
unsafe fn __init_state() -> io::Result<*mut backtrace_state> {

    static mut STATE: *mut backtrace_state = ptr::null_mut();

    if !STATE.is_null() { return Ok(STATE)  }

    let filename = match ::sys::backtrace::gnu::get_enclave_filename() {
        Ok(filename) => {
            // filename is purposely leaked here since libbacktrace requires
            // it to stay allocated permanently.
            let filename_ptr = filename.as_ptr();
            mem::forget(filename);
            filename_ptr
        },
        Err(_) => ptr::null(),
    };

    if filename.is_null() {
        return Err(io::Error::from_raw_os_error(libc::ENOENT));
    }

    STATE = backtrace_create_state(filename, 0, error_cb, ptr::null_mut());
    
    if STATE.is_null() {
        return Err(io::Error::from_raw_os_error(libc::ENOMEM));
    }

    Ok(STATE)
}
