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

use sgx_trts::libc;
use sgx_backtrace_sys::{
    backtrace_syminfo_callback, 
    backtrace_full_callback, 
    backtrace_error_callback, 
    backtrace_state,
    backtrace_create_state,
    backtrace_syminfo,
    backtrace_pcinfo};
use crate::ffi::CStr;
use crate::io;
use crate::sys::backtrace::BacktraceContext;
use crate::sys_common::backtrace::Frame;
use core::mem;
use core::ptr;

pub fn foreach_symbol_fileline<F>(frame: Frame,
                                  mut f: F,
                                  _: &BacktraceContext) -> io::Result<bool>
where F: FnMut(&[u8], u32) -> io::Result<()>
{
    // pcinfo may return an arbitrary number of file:line pairs,
    // in the order of stack trace (i.e. inlined calls first).
    // in order to avoid allocation, we stack-allocate a fixed size of entries.
    const FILELINE_SIZE: usize = 32;
    let mut fileline_buf = [(ptr::null(), !0); FILELINE_SIZE];
    let ret;
    let fileline_count = {
        let state = unsafe { (__init_state())? };
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
        let state = unsafe { __init_state()? };
        if state.is_null() {
            None
        } else {
            let mut data: *const libc::c_char = ptr::null();
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
// helper callbacks
////////////////////////////////////////////////////////////////////////

type FileLine = (*const libc::c_char, u32);

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
            buffer[0] = (filename, lineno as u32);
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

    let filename = match crate::sys::backtrace::gnu::get_enclave_filename() {
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
