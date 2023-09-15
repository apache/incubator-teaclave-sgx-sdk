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

use sgx_types::error::SgxResult;

use crate::arch::{Layout, LayoutEntry};
use crate::edmm::mem::init_segment_emas;
use crate::edmm::{PageInfo, PageType, ProtFlags};
use crate::emm::flags::AllocFlags;
use crate::emm::interior::Alloc;
use crate::emm::range::{RangeType, EMA_PROT_MASK};
use crate::enclave::MmLayout;
use crate::{arch, emm::range::RM};

use super::interior::{init_reserve_alloc, init_static_alloc};
use super::range::init_range_manage;

pub fn init_emm() {
    init_range_manage();
    init_static_alloc();
    init_reserve_alloc();
}

pub fn init_rts_emas() -> SgxResult {
    init_segment_emas()?;
    // let mut layout = arch::Global::get().layout_table();
    // layout = &layout[..(layout.len() - 1)];

    // let layout = arch::Global::get().layout_table().split_last().unwrap().1;
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
                    init_rts_contexts_emas(&table[i - layout.group.entry_count as usize..i], step)?;
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
        range_manage.init_static_region(
            addr,
            size,
            AllocFlags::RESERVED | AllocFlags::SYSTEM,
            PageInfo {
                typ: PageType::None,
                prot: ProtFlags::NONE,
            },
            None,
            None,
        )?;
        return Ok(());
    }

    let post_remove = (entry.attributes & arch::PAGE_ATTR_POST_REMOVE) != 0;
    let post_add = (entry.attributes & arch::PAGE_ATTR_POST_ADD) != 0;
    let static_min = ((entry.attributes & arch::PAGE_ATTR_EADD) != 0) && !post_remove;

    if post_remove {
        // TODO: maybe AllocFlags need more flags or PageType is not None
        range_manage.init_static_region(
            addr,
            size,
            AllocFlags::SYSTEM,
            PageInfo {
                typ: PageType::None,
                prot: ProtFlags::R | ProtFlags::W,
            },
            None,
            None,
        )?;

        range_manage.dealloc(addr, size, RangeType::Rts)?;
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
        range_manage.alloc_inner(
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
        )?;
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
        range_manage.init_static_region(addr, size, AllocFlags::SYSTEM, info, None, None)?;
    }

    Ok(())
}
