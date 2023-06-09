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

use crate::arch::Tcs;
use crate::enclave::is_within_host;
use crate::fence;
use crate::tcs::{self, list};
use core::mem;
use core::ptr::NonNull;
use sgx_types::error::{SgxResult, SgxStatus};

cfg_if! {
    if #[cfg(not(any(feature = "sim", feature = "hyper")))] {
        pub use hw::*;
    } else {
        pub use sw::*;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MkTcs {
    tcs: *mut Tcs,
}

pub fn mktcs(mk_tcs: NonNull<MkTcs>) -> SgxResult {
    let tc = tcs::current();
    ensure!(tc.is_utility(), SgxStatus::Unexpected);
    ensure!(
        is_within_host(mk_tcs.cast().as_ptr(), mem::size_of::<MkTcs>()),
        SgxStatus::Unexpected
    );

    fence::lfence();

    let mktcs = unsafe { *mk_tcs.as_ptr() };
    let tcs = NonNull::new(mktcs.tcs).ok_or(SgxStatus::Unexpected)?;

    fence::lfence();

    list::TCS_LIST.lock().save_tcs(tcs);
    if add_tcs(tcs).is_err() {
        list::TCS_LIST.lock().del_tcs(tcs);
    }

    Ok(())
}

#[cfg(not(any(feature = "sim", feature = "hyper")))]
mod hw {
    use crate::arch::{self, Layout, Tcs};
    use crate::edmm::epc::{Page, PageInfo, PageType, ProtFlags};
    use crate::enclave::MmLayout;
    use crate::tcs::list;
    use core::ptr;
    use core::ptr::NonNull;
    use sgx_types::error::{SgxResult, SgxStatus};

    pub fn add_tcs(mut tcs: NonNull<Tcs>) -> SgxResult {
        use crate::call::{ocall, OCallIndex};
        use crate::edmm::{self, layout::LayoutTable};

        let base = MmLayout::image_base();
        let table = LayoutTable::new();
        let mut offset: usize = 0;
        table
            .check_dyn_range(tcs.as_ptr() as usize, 1, Some(&mut offset))
            .ok_or(SgxStatus::Unexpected)?;

        let layout = table
            .layout_by_id(arch::LAYOUT_ID_TCS_DYN)
            .ok_or(SgxStatus::Unexpected)?;
        if base + unsafe { layout.entry.rva as usize } + offset != tcs.as_ptr() as usize {
            bail!(SgxStatus::Unexpected);
        }

        // adding page for all the dynamic entries
        for id in arch::LAYOUT_ID_TCS_DYN..=arch::LAYOUT_ID_STACK_DYN_MIN {
            if let Some(layout) = table.layout_by_id(id) {
                if unsafe { layout.entry.attributes & arch::PAGE_ATTR_DYN_THREAD } != 0 {
                    let addr = base + unsafe { layout.entry.rva as usize } + offset;
                    let count = unsafe { layout.entry.page_count };
                    edmm::mem::apply_epc_pages(addr, count as usize)?;
                }
            }
        }

        // copy and initialize TCS
        let tcs_template = &arch::Global::get().tcs_template;
        unsafe {
            ptr::copy_nonoverlapping(
                tcs_template as *const u8,
                tcs.as_ptr().cast(),
                arch::TCS_TEMPLATE_SIZE,
            );
        }

        let tcs_ptr = tcs.as_ptr() as u64;
        let tc = unsafe { tcs.as_mut() };
        tc.ossa = tcs_ptr + tc.ossa - base as u64;
        tc.ofsbase = tcs_ptr + tc.ofsbase - base as u64;
        tc.ogsbase = tcs_ptr + tc.ogsbase - base as u64;

        // ocall for MKTCS
        ocall(OCallIndex::OCall(0), Some(tc))?;

        // EACCEPT for MKTCS
        let page = Page::new(
            tcs.as_ptr() as usize,
            PageInfo {
                typ: PageType::Tcs,
                prot: ProtFlags::MODIFIED,
            },
        )?;
        page.accept()?;

        Ok(())
    }

    #[inline]
    pub fn add_static_tcs() -> SgxResult {
        reentrant_add_static_tcs(arch::Global::get().layout_table(), 0)
    }

    #[inline]
    pub fn clear_static_tcs() -> SgxResult {
        list::TCS_LIST.lock().clear();
        Ok(())
    }

    fn reentrant_add_static_tcs(table: &[Layout], offset: usize) -> SgxResult {
        let base = MmLayout::image_base();
        unsafe {
            for (i, layout) in table.iter().enumerate() {
                if is_group_id!(layout.group.id) {
                    let mut step = 0_usize;
                    for _ in 0..layout.group.load_times {
                        step += layout.group.load_step as usize;
                        reentrant_add_static_tcs(
                            &table[i - layout.group.entry_count as usize..i],
                            step,
                        )?;
                    }
                } else if (layout.entry.si_flags & arch::SI_FLAGS_TCS != 0)
                    && (layout.entry.attributes == (arch::PAGE_ATTR_EADD | arch::PAGE_ATTR_EEXTEND))
                {
                    let tcs_addr = base + layout.entry.rva as usize + offset;
                    list::TCS_LIST
                        .lock()
                        .save_tcs(NonNull::new_unchecked(tcs_addr as *mut Tcs));
                }
            }
            Ok(())
        }
    }

    pub fn accept_trim_tcs(tcs: &Tcs) -> SgxResult {
        let mut list_guard = list::TCS_LIST.lock();
        for tcs in list_guard.iter_mut().filter(|&t| !ptr::eq(t.as_ptr(), tcs)) {
            let page = Page::new(
                tcs.as_ptr() as usize,
                PageInfo {
                    typ: PageType::Trim,
                    prot: ProtFlags::MODIFIED,
                },
            )?;
            page.accept()?;
        }

        list_guard.clear();
        Ok(())
    }
}

#[cfg(any(feature = "sim", feature = "hyper"))]
mod sw {
    use crate::arch::Tcs;
    use core::ptr::NonNull;
    use sgx_types::error::SgxResult;

    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    pub fn add_tcs(_tcs: NonNull<Tcs>) -> SgxResult {
        Ok(())
    }
}
