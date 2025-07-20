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

use crate::internal::InnerSealedData;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::marker::PhantomData;
use core::mem;
use core::slice;
use sgx_trts::trts::EnclaveRange;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::ContiguousMemory;
use sgx_types::types::{Attributes, KeyPolicy, Mac};

#[cfg(feature = "serialize")]
use sgx_serialize::{Deserialize, Serialize};

/// The structure about the mac data, for authenticate and verify.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
pub struct MacAad<T: ?Sized> {
    inner: InnerSealedData,
    marker: PhantomData<T>,
}

impl<T: ContiguousMemory + ?Sized> MacAad<T> {
    pub fn mac(aad: &T) -> SgxResult<MacAad<T>> {
        let size = mem::size_of_val(aad);
        ensure!(size != 0, SgxStatus::InvalidParameter);

        let aad = unsafe { slice::from_raw_parts(aad as *const _ as *const u8, size) };
        InnerSealedData::mac(aad).map(|inner| MacAad {
            inner,
            marker: PhantomData,
        })
    }

    pub fn mac_with_key_policy(
        key_policy: KeyPolicy,
        attribute_mask: Attributes,
        misc_mask: u32,
        aad: &T,
    ) -> SgxResult<MacAad<T>> {
        let size = mem::size_of_val(aad);
        ensure!(size != 0, SgxStatus::InvalidParameter);

        let aad = unsafe { slice::from_raw_parts(aad as *const _ as *const u8, size) };
        InnerSealedData::mac_with_key_policy(key_policy, attribute_mask, misc_mask, aad).map(
            |inner| MacAad {
                inner,
                marker: PhantomData,
            },
        )
    }

    #[inline]
    pub fn into_bytes(self) -> SgxResult<Vec<u8>> {
        self.inner.into_bytes()
    }

    #[inline]
    pub fn to_bytes(&self) -> SgxResult<Vec<u8>> {
        self.inner.to_bytes()
    }

    #[inline]
    pub fn into_bytes_in<A: Allocator>(self, alloc: A) -> SgxResult<Vec<u8, A>> {
        self.inner.into_bytes_in(alloc)
    }
    #[inline]
    pub fn to_bytes_in<A: Allocator>(&self, alloc: A) -> SgxResult<Vec<u8, A>> {
        self.inner.to_bytes_in(alloc)
    }

    #[inline]
    pub fn from_bytes<A: Allocator>(data: Vec<u8, A>) -> SgxResult<MacAad<T>> {
        InnerSealedData::from_bytes(data).map(|inner| MacAad {
            inner,
            marker: PhantomData,
        })
    }

    #[inline]
    pub fn from_slice(data: &[u8]) -> SgxResult<MacAad<T>> {
        InnerSealedData::from_slice(data).map(|inner| MacAad {
            inner,
            marker: PhantomData,
        })
    }

    #[inline]
    pub fn tag(&self) -> Mac {
        self.inner.payload.tag
    }

    #[inline]
    pub fn payload_size(&self) -> u32 {
        self.inner.payload.len
    }
}

impl<T: ContiguousMemory> MacAad<T> {
    pub fn unmac(self) -> SgxResult<Box<T>> {
        self.inner.verify().map(|inner| {
            let ptr = Box::into_raw(inner.aad);
            unsafe { Box::from_raw(ptr as *mut T) }
        })
    }
}

impl<T: ContiguousMemory> MacAad<[T]> {
    pub fn unmac(self) -> SgxResult<Box<[T]>> {
        self.inner
            .verify()
            .map(|inner| unsafe { mem::transmute(inner.aad) })
    }
}

impl<T: ContiguousMemory + ?Sized> EnclaveRange for MacAad<T> {
    fn is_enclave_range(&self) -> bool {
        self.inner.is_enclave_range()
    }
    fn is_host_range(&self) -> bool {
        self.inner.is_host_range()
    }
}
