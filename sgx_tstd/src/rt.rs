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

//! Runtime services

#![allow(unused_macros)]

// Re-export some of our utilities which are expected by other crates.
pub use crate::panicking::{begin_panic, panic_count};
pub use crate::sys_common::at_exit;
pub use core::panicking::{panic_display, panic_fmt};

use crate::enclave::Enclave;
use crate::ffi::CString;
use crate::slice;
use crate::str;
use crate::sync::Once;
use crate::sys;

// Prints to the "panic output", depending on the platform this may be:
// - the standard error output
// - some dedicated platform specific output
// - nothing (so this macro is a no-op)
#[cfg(feature = "stdio")]
macro_rules! rtprintpanic {
    ($($t:tt)*) => {
        if let Some(mut out) = crate::sys::stdio::panic_output() {
            let _ = crate::io::Write::write_fmt(&mut out, format_args!($($t)*));
        }
    }
}

#[cfg(not(feature = "stdio"))]
macro_rules! rtprintpanic {
    ($($t:tt)*) => {
        format_args!($($t)*);
    }
}

macro_rules! rtabort {
    ($($t:tt)*) => {
        {
            rtprintpanic!("fatal runtime error: {}\n", format_args!($($t)*));
            crate::sys::abort_internal();
        }
    }
}

macro_rules! rtassert {
    ($e:expr) => {
        if !$e {
            rtabort!(concat!("assertion failed: ", stringify!($e)));
        }
    };
}

macro_rules! rtunwrap {
    ($ok:ident, $e:expr) => {
        match $e {
            $ok(v) => v,
            ref err => {
                let err = err.as_ref().map(drop); // map Ok/Some which might not be Debug
                rtabort!(concat!("unwrap failed: ", stringify!($e), " = {:?}"), err)
            }
        }
    };
}

macro_rules! should_panic {
    ($fmt:expr) => {{
        match crate::panic::catch_unwind(crate::panic::AssertUnwindSafe(|| $fmt)).is_err() {
            true => {}
            false => crate::rt::begin_panic($fmt),
        }
    }};
}

macro_rules! global_ctors_object {
    ($var_name:ident, $func_name:ident = $func:block) => {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                #[link_section = ".init_array"]
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else if #[cfg(target_os = "windows")]  {
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else if #[cfg(target_os = "macos")]  {
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else {

            }
        }
        #[no_mangle]
        pub fn $func_name() {
            {
                $func
            };
        }
    };
}

macro_rules! global_dtors_object {
    ($var_name:ident, $func_name:ident = $func:block) => {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                #[link_section = ".fini_array"]
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else if #[cfg(target_os = "windows")]  {
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else if #[cfg(target_os = "macos")]  {
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else {

            }
        }
        #[no_mangle]
        pub fn $func_name() {
            {
                $func
            };
        }
    };
}

static INIT: Once = Once::new();
static EXIT: Once = Once::new();

#[no_mangle]
unsafe extern "C" fn global_init_ecall(
    eid: u64,
    path: *const u8,
    path_len: usize,
    env: *const u8,
    env_len: usize,
    args: *const u8,
    args_len: usize,
) {
    INIT.call_once(|| {
        if eid > 0 {
            Enclave::set_id(eid);
        }
        if !path.is_null() && path_len > 0 {
            if let Ok(s) = str::from_utf8(slice::from_raw_parts(path, path_len)) {
                Enclave::set_path_unchecked(s);
            }
        }

        let parse_vec = |ptr: *const u8, len: usize| -> Vec<CString> {
            if !ptr.is_null() && len > 0 {
                let buf = slice::from_raw_parts(ptr, len);
                buf.split(|&c| c == 0)
                    .filter_map(|bytes| {
                        if !bytes.is_empty() {
                            CString::new(bytes).ok()
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        };

        let env = parse_vec(env, env_len);
        let args = parse_vec(args, args_len);
        sys::init(env, args);
    });
}

#[no_mangle]
unsafe extern "C" fn global_exit_ecall() {}

global_dtors_object! {
    GLOBAL_DTORS, global_dtors = {
        let _ = crate::panic::catch_unwind(cleanup);
    }
}

// One-time runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub (crate) fn cleanup() {
    static CLEANUP: Once = Once::new();
    CLEANUP.call_once(|| {
        // Flush stdout and disable buffering.
        #[cfg(feature = "stdio")]
        crate::io::cleanup();

        crate::sys_common::at_exit_imp::cleanup();
        // SAFETY: Only called once during runtime cleanup.
        unsafe { sys::cleanup() };
    });
}
