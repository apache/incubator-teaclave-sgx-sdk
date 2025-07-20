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

use core::fmt;
use sgx_types::marker::ContiguousMemory;

#[derive(Debug)]
#[repr(C)]
pub struct Dynamic<P>
where
    Tag_<P>: fmt::Debug,
{
    tag: Tag_<P>,
    un: P,
}

unsafe impl<P> ContiguousMemory for Dynamic<P> where Tag_<P>: fmt::Debug {}

#[derive(Clone, Copy)]
pub struct Tag_<P>(P);

#[derive(Debug, Eq, PartialEq)]
pub enum Tag<P> {
    Null,
    Needed,
    PltRelSize,
    Pltgot,
    Hash,
    StrTab,
    SymTab,
    Rela,
    RelaSize,
    RelaEnt,
    StrSize,
    SymEnt,
    Init,
    Fini,
    SoName,
    RPath,
    Symbolic,
    Rel,
    RelSize,
    RelEnt,
    PltRel,
    Debug,
    TextRel,
    JmpRel,
    BindNow,
    InitArray,
    FiniArray,
    InitArraySize,
    FiniArraySize,
    RunPath,
    Flags,
    PreInitArray,
    PreInitArraySize,
    SymTabShIndex,
    Flags1,
    OsSpecific(P),
    ProcessorSpecific(P),
}

macro_rules! impls {
    ($p: ident) => {
        impl Dynamic<$p> {
            pub fn get_tag(&self) -> Result<Tag<$p>, &'static str> {
                self.tag.as_tag()
            }

            pub fn get_val(&self) -> Result<$p, &'static str> {
                match self.get_tag() {
                    Ok(tag) => match tag {
                        Tag::Needed
                        | Tag::PltRelSize
                        | Tag::RelaSize
                        | Tag::RelaEnt
                        | Tag::StrSize
                        | Tag::SymEnt
                        | Tag::SoName
                        | Tag::RPath
                        | Tag::RelSize
                        | Tag::RelEnt
                        | Tag::PltRel
                        | Tag::InitArraySize
                        | Tag::FiniArraySize
                        | Tag::RunPath
                        | Tag::Flags
                        | Tag::PreInitArraySize
                        | Tag::Flags1
                        | Tag::OsSpecific(_)
                        | Tag::ProcessorSpecific(_) => Ok(self.un),
                        _ => Err("Invalid value"),
                    },
                    Err(e) => Err(e),
                }
            }

            pub fn get_ptr(&self) -> Result<$p, &'static str> {
                match self.get_tag() {
                    Ok(tag) => match tag {
                        Tag::Pltgot
                        | Tag::Hash
                        | Tag::StrTab
                        | Tag::SymTab
                        | Tag::Rela
                        | Tag::Init
                        | Tag::Fini
                        | Tag::Rel
                        | Tag::Debug
                        | Tag::JmpRel
                        | Tag::InitArray
                        | Tag::FiniArray
                        | Tag::PreInitArray
                        | Tag::SymTabShIndex
                        | Tag::OsSpecific(_)
                        | Tag::ProcessorSpecific(_) => Ok(self.un),
                        _ => Err("Invalid ptr"),
                    },
                    Err(e) => Err(e),
                }
            }
        }

        impl Tag_<$p> {
            #[allow(clippy::manual_range_contains)]
            fn as_tag(self) -> Result<Tag<$p>, &'static str> {
                match self.0 {
                    0 => Ok(Tag::Null),
                    1 => Ok(Tag::Needed),
                    2 => Ok(Tag::PltRelSize),
                    3 => Ok(Tag::Pltgot),
                    4 => Ok(Tag::Hash),
                    5 => Ok(Tag::StrTab),
                    6 => Ok(Tag::SymTab),
                    7 => Ok(Tag::Rela),
                    8 => Ok(Tag::RelaSize),
                    9 => Ok(Tag::RelaEnt),
                    10 => Ok(Tag::StrSize),
                    11 => Ok(Tag::SymEnt),
                    12 => Ok(Tag::Init),
                    13 => Ok(Tag::Fini),
                    14 => Ok(Tag::SoName),
                    15 => Ok(Tag::RPath),
                    16 => Ok(Tag::Symbolic),
                    17 => Ok(Tag::Rel),
                    18 => Ok(Tag::RelSize),
                    19 => Ok(Tag::RelEnt),
                    20 => Ok(Tag::PltRel),
                    21 => Ok(Tag::Debug),
                    22 => Ok(Tag::TextRel),
                    23 => Ok(Tag::JmpRel),
                    24 => Ok(Tag::BindNow),
                    25 => Ok(Tag::InitArray),
                    26 => Ok(Tag::FiniArray),
                    27 => Ok(Tag::InitArraySize),
                    28 => Ok(Tag::FiniArraySize),
                    29 => Ok(Tag::RunPath),
                    30 => Ok(Tag::Flags),
                    32 => Ok(Tag::PreInitArray),
                    33 => Ok(Tag::PreInitArraySize),
                    34 => Ok(Tag::SymTabShIndex),
                    0x6ffffffb => Ok(Tag::Flags1),
                    t if t >= 0x6000000D && t <= 0x6fffffff => Ok(Tag::OsSpecific(t)),
                    t if t >= 0x70000000 && t <= 0x7fffffff => Ok(Tag::ProcessorSpecific(t)),
                    _ => Err("Invalid tag value"),
                }
            }
        }

        impl fmt::Debug for Tag_<$p> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.as_tag().fmt(f)
            }
        }
    };
}

impls!(u32);
impls!(u64);

/* Flag values used in the DT_FLAGS_1 .dynamic entry.  */
pub const FLAG_NOW: u64 = 0x00000001;
pub const FLAG_GLOBAL: u64 = 0x00000002;
pub const FLAG_GROUP: u64 = 0x00000004;
pub const FLAG_NODELETE: u64 = 0x00000008;
pub const FLAG_LOADFLTR: u64 = 0x00000010;
pub const FLAG_INITFIRST: u64 = 0x00000020;
pub const FLAG_NOOPEN: u64 = 0x00000040;
pub const FLAG_ORIGIN: u64 = 0x00000080;
pub const FLAG_DIRECT: u64 = 0x00000100;
pub const FLAG_TRANS: u64 = 0x00000200;
pub const FLAG_INTERPOSE: u64 = 0x00000400;
pub const FLAG_NODEFLIB: u64 = 0x00000800;
pub const FLAG_NODUMP: u64 = 0x00001000;
pub const FLAG_CONFALT: u64 = 0x00002000;
pub const FLAG_ENDFILTEE: u64 = 0x00004000;
pub const FLAG_DISPRELDNE: u64 = 0x00008000;
pub const FLAG_DISPRELPND: u64 = 0x00010000;
pub const FLAG_NODIRECT: u64 = 0x00020000;
pub const FLAG_IGNMULDEF: u64 = 0x00040000;
pub const FLAG_NOKSYMS: u64 = 0x00080000;
pub const FLAG_NOHDR: u64 = 0x00100000;
pub const FLAG_EDITED: u64 = 0x00200000;
pub const FLAG_NORELOC: u64 = 0x00400000;
pub const FLAG_SYMINTPOSE: u64 = 0x00800000;
pub const FLAG_GLOBAUDIT: u64 = 0x01000000;
pub const FLAG_SINGLETON: u64 = 0x02000000;
pub const FLAG_STUB: u64 = 0x04000000;
pub const FLAG_PIE: u64 = 0x08000000;
