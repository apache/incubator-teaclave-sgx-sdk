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

#![allow(clippy::enum_variant_names)]

use crate::edmm::{self, PageType};
use crate::tcs::tc;
use crate::version::*;
use crate::xsave;
use core::convert::From;
use core::fmt;
use core::mem;
use core::slice;
use sgx_types::types::{Attributes, ConfigId, Measurement, MiscSelect};

pub const SE_PAGE_SHIFT: usize = 12;
pub const SE_PAGE_SIZE: usize = 0x1000;
pub const SE_GUARD_PAGE_SHIFT: usize = 16;
pub const SE_GUARD_PAGE_SIZE: usize = 0x10000;
pub const RED_ZONE_SIZE: usize = 128;
pub const RSVD_SIZE_OF_MITIGATION_STACK_AREA: usize = 15 * 8;

macro_rules! is_page_aligned {
    ($num:expr) => {
        $num & (crate::arch::SE_PAGE_SIZE - 1) == 0
    };
}

macro_rules! round_to_page {
    ($num:expr) => {
        ($num + crate::arch::SE_PAGE_SIZE - 1) & (!(crate::arch::SE_PAGE_SIZE - 1))
    };
}

macro_rules! trim_to_page {
    ($num:expr) => {
        $num & (!(crate::arch::SE_PAGE_SIZE - 1))
    };
}

#[link_section = ".niprod"]
#[no_mangle]
pub static mut g_global_data: GlobalData = GlobalData {
    version: VERSION_UINT,
    data: [0_u8; GLOBAL_DATA_SIZE],
};

const GLOBAL_DATA_SIZE: usize = mem::size_of::<Global>() - mem::size_of::<usize>();

#[repr(C)]
pub struct GlobalData {
    pub version: usize,
    pub data: [u8; GLOBAL_DATA_SIZE],
}

macro_rules! impl_align {
    ($($t:ty;)*) => {$(
        impl<T: ::core::marker::Copy> ::core::marker::Copy for $t {}

        impl<T: ::core::clone::Clone> ::core::clone::Clone for $t {
           fn clone(&self) -> $t {
            Self(self.0.clone())
           }
        }

        impl<T: ::core::default::Default> ::core::default::Default for $t {
            fn default() -> $t {
                Self(T::default())
            }
        }

        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for $t {
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                self.0.fmt(fmt)
            }
        }
    )*}
}
/// Wrapper struct to force 16-byte alignment.
#[repr(align(16))]
pub struct Align16<T>(pub T);

/// Wrapper struct to force 32-byte alignment.
#[repr(align(32))]
pub struct Align32<T>(pub T);

/// Wrapper struct to force 16-byte alignment.
#[repr(align(64))]
pub struct Align64<T>(pub T);

/// Wrapper struct to force 128-byte alignment.
#[repr(align(128))]
pub struct Align128<T>(pub T);

/// Wrapper struct to force 256-byte alignment.
#[repr(align(256))]
pub struct Align256<T>(pub T);

/// Wrapper struct to force 512-byte alignment.
#[repr(align(512))]
pub struct Align512<T>(pub T);

impl_align! {
    Align16<T>;
    Align32<T>;
    Align64<T>;
    Align128<T>;
    Align256<T>;
    Align512<T>;
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Encls {
        ECreate =  0,
        EAdd    =  1,
        EInit   =  2,
        ERemove =  3,
        EDbgrd  =  4,
        EDbgwr  =  5,
        EExtend =  6,
        ELdb    =  7,
        ELdu    =  8,
        EBlock  =  9,
        EPa     = 10,
        EWb     = 11,
        ETrack  = 12,
        EAug    = 13,
        EModpr  = 14,
        EModt   = 15,
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Enclu {
        EReport     = 0,
        EGetkey     = 1,
        EEnter      = 2,
        EResume     = 3,
        EExit       = 4,
        EAccept     = 5,
        EModpe      = 6,
        EAcceptcopy = 7,
        EVerifyReport2 = 8,
    }
}

#[cfg(not(feature = "hyper"))]
pub const SECS_RESERVED1_BYTES: usize = 24;
#[cfg(feature = "hyper")]
pub const SECS_RESERVED1_BYTES: usize = 16;
pub const SECS_RESERVED2_BYTES: usize = 32;
pub const SECS_RESERVED3_BYTES: usize = 32;
pub const SECS_RESERVED4_BYTES: usize = 3834;

impl_copy_clone! {
    #[repr(C, align(4096))]
    pub struct Secs {
        pub size: u64,
        pub base: u64,
        pub ssa_frame_size: u32,
        pub misc_select: MiscSelect,
        pub reserved1: [u8; SECS_RESERVED1_BYTES],
        #[cfg(feature = "hyper")]
        pub msbuf_size: u64,
        pub attributes: Attributes,
        pub mr_enclave: Measurement,
        pub reserved2: [u8; SECS_RESERVED2_BYTES],
        pub mr_signer: Measurement,
        pub reserved3: [u8; SECS_RESERVED3_BYTES],
        pub config_id: ConfigId,
        pub isv_prod_id: u16,
        pub isv_svn: u16,
        pub config_svn: u16,
        pub reserved4: [u8; SECS_RESERVED4_BYTES],
    }
}

impl_struct_default! {
    Secs;
}

impl_struct_ContiguousMemory! {
    Secs;
}

impl Secs {
    pub const ALIGN_SIZE: usize = mem::size_of::<Secs>();
}

impl AsRef<[u8; Secs::ALIGN_SIZE]> for Secs {
    fn as_ref(&self) -> &[u8; Secs::ALIGN_SIZE] {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl fmt::Debug for Secs {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Secs")
            .field("size", &self.size)
            .field("base", &self.base)
            .field("ssa_frame_size", &self.ssa_frame_size)
            .field("misc_select", &self.misc_select)
            .field("attributes", &self.attributes)
            .field("mr_enclave", &self.mr_enclave)
            .field("mr_signer", &self.mr_signer)
            .field("config_id", &self.config_id)
            .field("isv_prod_id", &self.isv_prod_id)
            .field("isv_svn", &self.isv_svn)
            .field("config_svn", &self.config_svn)
            .finish()
    }
}

pub const TCS_RESERVED_BYTES: usize = 4024;

impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct TcsFlags: u64 {
        const DBGOPTIN     = 0x0001;
        const AEXNOTIFY    = 0x0002;
    }
}

impl_copy_clone! {
    #[repr(C, align(4096))]
    pub struct Tcs {
        pub reserved0: u64,
        pub flags: TcsFlags,
        pub ossa: u64,
        pub cssa: u32,
        pub nssa: u32,
        pub oentry: u64,
        pub reserved1: u64,
        pub ofsbase: u64,
        pub ogsbase: u64,
        pub ofslimit: u32,
        pub ogslimit: u32,
        pub reserved: [u8; TCS_RESERVED_BYTES],
    }
}

impl_struct_default! {
    Tcs;
}

impl_struct_ContiguousMemory! {
    Tcs;
}

impl Tcs {
    pub const ALIGN_SIZE: usize = mem::size_of::<Tcs>();

    #[inline]
    pub fn from_td(tds: &Tds) -> &Tcs {
        unsafe {
            let raw = (tds.stack_base + tc::STATIC_STACK_SIZE + SE_GUARD_PAGE_SIZE) as *const Tcs;
            &*raw
        }
    }

    #[inline]
    pub unsafe fn from_raw<'a>(tcs: *const Tcs) -> &'a Tcs {
        &*tcs
    }

    #[inline]
    pub unsafe fn from_raw_mut<'a>(tcs: *mut Tcs) -> &'a mut Tcs {
        &mut *tcs
    }
}

impl AsRef<[u8; Tcs::ALIGN_SIZE]> for Tcs {
    fn as_ref(&self) -> &[u8; Tcs::ALIGN_SIZE] {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl fmt::Debug for Tcs {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Tcs")
            .field("flags", &self.flags)
            .field("ossa", &self.ossa)
            .field("cssa", &self.cssa)
            .field("nssa", &self.nssa)
            .field("oentry", &self.oentry)
            .field("ofsbase", &self.ofsbase)
            .field("ogsbase", &self.ogsbase)
            .field("ofslimit", &self.ofslimit)
            .field("ogslimit", &self.ogslimit)
            .finish()
    }
}

impl_enum! {
    #[repr(usize)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum TcsPolicy {
        Bind  = 0,
        Unbind = 1,
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Tds {
    pub self_addr: usize,
    pub last_sp: usize,
    pub stack_base: usize,
    pub stack_limit: usize,
    pub first_ssa_gpr: usize,
    pub stack_guard: usize,
    pub flags: usize,
    pub xsave_size: usize,
    pub last_error: usize,
    pub aex_mitigation_list: usize,
    pub aex_notify_flag: usize,
    pub first_ssa_xsave: usize,
    pub m_next: usize,
    pub tls_addr: usize,
    pub tls_array: usize,
    pub exception_flag: isize,
    pub cxx_thread_info: [usize; 6],
    pub stack_commit: usize,
    pub aex_notify_entropy_cache: u32,
    pub aex_notify_entropy_remaining: i32,
    #[cfg(feature = "hyper")]
    pub index: usize,
}

impl Tds {
    #[inline]
    pub unsafe fn from_raw<'a>(tds: *const Tds) -> &'a Tds {
        &*tds
    }

    #[inline]
    pub unsafe fn from_raw_mut<'a>(tds: *mut Tds) -> &'a mut Tds {
        &mut *tds
    }

    #[inline]
    pub fn from_tcs<'a>(tcs: &Tcs) -> &'a Tds {
        unsafe {
            let raw =
                (tcs as *const _ as usize + Global::get().td_template.self_addr) as *const Tds;
            &*raw
        }
    }

    #[inline]
    pub fn from_tcs_mut<'a>(tcs: &Tcs) -> &'a mut Tds {
        unsafe {
            let raw = (tcs as *const _ as usize + Global::get().td_template.self_addr) as *mut Tds;
            &mut *raw
        }
    }

    #[inline]
    pub fn ssa_gpr_mut(&mut self) -> &mut SsaGpr {
        unsafe { &mut *(self.first_ssa_gpr as *mut SsaGpr) }
    }

    #[inline]
    pub fn ssa_gpr(&self) -> &SsaGpr {
        unsafe { &*(self.first_ssa_gpr as *const SsaGpr) }
    }

    #[inline]
    pub fn xsave_area(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.first_ssa_xsave as *const u8, xsave::xsave_size()) }
    }
}

pub const TCS_POLICY_BIND: usize = 0x0000_0000; //If set, the TCS is bound to the application thread
pub const TCS_POLICY_UNBIND: usize = 0x0000_0001;

pub const LAYOUT_ENTRY_NUM: usize = 43;
pub const TCS_TEMPLATE_SIZE: usize = 72;

#[repr(C)]
pub struct Global {
    pub sdk_version: usize,
    pub enclave_size: usize,
    pub heap_offset: usize,
    pub heap_size: usize,
    pub rsrv_offset: usize,
    pub rsrv_size: usize,
    pub rsrv_executable: usize,
    pub tcs_policy: usize,
    pub tcs_max_num: usize,
    pub tcs_num: usize,
    pub td_template: Tds,
    pub tcs_template: [u8; TCS_TEMPLATE_SIZE],
    pub layout_num: u32,
    pub reserved: u32,
    pub layouts: [Layout; LAYOUT_ENTRY_NUM],
    pub enclave_image_base: u64,
    pub elrange_start_base: u64,
    pub elrange_size: u64,
    pub edmm_bk_overhead: usize,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct LayoutEntry {
    pub id: u16,
    pub attributes: u16,
    pub page_count: u32,
    pub rva: u64,
    pub content_size: u32,
    pub content_offset: u32,
    pub si_flags: u64,
}

pub const LAYOUT_GROUP_RESERVED_BYTES: usize = 4;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct LayoutGroup {
    pub id: u16,
    pub entry_count: u16,
    pub load_times: u32,
    pub load_step: u64,
    pub reserved: [u32; LAYOUT_GROUP_RESERVED_BYTES],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union Layout {
    pub entry: LayoutEntry,
    pub group: LayoutGroup,
}

impl_struct_ContiguousMemory! {
    LayoutEntry;
    LayoutGroup;
    Layout;
    Global;
}

impl Global {
    #[inline]
    pub fn get() -> &'static Global {
        unsafe {
            let ptr = &g_global_data as *const _ as *const Global;
            &*ptr
        }
    }

    #[inline]
    pub fn layout_table(&self) -> &[Layout] {
        &self.layouts[0..self.layout_num as usize]
    }
}

macro_rules! group_id {
    ($gid:expr) => {
        (crate::arch::GROUP_FLAG | $gid)
    };
}

#[allow(unused_macros)]
macro_rules! is_group_id {
    ($gid:expr) => {
        (($gid & crate::arch::GROUP_FLAG) != 0)
    };
}

pub const GROUP_FLAG: u16 = 1 << 12;
pub const LAYOUT_ID_HEAP_MIN: u16 = 1;
pub const LAYOUT_ID_HEAP_INIT: u16 = 2;
pub const LAYOUT_ID_HEAP_MAX: u16 = 3;
pub const LAYOUT_ID_TCS: u16 = 4;
pub const LAYOUT_ID_TD: u16 = 5;
pub const LAYOUT_ID_SSA: u16 = 6;
pub const LAYOUT_ID_STACK_MAX: u16 = 7;
pub const LAYOUT_ID_STACK_MIN: u16 = 8;
pub const LAYOUT_ID_THREAD_GROUP: u16 = group_id!(9);
pub const LAYOUT_ID_GUARD: u16 = 10;
pub const LAYOUT_ID_HEAP_DYN_MIN: u16 = 11;
pub const LAYOUT_ID_HEAP_DYN_INIT: u16 = 12;
pub const LAYOUT_ID_HEAP_DYN_MAX: u16 = 13;
pub const LAYOUT_ID_TCS_DYN: u16 = 14;
pub const LAYOUT_ID_TD_DYN: u16 = 15;
pub const LAYOUT_ID_SSA_DYN: u16 = 16;
pub const LAYOUT_ID_STACK_DYN_MAX: u16 = 17;
pub const LAYOUT_ID_STACK_DYN_MIN: u16 = 18;
pub const LAYOUT_ID_THREAD_GROUP_DYN: u16 = group_id!(19);
pub const LAYOUT_ID_RSRV_MIN: u16 = 20;
pub const LAYOUT_ID_RSRV_INIT: u16 = 21;
pub const LAYOUT_ID_RSRV_MAX: u16 = 22;
pub const LAYOUT_ID_USER_REGION: u16 = 23;

// se_page_attr.h
pub const PAGE_ATTR_EADD: u16 = 1 << 0;
pub const PAGE_ATTR_EEXTEND: u16 = 1 << 1;
pub const PAGE_ATTR_EREMOVE: u16 = 1 << 2;
pub const PAGE_ATTR_POST_ADD: u16 = 1 << 3;
pub const PAGE_ATTR_POST_REMOVE: u16 = 1 << 4;
pub const PAGE_ATTR_DYN_THREAD: u16 = 1 << 5;
pub const PAGE_DIR_GROW_DOWN: u16 = 1 << 6;
pub const ADD_PAGE_ONLY: u16 = PAGE_ATTR_EADD;
pub const ADD_EXTEND_PAGE: u16 = PAGE_ATTR_EADD | PAGE_ATTR_EEXTEND;
pub const PAGE_ATTR_MASK: u16 = !(PAGE_ATTR_EADD
    | PAGE_ATTR_EEXTEND
    | PAGE_ATTR_EREMOVE
    | PAGE_ATTR_POST_ADD
    | PAGE_ATTR_POST_REMOVE
    | PAGE_ATTR_DYN_THREAD
    | PAGE_DIR_GROW_DOWN);

// arch.h
pub const SI_FLAG_NONE: u64 = 0x0;
pub const SI_FLAG_R: u64 = 0x1; //Read Access
pub const SI_FLAG_W: u64 = 0x2; //Write Access
pub const SI_FLAG_X: u64 = 0x4; //Execute Access
pub const SI_FLAG_PT_LOW_BIT: u64 = 0x8; // PT low bit
pub const SI_FLAG_PT_MASK: u64 = 0xFF << SI_FLAG_PT_LOW_BIT; //Page Type Mask [15:8]
pub const SI_FLAG_SECS: u64 = 0x00 << SI_FLAG_PT_LOW_BIT; //SECS
pub const SI_FLAG_TCS: u64 = 0x01 << SI_FLAG_PT_LOW_BIT; //TCS
pub const SI_FLAG_REG: u64 = 0x02 << SI_FLAG_PT_LOW_BIT; //Regular Page
pub const SI_FLAG_TRIM: u64 = 0x04 << SI_FLAG_PT_LOW_BIT; //Trim Page
pub const SI_FLAG_PENDING: u64 = 0x8;
pub const SI_FLAG_MODIFIED: u64 = 0x10;
pub const SI_FLAG_PR: u64 = 0x20;

pub const SI_FLAGS_EXTERNAL: u64 = SI_FLAG_PT_MASK | SI_FLAG_R | SI_FLAG_W | SI_FLAG_X; //Flags visible/usable by instructions
pub const SI_FLAGS_R: u64 = SI_FLAG_R | SI_FLAG_REG;
pub const SI_FLAGS_RW: u64 = SI_FLAG_R | SI_FLAG_W | SI_FLAG_REG;
pub const SI_FLAGS_RX: u64 = SI_FLAG_R | SI_FLAG_X | SI_FLAG_REG;
pub const SI_FLAGS_RWX: u64 = SI_FLAG_R | SI_FLAG_W | SI_FLAG_X | SI_FLAG_REG;
pub const SI_FLAGS_TCS: u64 = SI_FLAG_TCS;
pub const SI_FLAGS_SECS: u64 = SI_FLAG_SECS;
pub const SI_MASK_TCS: u64 = SI_FLAG_PT_MASK;
pub const SI_MASK_MEM_ATTRIBUTE: u64 = 0x7;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct OCallContext {
    pub shadow0: usize,
    pub shadow1: usize,
    pub shadow2: usize,
    pub shadow3: usize,
    pub ocall_flag: usize,
    pub ocall_index: usize,
    pub pre_last_sp: usize,
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rbx: usize,
    pub reserved: [usize; 3],
    pub ocall_depth: usize,
    pub ocall_ret: usize,
}

impl_struct_ContiguousMemory! {
    OCallContext;
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct SsaGpr {
    pub rax: u64,            /* (0) */
    pub rcx: u64,            /* (8) */
    pub rdx: u64,            /* (16) */
    pub rbx: u64,            /* (24) */
    pub rsp: u64,            /* (32) */
    pub rbp: u64,            /* (40) */
    pub rsi: u64,            /* (48) */
    pub rdi: u64,            /* (56) */
    pub r8: u64,             /* (64) */
    pub r9: u64,             /* (72) */
    pub r10: u64,            /* (80) */
    pub r11: u64,            /* (88) */
    pub r12: u64,            /* (96) */
    pub r13: u64,            /* (104) */
    pub r14: u64,            /* (112) */
    pub r15: u64,            /* (120) */
    pub rflags: u64,         /* (128) */
    pub rip: u64,            /* (136) */
    pub rsp_u: u64,          /* (144) untrusted stack pointer. saved by EENTER */
    pub rbp_u: u64,          /* (152) untrusted frame pointer. saved by EENTE */
    pub exit_info: ExitInfo, /* (160) contain information for exits  */
    pub reserved: [u8; 3],   /* (164) padding */
    pub aex_notify: u8,      /* (167) AEX Notify */
    pub fs: u64,             /* (168) FS register */
    pub gs: u64,             /* (176) GS register */
}

impl SsaGpr {
    pub const BYTE_SIZE: usize = mem::size_of::<SsaGpr>();
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MiscExInfo {
    pub maddr: u64,
    pub error_code: u32,
    pub reserved: u32,
}

impl MiscExInfo {
    pub const BYTE_SIZE: usize = mem::size_of::<MiscExInfo>();

    #[inline]
    pub fn from_ssa_gpr(ssa_gpr: &SsaGpr) -> &MiscExInfo {
        unsafe { &*((ssa_gpr as *const _ as usize - Self::BYTE_SIZE) as *const MiscExInfo) }
    }

    #[inline]
    pub fn from_ssa_gpr_mut(ssa_gpr: &mut SsaGpr) -> &mut MiscExInfo {
        unsafe { &mut *((ssa_gpr as *mut _ as usize - Self::BYTE_SIZE) as *mut MiscExInfo) }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct ExitInfo(u32);

impl ExitInfo {
    const VECTOR_OFFSET: u32 = 0;
    const VECTOR_MASK: u32 = 0x000000FF;
    const TYPE_OFFSET: u32 = 8;
    const TYPE_MASK: u32 = 0x00000700;
    const VALID_OFFSET: u32 = 31;
    const VALID_MASK: u32 = 0x80000000;

    #[inline]
    pub fn vector(&self) -> u32 {
        (self.0 & Self::VECTOR_MASK) >> Self::VECTOR_OFFSET
    }

    #[inline]
    pub fn exit_type(&self) -> u32 {
        (self.0 & Self::TYPE_MASK) >> Self::TYPE_OFFSET
    }

    #[inline]
    pub fn valid(&self) -> u32 {
        (self.0 & Self::VALID_MASK) >> Self::VALID_OFFSET
    }

    #[inline]
    pub fn set_valid(&mut self, valid: u32) {
        let valid = (valid << Self::VALID_OFFSET) & Self::VALID_MASK;
        self.0 = (self.0 & (!Self::VALID_MASK)) | valid;
    }

    #[inline]
    pub fn set_exit_type(&mut self, exit_type: u32) {
        let exit_type = (exit_type << Self::TYPE_OFFSET) & Self::TYPE_MASK;
        self.0 = (self.0 & (!Self::TYPE_MASK)) | exit_type;
    }

    #[inline]
    pub fn set_vector(&mut self, vector: u32) {
        let vector = (vector << Self::VECTOR_OFFSET) & Self::VECTOR_MASK;
        self.0 = (self.0 & (!Self::VECTOR_MASK)) | vector;
    }
}

impl_bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct SecInfoFlags: u64 {
        const R        = 0b0000_0000_0000_0001;
        const W        = 0b0000_0000_0000_0010;
        const X        = 0b0000_0000_0000_0100;
        const PENDING  = 0b0000_0000_0000_1000;
        const MODIFIED = 0b0000_0000_0001_0000;
        const PR       = 0b0000_0000_0010_0000;
        const PT_MASK  = 0b1111_1111_0000_0000;
        const PT_B0    = 0b0000_0001_0000_0000;
        const PT_B1    = 0b0000_0010_0000_0000;
        const PT_B2    = 0b0000_0100_0000_0000;
        const PT_B3    = 0b0000_1000_0000_0000;
        const PT_B4    = 0b0001_0000_0000_0000;
        const PT_B5    = 0b0010_0000_0000_0000;
        const PT_B6    = 0b0100_0000_0000_0000;
        const PT_B7    = 0b1000_0000_0000_0000;
    }
}

impl SecInfoFlags {
    pub fn page_type(&self) -> u8 {
        (((*self & SecInfoFlags::PT_MASK).bits()) >> 8) as u8
    }

    pub fn page_type_mut(&mut self) -> &mut u8 {
        unsafe {
            let page_type: &mut [u8; 8] = &mut *(&mut self.bits() as *mut u64 as *mut [u8; 8]);
            &mut page_type[1]
        }
    }
}

impl From<PageType> for SecInfoFlags {
    fn from(data: PageType) -> SecInfoFlags {
        SecInfoFlags::from_bits_truncate((data as u64) << 8)
    }
}

impl From<edmm::PageInfo> for SecInfoFlags {
    fn from(data: edmm::PageInfo) -> SecInfoFlags {
        let typ = data.typ as u64;
        let flags = data.flags.bits() as u64;
        SecInfoFlags::from_bits_truncate((typ << 8) | flags)
    }
}

#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct SecInfo {
    pub flags: SecInfoFlags,
    pub _reserved1: [u8; 56],
}

impl fmt::Debug for SecInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("SecInfo")
            .field("flags", &self.flags.bits())
            .finish()
    }
}

impl SecInfo {
    pub fn new(flags: SecInfoFlags) -> SecInfo {
        SecInfo {
            flags,
            _reserved1: [0_u8; 56],
        }
    }
}

impl Default for SecInfo {
    fn default() -> SecInfo {
        SecInfo {
            flags: SecInfoFlags::empty(),
            _reserved1: [0_u8; 56],
        }
    }
}

impl SecInfo {
    pub const ALIGN_SIZE: usize = mem::size_of::<SecInfo>();
}

impl AsRef<[u8; SecInfo::ALIGN_SIZE]> for SecInfo {
    fn as_ref(&self) -> &[u8; SecInfo::ALIGN_SIZE] {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl AsRef<Align64<[u8; SecInfo::ALIGN_SIZE]>> for SecInfo {
    fn as_ref(&self) -> &Align64<[u8; SecInfo::ALIGN_SIZE]> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl From<SecInfoFlags> for SecInfo {
    fn from(flags: SecInfoFlags) -> SecInfo {
        SecInfo::new(flags)
    }
}

impl From<edmm::PageInfo> for SecInfo {
    fn from(data: edmm::PageInfo) -> SecInfo {
        SecInfo::from(SecInfoFlags::from(data))
    }
}

#[repr(C, align(32))]
#[derive(Clone, Copy, Debug)]
pub struct PageInfo {
    pub linaddr: u64,
    pub srcpage: u64,
    pub secinfo: u64,
    pub secs: u64,
}

impl PageInfo {
    pub const ALIGN_SIZE: usize = mem::size_of::<PageInfo>();
}

impl AsRef<[u8; PageInfo::ALIGN_SIZE]> for PageInfo {
    fn as_ref(&self) -> &[u8; PageInfo::ALIGN_SIZE] {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl AsRef<Align32<[u8; PageInfo::ALIGN_SIZE]>> for PageInfo {
    fn as_ref(&self) -> &Align32<[u8; PageInfo::ALIGN_SIZE]> {
        unsafe { &*(self as *const _ as *const _) }
    }
}
