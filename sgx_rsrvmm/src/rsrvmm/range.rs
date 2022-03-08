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

use alloc_crate::vec::Vec;
use core::fmt;
use core::slice;
use sgx_types::error::errno::*;
use sgx_types::error::OsResult;
use sgx_types::metadata::SE_PAGE_SIZE;

macro_rules! is_page_aligned {
    ($num:expr) => {
        $num & (SE_PAGE_SIZE - 1) == 0
    };
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct MmRange {
    start: usize,
    end: usize,
}

impl MmRange {
    pub fn new(start: usize, end: usize) -> OsResult<MmRange> {
        ensure!(is_page_aligned!(start) && is_page_aligned!(end), EINVAL);
        Ok(MmRange { start, end })
    }

    pub fn new_with_size(start: usize, size: usize) -> OsResult<MmRange> {
        ensure!(is_page_aligned!(start) && is_page_aligned!(size), EINVAL);
        Ok(MmRange {
            start,
            end: start.checked_add(size).ok_or(EINVAL)?,
        })
    }

    pub fn new_empty(start: usize) -> OsResult<MmRange> {
        ensure!(is_page_aligned!(start), EINVAL);
        Ok(MmRange { start, end: start })
    }

    #[inline]
    pub unsafe fn from_unchecked(start: usize, end: usize) -> MmRange {
        debug_assert!(is_page_aligned!(start));
        debug_assert!(is_page_aligned!(end));
        debug_assert!(start <= end);
        MmRange { start, end }
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    pub fn resize(&mut self, size: usize) {
        if size == 0 {
            self.end = self.start;
        } else {
            assert!(is_page_aligned!(size));
            let end = self.start.checked_add(size);
            assert!(end.is_some());
            self.end = end.unwrap();
        }
    }

    #[inline]
    pub fn add_size(&mut self, size: usize) {
        if size == 0 {
            return;
        }
        assert!(is_page_aligned!(size));
        let end = self.end.checked_add(size);
        assert!(end.is_some());
        self.end = end.unwrap();
    }

    #[inline]
    pub fn sub_size(&mut self, size: usize) {
        if size == 0 {
            return;
        }
        assert!(is_page_aligned!(size));
        if size <= self.size() {
            self.end -= size;
        }
    }

    #[inline]
    pub fn set_start(&mut self, start: usize) {
        assert!(is_page_aligned!(start) && start <= self.end);
        self.start = start;
    }

    #[inline]
    pub fn set_end(&mut self, end: usize) {
        assert!(is_page_aligned!(end) && end >= self.start);
        self.end = end;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub fn is_superset_of(&self, other: &MmRange) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    #[inline]
    pub fn contains(&self, addr: usize) -> bool {
        self.start <= addr && addr < self.end
    }

    pub fn contiguous_with(&self, other: &MmRange) -> bool {
        let intersection_start = self.start.max(other.start);
        let intersection_end = self.end.min(other.end);
        intersection_start == intersection_end
    }

    #[allow(dead_code)]
    pub fn overlap_with(&self, other: &MmRange) -> bool {
        let intersection_start = self.start.max(other.start);
        let intersection_end = self.end.min(other.end);
        intersection_start < intersection_end
    }

    pub fn intersect(&self, other: &MmRange) -> Option<MmRange> {
        let intersection_start = self.start.max(other.start);
        let intersection_end = self.end.min(other.end);
        if intersection_start >= intersection_end {
            None
        } else {
            unsafe {
                Some(MmRange::from_unchecked(
                    intersection_start,
                    intersection_end,
                ))
            }
        }
    }

    pub fn subtract(&self, other: &MmRange) -> Vec<MmRange> {
        if self.is_empty() {
            return Vec::new();
        }

        let intersection = match self.intersect(other) {
            None => return vec![*self],
            Some(intersection) => intersection,
        };

        let self_start = self.start;
        let self_end = self.end;
        let inter_start = intersection.start;
        let inter_end = intersection.end;
        debug_assert!(self_start <= inter_start);
        debug_assert!(inter_end <= self_end);

        match (self_start < inter_start, inter_end < self_end) {
            (false, false) => Vec::new(),
            (false, true) => unsafe { vec![MmRange::from_unchecked(inter_end, self_end)] },
            (true, false) => unsafe { vec![MmRange::from_unchecked(self_start, inter_start)] },
            (true, true) => unsafe {
                vec![
                    MmRange::from_unchecked(self_start, inter_start),
                    MmRange::from_unchecked(inter_end, self_end),
                ]
            },
        }
    }

    #[inline]
    pub unsafe fn as_slice(&self) -> &[u8] {
        slice::from_raw_parts(self.start as *const u8, self.size())
    }

    #[inline]
    pub unsafe fn as_slice_mut(&mut self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.start as *mut u8, self.size())
    }
}

impl fmt::Debug for MmRange {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("MmRange")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("size", &self.size())
            .finish()
    }
}
