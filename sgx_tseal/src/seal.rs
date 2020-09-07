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

//!
//! Intel(R) Software Guard Extensions Sealing and Unsealing Functions
//!
//! # Intel(R) Software Guard Extensions Sealing and Unsealing Functions
//!
//! The API of the model provides the following functions:
//!
//! * Exposes APIs to create sealed data which is both confidentiality andintegrity protected.
//! * Exposes an API to unseal sealed data inside the enclave.
//!
//! The library also provides APIs to help calculate the sealed data size, encrypt text length, and Message Authentication Code (MAC) text length.
//!
use crate::internal::*;
use alloc::boxed::Box;
use alloc::slice;
use core::marker::PhantomData;
use core::mem;
use sgx_types::marker::ContiguousMemory;
use sgx_types::*;

/// The structure about the unsealed data.
pub struct SgxUnsealedData<'a, T: 'a + ?Sized> {
    pub payload_size: u32,
    pub decrypt: Box<T>,
    pub additional: Box<[u8]>,
    marker: PhantomData<&'a T>,
}

impl<'a, T: 'a + ?Sized> SgxUnsealedData<'a, T> {
    ///
    /// Get the payload size of the SgxUnsealedData.
    ///
    pub fn get_payload_size(&self) -> u32 {
        self.payload_size
    }
    ///
    /// Get the pointer of decrypt buffer in SgxUnsealedData.
    ///
    pub fn get_decrypt_txt(&self) -> &T {
        &*self.decrypt
    }
    ///
    /// Get the pointer of additional buffer in SgxUnsealedData.
    ///
    pub fn get_additional_txt(&self) -> &[u8] {
        &*self.additional
    }
}

impl<'a, T: 'a + Default> Default for SgxUnsealedData<'a, T> {
    fn default() -> SgxUnsealedData<'a, T> {
        SgxUnsealedData {
            payload_size: 0_u32,
            decrypt: Box::<T>::default(),
            additional: Box::<[u8]>::default(),
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a + Default> Default for SgxUnsealedData<'a, [T]> {
    fn default() -> SgxUnsealedData<'a, [T]> {
        SgxUnsealedData {
            payload_size: 0_u32,
            decrypt: Box::<[T]>::default(),
            additional: Box::<[u8]>::default(),
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a + Clone + ?Sized> Clone for SgxUnsealedData<'a, T> {
    fn clone(&self) -> SgxUnsealedData<'a, T> {
        SgxUnsealedData {
            payload_size: self.payload_size,
            decrypt: self.decrypt.clone(),
            additional: self.additional.clone(),
            marker: PhantomData,
        }
    }
}

/// The structure about the sealed data.
pub struct SgxSealedData<'a, T: 'a + ?Sized> {
    inner: SgxInternalSealedData,
    marker: PhantomData<&'a T>,
}

impl<'a, T: 'a + ?Sized> Default for SgxSealedData<'a, T> {
    fn default() -> SgxSealedData<'a, T> {
        SgxSealedData {
            inner: SgxInternalSealedData::new(),
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a + Clone + ?Sized> Clone for SgxSealedData<'a, T> {
    fn clone(&self) -> SgxSealedData<'a, T> {
        SgxSealedData {
            inner: self.inner.clone(),
            marker: PhantomData,
        }
    }
}

/// The encrypt_text to seal is T, and T must have Copy and ContiguousMemory trait.
impl<'a, T: 'a + Copy + ContiguousMemory> SgxSealedData<'a, T> {
    ///
    /// This function is used to AES-GCM encrypt the input data. Two input data sets
    /// are provided: one is the data to be encrypted; the second is optional additional data
    /// that will not be encrypted but will be part of the GCM MAC calculation which also covers the data to be encrypted.
    ///
    /// # Description
    ///
    /// The seal_data function retrieves a key unique to the enclave and uses
    /// that key to encrypt the input data buffer. This function can be utilized to preserve secret
    /// data after the enclave is destroyed. The sealed data blob can be
    /// unsealed on future instantiations of the enclave.
    /// The additional data buffer will not be encrypted but will be part of the MAC
    /// calculation that covers the encrypted data as well. This data may include
    /// information about the application, version, data, etc which can be utilized to
    /// identify the sealed data blob since it will remain plain text
    /// Use `calc_raw_sealed_data_size` to calculate the number of bytes to
    /// allocate for the `SgxSealedData` structure. The input sealed data buffer and
    /// text2encrypt buffers must be allocated within the enclave.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Parameters
    ///
    /// **additional_text**
    ///
    /// Pointer to the additional Message Authentication Code (MAC) data.
    /// This additional data is optional and no data is necessary.
    ///
    /// **encrypt_text**
    ///
    /// Pointer to the data stream to be encrypted, which is &T. Must be within the enclave.
    ///
    /// # Return value
    ///
    /// The sealed data in SgxSealedData.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Indicates an error if the parameters do not meet any of the following conditions:
    ///
    /// * additional_text buffer can be within or outside the enclave, but cannot cross the enclave boundary.
    /// * encrypt_text must be non-zero.
    /// * encrypt_text buffer must be within the enclave.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// The enclave is out of memory.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates a crypto library failure or the RDRAND instruction fails to generate a
    /// random number.
    ///
    pub fn seal_data(additional_text: &[u8], encrypt_text: &'a T) -> SgxResult<Self> {
        let size = mem::size_of::<T>();
        if size == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        let encrypt_slice: &[u8] = unsafe {
            slice::from_raw_parts(
                encrypt_text as *const _ as *const u8,
                mem::size_of_val(encrypt_text),
            )
        };
        let result = SgxInternalSealedData::seal_data(additional_text, encrypt_slice);
        result.map(|x| SgxSealedData {
            inner: x,
            marker: PhantomData,
        })
    }

    ///
    /// This function is used to AES-GCM encrypt the input data. Two input data sets
    /// are provided: one is the data to be encrypted; the second is optional additional
    /// data that will not be encrypted but will be part of the GCM MAC calculation
    /// which also covers the data to be encrypted. This is the expert mode
    /// version of function `seal_data`.
    ///
    /// # Descryption
    ///
    /// The `seal_data_ex` is an extended version of `seal_data`. It
    /// provides parameters for you to identify how to derive the sealing key (key
    /// policy and attributes_mask). Typical callers of the seal library should be
    /// able to use `seal_data` and the default values provided for key_
    /// policy (MR_SIGNER) and an attribute mask which includes the RESERVED,
    /// INITED and DEBUG bits. Users of this function should have a clear understanding
    /// of the impact on using a policy and/or attribute_mask that is different from that in seal_data.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Parameters
    ///
    /// **key_policy**
    ///
    /// Specifies the policy to use in the key derivation. Function sgx_seal_data uses the MRSIGNER policy.
    ///
    /// Key policy name | Value | Description
    /// ---|---|---
    /// KEYPOLICY_MRENCLAVE | 0x0001 | -Derive key using the enclave??s ENCLAVE measurement register
    /// KEYPOLICY_MRSIGNER |0x0002 | -Derive key using the enclave??s SIGNER measurement register
    ///
    /// **attribute_mask**
    ///
    /// Identifies which platform/enclave attributes to use in the key derivation. See
    /// the definition of sgx_attributes_t to determine which attributes will be
    /// checked.  Function sgx_seal_data uses flags=0xfffffffffffffff3,?xfrm=0.
    ///
    /// **misc_mask**
    ///
    /// The misc mask bits for the enclave. Reserved for future function extension.
    ///
    /// **additional_text**
    ///
    /// Pointer to the additional Message Authentication Code (MAC) data.
    /// This additional data is optional and no data is necessary.
    ///
    /// **encrypt_text**
    ///
    /// Pointer to the data stream to be encrypted, which is &T. Must not be NULL. Must be within the enclave.
    ///
    /// # Return value
    ///
    /// The sealed data in SgxSealedData.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Indicates an error if the parameters do not meet any of the following conditions:
    ///
    /// * additional_text buffer can be within or outside the enclave, but cannot cross the enclave boundary.
    /// * encrypt_text must be non-zero.
    /// * encrypt_text buffer must be within the enclave.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// The enclave is out of memory.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates a crypto library failure or the RDRAND instruction fails to generate a
    /// random number.
    ///
    pub fn seal_data_ex(
        key_policy: u16,
        attribute_mask: sgx_attributes_t,
        misc_mask: sgx_misc_select_t,
        additional_text: &[u8],
        encrypt_text: &'a T,
    ) -> SgxResult<Self> {
        let size = mem::size_of::<T>();
        if size == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        let encrypt_slice: &[u8] = unsafe {
            slice::from_raw_parts(
                encrypt_text as *const _ as *const u8,
                mem::size_of_val(encrypt_text),
            )
        };
        let result = SgxInternalSealedData::seal_data_ex(
            key_policy,
            attribute_mask,
            misc_mask,
            additional_text,
            encrypt_slice,
        );
        result.map(|x| SgxSealedData {
            inner: x,
            marker: PhantomData,
        })
    }

    ///
    /// This function is used to AES-GCM decrypt the input sealed data structure.
    /// Two output data sets result: one is the decrypted data; the second is the
    /// optional additional data that was part of the GCM MAC calculation but was not
    /// encrypted. This function provides the converse of seal_data and
    /// seal_data_ex.
    ///
    /// # Descryption
    ///
    /// The unseal_data function AES-GCM decrypts the sealed data so that
    /// the enclave data can be restored. This function can be utilized to restore
    /// secret data that was preserved after an earlier instantiation of this enclave
    /// saved this data.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Return value
    ///
    /// The unsealed data in SgxUnsealedData.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The size of T may be zero.
    ///
    /// **SGX_ERROR_INVALID_CPUSVN**
    ///
    /// The CPUSVN in the sealed data blob is beyond the CPUSVN value of the platform.
    /// SGX_ERROR_INVALID_ISVSVN The ISVSVN in the sealed data blob is greater than the ISVSVN value of the enclave.
    ///
    /// **SGX_ERROR_MAC_MISMATCH**
    ///
    /// The tag verification failed during unsealing. The error may be caused by a platform update,
    /// software update, or sealed data blob corruption. This error is also reported if other corruption
    /// of the sealed data structure is detected.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// The enclave is out of memory.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates a crypto library failure or the RDRAND instruction fails to generate a
    /// random number.
    ///
    pub fn unseal_data(&self) -> SgxResult<SgxUnsealedData<'a, T>> {
        let size = mem::size_of::<T>();
        if size == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        let encrypt_len = self.get_encrypt_txt_len() as usize;
        if size != encrypt_len {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        self.inner.unseal_data().map(|x| {
            let ptr = Box::into_raw(x.decrypt);
            SgxUnsealedData {
                payload_size: x.payload_size,
                decrypt: unsafe { Box::from_raw(ptr as *mut T) },
                additional: x.additional,
                marker: PhantomData,
            }
        })
    }

    ///
    /// Convert a pointer of sgx_sealed_data_t buffer to SgxSealedData.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Parameters
    ///
    /// **p**
    ///
    /// The mutable pointer of sgx_sealed_data_t buffer.
    ///
    /// **len**
    ///
    /// The size of the parameter `p`.
    ///
    /// # Return value
    ///
    /// **Some(SgxSealedData)**
    ///
    /// Indicates the conversion is successfully. The return value is SgxSealedData.
    ///
    /// **None**
    ///
    /// Maybe the size of T is zero.
    ///
    pub unsafe fn from_raw_sealed_data_t(p: *mut sgx_sealed_data_t, len: u32) -> Option<Self> {
        let size = mem::size_of::<T>();
        if size == 0 {
            return None;
        }
        let opt = SgxInternalSealedData::from_raw_sealed_data_t(p, len);
        opt.map(|x| SgxSealedData {
            inner: x,
            marker: PhantomData,
        })
    }

    ///
    /// Convert SgxSealedData to the pointer of sgx_sealed_data_t.
    ///
    /// # Parameters
    ///
    /// **p**
    ///
    /// The pointer of sgx_sealed_data_t to save the data in SgxSealedData.
    ///
    /// **len**
    ///
    /// The size of the pointer of sgx_sealed_data_t.
    ///
    /// # Error
    ///
    /// **Some(*mut sgx_sealed_data_t)**
    ///
    /// Indicates the conversion is successfully. The return value is the pointer of sgx_sealed_data_t.
    ///
    /// **None**
    ///
    /// May be the parameter p and len is not avaliable.
    ///
    pub unsafe fn to_raw_sealed_data_t(
        &self,
        p: *mut sgx_sealed_data_t,
        len: u32,
    ) -> Option<*mut sgx_sealed_data_t> {
        self.inner.to_raw_sealed_data_t(p, len)
    }
}

/// The encrypt_text to seal is [T], and T must have Copy and ContiguousMemory trait.
impl<'a, T: 'a + Copy + ContiguousMemory> SgxSealedData<'a, [T]> {
    ///
    /// This function is used to AES-GCM encrypt the input data. Two input data sets
    /// are provided: one is the data to be encrypted; the second is optional additional data
    /// that will not be encrypted but will be part of the GCM MAC calculation which also covers the data to be encrypted.
    ///
    /// # Descryption
    ///
    /// The seal_data function retrieves a key unique to the enclave and uses
    /// that key to encrypt the input data buffer. This function can be utilized to preserve secret
    /// data after the enclave is destroyed. The sealed data blob can be
    /// unsealed on future instantiations of the enclave.
    /// The additional data buffer will not be encrypted but will be part of the MAC
    /// calculation that covers the encrypted data as well. This data may include
    /// information about the application, version, data, etc which can be utilized to
    /// identify the sealed data blob since it will remain plain text
    /// Use `calc_raw_sealed_data_size` to calculate the number of bytes to
    /// allocate for the `SgxSealedData` structure. The input sealed data buffer and
    /// text2encrypt buffers must be allocated within the enclave.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Parameters
    ///
    /// **additional_text**
    ///
    /// Pointer to the additional Message Authentication Code (MAC) data.
    /// This additional data is optional and no data is necessary.
    ///
    /// **encrypt_text**
    ///
    /// Pointer to the data stream to be encrypted, which is &[T]. Must be within the enclave.
    ///
    /// # Return value
    ///
    /// The sealed data in SgxSealedData.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Indicates an error if the parameters do not meet any of the following conditions:
    ///
    /// * additional_text buffer can be within or outside the enclave, but cannot cross the enclave boundary.
    /// * encrypt_text must be non-zero.
    /// * encrypt_text buffer must be within the enclave.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// The enclave is out of memory.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates a crypto library failure or the RDRAND instruction fails to generate a
    /// random number.
    ///
    pub fn seal_data(additional_text: &[u8], encrypt_text: &'a [T]) -> SgxResult<Self> {
        let size = mem::size_of::<T>();
        let len = mem::size_of_val(encrypt_text);
        if size == 0 || len == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        let encrypt_slice: &[u8] =
            unsafe { slice::from_raw_parts(encrypt_text.as_ptr() as *const u8, len) };

        let result = SgxInternalSealedData::seal_data(additional_text, encrypt_slice);
        result.map(|x| SgxSealedData {
            inner: x,
            marker: PhantomData,
        })
    }

    ///
    /// This function is used to AES-GCM encrypt the input data. Two input data sets
    /// are provided: one is the data to be encrypted; the second is optional additional
    /// data that will not be encrypted but will be part of the GCM MAC calculation
    /// which also covers the data to be encrypted. This is the expert mode
    /// version of function `seal_data`.
    ///
    /// # Descryption
    ///
    /// The `seal_data_ex` is an extended version of `seal_data`. It
    /// provides parameters for you to identify how to derive the sealing key (key
    /// policy and attributes_mask). Typical callers of the seal library should be
    /// able to use `seal_data` and the default values provided for key_
    /// policy (MR_SIGNER) and an attribute mask which includes the RESERVED,
    /// INITED and DEBUG bits. Users of this function should have a clear understanding
    /// of the impact on using a policy and/or attribute_mask that is different from that in seal_data.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Parameters
    ///
    /// **key_policy**
    ///
    /// Specifies the policy to use in the key derivation. Function sgx_seal_data uses the MRSIGNER policy.
    ///
    /// Key policy name | Value | Description
    /// ---|---|---
    /// KEYPOLICY_MRENCLAVE | 0x0001 | -Derive key using the enclave??s ENCLAVE measurement register
    /// KEYPOLICY_MRSIGNER |0x0002 | -Derive key using the enclave??s SIGNER measurement register
    ///
    /// **attribute_mask**
    ///
    /// Identifies which platform/enclave attributes to use in the key derivation. See
    /// the definition of sgx_attributes_t to determine which attributes will be
    /// checked.  Function sgx_seal_data uses flags=0xfffffffffffffff3,?xfrm=0.
    ///
    /// **misc_mask**
    ///
    /// The misc mask bits for the enclave. Reserved for future function extension.
    ///
    /// **additional_text**
    ///
    /// Pointer to the additional Message Authentication Code (MAC) data.
    /// This additional data is optional and no data is necessary.
    ///
    /// **encrypt_text**
    ///
    /// Pointer to the data stream to be encrypted, which is &[T]. Must not be NULL. Must be within the enclave.
    ///
    /// # Return value
    ///
    /// The sealed data in SgxSealedData.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Indicates an error if the parameters do not meet any of the following conditions:
    ///
    /// * additional_text buffer can be within or outside the enclave, but cannot cross the enclave boundary.
    /// * encrypt_text must be non-zero.
    /// * encrypt_text buffer must be within the enclave.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// The enclave is out of memory.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates a crypto library failure or the RDRAND instruction fails to generate a
    /// random number.
    ///
    pub fn seal_data_ex(
        key_policy: u16,
        attribute_mask: sgx_attributes_t,
        misc_mask: sgx_misc_select_t,
        additional_text: &[u8],
        encrypt_text: &'a [T],
    ) -> SgxResult<Self> {
        let size = mem::size_of::<T>();
        let len = mem::size_of_val(encrypt_text);
        if size == 0 || len == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        let encrypt_slice: &[u8] =
            unsafe { slice::from_raw_parts(encrypt_text.as_ptr() as *const u8, len) };

        let result = SgxInternalSealedData::seal_data_ex(
            key_policy,
            attribute_mask,
            misc_mask,
            additional_text,
            encrypt_slice,
        );
        result.map(|x| SgxSealedData {
            inner: x,
            marker: PhantomData,
        })
    }

    ///
    /// This function is used to AES-GCM decrypt the input sealed data structure.
    /// Two output data sets result: one is the decrypted data; the second is the
    /// optional additional data that was part of the GCM MAC calculation but was not
    /// encrypted. This function provides the converse of seal_data and
    /// seal_data_ex.
    ///
    /// # Descryption
    ///
    /// The unseal_data function AES-GCM decrypts the sealed data so that
    /// the enclave data can be restored. This function can be utilized to restore
    /// secret data that was preserved after an earlier instantiation of this enclave
    /// saved this data.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Return value
    ///
    /// The unsealed data in SgxUnsealedData.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The size of T may be zero.
    ///
    /// **SGX_ERROR_INVALID_CPUSVN**
    ///
    /// The CPUSVN in the sealed data blob is beyond the CPUSVN value of the platform.
    /// SGX_ERROR_INVALID_ISVSVN The ISVSVN in the sealed data blob is greater than the ISVSVN value of the enclave.
    ///
    /// **SGX_ERROR_MAC_MISMATCH**
    ///
    /// The tag verification failed during unsealing. The error may be caused by a platform update,
    /// software update, or sealed data blob corruption. This error is also reported if other corruption
    /// of the sealed data structure is detected.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// The enclave is out of memory.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates a crypto library failure or the RDRAND instruction fails to generate a
    /// random number.
    ///
    pub fn unseal_data(&self) -> SgxResult<SgxUnsealedData<'a, [T]>> {
        let size = mem::size_of::<T>();
        if size == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        let encrypt_len = self.get_encrypt_txt_len() as usize;
        if size > encrypt_len {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if (encrypt_len % size) != 0 {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }

        self.inner.unseal_data().map(|x| {
            let ptr = Box::into_raw(x.decrypt);
            let slice = unsafe { slice::from_raw_parts_mut(ptr as *mut T, encrypt_len / size) };
            SgxUnsealedData {
                payload_size: x.payload_size,
                decrypt: unsafe { Box::from_raw(slice as *mut [T]) },
                additional: x.additional,
                marker: PhantomData,
            }
        })
    }

    ///
    /// Convert a pointer of sgx_sealed_data_t buffer to SgxSealedData.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Parameters
    ///
    /// **p**
    ///
    /// The mutable pointer of sgx_sealed_data_t buffer.
    ///
    /// **len**
    ///
    /// The size of the parameter `p`.
    ///
    /// # Return value
    ///
    /// **Some(SgxSealedData)**
    ///
    /// Indicates the conversion is successfully. The return value is SgxSealedData.
    ///
    /// **None**
    ///
    /// Maybe the size of T is zero.
    ///
    pub unsafe fn from_raw_sealed_data_t(p: *mut sgx_sealed_data_t, len: u32) -> Option<Self> {
        let size = mem::size_of::<T>();
        if size == 0 {
            return None;
        }
        let opt = SgxInternalSealedData::from_raw_sealed_data_t(p, len);
        opt.map(|x| SgxSealedData {
            inner: x,
            marker: PhantomData,
        })
    }

    ///
    /// Convert SgxSealedData to the pointer of sgx_sealed_data_t.
    ///
    /// # Parameters
    ///
    /// **p**
    ///
    /// The pointer of sgx_sealed_data_t to save the data in SgxSealedData.
    ///
    /// **len**
    ///
    /// The size of the pointer of sgx_sealed_data_t.
    ///
    /// # Error
    ///
    /// **Some(*mut sgx_sealed_data_t)**
    ///
    /// Indicates the conversion is successfully. The return value is the pointer of sgx_sealed_data_t.
    ///
    /// **None**
    ///
    /// May be the parameter p and len is not avaliable.
    ///
    pub unsafe fn to_raw_sealed_data_t(
        &self,
        p: *mut sgx_sealed_data_t,
        len: u32,
    ) -> Option<*mut sgx_sealed_data_t> {
        self.inner.to_raw_sealed_data_t(p, len)
    }
}

impl<'a, T: 'a + ?Sized> SgxSealedData<'a, T> {
    ///
    /// Create a SgxSealedData with default values.
    ///
    pub fn new() -> Self {
        SgxSealedData::default()
    }

    ///
    /// Get the size of payload in SgxSealedData.
    ///
    pub fn get_payload_size(&self) -> u32 {
        self.inner.get_payload_size()
    }

    ///
    /// Get a slice of payload in SgxSealedData.
    ///
    pub fn get_payload_tag(&self) -> &[u8; SGX_SEAL_TAG_SIZE] {
        self.inner.get_payload_tag()
    }

    ///
    /// Get the pointer of sgx_key_request_t in SgxSealedData.
    ///
    pub fn get_key_request(&self) -> &sgx_key_request_t {
        self.inner.get_key_request()
    }

    ///
    /// Get a slice of encrypt text in SgxSealedData.
    ///
    pub fn get_encrypt_txt(&self) -> &[u8] {
        self.inner.get_encrypt_txt()
    }

    ///
    /// Get a slice of additional text in SgxSealedData.
    ///
    pub fn get_additional_txt(&self) -> &[u8] {
        self.inner.get_additional_txt()
    }

    ///
    /// Calculate the size of the sealed data in SgxSealedData.
    ///
    pub fn calc_raw_sealed_data_size(add_mac_txt_size: u32, encrypt_txt_size: u32) -> u32 {
        SgxInternalSealedData::calc_raw_sealed_data_size(add_mac_txt_size, encrypt_txt_size)
    }

    ///
    /// Get the size of the additional mactext in SgxSealedData.
    ///
    pub fn get_add_mac_txt_len(&self) -> u32 {
        self.inner.get_add_mac_txt_len()
    }

    ///
    /// Get the size of the encrypt text in SgxSealedData.
    ///
    pub fn get_encrypt_txt_len(&self) -> u32 {
        self.inner.get_encrypt_txt_len()
    }
}
