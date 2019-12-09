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

/// A wrapper mod for sys_common::backtrace, providing `enable_backtrace`
/// interface.

pub use crate::sys_common::backtrace::__rust_begin_short_backtrace;
pub use crate::sys_common::backtrace::PrintFormat;

use crate::path::Path;
use crate::io;
use crate::enclave;
use crate::sys_common::backtrace::set_enabled;

/// Enable backtrace for dumping call stack on crash.
///
/// Enabling backtrace here makes the panic information along with call stack
/// information. Very similar to normal `RUST_BACKTRACE=1` flag in Rust.
///
/// * `path` - The path of enclave's file image. This is essential for dumping
/// symbol information on crash point.
/// * Always return `Ok(())`
pub fn enable_backtrace<P: AsRef<Path>>(path: P, format: PrintFormat) -> io::Result<()> {

    enclave::set_enclave_path(path)?;
    set_enabled(format);
    Ok(())
}
