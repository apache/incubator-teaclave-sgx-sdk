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

use crate::map::{MapAddr, Nothing};
use crate::rsrvmm::area::{MmArea, MmPerm, MmState, MmType};
use crate::rsrvmm::range::MmRange;
use alloc_crate::vec::Vec;
use core::convert::From;
use core::mem;
use sgx_trts::trts::MmLayout;
use sgx_types::error::errno::*;
use sgx_types::error::OsResult;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MmAllocAddr {
    #[default]
    Any, // Free to choose any address
    Hint(usize),  // Prefer the address, but can use other address
    Need(usize),  // Need to use the address, otherwise report error
    Force(usize), // Force using the address by free first
}

impl From<MapAddr> for MmAllocAddr {
    fn from(addr: MapAddr) -> MmAllocAddr {
        match addr {
            MapAddr::Any => MmAllocAddr::Any,
            MapAddr::Hint(p) => MmAllocAddr::Hint(p.as_ptr() as usize),
            MapAddr::Need(p) => MmAllocAddr::Need(p.as_ptr() as usize),
            MapAddr::Force(p) => MmAllocAddr::Force(p.as_ptr() as usize),
        }
    }
}

#[derive(Debug)]
pub struct MmManager {
    range: MmRange,
    perm: MmPerm,
    vrds: Vec<MmArea>,
}

pub const INIT_MMAREA_COUNT: usize = 32;

impl MmManager {
    pub fn new(base: usize, size: usize, perm: MmPerm) -> OsResult<MmManager> {
        let range = MmRange::new_with_size(base, size)?;
        let mut vrds = Vec::with_capacity(INIT_MMAREA_COUNT);

        let vrd = MmArea::new::<Nothing>(
            MmRange::new(MmLayout::image_base(), base)?,
            MmPerm::None,
            MmState::Locked,
            MmType::Reg,
            None,
        )?;
        vrds.push(vrd);

        let vrd = MmArea::new::<Nothing>(
            MmRange::new_empty(range.end())?,
            MmPerm::None,
            MmState::Locked,
            MmType::Reg,
            None,
        )?;
        vrds.push(vrd);

        Ok(MmManager { range, perm, vrds })
    }

    #[allow(dead_code)]
    #[inline]
    pub fn range(&self) -> MmRange {
        self.range
    }

    #[allow(dead_code)]
    #[inline]
    pub fn base(&self) -> usize {
        self.range.start()
    }

    #[allow(dead_code)]
    #[inline]
    pub fn size(&self) -> usize {
        self.range.size()
    }

    #[allow(dead_code)]
    pub fn find_range(&self, addr: usize) -> Option<(usize, MmRange)> {
        self.vrds
            .iter()
            .enumerate()
            .find(|(_, vrd)| vrd.contains(addr))
            .map(|(idx, vrd)| (idx, vrd.range()))
    }

    pub fn find_vrd(&self, addr: usize) -> Option<(usize, &MmArea)> {
        self.vrds
            .iter()
            .enumerate()
            .find(|(_, vrd)| vrd.contains(addr))
    }

    #[allow(dead_code)]
    pub fn is_free_range(&self, range: &MmRange) -> bool {
        self.range.is_superset_of(range) && self.vrds.iter().all(|vrd| !vrd.overlap_with(range))
    }

    pub fn insert_vrd(&mut self, idx: usize, vrd: MmArea) {
        debug_assert!(self.check_range(&vrd.range()));
        debug_assert!(0 < idx && idx < self.vrds.len());

        let left_idx = idx - 1;
        let right_idx = idx;
        let left_vrd = &self.vrds[left_idx];
        let right_vrd = &self.vrds[right_idx];

        debug_assert!(left_vrd.end() <= vrd.start());
        debug_assert!(vrd.end() <= right_vrd.start());

        let left_combinable = vrd.can_combine(left_vrd);
        let right_combinable = vrd.can_combine(right_vrd);

        match (left_combinable, right_combinable) {
            (false, false) => {
                self.vrds.insert(idx, vrd);
            }
            (true, false) => {
                self.vrds[left_idx].set_end(vrd.end());
            }
            (false, true) => {
                self.vrds[right_idx].set_start(vrd.start());
            }
            (true, true) => {
                let left_end = self.vrds[right_idx].end();
                self.vrds[left_idx].set_end(left_end);
                self.vrds.remove(right_idx);
            }
        }
    }

    pub fn find_free_range(
        &mut self,
        addr: MmAllocAddr,
        size: usize,
    ) -> OsResult<(usize, MmRange)> {
        match addr {
            MmAllocAddr::Any => {
                if size > self.range.size() {
                    bail!(EINVAL);
                }
            }
            MmAllocAddr::Hint(addr) | MmAllocAddr::Need(addr) | MmAllocAddr::Force(addr) => {
                if !self.check_range(&MmRange::new_with_size(addr, size)?) {
                    bail!(EINVAL);
                }
            }
        };

        if let MmAllocAddr::Force(addr) = addr {
            self.free_vrds(unsafe { &MmRange::from_unchecked(addr, size) })?;
        }

        let mut result_free_range: Option<MmRange> = None;
        let mut result_idx: Option<usize> = None;

        for (idx, range_pair) in self.vrds.windows(2).enumerate() {
            let pre_range = &range_pair[0];
            let next_range = &range_pair[1];

            let mut free_range = {
                let free_range_start = pre_range.end();
                let free_range_end = next_range.start();

                let free_range_size = free_range_end - free_range_start;
                if free_range_size < size {
                    continue;
                }

                unsafe { MmRange::from_unchecked(free_range_start, free_range_end) }
            };

            match addr {
                MmAllocAddr::Any => {}
                MmAllocAddr::Hint(addr) => {
                    if free_range.contains(addr) && free_range.end() - addr >= size {
                        free_range.set_start(addr);
                        let insert_idx = idx + 1;
                        return Ok((insert_idx, free_range));
                    }
                }
                MmAllocAddr::Need(addr) | MmAllocAddr::Force(addr) => {
                    if free_range.start() > addr {
                        bail!(ENOMEM);
                    }
                    if !free_range.contains(addr) {
                        continue;
                    }
                    if free_range.end() - addr < size {
                        bail!(ENOMEM);
                    }
                    free_range.set_start(addr);
                    let insert_idx = idx + 1;
                    return Ok((insert_idx, free_range));
                }
            }

            if result_free_range.is_none()
                || result_free_range.as_ref().unwrap().size() > free_range.size()
            {
                // Record the minimal free range that satisfies the contraints
                result_free_range = Some(free_range);
                result_idx = Some(idx);
            }
        }

        if let Some(free_range) = result_free_range {
            let insert_idx = result_idx.unwrap() + 1;
            Ok((insert_idx, free_range))
        } else {
            Err(ENOMEM)
        }
    }

    pub fn alloc_range_from(
        &self,
        addr: MmAllocAddr,
        size: usize,
        free_range: &MmRange,
    ) -> MmRange {
        debug_assert!(free_range.size() >= size);

        let mut alloc_range = *free_range;

        if let MmAllocAddr::Need(addr) = addr {
            debug_assert!(addr == alloc_range.start());
        }
        if let MmAllocAddr::Force(addr) = addr {
            debug_assert!(addr == alloc_range.start());
        }

        alloc_range.resize(size);
        alloc_range
    }

    pub fn combine_vrds(&mut self, range: &MmRange) {
        debug_assert!(self.check_range(range));
        debug_assert!(!range.is_empty());

        let mut addr = range.start();
        let (mut idx, _) = match self.find_vrd(addr) {
            Some(v) => v,
            None => return,
        };

        while addr < range.end() {
            if idx > 0 && self.vrds[idx - 1].can_combine(&self.vrds[idx]) {
                let size = self.vrds[idx].size();
                self.vrds[idx - 1].add_size(size);
                self.vrds.remove(idx);
                idx -= 1;
            }

            if idx < self.vrds.len() - 1 {
                idx += 1;
                addr = self.vrds[idx].start();
            } else {
                return;
            }
        }

        if idx > 0 && idx < self.vrds.len() && self.vrds[idx - 1].can_combine(&self.vrds[idx]) {
            let size = self.vrds[idx].size();
            self.vrds[idx - 1].add_size(size);
            self.vrds.remove(idx);
        }
    }

    pub fn split_vrds(&mut self, range: &MmRange) -> OsResult {
        ensure!(!range.is_empty() && self.check_range(range), EINVAL);

        let start_addr = range.start();
        let (idx, vrd) = self.find_vrd(start_addr).ok_or(EINVAL)?;

        if vrd.start() < start_addr {
            let new_range = MmRange::new(start_addr, vrd.end())?;
            let new_vrd = MmArea::inherits_from(new_range, vrd);

            self.vrds[idx].sub_size(new_range.size());
            self.vrds.insert(idx + 1, new_vrd);
        }

        let end_addr = range.end();
        let (idx, vrd) = self.find_vrd(end_addr - 1).ok_or(EINVAL)?;

        if end_addr < vrd.end() {
            let new_range = MmRange::new(end_addr, vrd.end())?;
            let new_vrd = MmArea::inherits_from(new_range, vrd);

            self.vrds[idx].sub_size(new_vrd.size());
            self.vrds.insert(idx + 1, new_vrd);
        }
        Ok(())
    }

    pub fn apply_perm_vrds(&mut self, range: &MmRange, new_perm: MmPerm) -> OsResult {
        debug_assert!(self.check_range(range));
        debug_assert!(!range.is_empty());

        for vrd in &self.vrds {
            if vrd.start() >= range.end() {
                break;
            }

            if let Some(mut vrd) = vrd.intersect(range) {
                if vrd.typ() == MmType::Reg {
                    vrd.apply_perm(new_perm, MmArea::check_perm)?;
                }
            }
        }
        Ok(())
    }

    pub fn free_vrds(&mut self, range: &MmRange) -> OsResult {
        ensure!(!range.is_empty() && self.check_range(range), EINVAL);

        let old_vrds = {
            let mut old_vrds = Vec::new();
            mem::swap(&mut self.vrds, &mut old_vrds);
            old_vrds
        };
        let new_vrds = old_vrds
            .into_iter()
            .flat_map(|vrd| {
                let mut intersection = match vrd.intersect(range) {
                    None => return vec![vrd],
                    Some(intersection) => intersection,
                };

                let _ = intersection.write();

                let _ = intersection.apply_perm(self.perm, |_, _| true);

                vrd.subtract(&intersection)
            })
            .collect();
        self.vrds = new_vrds;
        Ok(())
    }

    pub fn flush_vrds(&mut self, range: &MmRange) -> OsResult {
        ensure!(!range.is_empty() && self.check_range(range), EINVAL);

        for vrd in &self.vrds {
            if vrd.start() >= range.end() {
                break;
            }

            if let Some(vrd) = vrd.intersect(range) {
                vrd.write()?;
                vrd.flush()?;
            }
        }
        Ok(())
    }

    #[inline]
    pub fn check_range(&self, range: &MmRange) -> bool {
        self.range.is_superset_of(range)
    }
}
