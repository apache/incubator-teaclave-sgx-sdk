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

use crate::elf::dynamic::Dynamic;
use crate::elf::symtabl;
use crate::elf::zero::read_str;
use core::slice;
use sgx_types::marker::ContiguousMemory;

#[derive(Debug)]
pub enum SectionData<'a> {
    Empty,
    Undefined(&'a [u8]),
    Group { flags: &'a u32, indicies: &'a [u32] },
    StrArray(&'a [u8]),
    FnArray32(&'a [u32]),
    FnArray64(&'a [u64]),
    SymbolTable32(&'a [symtabl::Entry32]),
    SymbolTable64(&'a [symtabl::Entry64]),
    DynSymbolTable32(&'a [symtabl::DynEntry32]),
    DynSymbolTable64(&'a [symtabl::DynEntry64]),
    SymTabShIndex(&'a [u32]),
    Note64(&'a NoteHeader, &'a [u8]),
    Rela32(&'a [Rela<u32>]),
    Rela64(&'a [Rela<u64>]),
    Rel32(&'a [Rel<u32>]),
    Rel64(&'a [Rel<u64>]),
    Dynamic32(&'a [Dynamic<u32>]),
    Dynamic64(&'a [Dynamic<u64>]),
}

pub struct FnType(extern "C" fn());
impl FnType {
    pub fn get_fn(&self) -> extern "C" fn() {
        self.0
    }
}
unsafe impl ContiguousMemory for FnType {}

pub struct FnArray64<'a>(&'a [FnType]);
impl<'a> FnArray64<'a> {
    pub fn new(array: &'a [FnType]) -> FnArray64<'a> {
        FnArray64(array)
    }

    pub fn get_array(&self) -> &'a [FnType] {
        self.0
    }
}

#[derive(Debug)]
pub struct Rela64Array<'a>(&'a [Rela<u64>]);
impl<'a> Rela64Array<'a> {
    pub fn new(array: &'a [Rela<u64>]) -> Rela64Array<'a> {
        Rela64Array(array)
    }

    pub fn get_array(&self) -> &'a [Rela<u64>] {
        self.0
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Rela<P> {
    offset: P,
    info: P,
    addend: P,
}

#[derive(Debug)]
#[repr(C)]
pub struct Rel<P> {
    offset: P,
    info: P,
}

unsafe impl<P> ContiguousMemory for Rela<P> {}
unsafe impl<P> ContiguousMemory for Rel<P> {}

impl Rela<u32> {
    pub fn get_offset(&self) -> u32 {
        self.offset
    }
    pub fn get_addend(&self) -> u32 {
        self.addend
    }
    pub fn get_symbol_table_index(&self) -> u32 {
        self.info >> 8
    }
    pub fn get_type(&self) -> u8 {
        self.info as u8
    }
}
impl Rela<u64> {
    pub fn get_offset(&self) -> u64 {
        self.offset
    }
    pub fn get_addend(&self) -> u64 {
        self.addend
    }
    pub fn get_symbol_table_index(&self) -> u32 {
        (self.info >> 32) as u32
    }
    pub fn get_type(&self) -> u32 {
        (self.info & 0xffffffff) as u32
    }
}
impl Rel<u32> {
    pub fn get_offset(&self) -> u32 {
        self.offset
    }
    pub fn get_symbol_table_index(&self) -> u32 {
        self.info >> 8
    }
    pub fn get_type(&self) -> u8 {
        self.info as u8
    }
}
impl Rel<u64> {
    pub fn get_offset(&self) -> u64 {
        self.offset
    }
    pub fn get_symbol_table_index(&self) -> u32 {
        (self.info >> 32) as u32
    }
    pub fn get_type(&self) -> u32 {
        (self.info & 0xffffffff) as u32
    }
}

pub const R_X86_64_NONE: u32 = 0;
pub const R_X86_64_64: u32 = 1;
pub const R_X86_64_PC32: u32 = 2;
pub const R_X86_64_GOT32: u32 = 3;
pub const R_X86_64_PLT32: u32 = 4;
pub const R_X86_64_COPY: u32 = 5;
pub const R_X86_64_GLOB_DAT: u32 = 6;
pub const R_X86_64_JMP_SLOT: u32 = 7;
pub const R_X86_64_RELATIVE: u32 = 8;
pub const R_X86_64_GOTPCREL: u32 = 9;
pub const R_X86_64_32: u32 = 10;
pub const R_X86_64_32S: u32 = 11;
pub const R_X86_64_16: u32 = 12;
pub const R_X86_64_PC16: u32 = 13;
pub const R_X86_64_8: u32 = 14;
pub const R_X86_64_PC8: u32 = 15;
pub const R_X86_64_DTPMOD64: u32 = 16;
pub const R_X86_64_DTPOFF64: u32 = 17;
pub const R_X86_64_TPOFF64: u32 = 18;
pub const R_X86_64_TLSGD: u32 = 19;
pub const R_X86_64_TLSLD: u32 = 20;
pub const R_X86_64_DTPOFF32: u32 = 21;
pub const R_X86_64_GOTTPOFF: u32 = 22;
pub const R_X86_64_TPOFF32: u32 = 23;
pub const R_X86_64_IRELATIVE: u32 = 37;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct NoteHeader {
    name_size: u32,
    desc_size: u32,
    type_: u32,
}

unsafe impl ContiguousMemory for NoteHeader {}

impl NoteHeader {
    pub fn type_(&self) -> u32 {
        self.type_
    }

    pub fn name<'a>(&'a self, input: &'a [u8]) -> &'a str {
        let result = read_str(input);
        // - 1 is due to null terminator
        assert_eq!(result.len(), (self.name_size - 1) as usize);
        result
    }

    pub fn desc<'a>(&'a self, input: &'a [u8]) -> &'a [u8] {
        // Account for padding to the next u32.
        unsafe {
            let offset = (self.name_size + 3) & !0x3;
            let ptr = (&input[0] as *const u8).offset(offset as isize);
            slice::from_raw_parts(ptr, self.desc_size as usize)
        }
    }
}
