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

use crate::arch::{self, Layout};
use crate::enclave::MmLayout;
use crate::feature::SysFeatures;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DynAttrbutes {
    pub flags: u64,
    pub attr: u16,
}

pub struct LayoutTable<'a> {
    table: &'a [Layout],
}

impl<'a> LayoutTable<'a> {
    pub fn new() -> LayoutTable<'a> {
        LayoutTable {
            table: arch::Global::get().layout_table(),
        }
    }

    pub fn layout_by_id(&self, id: u16) -> Option<&Layout> {
        self.table
            .iter()
            .find(|layout| unsafe { layout.entry.id == id })
    }

    fn check_heap_dyn_range(&self, addr: usize, count: usize) -> Option<DynAttrbutes> {
        let start = MmLayout::heap_base() + MmLayout::heap_min_size();
        let end = start + (MmLayout::heap_size() - MmLayout::heap_min_size());
        if start == 0 || end == 0 {
            return None;
        }
        if addr >= start && (addr + (count << arch::SE_PAGE_SHIFT)) <= end {
            Some(DynAttrbutes {
                flags: arch::SI_FLAGS_RW,
                attr: arch::PAGE_ATTR_POST_ADD,
            })
        } else {
            None
        }
    }

    fn check_rsrv_dyn_range(&self, addr: usize, count: usize) -> Option<DynAttrbutes> {
        let start = MmLayout::rsrvmem_base() + MmLayout::rsrvmem_min_size();
        let end = start + (MmLayout::rsrvmem_size() - MmLayout::rsrvmem_min_size());
        if start == 0 || end == 0 {
            return None;
        }
        if addr >= start && (addr + (count << arch::SE_PAGE_SHIFT)) <= end {
            Some(DynAttrbutes {
                flags: arch::SI_FLAGS_RW,
                attr: arch::PAGE_ATTR_POST_ADD,
            })
        } else {
            None
        }
    }

    fn check_entry_dyn_range(
        &self,
        addr: usize,
        count: usize,
        id: u16,
        offset: usize,
    ) -> Option<DynAttrbutes> {
        if !(arch::LAYOUT_ID_HEAP_MIN..=arch::LAYOUT_ID_STACK_DYN_MIN).contains(&id) {
            return None;
        }
        let layout = self.layout_by_id(id)?;

        let start = MmLayout::image_base() + unsafe { layout.entry.rva as usize } + offset;
        let end = start + (unsafe { layout.entry.page_count as usize } << arch::SE_PAGE_SHIFT);

        if addr >= start && (addr + (count << arch::SE_PAGE_SHIFT)) <= end {
            Some(DynAttrbutes {
                flags: unsafe { layout.entry.si_flags },
                attr: unsafe { layout.entry.attributes },
            })
        } else {
            None
        }
    }

    fn check_utility_tcs_dyn_stack(&self, addr: usize, count: usize) -> Option<DynAttrbutes> {
        self.check_entry_dyn_range(addr, count, arch::LAYOUT_ID_STACK_MAX, 0)
    }

    pub fn check_dyn_range(
        &self,
        addr: usize,
        count: usize,
        offset: Option<&mut usize>,
    ) -> Option<DynAttrbutes> {
        addr.checked_add(count << arch::SE_PAGE_SHIFT)?;
        if let Some(attr) = self.check_heap_dyn_range(addr, count) {
            return Some(attr);
        }

        if let Some(attr) = self.check_utility_tcs_dyn_stack(addr, count) {
            return Some(attr);
        }

        if let Some(attr) = self.check_rsrv_dyn_range(addr, count) {
            return Some(attr);
        }

        if let Some(layout) = self.layout_by_id(arch::LAYOUT_ID_THREAD_GROUP_DYN) {
            let load_times = unsafe { layout.group.load_times as usize };
            let load_step = unsafe { layout.group.load_step as usize };
            for id in arch::LAYOUT_ID_TCS_DYN..=arch::LAYOUT_ID_STACK_DYN_MIN {
                for i in 0..load_times + 1 {
                    if let Some(attr) = self.check_entry_dyn_range(addr, count, id, i * load_step) {
                        if let Some(offset) = offset {
                            *offset = i * load_step;
                        }
                        return Some(attr);
                    }
                }
            }
        } else {
            for id in arch::LAYOUT_ID_TCS_DYN..=arch::LAYOUT_ID_STACK_DYN_MIN {
                if let Some(attr) = self.check_entry_dyn_range(addr, count, id, 0) {
                    if let Some(offset) = offset {
                        *offset = 0;
                    }
                    return Some(attr);
                }
            }
        }
        None
    }

    pub fn dyn_stack_max_page(&self) -> usize {
        self.layout_by_id(arch::LAYOUT_ID_STACK_MAX)
            .map(|layout| unsafe { layout.entry.page_count as usize })
            .unwrap_or(0)
    }

    pub fn is_dyn_tcs_exist(&self) -> bool {
        if SysFeatures::get().is_edmm() {
            self.layout_by_id(arch::LAYOUT_ID_STACK_DYN_MIN).is_some()
        } else {
            false
        }
    }
}
