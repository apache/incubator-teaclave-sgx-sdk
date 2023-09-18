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

use super::alloc::{init_reserve_alloc, init_static_alloc};
use super::range::init_range_manage;

pub fn init_emm() {
    init_range_manage();
    init_static_alloc();
    init_reserve_alloc();
}

cfg_if! {
    if #[cfg(not(any(feature = "sim", feature = "hyper")))] {
        pub use hw::*;
    } else {
        pub use sw::*;
    }
}

#[cfg(not(any(feature = "sim", feature = "hyper")))]
mod hw {
    use crate::arch::{self, Layout, LayoutEntry};
    use crate::elf::program::Type;
    use crate::emm::alloc::Alloc;
    use crate::emm::layout::LayoutTable;
    use crate::emm::page::AllocFlags;
    use crate::emm::range::{RangeType, EMA_PROT_MASK, RM};
    use crate::emm::{PageInfo, PageType, ProtFlags};
    use crate::enclave::parse;
    use crate::enclave::MmLayout;
    use sgx_types::error::{SgxResult, SgxStatus};

    pub fn init_rts_emas() -> SgxResult {
        init_segment_emas()?;

        let layout = arch::Global::get().layout_table();
        init_rts_contexts_emas(layout, 0)?;
        Ok(())
    }

    fn init_rts_contexts_emas(table: &[Layout], offset: usize) -> SgxResult {
        unsafe {
            for (i, layout) in table.iter().enumerate() {
                if is_group_id!(layout.group.id) {
                    let mut step = 0_usize;
                    for _ in 0..layout.group.load_times {
                        step += layout.group.load_step as usize;
                        init_rts_contexts_emas(
                            &table[i - layout.group.entry_count as usize..i],
                            step,
                        )?;
                    }
                } else {
                    build_rts_context_emas(&layout.entry, offset)?;
                }
            }
            Ok(())
        }
    }

    fn build_rts_context_emas(entry: &LayoutEntry, offset: usize) -> SgxResult {
        if entry.id == arch::LAYOUT_ID_USER_REGION {
            return Ok(());
        }

        let rva = offset + (entry.rva as usize);
        assert!(is_page_aligned!(rva));

        // TODO: not sure get_enclave_base() equal to elrange_base or image_base
        let addr = MmLayout::image_base() + rva;
        let size = (entry.page_count << arch::SE_PAGE_SHIFT) as usize;
        let mut range_manage = RM.get().unwrap().lock();

        // entry is guard page or has EREMOVE, build a reserved ema
        if (entry.si_flags == 0) || (entry.attributes & arch::PAGE_ATTR_EREMOVE != 0) {
            range_manage
                .init_static_region(
                    addr,
                    size,
                    AllocFlags::RESERVED | AllocFlags::SYSTEM,
                    PageInfo {
                        typ: PageType::None,
                        prot: ProtFlags::NONE,
                    },
                    None,
                    None,
                )
                .map_err(|_| SgxStatus::Unexpected)?;
            return Ok(());
        }

        let post_remove = (entry.attributes & arch::PAGE_ATTR_POST_REMOVE) != 0;
        let post_add = (entry.attributes & arch::PAGE_ATTR_POST_ADD) != 0;
        let static_min = ((entry.attributes & arch::PAGE_ATTR_EADD) != 0) && !post_remove;

        if post_remove {
            // TODO: maybe AllocFlags need more flags or PageType is not None
            range_manage
                .init_static_region(
                    addr,
                    size,
                    AllocFlags::SYSTEM,
                    PageInfo {
                        typ: PageType::None,
                        prot: ProtFlags::R | ProtFlags::W,
                    },
                    None,
                    None,
                )
                .map_err(|_| SgxStatus::Unexpected)?;

            range_manage
                .dealloc(addr, size, RangeType::Rts)
                .map_err(|_| SgxStatus::Unexpected)?;
        }

        if post_add {
            let commit_direction = if entry.id == arch::LAYOUT_ID_STACK_MAX
                || entry.id == arch::LAYOUT_ID_STACK_DYN_MAX
                || entry.id == arch::LAYOUT_ID_STACK_DYN_MIN
            {
                AllocFlags::GROWSDOWN
            } else {
                AllocFlags::GROWSUP
            };

            // TODO: revise alloc and not use int
            range_manage
                .alloc(
                    Some(addr),
                    size,
                    AllocFlags::COMMIT_ON_DEMAND
                        | commit_direction
                        | AllocFlags::SYSTEM
                        | AllocFlags::FIXED,
                    PageInfo {
                        typ: PageType::Reg,
                        prot: ProtFlags::R | ProtFlags::W,
                    },
                    None,
                    None,
                    RangeType::Rts,
                    Alloc::Reserve,
                )
                .map_err(|_| SgxStatus::Unexpected)?;
        } else if static_min {
            let info = if entry.id == arch::LAYOUT_ID_TCS {
                PageInfo {
                    typ: PageType::Tcs,
                    prot: ProtFlags::NONE,
                }
            } else {
                PageInfo {
                    typ: PageType::Reg,
                    prot: ProtFlags::from_bits_truncate(
                        (entry.si_flags as usize & EMA_PROT_MASK) as u8,
                    ),
                }
            };
            range_manage
                .init_static_region(addr, size, AllocFlags::SYSTEM, info, None, None)
                .map_err(|_| SgxStatus::Unexpected)?;
        }

        Ok(())
    }

    pub fn expand_stack_epc_pages(addr: usize, count: usize) -> SgxResult {
        ensure!(addr != 0 && count != 0, SgxStatus::InvalidParameter);

        LayoutTable::new()
            .check_dyn_range(addr, count, None)
            .ok_or(SgxStatus::InvalidParameter)?;

        let mut range_manage = RM.get().unwrap().lock();
        range_manage
            .commit(addr, count << arch::SE_PAGE_SHIFT, RangeType::Rts)
            .map_err(|_| SgxStatus::Unexpected)?;

        Ok(())
    }

    pub fn change_perm() -> SgxResult {
        let elf = parse::new_elf()?;
        let text_relo = parse::has_text_relo()?;

        let base = MmLayout::image_base();
        let mut range_manage = RM.get().unwrap().lock();
        for phdr in elf.program_iter() {
            let typ = phdr.get_type().unwrap_or(Type::Null);
            if typ == Type::Load && text_relo && !phdr.flags().is_write() {
                let mut perm = 0_u64;
                let start = base + trim_to_page!(phdr.virtual_addr() as usize);
                let end =
                    base + round_to_page!(phdr.virtual_addr() as usize + phdr.mem_size() as usize);
                let size = end - start;

                if phdr.flags().is_read() {
                    perm |= arch::SGX_EMA_PROT_READ;
                }
                if phdr.flags().is_execute() {
                    perm |= arch::SGX_EMA_PROT_EXEC;
                }

                let prot = ProtFlags::from_bits_truncate(perm as u8);
                range_manage
                    .modify_perms(start, size, prot, RangeType::Rts)
                    .map_err(|_| SgxStatus::Unexpected)?;
            }
            if typ == Type::GnuRelro {
                let start = base + trim_to_page!(phdr.virtual_addr() as usize);
                let end =
                    base + round_to_page!(phdr.virtual_addr() as usize + phdr.mem_size() as usize);
                let size = end - start;

                if size > 0 {
                    range_manage
                        .modify_perms(start, size, ProtFlags::R, RangeType::Rts)
                        .map_err(|_| SgxStatus::Unexpected)?;
                }
            }
        }

        let layout_table = arch::Global::get().layout_table();
        if let Some(layout) = layout_table.iter().find(|layout| unsafe {
            (layout.entry.id == arch::LAYOUT_ID_RSRV_MIN)
                && (layout.entry.si_flags == arch::SI_FLAGS_RWX)
                && (layout.entry.page_count > 0)
        }) {
            let start = base + unsafe { layout.entry.rva as usize };
            let size = unsafe { layout.entry.page_count as usize } << arch::SE_PAGE_SHIFT;

            range_manage
                .modify_perms(start, size, ProtFlags::R, RangeType::Rts)
                .map_err(|_| SgxStatus::Unexpected)?;
        }
        Ok(())
    }

    pub fn init_segment_emas() -> SgxResult {
        let elf = parse::new_elf()?;
        let text_relo = parse::has_text_relo()?;

        let base = MmLayout::image_base();
        for phdr in elf.program_iter() {
            let typ = phdr.get_type().unwrap_or(Type::Null);

            if typ == Type::Load {
                let mut perm = ProtFlags::R;
                let start = base + trim_to_page!(phdr.virtual_addr() as usize);
                let end =
                    base + round_to_page!(phdr.virtual_addr() as usize + phdr.mem_size() as usize);

                if phdr.flags().is_write() || text_relo {
                    perm |= ProtFlags::W;
                }
                if phdr.flags().is_execute() {
                    perm |= ProtFlags::X;
                }

                let mut range_manage = RM.get().unwrap().lock();
                range_manage
                    .init_static_region(
                        start,
                        end - start,
                        AllocFlags::SYSTEM,
                        PageInfo {
                            typ: PageType::Reg,
                            prot: perm,
                        },
                        None,
                        None,
                    )
                    .map_err(|_| SgxStatus::Unexpected)?;
            }
        }

        Ok(())
    }
}
