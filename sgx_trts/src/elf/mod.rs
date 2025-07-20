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

use crate::elf::header::{Header, Header64Pt2, HeaderPt1, HeaderPt2};
use crate::elf::program::{ProgramHeader, ProgramIter};

macro_rules! check {
    ($e:expr) => {
        if !$e {
            return Err("");
        }
    };
    ($e:expr, $msg: expr) => {
        if !$e {
            return Err($msg);
        }
    };
}

#[derive(Debug)]
pub struct ElfFile64<'a> {
    pub input: &'a [u8],
    pub header1: &'a HeaderPt1,
    pub header2: &'a Header64Pt2,
}

impl<'a> ElfFile64<'a> {
    pub fn new(input: &'a [u8]) -> Result<ElfFile64<'a>, &'static str> {
        let header = match header::parse_header(input) {
            Ok(h) => h,
            Err(e) => return Err(e),
        };
        let pt2 = match header.pt2 {
            HeaderPt2::Header32(_) => return Err("Not support 32-bit ELF"),
            HeaderPt2::Header64(pt2) => pt2,
        };
        Ok(ElfFile64 {
            input,
            header1: header.pt1,
            header2: pt2,
        })
    }
}

#[derive(Debug)]
pub struct ElfFile<'a> {
    pub input: &'a [u8],
    pub header: Header<'a>,
}

impl<'a> ElfFile<'a> {
    pub fn new(input: &'a [u8]) -> Result<ElfFile<'a>, &'static str> {
        let header = match header::parse_header(input) {
            Ok(h) => h,
            Err(e) => return Err(e),
        };
        Ok(ElfFile { input, header })
    }

    pub fn program_header(&self, index: u16) -> Result<ProgramHeader<'a>, &'static str> {
        program::parse_program_header(self.input, self.header, index)
    }

    pub fn program_iter<'b>(&'b self) -> ProgramIter<'b, 'a> {
        ProgramIter {
            file: self,
            next_index: 0,
        }
    }
}

pub mod control_flow;
pub mod dynamic;
pub mod header;
pub mod program;
pub mod sections;
pub mod slice;
pub mod symtabl;
pub mod zero;
