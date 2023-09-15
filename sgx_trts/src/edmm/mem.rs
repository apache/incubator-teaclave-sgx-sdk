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

cfg_if! {
    if #[cfg(not(any(feature = "sim", feature = "hyper")))] {
        pub use hw::*;
    } else {
        pub use sw::*;
    }
}

#[cfg(not(any(feature = "sim", feature = "hyper")))]
mod hw {
    use crate::arch::{self, Layout};
    use crate::edmm::epc::{PageInfo, PageRange, PageType, ProtFlags};
    use crate::edmm::layout::LayoutTable;
    use crate::edmm::trim;
    use crate::elf::program::Type;
    use crate::emm::flags::AllocFlags;
    use crate::emm::range::{RangeType, RM};
    use crate::enclave::parse;
    use crate::enclave::MmLayout;
    use sgx_types::error::{SgxResult, SgxStatus};

    pub fn apply_epc_pages(addr: usize, count: usize) -> SgxResult {
        ensure!(addr != 0 && count != 0, SgxStatus::InvalidParameter);

        if let Some(attr) = LayoutTable::new().check_dyn_range(addr, count, None) {
            let pages = PageRange::new(
                addr,
                count,
                PageInfo {
                    typ: PageType::Reg,
                    prot: ProtFlags::R | ProtFlags::W | ProtFlags::PENDING,
                },
            )?;
            if (attr.attr & arch::PAGE_DIR_GROW_DOWN) == 0 {
                pages.accept_forward()
            } else {
                pages.accept_backward()
            }
        } else {
            Err(SgxStatus::InvalidParameter)
        }
    }

    pub fn trim_epc_pages(addr: usize, count: usize) -> SgxResult {
        ensure!(addr != 0 && count != 0, SgxStatus::InvalidParameter);

        LayoutTable::new()
            .check_dyn_range(addr, count, None)
            .ok_or(SgxStatus::InvalidParameter)?;

        trim::trim_range(addr, count)?;

        let pages = PageRange::new(
            addr,
            count,
            PageInfo {
                typ: PageType::Trim,
                prot: ProtFlags::MODIFIED,
            },
        )?;
        pages.accept_forward()?;

        trim::trim_range_commit(addr, count)?;

        Ok(())
    }

    pub fn expand_stack_epc_pages(addr: usize, count: usize) -> SgxResult {
        ensure!(addr != 0 && count != 0, SgxStatus::InvalidParameter);

        LayoutTable::new()
            .check_dyn_range(addr, count, None)
            .ok_or(SgxStatus::InvalidParameter)?;

        let mut range_manage = RM.get().unwrap().lock();
        range_manage.commit(addr, count << arch::SE_PAGE_SHIFT, RangeType::Rts)?;

        Ok(())
    }

    #[inline]
    pub fn accept_post_remove() -> SgxResult {
        reentrant_accept_post_remove(arch::Global::get().layout_table(), 0)
    }

    fn reentrant_accept_post_remove(table: &[Layout], offset: usize) -> SgxResult {
        let base = MmLayout::image_base();
        unsafe {
            for (i, layout) in table.iter().enumerate() {
                if is_group_id!(layout.group.id) {
                    let mut step = 0_usize;
                    for _ in 0..layout.group.load_times {
                        step += layout.group.load_step as usize;
                        reentrant_accept_post_remove(
                            &table[i - layout.group.entry_count as usize..i],
                            step,
                        )?;
                    }
                } else if (layout.entry.attributes & arch::PAGE_ATTR_POST_REMOVE) != 0 {
                    let addr = base + layout.entry.rva as usize + offset;
                    let count = layout.entry.page_count as usize;

                    let pages = PageRange::new(
                        addr,
                        count,
                        PageInfo {
                            typ: PageType::Trim,
                            prot: ProtFlags::MODIFIED,
                        },
                    )?;
                    pages.accept_forward()?;
                }
            }
            Ok(())
        }
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
                range_manage.modify_perms(start, size, prot, RangeType::Rts)?;
            }
            if typ == Type::GnuRelro {
                let start = base + trim_to_page!(phdr.virtual_addr() as usize);
                let end =
                    base + round_to_page!(phdr.virtual_addr() as usize + phdr.mem_size() as usize);
                let size = end - start;

                if size > 0 {
                    range_manage.modify_perms(start, size, ProtFlags::R, RangeType::Rts)?;
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

            range_manage.modify_perms(start, size, ProtFlags::R, RangeType::Rts)?;
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
                range_manage.init_static_region(
                    start,
                    end - start,
                    AllocFlags::SYSTEM,
                    PageInfo {
                        typ: PageType::Reg,
                        prot: perm,
                    },
                    None,
                    None,
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(any(feature = "sim", feature = "hyper"))]
mod sw {
    use sgx_types::error::SgxResult;

    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    pub fn apply_epc_pages(_addr: usize, _count: usize) -> SgxResult {
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    pub fn trim_epc_pages(_addr: usize, _count: usize) -> SgxResult {
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    pub fn expand_stack_epc_pages(_addr: usize, _count: usize) -> SgxResult {
        Ok(())
    }
}
