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

//! Utilities for parsing DWARF-encoded data streams.
//! See <http://www.dwarfstd.org>,
//! DWARF-4 standard, Section 7 - "Data Representation"

// This module is used only by x86_64-pc-windows-gnu for now, but we
// are compiling it everywhere to avoid regressions.
#![allow(unused)]

pub mod eh;

use core::mem;

pub struct DwarfReader {
    pub ptr: *const u8,
}

#[repr(C, packed)]
struct Unaligned<T>(T);

impl DwarfReader {
    pub fn new(ptr: *const u8) -> DwarfReader {
        DwarfReader { ptr }
    }

    // DWARF streams are packed, so e.g., a u32 would not necessarily be aligned
    // on a 4-byte boundary. This may cause problems on platforms with strict
    // alignment requirements. By wrapping data in a "packed" struct, we are
    // telling the backend to generate "misalignment-safe" code.
    pub unsafe fn read<T: Copy>(&mut self) -> T {
        let Unaligned(result) = *(self.ptr as *const Unaligned<T>);
        self.ptr = self.ptr.add(mem::size_of::<T>());
        result
    }

    // ULEB128 and SLEB128 encodings are defined in Section 7.6 - "Variable
    // Length Data".
    pub unsafe fn read_uleb128(&mut self) -> u64 {
        let mut shift: usize = 0;
        let mut result: u64 = 0;
        let mut byte: u8;
        loop {
            byte = self.read::<u8>();
            result |= ((byte & 0x7F) as u64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
        }
        result
    }

    pub unsafe fn read_sleb128(&mut self) -> i64 {
        let mut shift: u32 = 0;
        let mut result: u64 = 0;
        let mut byte: u8;
        loop {
            byte = self.read::<u8>();
            result |= ((byte & 0x7F) as u64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
        }
        // sign-extend
        if shift < u64::BITS && (byte & 0x40) != 0 {
            result |= (!0) << shift;
        }
        result as i64
    }
}
