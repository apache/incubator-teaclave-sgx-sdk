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

use core::arch::asm;
use core::ffi::c_void;

#[repr(C)]
pub union Dtv {
    counter: usize,
    pointer: Pointer,
}

#[repr(C)]
struct Pointer {
    value: usize,
    is_static: i32,
}

#[repr(C)]
struct TcbHead {
    tcb: *mut c_void,
    dtv: *mut Dtv,
    self_addr: *mut TcbHead,
}

impl Dtv {
    #[link_section = ".nipx"]
    #[inline]
    pub fn get<'a>() -> &'a mut Dtv {
        unsafe {
            let dtv: *mut Dtv;
            asm!("mov {}, fs:0x08", out(reg) dtv);
            &mut *dtv
        }
    }

    #[link_section = ".nipx"]
    #[inline]
    pub fn read_value(&self) -> usize {
        unsafe { self.pointer.value }
    }

    #[link_section = ".nipx"]
    #[inline]
    pub fn set_value(&mut self, value: usize) {
        self.pointer.value = value;
    }
}

#[link_section = ".nipx"]
#[inline]
pub fn get_fs_gs_0() -> usize {
    unsafe {
        let orig: usize;
        asm!("mov {}, fs:0x0", out(reg) orig);
        orig
    }
}

#[link_section = ".nipx"]
#[inline]
pub fn set_fs_gs_0(value: usize) {
    unsafe {
        asm!("mov fs:0x0, {}", in(reg) value);
    }
}
