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

use crate::rsrvmm::RsrvMem;
use alloc_crate::sync::Arc;
use core::any::Any;
use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::slice;
use sgx_types::error::errno::*;
use sgx_types::error::OsResult;
use sgx_types::types::ProtectPerm;

#[derive(Debug)]
pub struct Mmap<T: Map + 'static = Nothing> {
    addr: NonNull<u8>,
    size: NonZeroUsize,
    _mark: PhantomData<T>,
}

#[derive(Debug)]
pub struct MmapMut<T: Map + 'static = Nothing> {
    addr: NonNull<u8>,
    size: NonZeroUsize,
    _mark: PhantomData<T>,
}

impl<T: Map + 'static> Mmap<T> {
    #[inline]
    #[allow(clippy::self_named_constructors)]
    pub fn mmap(
        addr: MapAddr,
        size: NonZeroUsize,
        mut map_object: Option<MapObject<T>>,
    ) -> OsResult<Mmap<T>> {
        if let Some(map_obj) = map_object.as_mut() {
            match map_obj.perm {
                ProtectPerm::None => bail!(EACCES),
                _ => map_obj.set_perm(ProtectPerm::Read),
            }
        }

        unsafe {
            mmap(addr, size, Some(ProtectPerm::Read), map_object).map(|addr| Mmap {
                addr,
                size,
                _mark: PhantomData,
            })
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.addr.as_ptr(), self.size.get()) }
    }
}

impl<T: Map + 'static> MmapMut<T> {
    #[inline]
    pub fn mmap(
        addr: MapAddr,
        size: NonZeroUsize,
        mut map_object: Option<MapObject<T>>,
    ) -> OsResult<MmapMut<T>> {
        if let Some(map_obj) = map_object.as_mut() {
            match map_obj.perm {
                ProtectPerm::None => bail!(EACCES),
                ProtectPerm::Read => bail!(EACCES),
                ProtectPerm::ReadWrite => (),
                ProtectPerm::ReadExec => bail!(EACCES),
                ProtectPerm::ReadWriteExec => map_obj.set_perm(ProtectPerm::ReadWrite),
            }
        }

        unsafe {
            mmap(addr, size, Some(ProtectPerm::ReadWrite), map_object).map(|addr| MmapMut {
                addr,
                size,
                _mark: PhantomData,
            })
        }
    }

    #[inline]
    pub fn msync(&mut self) -> OsResult {
        unsafe { msync(self.addr, self.size) }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.addr.as_ptr(), self.size.get()) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.addr.as_ptr(), self.size.get()) }
    }
}

impl<T: Map + 'static> Deref for Mmap<T> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<T: Map + 'static> Deref for MmapMut<T> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<T: Map + 'static> DerefMut for MmapMut<T> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl<T: Map + 'static> Drop for Mmap<T> {
    fn drop(&mut self) {
        unsafe {
            let r = munmap(self.addr, self.size);
            debug_assert!(r.is_ok());
        }
    }
}

impl<T: Map + 'static> Drop for MmapMut<T> {
    fn drop(&mut self) {
        unsafe {
            let r = munmap(self.addr, self.size);
            debug_assert!(r.is_ok());
        }
    }
}

unsafe fn mmap<T: Map + 'static>(
    addr: MapAddr,
    size: NonZeroUsize,
    perm: Option<ProtectPerm>,
    map_object: Option<MapObject<T>>,
) -> OsResult<NonNull<u8>> {
    let rsrvmem = RsrvMem::get_or_init()?;
    let addr = rsrvmem.mmap(addr.into(), size.get(), perm, map_object)?;

    NonNull::new(addr as *mut u8).ok_or(ENOMEM)
}

unsafe fn munmap(addr: NonNull<u8>, size: NonZeroUsize) -> OsResult {
    let rsrvmem = RsrvMem::get_or_init()?;
    rsrvmem.munmap(addr.as_ptr() as usize, size.get())
}

#[allow(dead_code)]
unsafe fn mprotect(addr: NonNull<u8>, size: NonZeroUsize, perm: ProtectPerm) -> OsResult {
    let rsrvmem = RsrvMem::get_or_init()?;
    rsrvmem.mprotect(addr.as_ptr() as usize, size.get(), perm)
}

unsafe fn msync(addr: NonNull<u8>, size: NonZeroUsize) -> OsResult {
    let rsrvmem = RsrvMem::get_or_init()?;
    rsrvmem.msync(addr.as_ptr() as usize, size.get())
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MapAddr {
    /// Free to choose any address
    #[default]
    Any,
    /// Prefer the address, but can use other address
    Hint(NonNull<u8>),
    /// Need to use the address, otherwise report error
    Need(NonNull<u8>),
    /// Force using the address by free first
    Force(NonNull<u8>),
}

pub trait Map: Any {
    /// Reads a number of bytes starting from a given offset.
    ///
    /// Returns the number of bytes read.
    ///
    /// The offset is relative to the start of the file and thus independent
    /// from the current cursor.
    ///
    /// The current file cursor is not affected by this function.
    ///
    /// Note that similar to [`File::read`], it is not an error to return with a
    /// short read.
    fn read_at(&self, buf: &mut [u8], offset: usize) -> OsResult<usize>;

    /// Writes a number of bytes starting from a given offset.
    ///
    /// Returns the number of bytes written.
    ///
    /// The offset is relative to the start of the file and thus independent
    /// from the current cursor.
    ///
    /// The current file cursor is not affected by this function.
    ///
    /// When writing beyond the end of the file, the file is appropriately
    /// extended and the intermediate bytes are initialized with the value 0.
    ///
    /// Note that similar to [`File::write`], it is not an error to return a
    /// short write.
    fn write_at(&self, buf: &[u8], offset: usize) -> OsResult<usize>;

    /// Flush this map obejct, ensuring that all intermediately buffered
    /// contents reach their destination.
    fn flush(&self) -> OsResult;
}

#[derive(Clone)]
pub struct MapObject<T: Map = Nothing> {
    object: Arc<T>,
    perm: ProtectPerm,
    offset: usize,
}

impl<T: Map> MapObject<T> {
    #[inline]
    pub fn new(object: Arc<T>, perm: ProtectPerm, offset: usize) -> MapObject<T> {
        MapObject {
            object,
            perm,
            offset,
        }
    }

    #[inline]
    pub fn zero() -> MapObject<Zero> {
        MapObject::new(Arc::new(Zero), ProtectPerm::ReadWrite, 0)
    }

    #[inline]
    pub fn can_read(&self) -> bool {
        self.perm.can_read()
    }

    #[inline]
    pub fn can_write(&self) -> bool {
        self.perm.can_write()
    }

    #[inline]
    pub fn read(&self, buf: &mut [u8]) -> OsResult<usize> {
        self.object.read_at(buf, self.offset)
    }

    #[inline]
    pub fn write(&mut self, buf: &[u8]) -> OsResult<usize> {
        self.object.write_at(buf, self.offset)
    }

    #[inline]
    pub fn flush(&mut self) -> OsResult {
        self.object.flush()
    }

    #[inline]
    pub fn perm(&self) -> ProtectPerm {
        self.perm
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn set_perm(&mut self, perm: ProtectPerm) {
        self.perm = perm;
    }

    #[inline]
    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    #[inline]
    pub fn as_object(&self) -> &Arc<T> {
        &self.object
    }

    #[inline]
    pub fn into_object(self) -> Arc<T> {
        self.object
    }
}

impl fmt::Debug for MapObject {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("MapObject")
            .field("ptr", &Arc::as_ptr(&self.object))
            .field("perm", &self.perm)
            .field("offset", &self.offset)
            .finish()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Zero;

impl Map for Zero {
    #[inline]
    fn read_at(&self, buf: &mut [u8], _offset: usize) -> OsResult<usize> {
        unsafe {
            buf.as_mut_ptr().write_bytes(0, buf.len());
        }
        Ok(buf.len())
    }

    #[inline]
    fn write_at(&self, buf: &[u8], _offset: usize) -> OsResult<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn flush(&self) -> OsResult {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Nothing;

impl Map for Nothing {
    #[inline]
    fn read_at(&self, buf: &mut [u8], _offset: usize) -> OsResult<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn write_at(&self, buf: &[u8], _offset: usize) -> OsResult<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn flush(&self) -> OsResult {
        Ok(())
    }
}

impl Map for () {
    #[inline]
    fn read_at(&self, buf: &mut [u8], _offset: usize) -> OsResult<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn write_at(&self, buf: &[u8], _offset: usize) -> OsResult<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn flush(&self) -> OsResult {
        Ok(())
    }
}
