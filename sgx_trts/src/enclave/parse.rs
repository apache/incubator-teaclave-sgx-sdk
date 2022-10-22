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

use crate::elf::dynamic::Tag as DynTag;
use crate::elf::header::Type as HeaderType;
use crate::elf::header::{Header, HeaderPt2};
use crate::elf::program::{SegmentData, Type as ProgramType};
use crate::elf::sections::{self, FnArray64, Rela64Array};
use crate::elf::slice::{self, AsSlice};
use crate::elf::symtabl::{Binding, DynEntry64, Entry};
use crate::elf::zero::read_array;
use crate::elf::ElfFile;
use crate::elf::ElfFile64;
use crate::enclave::MmLayout;
use sgx_types::error::{SgxResult, SgxStatus};

macro_rules! try_option {
    ($option:expr) => {
        match $option {
            Some(t) => t,
            None => bail!(SgxStatus::Unexpected),
        }
    };
}

macro_rules! try_result {
    ($result:expr) => {
        match $result {
            Ok(t) => t,
            Err(_) => bail!(SgxStatus::Unexpected),
        }
    };
}

macro_rules! try_result_bool {
    ($result:expr) => {
        match $result {
            Ok(t) => t,
            Err(_) => return false,
        }
    };
}

pub fn relocate() -> SgxResult {
    let elf = try_result!(new_elf64());
    let elf = ElfFile {
        input: elf.input,
        header: Header {
            pt1: elf.header1,
            pt2: HeaderPt2::Header64(elf.header2),
        },
    };
    let phdr = elf.program_iter().search(|phdr| match phdr.get_type() {
        Ok(typ) => typ == ProgramType::Dynamic,
        Err(_) => false,
    });

    let phdr = try_option!(phdr);
    let segment = try_result!(phdr.get_data(&elf));

    let mut sym_offset = 0_u64;
    let mut rel_offset = 0_u64;
    let mut plt_offset = 0_u64;
    let mut rel_size = 0_u64;
    let mut plt_size = 0_u64;

    match segment {
        SegmentData::Dynamic64(dyns) => dyns.as_slice().iter().all(|d| {
            let tag = try_result_bool!(d.get_tag());
            match tag {
                DynTag::Null => return false,
                DynTag::SymTab => {
                    sym_offset = try_result_bool!(d.get_ptr());
                }
                DynTag::Rela => {
                    rel_offset = try_result_bool!(d.get_ptr());
                }
                DynTag::RelaSize => {
                    rel_size = try_result_bool!(d.get_val());
                }
                DynTag::JmpRel => {
                    plt_offset = try_result_bool!(d.get_ptr());
                }
                DynTag::PltRelSize => {
                    plt_size = try_result_bool!(d.get_val());
                }
                _ => (),
            };
            true
        }),
        _ => bail!(SgxStatus::Unexpected),
    };

    unsafe {
        if rel_offset != 0 && rel_size != 0 {
            relocate_elf_rela(&elf, sym_offset, rel_offset, rel_size);
        }
        if plt_offset != 0 && plt_size != 0 {
            relocate_elf_rela(&elf, sym_offset, plt_offset, plt_size);
        }
    }
    Ok(())
}

pub fn init_array<'a>() -> SgxResult<Option<FnArray64<'a>>> {
    let elf = new_elf().map_err(|_| SgxStatus::Unexpected)?;
    let segment = elf
        .program_iter()
        .find(|phdr| phdr.get_type().unwrap_or(ProgramType::Null) == ProgramType::Dynamic)
        .ok_or(SgxStatus::Unexpected)?
        .get_data(&elf)
        .map_err(|_| SgxStatus::Unexpected)?;

    let mut array_offset = 0_u64;
    let mut array_size = 0_u64;
    match segment {
        SegmentData::Dynamic64(dyns) => {
            for d in dyns {
                match d.get_tag().map_err(|_| SgxStatus::Unexpected)? {
                    DynTag::Null => break,
                    DynTag::InitArray => {
                        array_offset = d.get_ptr().map_err(|_| SgxStatus::Unexpected)?
                    }
                    DynTag::InitArraySize => {
                        array_size = d.get_val().map_err(|_| SgxStatus::Unexpected)?
                    }
                    _ => (),
                }
            }
        }
        _ => bail!(SgxStatus::Unexpected),
    }

    if array_offset != 0 && array_size != 0 {
        let raw_data = &elf.input[array_offset as usize..(array_offset + array_size) as usize];
        Ok(Some(FnArray64::new(read_array(raw_data))))
    } else {
        Ok(None)
    }
}

pub fn uninit_array<'a>() -> SgxResult<Option<FnArray64<'a>>> {
    let elf = new_elf().map_err(|_| SgxStatus::Unexpected)?;
    let segment = elf
        .program_iter()
        .find(|phdr| phdr.get_type().unwrap_or(ProgramType::Null) == ProgramType::Dynamic)
        .ok_or(SgxStatus::Unexpected)?
        .get_data(&elf)
        .map_err(|_| SgxStatus::Unexpected)?;

    let mut array_offset = 0_u64;
    let mut array_size = 0_u64;
    match segment {
        SegmentData::Dynamic64(dyns) => {
            for d in dyns {
                match d.get_tag().map_err(|_| SgxStatus::Unexpected)? {
                    DynTag::Null => break,
                    DynTag::FiniArray => {
                        array_offset = d.get_ptr().map_err(|_| SgxStatus::Unexpected)?
                    }
                    DynTag::FiniArraySize => {
                        array_size = d.get_val().map_err(|_| SgxStatus::Unexpected)?
                    }
                    _ => (),
                }
            }
        }
        _ => bail!(SgxStatus::Unexpected),
    }

    if array_offset != 0 && array_size != 0 {
        let raw_data = &elf.input[array_offset as usize..(array_offset + array_size) as usize];
        Ok(Some(FnArray64::new(read_array(raw_data))))
    } else {
        Ok(None)
    }
}

pub fn tls_info() -> SgxResult<Option<(*const u8, usize)>> {
    let elf = new_elf()?;
    let phdr = elf
        .program_iter()
        .find(|phdr| phdr.get_type().unwrap_or(ProgramType::Null) == ProgramType::Tls);

    if let Some(phdr) = phdr {
        Ok(Some((
            (elf.input.as_ptr() as u64 + phdr.virtual_addr()) as *const u8,
            phdr.file_size() as usize,
        )))
    } else {
        Ok(None)
    }
}

pub fn has_text_relo() -> SgxResult<bool> {
    let elf = new_elf().map_err(|_| SgxStatus::Unexpected)?;
    let segment = elf
        .program_iter()
        .find(|phdr| phdr.get_type().unwrap_or(ProgramType::Null) == ProgramType::Dynamic)
        .ok_or(SgxStatus::Unexpected)?
        .get_data(&elf)
        .map_err(|_| SgxStatus::Unexpected)?;

    let mut text_relo = false;
    match segment {
        SegmentData::Dynamic64(dyns) => {
            for d in dyns {
                match d.get_tag().map_err(|_| SgxStatus::Unexpected)? {
                    DynTag::Null => break,
                    DynTag::TextRel => {
                        text_relo = true;
                        break;
                    }
                    _ => (),
                }
            }
        }
        _ => bail!(SgxStatus::Unexpected),
    }
    Ok(text_relo)
}

pub fn new_elf<'a>() -> SgxResult<ElfFile<'a>> {
    let input = unsafe {
        slice::from_raw_parts(MmLayout::image_base() as *const u8, MmLayout::image_size())
    };

    let elf = match ElfFile::new(input) {
        Ok(elf) => elf,
        Err(_) => bail!(SgxStatus::Unexpected),
    };
    if elf.header.pt2.type_().as_type() == HeaderType::SharedObject {
        Ok(elf)
    } else {
        Err(SgxStatus::Unexpected)
    }
}

pub fn new_elf64<'a>() -> SgxResult<ElfFile64<'a>> {
    let input = unsafe {
        slice::from_raw_parts(MmLayout::image_base() as *const u8, MmLayout::image_size())
    };

    let elf = match ElfFile64::new(input) {
        Ok(elf) => elf,
        Err(_) => bail!(SgxStatus::Unexpected),
    };
    if elf.header2.type_.as_type() == HeaderType::SharedObject {
        Ok(elf)
    } else {
        Err(SgxStatus::Unexpected)
    }
}

fn tls_size(elf: &ElfFile) -> Option<u64> {
    let phdr = elf.program_iter().search(|phdr| match phdr.get_type() {
        Ok(typ) => typ == ProgramType::Tls,
        Err(_) => false,
    });
    let phdr = match phdr {
        Some(p) => p,
        None => return None,
    };

    let tls_size = phdr.mem_size();
    let align = phdr.align();
    if align == 0 || align == 1 {
        Some(tls_size)
    } else {
        Some((tls_size + align - 1) & (!(align - 1)))
    }
}

fn get_sym<'a>(symtabl: *const u8, idx: u32) -> Option<&'a DynEntry64> {
    let sym_ptr = symtabl as usize + core::mem::size_of::<DynEntry64>() * idx as usize;
    let sym = unsafe { &*(sym_ptr as *const DynEntry64) };
    let bind = match sym.get_binding() {
        Ok(b) => b,
        Err(_) => return None,
    };
    if bind == Binding::Weak && sym.value() == 0 {
        None
    } else {
        Some(sym)
    }
}

unsafe fn relocate_elf_rela(elf: &ElfFile, sym_offset: u64, rel_offset: u64, rel_size: u64) {
    let sym_table = (elf.input.as_slice().as_ptr() as usize + sym_offset as usize) as *const u8;
    let rel_raw = elf
        .input
        .as_slice()
        .into_slice_unchecked((rel_offset as usize, (rel_offset + rel_size) as usize));
    let rel_array = Rela64Array::new(read_array(rel_raw));

    rel_array.get_array().as_slice().iter().for_each(|rel| {
        let reloc_addr =
            (elf.input.as_slice().as_ptr() as usize + rel.get_offset() as usize) as *mut u64;

        match rel.get_type() {
            sections::R_X86_64_RELATIVE => {
                *reloc_addr = elf.input.as_slice().as_ptr() as u64 + rel.get_addend();
            }
            sections::R_X86_64_GLOB_DAT | sections::R_X86_64_JMP_SLOT | sections::R_X86_64_64 => {
                if let Some(sym) = get_sym(sym_table, rel.get_symbol_table_index()) {
                    *reloc_addr =
                        elf.input.as_slice().as_ptr() as u64 + sym.value() + rel.get_addend();
                }
            }
            sections::R_X86_64_DTPMOD64 => *reloc_addr = 1_u64,
            sections::R_X86_64_DTPOFF64 => {
                if let Some(sym) = get_sym(sym_table, rel.get_symbol_table_index()) {
                    *reloc_addr = sym.value() + rel.get_addend();
                }
            }
            sections::R_X86_64_TPOFF64 => {
                if let Some(sym) = get_sym(sym_table, rel.get_symbol_table_index()) {
                    if let Some(tls_size) = tls_size(elf) {
                        *reloc_addr = sym.value() + rel.get_addend() - tls_size;
                    }
                }
            }
            _ => (),
        }
    });
}
