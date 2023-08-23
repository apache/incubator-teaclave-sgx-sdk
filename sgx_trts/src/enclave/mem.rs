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

use crate::arch;
use crate::enclave::parse;
use crate::error;
use crate::feature::SysFeatures;
use core::mem::{self, MaybeUninit};
use core::ptr;
use sgx_types::marker::ContiguousMemory;
use sgx_types::types::ProtectPerm;

extern "C" {
    static __ImageBase: u8;
}

pub struct MmLayout;

impl MmLayout {
    #[inline(always)]
    pub fn image_base() -> usize {
        unsafe { &__ImageBase as *const _ as usize }
    }

    #[inline]
    pub fn image_size() -> usize {
        arch::Global::get().enclave_size
    }

    #[inline]
    pub fn elrange_base() -> usize {
        Image::get().elrange_base
    }

    #[inline]
    pub fn elrange_size() -> usize {
        Image::get().elrange_size
    }

    #[inline]
    pub fn entry_address() -> usize {
        Image::get().entry_address
    }

    #[inline]
    pub fn heap_base() -> usize {
        Heap::get_or_init().base
    }

    #[inline]
    pub fn heap_min_size() -> usize {
        Heap::get_or_init().min_size
    }

    #[inline]
    pub fn heap_size() -> usize {
        Heap::get_or_init().size
    }

    #[inline]
    pub fn rsrvmem_base() -> usize {
        RsrvMem::get_or_init().base
    }

    #[inline]
    pub fn rsrvmem_min_size() -> usize {
        RsrvMem::get_or_init().min_size
    }

    #[inline]
    pub fn rsrvmem_size() -> usize {
        RsrvMem::get_or_init().size
    }

    #[inline]
    pub fn rsrvmm_default_perm() -> ProtectPerm {
        RsrvMem::get_or_init().perm
    }

    #[inline]
    pub fn user_region_mem_base() -> usize {
        UserRegionMem::get_or_init().base
    }

    #[inline]
    pub fn user_region_mem_size() -> usize {
        UserRegionMem::get_or_init().size
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Image {
    pub image_base: usize,
    pub image_size: usize,
    pub elrange_base: usize,
    pub elrange_size: usize,
    pub entry_address: usize,
}

#[link_section = ".data.rel.ro"]
static mut IMAGE: MaybeUninit<Image> = MaybeUninit::uninit();

impl Image {
    pub fn init() {
        let mut image = unsafe { IMAGE.assume_init_mut() };
        image.image_base = Self::image_base();
        image.image_size = Self::image_size();
        image.elrange_base = Self::elrange_base();
        image.elrange_size = Self::elrange_size();
        image.entry_address = Self::entry_address();
    }

    #[inline]
    pub fn get() -> &'static Image {
        unsafe { IMAGE.assume_init_ref() }
    }

    #[inline]
    fn image_base() -> usize {
        unsafe { &__ImageBase as *const _ as usize }
    }

    #[inline]
    fn image_size() -> usize {
        arch::Global::get().enclave_size
    }

    #[inline]
    fn elrange_base() -> usize {
        let global_data = arch::Global::get();

        if global_data.enclave_image_base != 0 {
            if global_data.enclave_image_base as usize != Self::image_base() {
                error::abort();
            }
            global_data.elrange_start_base as usize
        } else {
            Self::image_base()
        }
    }

    #[inline]
    fn elrange_size() -> usize {
        arch::Global::get().elrange_size as usize
    }

    #[inline]
    fn entry_address() -> usize {
        let elf = match parse::new_elf64() {
            Ok(elf) => elf,
            Err(_) => return 0,
        };
        Self::image_base() + elf.header2.entry_point as usize
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Heap {
    pub base: usize,
    pub size: usize,
    pub min_size: usize,
}

static mut HEAP: Option<Heap> = None;

impl Heap {
    pub fn get_or_init() -> &'static Heap {
        unsafe {
            if let Some(ref heap) = HEAP {
                heap
            } else {
                HEAP = Some(Heap {
                    base: Self::base(),
                    size: Self::size(),
                    min_size: Self::min_size(),
                });
                HEAP.as_ref().unwrap()
            }
        }
    }

    #[inline]
    fn base() -> usize {
        MmLayout::image_base() + arch::Global::get().heap_offset
    }

    fn size() -> usize {
        let mut size = arch::Global::get().heap_size;
        if SysFeatures::get().is_edmm() {
            let layout_table = arch::Global::get().layout_table();
            size += layout_table
                .iter()
                .find(|layout| unsafe { layout.entry.id == arch::LAYOUT_ID_HEAP_MAX })
                .map(|layout| unsafe { (layout.entry.page_count as usize) << arch::SE_PAGE_SHIFT })
                .unwrap_or(0);
        }
        size
    }

    fn min_size() -> usize {
        let layout_table = arch::Global::get().layout_table();
        layout_table
            .iter()
            .find(|layout| unsafe { layout.entry.id == arch::LAYOUT_ID_HEAP_MIN })
            .map(|layout| unsafe { (layout.entry.page_count as usize) << arch::SE_PAGE_SHIFT })
            .unwrap_or(0)
    }

    #[inline]
    pub fn zero_memory(&self) {
        cfg_if! {
            if #[cfg(any(feature = "sim", feature = "hyper"))] {
                let zero_size = crate::arch::Global::get().heap_size;
            } else {
                let zero_size = if SysFeatures::get().is_edmm() {
                    self.min_size
                } else {
                    self.size
                };
            }
        }
        unsafe {
            ptr::write_bytes(self.base as *mut u8, 0, zero_size);
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct RsrvMem {
    pub base: usize,
    pub size: usize,
    pub min_size: usize,
    pub perm: ProtectPerm,
}

static mut RSRV_MEM: Option<RsrvMem> = None;

impl RsrvMem {
    pub fn get_or_init() -> &'static RsrvMem {
        unsafe {
            if let Some(ref rsrvmem) = RSRV_MEM {
                rsrvmem
            } else {
                RSRV_MEM = Some(RsrvMem {
                    base: Self::base(),
                    size: Self::size(),
                    min_size: Self::min_size(),
                    perm: Self::default_perm(),
                });
                RSRV_MEM.as_ref().unwrap()
            }
        }
    }

    #[inline]
    fn base() -> usize {
        let offset = arch::Global::get().rsrv_offset;
        if offset != 0 {
            MmLayout::image_base() + offset
        } else {
            0
        }
    }

    fn size() -> usize {
        if arch::Global::get().rsrv_offset == 0 {
            return 0;
        }
        let mut size = arch::Global::get().rsrv_size;
        if SysFeatures::get().is_edmm() {
            let layout_table = arch::Global::get().layout_table();
            size += layout_table
                .iter()
                .find(|layout| unsafe { layout.entry.id == arch::LAYOUT_ID_RSRV_MAX })
                .map(|layout| unsafe { (layout.entry.page_count as usize) << arch::SE_PAGE_SHIFT })
                .unwrap_or(0);
        }
        size
    }

    fn min_size() -> usize {
        if arch::Global::get().rsrv_offset == 0 {
            return 0;
        }
        let layout_table = arch::Global::get().layout_table();
        layout_table
            .iter()
            .find(|layout| unsafe { layout.entry.id == arch::LAYOUT_ID_RSRV_MIN })
            .map(|layout| unsafe { (layout.entry.page_count as usize) << arch::SE_PAGE_SHIFT })
            .unwrap_or(0)
    }

    fn default_perm() -> ProtectPerm {
        if !SysFeatures::get().is_edmm() && Self::is_executable() {
            ProtectPerm::ReadWriteExec
        } else {
            ProtectPerm::ReadWrite
        }
    }

    #[inline]
    fn is_executable() -> bool {
        arch::Global::get().rsrv_executable != 0
    }

    pub fn check(&self) -> bool {
        if self.base == 0 {
            return true;
        }
        if !(is_page_aligned!(self.base)
            && is_page_aligned!(self.size)
            && is_page_aligned!(self.min_size))
        {
            return false;
        }
        if self.size > usize::MAX - self.base {
            return false;
        }
        true
    }

    #[inline]
    pub fn zero_memory(&self) {
        if self.base != 0 {
            cfg_if! {
                if #[cfg(any(feature = "sim", feature = "hyper"))] {
                    let zero_size = crate::arch::Global::get().rsrv_size;
                } else {
                    let zero_size = if SysFeatures::get().is_edmm() {
                        self.min_size
                    } else {
                        self.size
                    };
                }
            }
            unsafe {
                ptr::write_bytes(self.base as *mut u8, 0, zero_size);
            }
        }
    }
}

pub struct UserRegionMem {
    pub base: usize,
    pub size: usize,
}

static mut USER_REGION_MEM: Option<UserRegionMem> = None;

impl UserRegionMem {
    pub fn get_or_init() -> &'static UserRegionMem {
        unsafe {
            if let Some(ref user_region_mem) = USER_REGION_MEM {
                user_region_mem
            } else {
                let (base, size) = Self::layout();
                USER_REGION_MEM = Some(UserRegionMem { base, size });
                USER_REGION_MEM.as_ref().unwrap()
            }
        }
    }

    fn layout() -> (usize, usize) {
        if SysFeatures::get().is_edmm() {
            let layout_table = arch::Global::get().layout_table();
            layout_table
                .iter()
                .find(|layout| unsafe { layout.entry.id == arch::LAYOUT_ID_USER_REGION })
                .map(|layout| unsafe {
                    (
                        MmLayout::image_base() + layout.entry.rva as usize,
                        (layout.entry.page_count as usize) << arch::SE_PAGE_SHIFT,
                    )
                })
                .unwrap_or((0, 0))
        } else {
            (0, 0)
        }
    }

    pub fn check(&self) -> bool {
        if self.base == 0 {
            return true;
        }
        if !(is_page_aligned!(self.base) && is_page_aligned!(self.size)) {
            return false;
        }
        if self.size > usize::MAX - self.base {
            return false;
        }
        true
    }
}

pub fn is_within_enclave(p: *const u8, len: usize) -> bool {
    let start = p as usize;
    let end = if len > 0 {
        if let Some(end) = start.checked_add(len - 1) {
            end
        } else {
            return false;
        }
    } else {
        start
    };
    let base = MmLayout::elrange_base();

    (start <= end) && (start >= base) && (end < base + MmLayout::elrange_size())
}

pub fn is_within_host(p: *const u8, len: usize) -> bool {
    let start = p as usize;
    let end = if len > 0 {
        if let Some(end) = start.checked_add(len - 1) {
            end
        } else {
            return false;
        }
    } else {
        start
    };
    let base = MmLayout::elrange_base();

    (start <= end) && ((end < base) || (start > base + MmLayout::elrange_size() - 1))
}

pub trait EnclaveRange {
    fn is_enclave_range(&self) -> bool;
    fn is_host_range(&self) -> bool;
}

impl<T> EnclaveRange for T
where
    T: Sized + ContiguousMemory,
{
    default fn is_enclave_range(&self) -> bool {
        is_within_enclave(self as *const _ as *const u8, mem::size_of::<T>())
    }

    default fn is_host_range(&self) -> bool {
        is_within_host(self as *const _ as *const u8, mem::size_of::<T>())
    }
}

impl<T> EnclaveRange for [T]
where
    T: Sized + ContiguousMemory,
{
    default fn is_enclave_range(&self) -> bool {
        is_within_enclave(
            self.as_ptr() as *const _ as *const u8,
            mem::size_of_val(self),
        )
    }

    default fn is_host_range(&self) -> bool {
        is_within_host(
            self.as_ptr() as *const _ as *const u8,
            mem::size_of_val(self),
        )
    }
}
