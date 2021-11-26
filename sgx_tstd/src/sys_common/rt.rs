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

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unused_macros)]

// One-time runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
// pub fn cleanup() {
//     static CLEANUP: Once = Once::new();
//     CLEANUP.call_once(|| {
//         // Flush stdout and disable buffering.
//         crate::io::cleanup();

//         super::at_exit_imp::cleanup();
//     });
// }

/// One-time runtime cleanup.
pub fn cleanup() {
    use crate::sync::SgxThreadSpinlock;

    static SPIN_LOCK: SgxThreadSpinlock = SgxThreadSpinlock::new();
    static mut IS_CLEAUP: bool = false;

    unsafe {
        SPIN_LOCK.lock();
        if !IS_CLEAUP {
            super::at_exit_imp::cleanup();
            IS_CLEAUP = true;
        }
        SPIN_LOCK.unlock();
    }
}

// Prints to the "panic output", depending on the platform this may be:
// - the standard error output
// - some dedicated platform specific output
// - nothing (so this macro is a no-op)
macro_rules! rtprintpanic {
    ($($t:tt)*) => {
        if let Some(mut out) = crate::sys::stdio::panic_output() {
            let _ = crate::io::Write::write_fmt(&mut out, format_args!($($t)*));
        }
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
