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

use crate::internal::InnerSealedData;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::convert::From;
use core::fmt;
use core::marker::PhantomData;
use core::mem;
use core::slice;
use sgx_trts::trts::{is_within_enclave, is_within_host, EnclaveRange};
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::marker::ContiguousMemory;
use sgx_types::types::{Attributes, KeyPolicy, Mac};

#[cfg(feature = "serialize")]
use sgx_serialize::{Deserialize, Serialize};

/// The structure about the sealed data.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
pub struct SealedData<T: ?Sized> {
    inner: InnerSealedData,
    marker: PhantomData<T>,
}

impl<T: ContiguousMemory + ?Sized> SealedData<T> {
    pub fn seal(data: &T, aad: Option<&[u8]>) -> SgxResult<SealedData<T>> {
        let size = mem::size_of_val(data);
        ensure!(size != 0, SgxStatus::InvalidParameter);

        let plaintext = unsafe { slice::from_raw_parts(data as *const _ as *const u8, size) };
        InnerSealedData::seal(plaintext, aad).map(|inner| SealedData {
            inner,
            marker: PhantomData,
        })
    }

    pub fn seal_with_key_policy(
        key_policy: KeyPolicy,
        attribute_mask: Attributes,
        misc_mask: u32,
        data: &T,
        aad: Option<&[u8]>,
    ) -> SgxResult<SealedData<T>> {
        let size = mem::size_of_val(data);
        ensure!(size != 0, SgxStatus::InvalidParameter);

        let plaintext = unsafe { slice::from_raw_parts(data as *const _ as *const u8, size) };
        InnerSealedData::seal_with_key_policy(key_policy, attribute_mask, misc_mask, plaintext, aad)
            .map(|inner| SealedData {
                inner,
                marker: PhantomData,
            })
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
    pub fn from_bytes<A: Allocator>(data: Vec<u8, A>) -> SgxResult<SealedData<T>> {
        InnerSealedData::from_bytes(data).map(|inner| SealedData {
            inner,
            marker: PhantomData,
        })
    }

    #[inline]
    pub fn from_slice(data: &[u8]) -> SgxResult<SealedData<T>> {
        InnerSealedData::from_slice(data).map(|inner| SealedData {
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

impl<T: ContiguousMemory> SealedData<T> {
    pub fn unseal(self) -> SgxResult<UnsealedData<T>> {
        self.inner.unseal().map(|inner| {
            let ptr = Box::into_raw(inner.plaintext);
            UnsealedData {
                payload_size: inner.payload_len,
                plaintext: unsafe { Box::from_raw(ptr as *mut T) },
                aad: inner.aad,
                marker: PhantomData,
            }
        })
    }
}

impl<T: ContiguousMemory> SealedData<[T]> {
    pub fn unseal(self) -> SgxResult<UnsealedData<[T]>> {
        self.inner.unseal().map(|inner| UnsealedData {
            payload_size: inner.payload_len,
            plaintext: unsafe { mem::transmute(inner.plaintext) },
            aad: inner.aad,
            marker: PhantomData,
        })
    }
}

/// The structure about the unsealed data.
pub struct UnsealedData<T: ?Sized> {
    payload_size: u32,
    plaintext: Box<T>,
    aad: Box<[u8]>,
    marker: PhantomData<T>,
}

impl<T: ContiguousMemory + ?Sized> UnsealedData<T> {
    #[inline]
    pub fn payload_size(&self) -> u32 {
        self.payload_size
    }

    #[inline]
    pub fn into_plaintext(self) -> Box<T> {
        self.plaintext
    }

    #[inline]
    pub fn to_plaintext(&self) -> &T {
        &self.plaintext
    }

    #[inline]
    pub fn into_aad(self) -> Box<[u8]> {
        self.aad
    }

    #[inline]
    pub fn to_aad(&self) -> &[u8] {
        &self.aad
    }
}

impl<T: ContiguousMemory> UnsealedData<T> {
    pub fn unseal_from_bytes<A: Allocator>(data: Vec<u8, A>) -> SgxResult<UnsealedData<T>> {
        let sealed_data = SealedData::<T>::from_bytes(data)?;
        sealed_data.unseal()
    }

    pub fn unseal_from_slice(data: &[u8]) -> SgxResult<UnsealedData<T>> {
        let sealed_data = SealedData::<T>::from_slice(data)?;
        sealed_data.unseal()
    }
}

impl<T: ContiguousMemory> UnsealedData<[T]> {
    pub fn unseal_from_bytes<A: Allocator>(data: Vec<u8, A>) -> SgxResult<UnsealedData<[T]>> {
        let sealed_data = SealedData::<[T]>::from_bytes(data)?;
        sealed_data.unseal()
    }

    pub fn unseal_from_slice(data: &[u8]) -> SgxResult<UnsealedData<[T]>> {
        let sealed_data = SealedData::<[T]>::from_slice(data)?;
        sealed_data.unseal()
    }
}

impl<T: ?Sized> From<UnsealedData<T>> for (Box<T>, Box<[u8]>) {
    fn from(unsealed_data: UnsealedData<T>) -> (Box<T>, Box<[u8]>) {
        (unsealed_data.plaintext, unsealed_data.aad)
    }
}

impl<T: Clone + ?Sized> Clone for UnsealedData<T> {
    fn clone(&self) -> UnsealedData<T> {
        UnsealedData {
            payload_size: self.payload_size,
            plaintext: self.plaintext.clone(),
            aad: self.aad.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for UnsealedData<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("UnsealedData")
            .field("payload_size", &self.payload_size)
            .field("plaintext", &self.plaintext)
            .field("aad", &self.aad)
            .finish()
    }
}

impl<T: ContiguousMemory + ?Sized> EnclaveRange for SealedData<T> {
    fn is_enclave_range(&self) -> bool {
        self.inner.is_enclave_range()
    }
    fn is_host_range(&self) -> bool {
        self.inner.is_host_range()
    }
}

impl<T: ContiguousMemory> EnclaveRange for UnsealedData<T> {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(self as *const _ as *const u8, mem::size_of_val(self)) {
            return false;
        }

        if !is_within_enclave(
            self.plaintext.as_ref() as *const _ as *const u8,
            mem::size_of::<T>(),
        ) {
            return false;
        }

        if !self.aad.is_empty() && !is_within_enclave(self.aad.as_ptr(), self.aad.len()) {
            return false;
        }

        true
    }

    fn is_host_range(&self) -> bool {
        if !is_within_host(self as *const _ as *const u8, mem::size_of_val(self)) {
            return false;
        }

        if !is_within_host(
            self.plaintext.as_ref() as *const _ as *const u8,
            mem::size_of::<T>(),
        ) {
            return false;
        }

        if !self.aad.is_empty() && !is_within_host(self.aad.as_ptr(), self.aad.len()) {
            return false;
        }

        true
    }
}

impl<T: ContiguousMemory> EnclaveRange for UnsealedData<[T]> {
    fn is_enclave_range(&self) -> bool {
        if !is_within_enclave(self as *const _ as *const u8, mem::size_of_val(self)) {
            return false;
        }

        if !is_within_enclave(
            self.plaintext.as_ptr() as *const _ as *const u8,
            self.plaintext.len() * mem::size_of::<T>(),
        ) {
            return false;
        }

        if !self.aad.is_empty() && !is_within_enclave(self.aad.as_ptr(), self.aad.len()) {
            return false;
        }

        true
    }

    fn is_host_range(&self) -> bool {
        if !is_within_host(self as *const _ as *const u8, mem::size_of_val(self)) {
            return false;
        }

        if !is_within_host(
            self.plaintext.as_ptr() as *const _ as *const u8,
            self.plaintext.len() * mem::size_of::<T>(),
        ) {
            return false;
        }

        if !self.aad.is_empty() && !is_within_host(self.aad.as_ptr(), self.aad.len()) {
            return false;
        }

        true
    }
}
