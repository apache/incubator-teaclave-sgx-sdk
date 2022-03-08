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
use core::mem;
use sgx_types::marker::ContiguousMemory;

#[derive(Debug)]
#[repr(C)]
struct Entry32_ {
    name: u32,
    value: u32,
    size: u32,
    info: u8,
    other: Visibility_,
    shndx: u16,
}

#[derive(Debug)]
#[repr(C)]
struct Entry64_ {
    name: u32,
    info: u8,
    other: Visibility_,
    shndx: u16,
    value: u64,
    size: u64,
}

unsafe impl ContiguousMemory for Entry32_ {}
unsafe impl ContiguousMemory for Entry64_ {}

#[derive(Debug)]
#[repr(C)]
pub struct Entry32(Entry32_);

#[derive(Debug)]
#[repr(C)]
pub struct Entry64(Entry64_);

unsafe impl ContiguousMemory for Entry32 {}
unsafe impl ContiguousMemory for Entry64 {}

#[derive(Debug)]
#[repr(C)]
pub struct DynEntry32(Entry32_);

#[derive(Debug)]
#[repr(C)]
pub struct DynEntry64(Entry64_);

unsafe impl ContiguousMemory for DynEntry32 {}
unsafe impl ContiguousMemory for DynEntry64 {}

pub trait Entry {
    fn name(&self) -> u32;
    fn info(&self) -> u8;
    fn other(&self) -> Visibility_;
    fn shndx(&self) -> u16;
    fn value(&self) -> u64;
    fn size(&self) -> u64;

    fn get_other(&self) -> Visibility {
        self.other().as_visibility()
    }

    fn get_binding(&self) -> Result<Binding, &'static str> {
        Binding_(self.info() >> 4).as_binding()
    }

    fn get_type(&self) -> Result<Type, &'static str> {
        Type_(self.info() & 0xf).as_type()
    }
}

impl fmt::Display for dyn Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Symbol table entry:")?;
        writeln!(f, "    name:             {:?}", self.name())?;
        writeln!(f, "    binding:          {:?}", self.get_binding())?;
        writeln!(f, "    type:             {:?}", self.get_type())?;
        writeln!(f, "    other:            {:?}", self.get_other())?;
        writeln!(f, "    shndx:            {:?}", self.shndx())?;
        writeln!(f, "    value:            {:?}", self.value())?;
        writeln!(f, "    size:             {:?}", self.size())?;
        Ok(())
    }
}

macro_rules! impl_entry {
    ($name: ident) => {
        impl Entry for $name {
            fn name(&self) -> u32 {
                self.0.name
            }
            fn info(&self) -> u8 {
                self.0.info
            }
            fn other(&self) -> Visibility_ {
                self.0.other
            }
            fn shndx(&self) -> u16 {
                self.0.shndx
            }
            fn value(&self) -> u64 {
                self.0.value as u64
            }
            fn size(&self) -> u64 {
                self.0.size as u64
            }
        }
    };
}
impl_entry!(Entry32);
impl_entry!(Entry64);
impl_entry!(DynEntry32);
impl_entry!(DynEntry64);

#[derive(Clone, Copy, Debug)]
pub struct Visibility_(u8);

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Visibility {
    Default = 0,
    Internal = 1,
    Hidden = 2,
    Protected = 3,
}

impl Visibility_ {
    pub fn as_visibility(self) -> Visibility {
        unsafe { mem::transmute(self.0 & 0x3) }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Binding_(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Binding {
    Local,
    Global,
    Weak,
    OsSpecific(u8),
    ProcessorSpecific(u8),
}

impl Binding_ {
    #[allow(clippy::manual_range_contains)]
    pub fn as_binding(self) -> Result<Binding, &'static str> {
        match self.0 {
            0 => Ok(Binding::Local),
            1 => Ok(Binding::Global),
            2 => Ok(Binding::Weak),
            b if b >= 10 && b <= 12 => Ok(Binding::OsSpecific(b)),
            b if b >= 13 && b <= 15 => Ok(Binding::ProcessorSpecific(b)),
            _ => Err("Invalid value for binding"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Type_(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    No,
    Object,
    Func,
    Section,
    File,
    Common,
    Tls,
    OsSpecific(u8),
    ProcessorSpecific(u8),
}

impl Type_ {
    #[allow(clippy::manual_range_contains)]
    pub fn as_type(self) -> Result<Type, &'static str> {
        match self.0 {
            0 => Ok(Type::No),
            1 => Ok(Type::Object),
            2 => Ok(Type::Func),
            3 => Ok(Type::Section),
            4 => Ok(Type::File),
            5 => Ok(Type::Common),
            6 => Ok(Type::Tls),
            b if b >= 10 && b <= 12 => Ok(Type::OsSpecific(b)),
            b if b >= 13 && b <= 15 => Ok(Type::ProcessorSpecific(b)),
            _ => Err("Invalid value for type"),
        }
    }
}
