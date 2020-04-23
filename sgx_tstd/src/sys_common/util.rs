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

use core::fmt;
use crate::io::prelude::*;
#[cfg(feature = "stdio")]
use crate::sys::stdio::panic_output;

#[cfg(feature = "stdio")]
pub fn dumb_print(args: fmt::Arguments<'_>) {
    if let Some(mut out) = panic_output() {
        let _ = out.write_fmt(args);
    }
}

#[cfg(not(feature = "stdio"))]
pub fn dumb_print(args: fmt::Arguments<'_>) {

}

// Other platforms should use the appropriate platform-specific mechanism for
// aborting the process.  If no platform-specific mechanism is available,
// crate::intrinsics::abort() may be used instead.  The above implementations cover
// all targets currently supported by libstd.

pub fn abort(args: fmt::Arguments<'_>) -> ! {
    dumb_print(format_args!("fatal runtime error: {}\n", args));
    unsafe {
        crate::sys::abort_internal();
    }
}
