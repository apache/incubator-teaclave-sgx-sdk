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

#![allow(dead_code)]

use core::fmt::{Formatter, Result, Write};
use core::str::lossy::{Utf8Lossy, Utf8LossyChunk};

pub fn debug_fmt_bytestring(slice: &[u8], f: &mut Formatter<'_>) -> Result {
    // Writes out a valid unicode string with the correct escape sequences
    fn write_str_escaped(f: &mut Formatter<'_>, s: &str) -> Result {
        for c in s.chars().flat_map(|c| c.escape_debug()) {
            f.write_char(c)?
        }
        Ok(())
    }

    f.write_str("\"")?;
    for Utf8LossyChunk { valid, broken } in Utf8Lossy::from_bytes(slice).chunks() {
        write_str_escaped(f, valid)?;
        for b in broken {
            write!(f, "\\x{:02X}", b)?;
        }
    }
    f.write_str("\"")
}