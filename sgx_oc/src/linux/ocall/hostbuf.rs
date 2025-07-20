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

use super::*;
use core::convert::From;
use core::slice;
use sgx_trts::fence;

#[derive(Debug)]
pub struct HostBuffer(HostBufInner);

#[derive(Debug)]
enum HostBufInner {
    Heap(HeapBuffer),
    Oc(OcBuffer),
}

impl !Send for HostBufInner {}
impl !Send for HostBuffer {}

impl HostBuffer {
    const MAX_OCALL_MS_SIZE: usize = 0x100;

    pub fn alloc(size: usize) -> OCallResult<Self> {
        let size = match NonZeroUsize::new(size) {
            Some(size) => size,
            None => {
                bail!(ecust!("Trying to allocate zero byte host memory."));
            }
        };

        let mut remain_size = OcBuffer::remain_size();
        ensure!(remain_size > Self::MAX_OCALL_MS_SIZE, eos!(ENOMEM));
        remain_size -= Self::MAX_OCALL_MS_SIZE;

        if size.get() <= remain_size {
            Ok(HostBuffer(HostBufInner::Oc(OcBuffer::alloc(size)?)))
        } else {
            Ok(HostBuffer(HostBufInner::Heap(HeapBuffer::alloc(size)?)))
        }
    }

    pub fn alloc_zeroed(size: usize) -> OCallResult<Self> {
        let size = match NonZeroUsize::new(size) {
            Some(size) => size,
            None => {
                bail!(ecust!("Trying to allocate zero byte host memory."));
            }
        };

        let mut remain_size = OcBuffer::remain_size();
        ensure!(remain_size > Self::MAX_OCALL_MS_SIZE, eos!(ENOMEM));
        remain_size -= Self::MAX_OCALL_MS_SIZE;

        if size.get() <= remain_size {
            Ok(HostBuffer(HostBufInner::Oc(OcBuffer::alloc_zeroed(size)?)))
        } else {
            Ok(HostBuffer(HostBufInner::Heap(HeapBuffer::alloc_zeroed(
                size,
            )?)))
        }
    }

    pub fn from_enclave_slice(encl_buf: &[u8]) -> OCallResult<Self> {
        let enclsz = encl_buf.len();
        let mut host_buf = Self::alloc(enclsz)?;
        host_buf.write(encl_buf)?;

        Ok(host_buf)
    }

    #[inline]
    pub fn to_enclave_slice(&self, encl_buf: &mut [u8]) -> OCallResult<usize> {
        self.read(encl_buf)
    }

    pub fn as_slice(&self) -> HostSlice<'_> {
        match self.0 {
            HostBufInner::Heap(ref buf) => HostSlice {
                typ: HostSliceType::Heap,
                slice: buf.as_ref(),
            },
            HostBufInner::Oc(ref buf) => HostSlice {
                typ: HostSliceType::Oc,
                slice: buf.as_ref(),
            },
        }
    }

    pub fn as_mut_slice(&mut self) -> HostSliceMut<'_> {
        match self.0 {
            HostBufInner::Heap(ref mut buf) => HostSliceMut {
                typ: HostSliceType::Heap,
                slice: buf.as_mut(),
            },
            HostBufInner::Oc(ref mut buf) => HostSliceMut {
                typ: HostSliceType::Oc,
                slice: buf.as_mut(),
            },
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.as_slice().as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_slice().as_mut_ptr()
    }

    #[inline]
    pub fn write(&mut self, encl_buf: &[u8]) -> OCallResult<usize> {
        let enclsz = encl_buf.len();
        let slice = self.as_mut_slice();
        ensure!(enclsz <= slice.len());

        self.as_mut_slice()
            .range_mut(..enclsz)
            .copy_from_enclave(encl_buf)?;
        Ok(enclsz)
    }

    #[inline]
    pub fn read(&self, encl_buf: &mut [u8]) -> OCallResult<usize> {
        let enclsz = encl_buf.len();
        let slice = self.as_slice();
        ensure!(enclsz <= slice.len());

        self.as_slice().range(..enclsz).copy_to_enclave(encl_buf)?;
        Ok(enclsz)
    }

    pub fn get_range<I>(&self, index: I) -> Option<HostSlice<'_>>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        self.as_slice().get_range(index)
    }

    pub fn get_range_mut<I>(&mut self, index: I) -> Option<HostSliceMut<'_>>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        self.as_mut_slice().get_range_mut(index)
    }

    pub fn range<I>(&self, index: I) -> HostSlice<'_>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        self.as_slice().range(index)
    }

    pub fn range_mut<I>(&mut self, index: I) -> HostSliceMut<'_>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        self.as_mut_slice().range_mut(index)
    }
}

impl HostBuffer {
    #[inline]
    pub unsafe fn from_heap_buffer(heap: HeapBuffer) -> Self {
        Self(HostBufInner::Heap(heap))
    }

    #[inline]
    pub unsafe fn from_oc_buffer(oc: OcBuffer) -> Self {
        Self(HostBufInner::Oc(oc))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HostSliceType {
    Heap,
    Oc,
    Raw,
}

#[derive(Debug)]
pub struct HostSlice<'a> {
    typ: HostSliceType,
    slice: &'a [u8],
}

#[derive(Debug)]
pub struct HostSliceMut<'a> {
    typ: HostSliceType,
    slice: &'a mut [u8],
}

impl<'a> HostSlice<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.as_slice().as_ptr()
    }

    pub fn copy_to_enclave(&self, encl_buf: &mut [u8]) -> OCallResult<()> {
        let enclsz = encl_buf.len();
        ensure!(enclsz == self.len());

        if enclsz > 0 {
            check_trusted_enclave_buffer(encl_buf)?;

            match enclave_mode() {
                EnclaveMode::Hyper => match self.typ {
                    HostSliceType::Oc => encl_buf.copy_from_slice(&self.slice[..enclsz]),
                    HostSliceType::Heap | HostSliceType::Raw => unsafe {
                        read_hostbuf(self.slice, encl_buf)?;
                    },
                },
                _ => encl_buf.copy_from_slice(&self.slice[..enclsz]),
            }
        }
        Ok(())
    }

    pub fn get_range<I>(self, index: I) -> Option<HostSlice<'a>>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        let slice = index.get(self.slice)?;
        Some(HostSlice {
            typ: self.typ,
            slice,
        })
    }

    #[must_use]
    pub fn range<I>(self, index: I) -> HostSlice<'a>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        HostSlice {
            typ: self.typ,
            slice: index.index(self.slice),
        }
    }
}

impl<'a> HostSliceMut<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.as_slice().as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_slice().as_mut_ptr()
    }

    pub fn copy_from_enclave(&mut self, encl_buf: &[u8]) -> OCallResult<()> {
        let enclsz = encl_buf.len();
        ensure!(enclsz == self.len());

        if enclsz > 0 {
            check_trusted_enclave_buffer(encl_buf)?;

            match enclave_mode() {
                EnclaveMode::Hyper => match self.typ {
                    HostSliceType::Oc => self.slice[..enclsz].copy_from_slice(encl_buf),
                    HostSliceType::Heap | HostSliceType::Raw => unsafe {
                        write_hostbuf(&mut self.slice[..enclsz], encl_buf)?;
                    },
                },
                _ => self.slice[..enclsz].copy_from_slice(encl_buf),
            }
        }
        Ok(())
    }

    #[inline]
    pub fn copy_to_enclave(&self, encl_buf: &mut [u8]) -> OCallResult<()> {
        let slice = HostSlice {
            typ: self.typ,
            slice: self.slice,
        };
        slice.copy_to_enclave(encl_buf)
    }

    pub fn get_range<I>(self, index: I) -> Option<HostSlice<'a>>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        let slice: HostSlice = self.into();
        slice.get_range(index)
    }

    pub fn get_range_mut<I>(self, index: I) -> Option<HostSliceMut<'a>>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        let slice = index.get_mut(self.slice)?;
        Some(HostSliceMut {
            typ: self.typ,
            slice,
        })
    }

    pub fn range<I>(self, index: I) -> HostSlice<'a>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        let slice: HostSlice = self.into();
        slice.range(index)
    }

    #[must_use]
    pub fn range_mut<I>(self, index: I) -> HostSliceMut<'a>
    where
        I: SliceIndex<[u8], Output = [u8]>,
    {
        HostSliceMut {
            typ: self.typ,
            slice: index.index_mut(self.slice),
        }
    }
}

impl<'a> HostSlice<'a> {
    pub unsafe fn from_raw_parts(data: *const u8, len: usize) -> OCallResult<Self> {
        ensure!(!data.is_null() && len > 0);

        let slice = slice::from_raw_parts(data, len);
        check_host_buffer(slice)?;
        fence::lfence();

        Ok(Self {
            typ: HostSliceType::Raw,
            slice,
        })
    }
}

impl<'a> HostSliceMut<'a> {
    pub unsafe fn from_raw_parts_mut(data: *mut u8, len: usize) -> OCallResult<Self> {
        ensure!(!data.is_null() && len > 0);

        let slice = slice::from_raw_parts_mut(data, len);
        check_host_buffer(slice)?;
        fence::lfence();

        Ok(Self {
            typ: HostSliceType::Raw,
            slice,
        })
    }
}

impl<'a> HostSlice<'a> {
    #[inline]
    pub(crate) fn as_slice(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> HostSliceMut<'a> {
    #[inline]
    pub(crate) fn as_slice(&self) -> &[u8] {
        self.slice
    }

    #[inline]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [u8] {
        self.slice
    }
}

impl<'a> From<HostSliceMut<'a>> for HostSlice<'a> {
    fn from(slice: HostSliceMut<'a>) -> HostSlice<'a> {
        HostSlice {
            typ: slice.typ,
            slice: slice.slice,
        }
    }
}
