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
// under the License.

use crate::map::{Map, MapObject};
use crate::rsrvmm::range::MmRange;
use alloc_crate::sync::Arc;
use alloc_crate::vec::Vec;
use core::any::TypeId;
use core::cmp::{self, Ordering};
use core::convert::From;
use core::fmt;
use core::ops::{Deref, DerefMut};
use sgx_trts::edmm::{modpr_ocall, mprotect_ocall};
use sgx_trts::edmm::{PageFlags, PageInfo, PageRange, PageType};
use sgx_trts::trts;
use sgx_types::error::errno::*;
use sgx_types::error::OsResult;
use sgx_types::metadata::SE_PAGE_SHIFT;
use sgx_types::types::ProtectPerm;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum MmPerm {
    #[default]
    None,
    R,
    RW,
    RX,
    RWX,
}

impl MmPerm {
    pub fn can_read(&self) -> bool {
        *self != Self::None
    }

    pub fn can_write(&self) -> bool {
        !matches!(*self, Self::None | Self::R | Self::RX)
    }

    #[allow(dead_code)]
    pub fn can_execute(&self) -> bool {
        !matches!(*self, Self::None | Self::R | Self::RW)
    }
}

impl From<ProtectPerm> for MmPerm {
    fn from(perm: ProtectPerm) -> MmPerm {
        match perm {
            ProtectPerm::None => MmPerm::None,
            ProtectPerm::Read => MmPerm::R,
            ProtectPerm::ReadWrite => MmPerm::RW,
            ProtectPerm::ReadExec => MmPerm::RX,
            ProtectPerm::ReadWriteExec => MmPerm::RWX,
        }
    }
}

impl From<MmPerm> for ProtectPerm {
    fn from(p: MmPerm) -> ProtectPerm {
        match p {
            MmPerm::None => ProtectPerm::None,
            MmPerm::R => ProtectPerm::Read,
            MmPerm::RW => ProtectPerm::ReadWrite,
            MmPerm::RX => ProtectPerm::ReadExec,
            MmPerm::RWX => ProtectPerm::ReadWriteExec,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MmType {
    #[default]
    None,
    Reg,
    Tcs,
    Trim,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MmState {
    #[default]
    Free,
    Reserved,
    SystemReserved,
    Committed,
    SystemCommitted,
    Locked,
}

#[derive(Clone)]
pub struct MmObject {
    object: Arc<dyn Map>,
    type_id: TypeId,
    perm: MmPerm,
    offset: usize,
    loaded_len: usize,
}

impl MmObject {
    pub fn new(
        object: Arc<dyn Map>,
        type_id: TypeId,
        perm: MmPerm,
        offset: usize,
        loaded_len: usize,
    ) -> MmObject {
        MmObject {
            object,
            type_id,
            perm,
            offset,
            loaded_len,
        }
    }
}

#[derive(Clone)]
pub struct MmArea {
    range: MmRange,
    perm: MmPerm,
    state: MmState,
    typ: MmType,
    mm_object: Option<MmObject>,
}

impl MmArea {
    pub fn new<T: Map + 'static>(
        range: MmRange,
        perm: MmPerm,
        state: MmState,
        typ: MmType,
        object: Option<MapObject<T>>,
    ) -> OsResult<MmArea> {
        let mm_object = object.map(|obj| {
            let offset = obj.offset();
            let perm = obj.perm().into();
            let type_id = TypeId::of::<T>();
            MmObject::new(obj.into_object() as Arc<dyn Map>, type_id, perm, offset, 0)
        });

        let mm_area = MmArea {
            range,
            perm,
            state,
            typ,
            mm_object,
        };
        ensure!(mm_area.check_perm(mm_area.perm), EACCES);

        Ok(mm_area)
    }

    pub fn inherits_from(range: MmRange, vrd: &MmArea) -> MmArea {
        assert!(vrd.is_superset_of(&range));

        let mm_object = vrd.mm_object.as_ref().map(|mm_obj| {
            let new_object = mm_obj.object.clone();

            let (new_offset, new_loaded_len) = match vrd.start().cmp(&range.start()) {
                Ordering::Less => {
                    let vrd_offset = range.start() - vrd.start();
                    (
                        mm_obj.offset + vrd_offset,
                        cmp::min(mm_obj.loaded_len.saturating_sub(vrd_offset), range.size()),
                    )
                }
                Ordering::Equal => (mm_obj.offset, cmp::min(mm_obj.loaded_len, range.size())),
                Ordering::Greater => {
                    // Should not come here
                    let vrd_offset = vrd.start() - range.start();
                    debug_assert!(mm_obj.offset > vrd_offset);
                    (mm_obj.offset - vrd_offset, 0)
                }
            };
            MmObject::new(
                new_object,
                mm_obj.type_id,
                mm_obj.perm,
                new_offset,
                new_loaded_len,
            )
        });

        MmArea {
            range,
            perm: vrd.perm,
            state: vrd.state,
            typ: vrd.typ,
            mm_object,
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn range(&self) -> MmRange {
        self.range
    }

    #[allow(dead_code)]
    #[inline]
    pub fn perm(&self) -> MmPerm {
        self.perm
    }

    #[allow(dead_code)]
    #[inline]
    pub fn state(&self) -> MmState {
        self.state
    }

    #[inline]
    pub fn typ(&self) -> MmType {
        self.typ
    }

    #[allow(dead_code)]
    #[inline]
    pub fn set_state(&mut self, state: MmState) {
        self.state = state;
    }

    #[inline]
    pub fn object(&self) -> &Option<MmObject> {
        &self.mm_object
    }

    #[allow(clippy::vtable_address_comparisons)]
    pub fn can_combine(&self, other: &MmArea) -> bool {
        if !(self.contiguous_with(other)
            && self.perm == other.perm
            && self.state == other.state
            && self.typ == other.typ)
        {
            return false;
        }

        match (self.object(), other.object()) {
            (None, None) => true,
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (Some(mm_obj), Some(other_mm_obj)) => {
                if !Arc::ptr_eq(&mm_obj.object, &other_mm_obj.object) {
                    return false;
                }
                if mm_obj.type_id != other_mm_obj.type_id {
                    return false;
                }
                if mm_obj.perm != other_mm_obj.perm {
                    return false;
                }

                match mm_obj.offset.cmp(&other_mm_obj.offset) {
                    Ordering::Greater if mm_obj.offset - other_mm_obj.offset == other.size() => {
                        true
                    }
                    Ordering::Less if other_mm_obj.offset - mm_obj.offset == self.size() => true,
                    _ => false,
                }
            }
        }
    }

    pub fn intersect(&self, other: &MmRange) -> Option<MmArea> {
        let new_range = self.range().intersect(other)?;
        Some(MmArea::inherits_from(new_range, self))
    }

    pub fn subtract(&self, other: &MmRange) -> Vec<MmArea> {
        self.deref()
            .subtract(other)
            .into_iter()
            .map(|range| MmArea::inherits_from(range, self))
            .collect()
    }

    pub fn load(&mut self) -> OsResult {
        if let Some(mm_obj) = self.mm_object.as_mut() {
            let buf = unsafe { self.range.as_slice_mut() };
            let read_len = mm_obj.object.read_at(buf, mm_obj.offset)?;
            mm_obj.loaded_len = read_len;
            unsafe {
                buf[read_len..]
                    .as_mut_ptr()
                    .write_bytes(0, buf.len() - read_len);
            }
        }
        Ok(())
    }

    pub fn write(&self) -> OsResult {
        if let Some(mm_obj) = self.object() {
            if mm_obj.perm.can_write() {
                let buf = unsafe { &self.as_slice()[..mm_obj.loaded_len] };
                let _written = mm_obj.object.write_at(buf, mm_obj.offset)?;
            }
        }
        Ok(())
    }

    pub fn flush(&self) -> OsResult {
        if let Some(mm_obj) = self.object() {
            if mm_obj.perm.can_write() {
                mm_obj.object.flush()?;
            }
        }
        Ok(())
    }

    pub fn apply_perm<F: Fn(&MmArea, MmPerm) -> bool>(
        &mut self,
        new_perm: MmPerm,
        check_perm: F,
    ) -> OsResult {
        ensure!(check_perm(self, new_perm), EACCES);

        if self.perm == new_perm {
            return Ok(());
        }

        let count = self.size() >> SE_PAGE_SHIFT;
        let perm: ProtectPerm = new_perm.into();

        if trts::is_supported_edmm() {
            let (pe_needed, pr_needed) = self.is_needed_modify_perm(new_perm)?;

            if pe_needed || pr_needed {
                modpr_ocall(self.start(), count, perm).unwrap();
            }

            let pages = PageRange::new(
                self.start(),
                count,
                PageInfo {
                    typ: PageType::Reg,
                    flags: PageFlags::from_bits_truncate(perm.into()) | PageFlags::PR,
                },
            )
            .map_err(|_| EINVAL)?;

            if pe_needed {
                let _ = pages.modpe();
            }

            if pr_needed && new_perm != MmPerm::RWX {
                let _ = pages.accept_forward();
            }

            if pr_needed && new_perm == MmPerm::None {
                mprotect_ocall(self.start(), count, perm).unwrap();
            }
        } else {
            mprotect_ocall(self.start(), count, perm).unwrap();
        }

        self.perm = new_perm;
        Ok(())
    }

    pub fn is_needed_modify_perm(&self, new_perm: MmPerm) -> OsResult<(bool, bool)> {
        ensure!(!self.is_empty(), EFAULT);
        ensure!(
            self.state == MmState::Committed || self.state == MmState::SystemCommitted,
            EFAULT
        );

        let mut pe = false;
        let mut pr = false;
        if (self.perm < new_perm) || (self.perm == MmPerm::RX && new_perm == MmPerm::RW) {
            pe = true;
        }
        if (self.perm > new_perm) || (self.perm == MmPerm::RW && new_perm == MmPerm::RX) {
            pr = true;
        }

        if (self.perm > MmPerm::RW && new_perm < MmPerm::RX)
            || (self.perm < MmPerm::RX && new_perm > MmPerm::RW)
        {
            pr = true;
        }
        Ok((pe, pr))
    }

    pub fn check_perm(&self, perm: MmPerm) -> bool {
        if let Some(mm_obj) = self.mm_object.as_ref() {
            match perm {
                MmPerm::None => true,
                MmPerm::R => mm_obj.perm.can_read(),
                MmPerm::RW => mm_obj.perm.can_read() && perm.can_write(),
                MmPerm::RX => mm_obj.perm.can_read() && perm.can_execute(),
                MmPerm::RWX => mm_obj.perm.can_read() && perm.can_write() && perm.can_execute(),
            }
        } else {
            true
        }
    }
}

impl Deref for MmArea {
    type Target = MmRange;

    fn deref(&self) -> &Self::Target {
        &self.range
    }
}

impl DerefMut for MmArea {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.range
    }
}

impl fmt::Debug for MmArea {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(object) = self.object() {
            fmt.debug_struct("MmArea")
                .field("range", &self.range)
                .field("perm", &self.state)
                .field("state", &self.state)
                .field("typ", &self.typ)
                .field("object", &Arc::as_ptr(&object.object))
                .field("offset", &object.offset)
                .field("loaded_len", &object.loaded_len)
                .finish()
        } else {
            fmt.debug_struct("MmArea")
                .field("range", &self.range)
                .field("perm", &self.state)
                .field("state", &self.state)
                .field("typ", &self.typ)
                .field("object", &Option::<()>::None)
                .finish()
        }
    }
}
