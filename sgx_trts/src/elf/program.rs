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
// under the License.

use crate::elf::control_flow::ControlFlow;
use crate::elf::dynamic::Dynamic;
use crate::elf::header::{Class, Header};
use crate::elf::sections::NoteHeader;
use crate::elf::slice::AsSlice;
use crate::elf::zero::{read, read_array};
use crate::elf::ElfFile;
use crate::error::abort;
use core::fmt;
use core::mem;
use sgx_types::marker::ContiguousMemory;

pub fn parse_program_header<'a>(
    input: &'a [u8],
    header: Header<'a>,
    index: u16,
) -> Result<ProgramHeader<'a>, &'static str> {
    let pt2 = &header.pt2;
    if !(index < pt2.ph_count() && pt2.ph_offset() > 0 && pt2.ph_entry_size() > 0) {
        return Err("There are no program headers in this file");
    }

    let start = pt2.ph_offset() as usize + index as usize * pt2.ph_entry_size() as usize;
    let end = start + pt2.ph_entry_size() as usize;

    match header.pt1.class() {
        Class::ThirtyTwo => Ok(ProgramHeader::Ph32(read(unsafe {
            input.as_slice().into_slice_unchecked((start, end))
        }))),
        Class::SixtyFour => Ok(ProgramHeader::Ph64(read(unsafe {
            input.as_slice().into_slice_unchecked((start, end))
        }))),
        Class::None | Class::Other(_) => abort(),
    }
}

#[derive(Debug)]
pub struct ProgramIter<'b, 'a: 'b> {
    pub file: &'b ElfFile<'a>,
    pub next_index: u16,
}

impl<'b, 'a> ProgramIter<'b, 'a> {
    #[inline]
    #[allow(clippy::while_let_on_iterator)]
    pub fn search<P>(&mut self, predicate: P) -> Option<ProgramHeader<'a>>
    where
        Self: Sized,
        P: FnMut(&ProgramHeader<'a>) -> bool,
    {
        #[inline]
        fn check<T>(mut predicate: impl FnMut(&T) -> bool) -> impl FnMut(T) -> ControlFlow<(), T> {
            move |x| {
                if predicate(&x) {
                    ControlFlow::Break(x)
                } else {
                    ControlFlow::CONTINUE
                }
            }
        }

        let mut f = check(predicate);
        while let Some(x) = self.next_item() {
            if let ControlFlow::Break(b) = f(x) {
                return Some(b);
            }
        }
        None
    }

    fn next_item(&mut self) -> Option<ProgramHeader<'a>> {
        let count = self.file.header.pt2.ph_count();
        if self.next_index >= count {
            return None;
        }

        let result = self.file.program_header(self.next_index);
        self.next_index += 1;
        result.ok()
    }
}

impl<'b, 'a> Iterator for ProgramIter<'b, 'a> {
    type Item = ProgramHeader<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_item()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ProgramHeader<'a> {
    Ph32(&'a ProgramHeader32),
    Ph64(&'a ProgramHeader64),
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct ProgramHeader32 {
    pub type_: Type_,
    pub offset: u32,
    pub virtual_addr: u32,
    pub physical_addr: u32,
    pub file_size: u32,
    pub mem_size: u32,
    pub flags: Flags,
    pub align: u32,
}

unsafe impl ContiguousMemory for ProgramHeader32 {}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct ProgramHeader64 {
    pub type_: Type_,
    pub flags: Flags,
    pub offset: u64,
    pub virtual_addr: u64,
    pub physical_addr: u64,
    pub file_size: u64,
    pub mem_size: u64,
    pub align: u64,
}

unsafe impl ContiguousMemory for ProgramHeader64 {}

macro_rules! getter {
    ($name: ident, $typ: ident) => {
        pub fn $name(&self) -> $typ {
            match *self {
                ProgramHeader::Ph32(h) => h.$name as $typ,
                ProgramHeader::Ph64(h) => h.$name as $typ,
            }
        }
    };
}

impl<'a> ProgramHeader<'a> {
    pub fn get_type(&self) -> Result<Type, &'static str> {
        match *self {
            ProgramHeader::Ph32(ph) => ph.get_type(),
            ProgramHeader::Ph64(ph) => ph.get_type(),
        }
    }

    pub fn get_data(&self, elf_file: &ElfFile<'a>) -> Result<SegmentData<'a>, &'static str> {
        match *self {
            ProgramHeader::Ph32(ph) => ph.get_data(elf_file),
            ProgramHeader::Ph64(ph) => ph.get_data(elf_file),
        }
    }

    getter!(align, u64);
    getter!(file_size, u64);
    getter!(mem_size, u64);
    getter!(offset, u64);
    getter!(physical_addr, u64);
    getter!(virtual_addr, u64);
    getter!(flags, Flags);
}

impl<'a> fmt::Display for ProgramHeader<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProgramHeader::Ph32(ph) => ph.fmt(f),
            ProgramHeader::Ph64(ph) => ph.fmt(f),
        }
    }
}
macro_rules! ph_impl {
    ($ph: ident) => {
        impl $ph {
            pub fn get_type(&self) -> Result<Type, &'static str> {
                self.type_.as_type()
            }

            pub fn get_data<'a>(
                &self,
                elf_file: &ElfFile<'a>,
            ) -> Result<SegmentData<'a>, &'static str> {
                match self.get_type() {
                    Ok(typ) => match typ {
                        Type::Null => Ok(SegmentData::Empty),
                        Type::Load
                        | Type::Interp
                        | Type::ShLib
                        | Type::Phdr
                        | Type::Tls
                        | Type::GnuRelro
                        | Type::OsSpecific(_)
                        | Type::ProcessorSpecific(_) => {
                            Ok(SegmentData::Undefined(self.raw_data(elf_file)))
                        }
                        Type::Dynamic => {
                            let data = self.raw_data(elf_file);
                            match elf_file.header.pt1.class() {
                                Class::ThirtyTwo => Ok(SegmentData::Dynamic32(read_array(data))),
                                Class::SixtyFour => Ok(SegmentData::Dynamic64(read_array(data))),
                                Class::None | Class::Other(_) => abort(),
                            }
                        }
                        Type::Note => {
                            let data = self.raw_data(elf_file);
                            match elf_file.header.pt1.class() {
                                Class::ThirtyTwo => abort(),
                                Class::SixtyFour => {
                                    let header: &'a NoteHeader = read(&data[0..12]);
                                    let index = &data[12..];
                                    Ok(SegmentData::Note64(header, index))
                                }
                                Class::None | Class::Other(_) => abort(),
                            }
                        }
                    },
                    Err(e) => Err(e),
                }
            }

            pub fn raw_data<'a>(&self, elf_file: &ElfFile<'a>) -> &'a [u8] {
                assert!(match self.get_type() {
                    Ok(_) => self.type_.0 != 0,
                    Err(_) => false,
                });
                unsafe {
                    elf_file.input.as_slice().into_slice_unchecked((
                        self.physical_addr as usize,
                        (self.physical_addr + self.file_size) as usize,
                    ))
                }
            }
        }

        impl fmt::Display for $ph {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                writeln!(f, "Program header:")?;
                writeln!(f, "    type:             {:?}", self.get_type())?;
                writeln!(f, "    flags:            {}", self.flags)?;
                writeln!(f, "    offset:           {:#x}", self.offset)?;
                writeln!(f, "    virtual address:  {:#x}", self.virtual_addr)?;
                writeln!(f, "    physical address: {:#x}", self.physical_addr)?;
                writeln!(f, "    file size:        {:#x}", self.file_size)?;
                writeln!(f, "    memory size:      {:#x}", self.mem_size)?;
                writeln!(f, "    align:            {:#x}", self.align)?;
                Ok(())
            }
        }
    };
}

ph_impl!(ProgramHeader32);
ph_impl!(ProgramHeader64);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Flags(pub u32);

impl Flags {
    pub fn is_execute(&self) -> bool {
        self.0 & FLAG_X == FLAG_X
    }

    pub fn is_write(&self) -> bool {
        self.0 & FLAG_W == FLAG_W
    }

    pub fn is_read(&self) -> bool {
        self.0 & FLAG_R == FLAG_R
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.0 & FLAG_X == FLAG_X { 'X' } else { ' ' },
            if self.0 & FLAG_W == FLAG_W { 'W' } else { ' ' },
            if self.0 & FLAG_R == FLAG_R { 'R' } else { ' ' }
        )
    }
}

impl fmt::LowerHex for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = self.0;
        write!(f, "{:#x}", val) // delegate to i32's implementation
    }
}

#[derive(Clone, Copy, Default)]
pub struct Type_(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    Null,
    Load,
    Dynamic,
    Interp,
    Note,
    ShLib,
    Phdr,
    Tls,
    GnuRelro,
    OsSpecific(u32),
    ProcessorSpecific(u32),
}

impl Type_ {
    #[allow(clippy::manual_range_contains)]
    fn as_type(&self) -> Result<Type, &'static str> {
        match self.0 {
            0 => Ok(Type::Null),
            1 => Ok(Type::Load),
            2 => Ok(Type::Dynamic),
            3 => Ok(Type::Interp),
            4 => Ok(Type::Note),
            5 => Ok(Type::ShLib),
            6 => Ok(Type::Phdr),
            7 => Ok(Type::Tls),
            TYPE_GNU_RELRO => Ok(Type::GnuRelro),
            t if t >= TYPE_LOOS && t <= TYPE_HIOS => Ok(Type::OsSpecific(t)),
            t if t >= TYPE_LOPROC && t <= TYPE_HIPROC => Ok(Type::ProcessorSpecific(t)),
            _ => Err("Invalid type"),
        }
    }
}

impl fmt::Debug for Type_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_type().fmt(f)
    }
}

#[derive(Debug)]
pub enum SegmentData<'a> {
    Empty,
    Undefined(&'a [u8]),
    Dynamic32(&'a [Dynamic<u32>]),
    Dynamic64(&'a [Dynamic<u64>]),
    Note64(&'a NoteHeader, &'a [u8]),
}

pub const TYPE_LOOS: u32 = 0x60000000;
pub const TYPE_HIOS: u32 = 0x6fffffff;
pub const TYPE_LOPROC: u32 = 0x70000000;
pub const TYPE_HIPROC: u32 = 0x7fffffff;
pub const TYPE_GNU_RELRO: u32 = TYPE_LOOS + 0x474e552;

pub const FLAG_X: u32 = 0x1;
pub const FLAG_W: u32 = 0x2;
pub const FLAG_R: u32 = 0x4;
pub const FLAG_MASKOS: u32 = 0x0ff00000;
pub const FLAG_MASKPROC: u32 = 0xf0000000;

pub fn sanity_check<'a>(ph: ProgramHeader<'a>, elf_file: &ElfFile<'a>) -> Result<(), &'static str> {
    let header = elf_file.header;
    match ph {
        ProgramHeader::Ph32(ph) => {
            check!(
                mem::size_of_val(ph) == header.pt2.ph_entry_size() as usize,
                "program header size mismatch"
            );
        }
        ProgramHeader::Ph64(ph) => {
            check!(
                mem::size_of_val(ph) == header.pt2.ph_entry_size() as usize,
                "program header size mismatch"
            );
        }
    }
    Ok(())
}
