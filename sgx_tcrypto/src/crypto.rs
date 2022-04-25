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
//! Cryptographic Functions
//!
use core::cell::{Cell, RefCell};
use core::mem;
use core::ops::{DerefMut, Drop};
use core::ptr;
use sgx_types::marker::ContiguousMemory;
use sgx_types::*;

///
/// The rsgx_sha256_msg function performs a standard SHA256 hash over the input data buffer.
///
/// # Description
///
/// The rsgx_sha256_msg function performs a standard SHA256 hash over the input data buffer.
/// Only a 256-bit version of the SHA hash is supported. (Other sizes, for example 512, are
/// not supported in this minimal cryptography library).
///
/// The function should be used if the complete input data stream is available.
/// Otherwise, the Init, Update… Update, Final procedure should be used to compute
/// a SHA256 bit hash over multiple input data sets.
///
/// # Parameters
///
/// **src**
///
/// A pointer to the input data stream to be hashed.
///
/// # Requirements
///
/// Library: libsgx_tcrypto.a
///
/// # Return value
///
/// The 256-bit hash that has been SHA256 calculated
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Input pointers are invalid.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// The SHA256 hash calculation failed.
///
pub fn rsgx_sha256_msg<T>(src: &T) -> SgxResult<sgx_sha256_hash_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut hash = sgx_sha256_hash_t::default();
    let ret = unsafe {
        sgx_sha256_msg(
            src as *const _ as *const u8,
            size as u32,
            &mut hash as *mut sgx_sha256_hash_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(hash),
        _ => Err(ret),
    }
}

///
/// The rsgx_sha256_slice function performs a standard SHA256 hash over the input data buffer.
///
pub fn rsgx_sha256_slice<T>(src: &[T]) -> SgxResult<sgx_sha256_hash_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut hash = sgx_sha256_hash_t::default();
    let ret = unsafe {
        sgx_sha256_msg(
            src.as_ptr() as *const u8,
            size as u32,
            &mut hash as *mut sgx_sha256_hash_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(hash),
        _ => Err(ret),
    }
}

fn rsgx_sha256_init(sha_handle: &mut sgx_sha_state_handle_t) -> sgx_status_t {
    unsafe { sgx_sha256_init(sha_handle as *mut sgx_sha_state_handle_t) }
}

fn rsgx_sha256_update_msg<T>(src: &T, sha_handle: sgx_sha_state_handle_t) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe { sgx_sha256_update(src as *const _ as *const u8, size as u32, sha_handle) }
}

fn rsgx_sha256_update_slice<T>(src: &[T], sha_handle: sgx_sha_state_handle_t) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    unsafe { sgx_sha256_update(src.as_ptr() as *const u8, size as u32, sha_handle) }
}

fn rsgx_sha256_get_hash(
    sha_handle: sgx_sha_state_handle_t,
    hash: &mut sgx_sha256_hash_t,
) -> sgx_status_t {
    unsafe { sgx_sha256_get_hash(sha_handle, hash as *mut sgx_sha256_hash_t) }
}

fn rsgx_sha256_close(sha_handle: sgx_sha_state_handle_t) -> sgx_status_t {
    unsafe { sgx_sha256_close(sha_handle) }
}

///
/// The rsgx_sha384_msg function performs a standard SHA384 hash over the input data buffer.
///
pub fn rsgx_sha384_msg<T>(src: &T) -> SgxResult<sgx_sha384_hash_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut hash: sgx_sha384_hash_t = [0_u8; SGX_SHA384_HASH_SIZE];
    let ret = unsafe {
        sgx_sha384_msg(
            src as *const _ as *const u8,
            size as u32,
            &mut hash as *mut sgx_sha384_hash_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(hash),
        _ => Err(ret),
    }
}

///
/// The rsgx_sha384_slice function performs a standard SHA384 hash over the input data buffer.
///
pub fn rsgx_sha384_slice<T>(src: &[T]) -> SgxResult<sgx_sha384_hash_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut hash: sgx_sha384_hash_t = [0_u8; SGX_SHA384_HASH_SIZE];
    let ret = unsafe {
        sgx_sha384_msg(
            src.as_ptr() as *const u8,
            size as u32,
            &mut hash as *mut sgx_sha384_hash_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(hash),
        _ => Err(ret),
    }
}

fn rsgx_sha384_init(sha_handle: &mut sgx_sha_state_handle_t) -> sgx_status_t {
    unsafe { sgx_sha384_init(sha_handle as *mut sgx_sha_state_handle_t) }
}

fn rsgx_sha384_update_msg<T>(src: &T, sha_handle: sgx_sha_state_handle_t) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe { sgx_sha384_update(src as *const _ as *const u8, size as u32, sha_handle) }
}

fn rsgx_sha384_update_slice<T>(src: &[T], sha_handle: sgx_sha_state_handle_t) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    unsafe { sgx_sha384_update(src.as_ptr() as *const u8, size as u32, sha_handle) }
}

fn rsgx_sha384_get_hash(
    sha_handle: sgx_sha_state_handle_t,
    hash: &mut sgx_sha384_hash_t,
) -> sgx_status_t {
    unsafe { sgx_sha384_get_hash(sha_handle, hash as *mut sgx_sha384_hash_t) }
}

fn rsgx_sha384_close(sha_handle: sgx_sha_state_handle_t) -> sgx_status_t {
    unsafe { sgx_sha384_close(sha_handle) }
}

pub fn rsgx_sha1_msg<T>(src: &T) -> SgxResult<sgx_sha1_hash_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut hash = sgx_sha1_hash_t::default();
    let ret = unsafe {
        sgx_sha1_msg(
            src as *const _ as *const u8,
            size as u32,
            &mut hash as *mut sgx_sha1_hash_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(hash),
        _ => Err(ret),
    }
}

pub fn rsgx_sha1_slice<T>(src: &[T]) -> SgxResult<sgx_sha1_hash_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut hash = sgx_sha1_hash_t::default();
    let ret = unsafe {
        sgx_sha1_msg(
            src.as_ptr() as *const u8,
            size as u32,
            &mut hash as *mut sgx_sha1_hash_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(hash),
        _ => Err(ret),
    }
}

fn rsgx_sha1_init(sha_handle: &mut sgx_sha_state_handle_t) -> sgx_status_t {
    unsafe { sgx_sha1_init(sha_handle as *mut sgx_sha_state_handle_t) }
}

fn rsgx_sha1_update_msg<T>(src: &T, sha_handle: sgx_sha_state_handle_t) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe { sgx_sha1_update(src as *const _ as *const u8, size as u32, sha_handle) }
}

fn rsgx_sha1_update_slice<T>(src: &[T], sha_handle: sgx_sha_state_handle_t) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    unsafe { sgx_sha1_update(src.as_ptr() as *const u8, size as u32, sha_handle) }
}

fn rsgx_sha1_get_hash(
    sha_handle: sgx_sha_state_handle_t,
    hash: &mut sgx_sha1_hash_t,
) -> sgx_status_t {
    unsafe { sgx_sha1_get_hash(sha_handle, hash as *mut sgx_sha1_hash_t) }
}

fn rsgx_sha1_close(sha_handle: sgx_sha_state_handle_t) -> sgx_status_t {
    unsafe { sgx_sha1_close(sha_handle) }
}

///
/// SHA256 algorithm context state.
///
/// This is a handle to the context state used by the cryptography library to perform an iterative SHA256 hash.
/// The algorithm stores the intermediate results of performing the hash calculation over data sets.
///
pub struct SgxShaHandle {
    handle: RefCell<sgx_sha_state_handle_t>,
    initflag: Cell<bool>,
}

impl SgxShaHandle {
    ///
    /// Constructs a new, empty SgxShaHandle.
    ///
    pub fn new() -> SgxShaHandle {
        SgxShaHandle {
            handle: RefCell::new(ptr::null_mut() as sgx_sha_state_handle_t),
            initflag: Cell::new(false),
        }
    }

    ///
    /// init returns an allocated and initialized SHA algorithm context state.
    ///
    /// This should be part of the Init, Update … Update, Final process when the SHA hash is to be performed
    /// over multiple datasets. If a complete dataset is available, the recommend call is rsgx_sha256_msg to
    /// perform the hash in a single call.
    ///
    /// # Description
    ///
    /// Calling init is the first set in performing a SHA256 hash over multiple datasets. The caller does not
    /// allocate memory for the SHA256 state that this function returns. The state is specific to the implementation
    /// of the cryptography library; thus the allocation is performed by the library itself. If the hash over the
    /// desired datasets is completed or any error occurs during the hash calculation process, sgx_sha256_close should
    /// be called to free the state allocated by this algorithm.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// The SHA256 state is not initialized properly due to an internal cryptography library failure.
    ///
    pub fn init(&self) -> SgxError {
        if self.initflag.get() {
            return Ok(());
        }

        let ret = rsgx_sha256_init(self.handle.borrow_mut().deref_mut());
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(true);
                Ok(())
            }
            _ => Err(ret),
        }
    }

    ///
    /// update_msg performs a SHA256 hash over the input dataset provided.
    ///
    /// This function supports an iterative calculation of the hash over multiple datasets where the
    /// sha_handle contains the intermediate results of the hash calculation over previous datasets.
    ///
    /// # Description
    ///
    /// This function should be used as part of a SHA256 calculation over multiple datasets.
    /// If a SHA256 hash is needed over a single data set, function rsgx_sha256_msg should be used instead.
    /// Prior to calling this function on the first dataset, the init function must be called first to allocate
    /// and initialize the SHA256 state structure which will hold intermediate hash results over earlier datasets.
    /// The function get_hash should be used to obtain the hash after the final dataset has been processed
    /// by this function.
    ///
    /// # Parameters
    ///
    /// **src**
    ///
    /// A pointer to the input data stream to be hashed.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The SHA256 state is not initialized.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// An internal cryptography library failure occurred while performing the SHA256 hash calculation.
    ///
    pub fn update_msg<T>(&self, src: &T) -> SgxError
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_sha256_update_msg(src, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    ///
    /// update_slice performs a SHA256 hash over the input dataset provided.
    ///
    pub fn update_slice<T>(&self, src: &[T]) -> SgxError
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_sha256_update_slice(src, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    ///
    /// get_hash obtains the SHA256 hash after the final dataset has been processed.
    ///
    /// # Description
    ///
    /// This function returns the hash after performing the SHA256 calculation over one or more datasets
    /// using the update function.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Return value
    ///
    /// The 256-bit hash that has been SHA256 calculated
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The SHA256 state is not initialized.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// The SHA256 state passed in is likely problematic causing an internal cryptography library failure.
    ///
    pub fn get_hash(&self) -> SgxResult<sgx_sha256_hash_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut hash = sgx_sha256_hash_t::default();
        let ret = rsgx_sha256_get_hash(*self.handle.borrow(), &mut hash);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(hash),
            _ => Err(ret),
        }
    }

    ///
    /// close cleans up and deallocates the SHA256 state that was allocated in function init.
    ///
    /// # Description
    ///
    /// Calling close is the last step after performing a SHA256 hash over multiple datasets.
    /// The caller uses this function to deallocate memory used to store the SHA256 calculation state.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The input handle is invalid.
    ///
    pub fn close(&self) -> SgxError {
        if !self.initflag.get() {
            return Ok(());
        }

        let ret = {
            let handle = *self.handle.borrow();
            if handle.is_null() {
                sgx_status_t::SGX_SUCCESS
            } else {
                rsgx_sha256_close(handle)
            }
        };

        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(false);
                *self.handle.borrow_mut() = ptr::null_mut();
                Ok(())
            }
            _ => Err(ret),
        }
    }
}

impl Default for SgxShaHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SgxShaHandle {
    ///
    /// drop cleans up and deallocates the SHA256 state that was allocated in function init.
    ///
    fn drop(&mut self) {
        let _ = self.close();
    }
}

///
/// SHA384 algorithm context state.
///
/// This is a handle to the context state used by the cryptography library to perform an iterative SHA384 hash.
/// The algorithm stores the intermediate results of performing the hash calculation over data sets.
///
pub struct SgxSha384Handle {
    handle: RefCell<sgx_sha_state_handle_t>,
    initflag: Cell<bool>,
}

impl SgxSha384Handle {
    ///
    /// Constructs a new, empty SgxShaHandle.
    ///
    pub fn new() -> SgxSha384Handle {
        SgxSha384Handle {
            handle: RefCell::new(ptr::null_mut() as sgx_sha_state_handle_t),
            initflag: Cell::new(false),
        }
    }

    ///
    /// init returns an allocated and initialized SHA384 algorithm context state.
    ///
    pub fn init(&self) -> SgxError {
        if self.initflag.get() {
            return Ok(());
        }

        let ret = rsgx_sha384_init(self.handle.borrow_mut().deref_mut());
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(true);
                Ok(())
            }
            _ => Err(ret),
        }
    }

    ///
    /// update_msg performs a SHA384 hash over the input dataset provided.
    ///
    pub fn update_msg<T>(&self, src: &T) -> SgxError
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_sha384_update_msg(src, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    ///
    /// update_slice performs a SHA384 hash over the input dataset provided.
    ///
    pub fn update_slice<T>(&self, src: &[T]) -> SgxError
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_sha384_update_slice(src, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    ///
    /// get_hash obtains the SHA384 hash after the final dataset has been processed.
    ///
    pub fn get_hash(&self) -> SgxResult<sgx_sha384_hash_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut hash: sgx_sha384_hash_t = [0_u8; SGX_SHA384_HASH_SIZE];
        let ret = rsgx_sha384_get_hash(*self.handle.borrow(), &mut hash);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(hash),
            _ => Err(ret),
        }
    }

    ///
    /// close cleans up and deallocates the SHA384 state that was allocated in function init.
    ///
    pub fn close(&self) -> SgxError {
        if !self.initflag.get() {
            return Ok(());
        }

        let ret = {
            let handle = *self.handle.borrow();
            if handle.is_null() {
                sgx_status_t::SGX_SUCCESS
            } else {
                rsgx_sha384_close(handle)
            }
        };

        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(false);
                *self.handle.borrow_mut() = ptr::null_mut();
                Ok(())
            }
            _ => Err(ret),
        }
    }
}

impl Default for SgxSha384Handle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SgxSha384Handle {
    ///
    /// drop cleans up and deallocates the SHA384 state that was allocated in function init.
    ///
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub struct SgxSha1Handle {
    handle: RefCell<sgx_sha_state_handle_t>,
    initflag: Cell<bool>,
}

impl SgxSha1Handle {
    pub fn new() -> SgxSha1Handle {
        SgxSha1Handle {
            handle: RefCell::new(ptr::null_mut() as sgx_sha_state_handle_t),
            initflag: Cell::new(false),
        }
    }

    pub fn init(&self) -> SgxError {
        if self.initflag.get() {
            return Ok(());
        }

        let ret = rsgx_sha1_init(self.handle.borrow_mut().deref_mut());
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(true);
                Ok(())
            }
            _ => Err(ret),
        }
    }

    pub fn update_msg<T>(&self, src: &T) -> SgxError
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_sha1_update_msg(src, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    pub fn update_slice<T>(&self, src: &[T]) -> SgxError
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_sha1_update_slice(src, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    pub fn get_hash(&self) -> SgxResult<sgx_sha1_hash_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut hash = sgx_sha1_hash_t::default();
        let ret = rsgx_sha1_get_hash(*self.handle.borrow(), &mut hash);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(hash),
            _ => Err(ret),
        }
    }

    pub fn close(&self) -> SgxError {
        if !self.initflag.get() {
            return Ok(());
        }

        let ret = {
            let handle = *self.handle.borrow();
            if handle.is_null() {
                sgx_status_t::SGX_SUCCESS
            } else {
                rsgx_sha1_close(handle)
            }
        };

        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(false);
                *self.handle.borrow_mut() = ptr::null_mut();
                Ok(())
            }
            _ => Err(ret),
        }
    }
}

impl Default for SgxSha1Handle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SgxSha1Handle {
    ///
    /// drop cleans up and deallocates the SHA1 state that was allocated in function init.
    ///
    fn drop(&mut self) {
        let _ = self.close();
    }
}

///
/// rsgx_rijndael128GCM_encrypt performs a Rijndael AES-GCM encryption operation.
///
/// Only a 128bit key size is supported by this Intel(R) SGX SDK cryptography library.
///
/// # Description
///
/// The Galois/Counter Mode (GCM) is a mode of operation of the AES algorithm.
/// GCM [NIST SP 800-38D] uses a variation of the counter mode of operation for
/// encryption. GCM assures authenticity of the confidential data (of up to about
/// 64 GB per invocation) using a universal hash function defined over a binary
/// finite field (the Galois field).
///
/// GCM can also provide authentication assurance for additional data (of practically
/// unlimited length per invocation) that is not encrypted. GCM provides
/// stronger authentication assurance than a (non-cryptographic) checksum or
/// error detecting code. In particular, GCM can detect both accidental modifications
/// of the data and intentional, unauthorized modifications.
///
/// It is recommended that the source and destination data buffers are allocated
/// within the enclave. The AAD buffer could be allocated within or outside
/// enclave memory. The use of AAD data buffer could be information identifying
/// the encrypted data since it will remain in clear text.
///
/// # Parameters
///
/// **key**
///
/// A pointer to key to be used in the AES-GCM encryption operation. The size must be 128 bits.
///
/// **src**
///
/// A pointer to the input data stream to be encrypted. Buffer content could be empty if there is AAD text.
///
/// **iv**
///
/// A pointer to the initialization vector to be used in the AES-GCM calculation. NIST AES-GCM recommended
/// IV size is 96 bits (12 bytes).
///
/// **aad**
///
/// A pointer to an optional additional authentication data buffer which is used in the GCM MAC calculation.
/// The data in this buffer will not be encrypted. The field is optional and content could be empty.
///
/// **dst**
///
/// A pointer to the output encrypted data buffer. This buffer should be allocated by the calling code.
///
/// **mac**
///
/// This is the output GCM MAC performed over the input data buffer (data to be encrypted) as well as
/// the additional authentication data (this is optional data). The calling code should allocate this buffer.
///
/// # Requirements
///
/// Library: libsgx_tcrypto.a
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// If both source buffer and AAD buffer content are empty.
///
/// If IV Length is not equal to 12 (bytes).
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// An internal cryptography library failure occurred.
///
pub fn rsgx_rijndael128GCM_encrypt(
    key: &sgx_aes_gcm_128bit_key_t,
    src: &[u8],
    iv: &[u8],
    aad: &[u8],
    dst: &mut [u8],
    mac: &mut sgx_aes_gcm_128bit_tag_t,
) -> SgxError {
    let src_len = src.len();
    if src_len > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    let iv_len = iv.len();
    if iv_len != SGX_AESGCM_IV_SIZE {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    let aad_len = aad.len();
    if aad_len > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    let dst_len = dst.len();
    if dst_len > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if dst_len < src_len {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let ret = unsafe {
        let p_aad = if aad_len != 0 {
            aad.as_ptr()
        } else {
            ptr::null()
        };

        let (p_src, p_dst) = if src_len != 0 {
            (src.as_ptr(), dst.as_mut_ptr())
        } else {
            (ptr::null(), ptr::null_mut())
        };

        sgx_rijndael128GCM_encrypt(
            key as *const sgx_aes_gcm_128bit_key_t,
            p_src,
            src_len as u32,
            p_dst,
            iv.as_ptr(),
            iv_len as u32,
            p_aad,
            aad_len as u32,
            mac as *mut sgx_aes_gcm_128bit_tag_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

///
/// rsgx_rijndael128GCM_decrypt performs a Rijndael AES-GCM decryption operation.
///
/// Only a 128bit key size is supported by this Intel(R) SGX SDK cryptography library.
///
/// # Description
///
/// The Galois/Counter Mode (GCM) is a mode of operation of the AES algorithm.
/// GCM [NIST SP 800-38D] uses a variation of the counter mode of operation for
/// encryption. GCM assures authenticity of the confidential data (of up to about
/// 64 GB per invocation) using a universal hash function defined over a binary
/// finite field (the Galois field).
///
/// GCM can also provide authentication assurance for additional data (of practically
/// unlimited length per invocation) that is not encrypted. GCM provides
/// stronger authentication assurance than a (non-cryptographic) checksum or
/// error detecting code. In particular, GCM can detect both accidental modifications
/// of the data and intentional, unauthorized modifications.
///
/// It is recommended that the destination data buffer is allocated within the
/// enclave. The AAD buffer could be allocated within or outside enclave memory.
///
/// # Parameters
///
/// **key**
///
/// A pointer to key to be used in the AES-GCM decryption operation. The size must be 128 bits.
///
/// **src**
///
/// A pointer to the input data stream to be decrypted. Buffer content could be empty if there is AAD text.
///
/// **iv**
///
/// A pointer to the initialization vector to be used in the AES-GCM calculation. NIST AES-GCM recommended
/// IV size is 96 bits (12 bytes).
///
/// **aad**
///
/// A pointer to an optional additional authentication data buffer which is provided for the GCM MAC calculation
/// when encrypting. The data in this buffer was not encrypted. The field is optional and content could be empty.
///
/// **mac**
///
/// This is the GCM MAC that was performed over the input data buffer (data to be encrypted) as well as
/// the additional authentication data (this is optional data) during the encryption process (call to
/// rsgx_rijndael128GCM_encrypt).
///
/// **dst**
///
/// A pointer to the output decrypted data buffer. This buffer should be allocated by the calling code.
///
/// # Requirements
///
/// Library: libsgx_tcrypto.a
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// If both source buffer and AAD buffer content are empty.
///
/// If IV Length is not equal to 12 (bytes).
///
/// **SGX_ERROR_MAC_MISMATCH**
///
/// The input MAC does not match the MAC calculated.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// An internal cryptography library failure occurred.
///
pub fn rsgx_rijndael128GCM_decrypt(
    key: &sgx_aes_gcm_128bit_key_t,
    src: &[u8],
    iv: &[u8],
    aad: &[u8],
    mac: &sgx_aes_gcm_128bit_tag_t,
    dst: &mut [u8],
) -> SgxError {
    let src_len = src.len();
    if src_len > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    let iv_len = iv.len();
    if iv_len != SGX_AESGCM_IV_SIZE {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    let aad_len = aad.len();
    if aad_len > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    let dst_len = dst.len();
    if dst_len > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if dst_len < src_len {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let ret = unsafe {
        let p_aad = if !aad.is_empty() {
            aad.as_ptr()
        } else {
            ptr::null()
        };

        let (p_src, p_dst) = if src_len != 0 {
            (src.as_ptr(), dst.as_mut_ptr())
        } else {
            (ptr::null(), ptr::null_mut())
        };

        sgx_rijndael128GCM_decrypt(
            key as *const sgx_aes_gcm_128bit_key_t,
            p_src,
            src_len as u32,
            p_dst,
            iv.as_ptr(),
            iv_len as u32,
            p_aad,
            aad_len as u32,
            mac as *const sgx_aes_gcm_128bit_tag_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

///
/// The rsgx_rijndael128_cmac_msg function performs a standard 128bit CMAC hash over the input data buffer.
///
/// # Description
///
/// The rsgx_rijndael128_cmac_msg function performs a standard CMAC hash over the input data buffer.
/// Only a 128-bit version of the CMAC hash is supported.
///
/// The function should be used if the complete input data stream is available.
/// Otherwise, the Init, Update… Update, Final procedure should be used to compute
/// a CMAC hash over multiple input data sets.
///
/// # Parameters
///
/// **key**
///
/// A pointer to key to be used in the CMAC hash operation. The size must be 128 bits.
///
/// **src**
///
/// A pointer to the input data stream to be hashed.
///
/// # Requirements
///
/// Library: libsgx_tcrypto.a
///
/// # Return value
///
/// The 128-bit hash that has been CMAC calculated
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// The pointer is invalid.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// An internal cryptography library failure occurred.
///
pub fn rsgx_rijndael128_cmac_msg<T>(
    key: &sgx_cmac_128bit_key_t,
    src: &T,
) -> SgxResult<sgx_cmac_128bit_tag_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut mac = sgx_cmac_128bit_tag_t::default();
    let ret = unsafe {
        sgx_rijndael128_cmac_msg(
            key as *const sgx_cmac_128bit_key_t,
            src as *const _ as *const u8,
            size as u32,
            &mut mac as *mut sgx_cmac_128bit_tag_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(mac),
        _ => Err(ret),
    }
}

pub fn rsgx_rijndael128_align_cmac_msg<T>(
    key: &sgx_cmac_128bit_key_t,
    src: &T,
) -> SgxResult<sgx_align_mac_128bit_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut align_mac = sgx_align_mac_128bit_t::default();
    let ret = unsafe {
        sgx_rijndael128_cmac_msg(
            key as *const sgx_cmac_128bit_key_t,
            src as *const _ as *const u8,
            size as u32,
            &mut align_mac.mac as *mut sgx_cmac_128bit_tag_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(align_mac),
        _ => Err(ret),
    }
}

///
/// The rsgx_rijndael128_cmac_slice function performs a standard 128bit CMAC hash over the input data buffer.
///
pub fn rsgx_rijndael128_cmac_slice<T>(
    key: &sgx_cmac_128bit_key_t,
    src: &[T],
) -> SgxResult<sgx_cmac_128bit_tag_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut mac = sgx_cmac_128bit_tag_t::default();
    let ret = unsafe {
        sgx_rijndael128_cmac_msg(
            key as *const sgx_cmac_128bit_key_t,
            src.as_ptr() as *const u8,
            size as u32,
            &mut mac as *mut sgx_cmac_128bit_tag_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(mac),
        _ => Err(ret),
    }
}

pub fn rsgx_rijndael128_align_cmac_slice<T>(
    key: &sgx_cmac_128bit_key_t,
    src: &[T],
) -> SgxResult<sgx_align_mac_128bit_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut align_mac = sgx_align_mac_128bit_t::default();
    let ret = unsafe {
        sgx_rijndael128_cmac_msg(
            key as *const sgx_cmac_128bit_key_t,
            src.as_ptr() as *const u8,
            size as u32,
            &mut align_mac.mac as *mut sgx_cmac_128bit_tag_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(align_mac),
        _ => Err(ret),
    }
}

fn rsgx_cmac128_init(
    key: &sgx_cmac_128bit_key_t,
    cmac_handle: &mut sgx_cmac_state_handle_t,
) -> sgx_status_t {
    unsafe {
        sgx_cmac128_init(
            key as *const sgx_cmac_128bit_key_t,
            cmac_handle as *mut sgx_cmac_state_handle_t,
        )
    }
}

fn rsgx_cmac128_update_msg<T>(src: &T, cmac_handle: sgx_cmac_state_handle_t) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    unsafe { sgx_cmac128_update(src as *const _ as *const u8, size as u32, cmac_handle) }
}

fn rsgx_cmac128_update_slice<T>(src: &[T], cmac_handle: sgx_cmac_state_handle_t) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    unsafe {
        sgx_cmac128_update(
            src.as_ptr() as *const _ as *const u8,
            size as u32,
            cmac_handle,
        )
    }
}

fn rsgx_cmac128_final(
    cmac_handle: sgx_cmac_state_handle_t,
    hash: &mut sgx_cmac_128bit_tag_t,
) -> sgx_status_t {
    unsafe { sgx_cmac128_final(cmac_handle, hash as *mut sgx_cmac_128bit_tag_t) }
}

fn rsgx_cmac128_close(cmac_handle: sgx_cmac_state_handle_t) -> sgx_status_t {
    unsafe { sgx_cmac128_close(cmac_handle) }
}

///
/// CMAC algorithm context state.
///
/// This is a handle to the context state used by the cryptography library to perform an
/// iterative CMAC 128-bit hash. The algorithm stores the intermediate results of performing
/// the hash calculation over data sets.
///
pub struct SgxCmacHandle {
    handle: RefCell<sgx_cmac_state_handle_t>,
    initflag: Cell<bool>,
}

impl SgxCmacHandle {
    ///
    /// Constructs a new, empty SgxCmacHandle.
    ///
    pub fn new() -> SgxCmacHandle {
        SgxCmacHandle {
            handle: RefCell::new(ptr::null_mut() as sgx_cmac_state_handle_t),
            initflag: Cell::new(false),
        }
    }

    ///
    /// init returns an allocated and initialized CMAC algorithm context state.
    ///
    /// This should be part of the Init, Update … Update, Final process when the CMAC hash is to be
    /// performed over multiple datasets. If a complete dataset is available, the recommended call
    /// is rsgx_rijndael128_cmac_msg to perform the hash in a single call.
    ///
    /// # Description
    ///
    /// Calling init is the first set in performing a CMAC 128-bit hash over multiple datasets.
    /// The caller does not allocate memory for the CMAC state that this function returns.
    /// The state is specific to the implementation of the cryptography library and thus the
    /// allocation is performed by the library itself. If the hash over the desired datasets is
    /// completed or any error occurs during the hash calculation process, sgx_cmac128_close should
    /// be called to free the state allocated by this algorithm.
    ///
    /// # Parameters
    ///
    /// **key**
    ///
    /// A pointer to key to be used in the CMAC hash operation. The size must be 128 bits.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// An internal cryptography library failure occurred.
    ///
    pub fn init(&self, key: &sgx_cmac_128bit_key_t) -> SgxError {
        if self.initflag.get() {
            return Ok(());
        }

        let ret = rsgx_cmac128_init(key, self.handle.borrow_mut().deref_mut());
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(true);
                Ok(())
            }
            _ => Err(ret),
        }
    }

    ///
    /// update_msg performs a CMAC 128-bit hash over the input dataset provided.
    ///
    /// This function supports an iterative calculation of the hash over multiple datasets where the
    /// cmac_handle contains the intermediate results of the hash calculation over previous datasets.
    ///
    /// # Description
    ///
    /// This function should be used as part of a CMAC 128-bit hash calculation over
    /// multiple datasets. If a CMAC hash is needed over a single data set, function
    /// rsgx_rijndael128_cmac128_msg should be used instead. Prior to calling
    /// this function on the first dataset, the init function must be called first to
    /// allocate and initialize the CMAC state structure which will hold intermediate
    /// hash results over earlier datasets. The function get_hash should be used
    /// to obtain the hash after the final dataset has been processed by this function.
    ///
    /// # Parameters
    ///
    /// **src**
    ///
    /// A pointer to the input data stream to be hashed.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The CMAC state is not initialized.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// An internal cryptography library failure occurred while performing the CMAC hash calculation.
    ///
    pub fn update_msg<T>(&self, src: &T) -> SgxError
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_cmac128_update_msg(src, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    ///
    /// update_slice performs a CMAC 128-bit hash over the input dataset provided.
    ///
    pub fn update_slice<T>(&self, src: &[T]) -> SgxError
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_cmac128_update_slice(src, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    ///
    /// get_hash obtains the CMAC 128-bit hash after the final dataset has been processed.
    ///
    /// # Description
    ///
    /// This function returns the hash after performing the CMAC 128-bit hash calculation
    /// over one or more datasets using the update function.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Return value
    ///
    /// The 128-bit hash that has been CMAC calculated
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The CMAC state is not initialized.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// The CMAC state passed in is likely problematic causing an internal cryptography library failure.
    ///
    pub fn get_hash(&self) -> SgxResult<sgx_cmac_128bit_tag_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut hash = sgx_cmac_128bit_tag_t::default();
        let ret = rsgx_cmac128_final(*self.handle.borrow(), &mut hash);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(hash),
            _ => Err(ret),
        }
    }

    pub fn get_align_hash(&self) -> SgxResult<sgx_align_mac_128bit_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut align_hash = sgx_align_mac_128bit_t::default();
        let ret = rsgx_cmac128_final(*self.handle.borrow(), &mut align_hash.mac);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(align_hash),
            _ => Err(ret),
        }
    }

    ///
    /// close cleans up and deallocates the CMAC algorithm context state that was allocated in function init.
    ///
    /// # Description
    ///
    /// Calling close is the last step after performing a CMAC hash over multiple datasets.
    /// The caller uses this function to deallocate memory used for storing the CMAC algorithm context state.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The input handle is invalid.
    ///
    pub fn close(&self) -> SgxError {
        if !self.initflag.get() {
            return Ok(());
        }

        let ret = {
            let handle = *self.handle.borrow();
            if handle.is_null() {
                sgx_status_t::SGX_SUCCESS
            } else {
                rsgx_cmac128_close(handle)
            }
        };

        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(false);
                *self.handle.borrow_mut() = ptr::null_mut();
                Ok(())
            }
            _ => Err(ret),
        }
    }
}

impl Default for SgxCmacHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SgxCmacHandle {
    ///
    /// drop cleans up and deallocates the CMAC algorithm context state that was allocated in function init.
    ///
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub fn rsgx_hmac_sha256_msg<T>(
    key: &sgx_hmac_256bit_key_t,
    src: &T,
) -> SgxResult<sgx_hmac_256bit_tag_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut mac = sgx_hmac_256bit_tag_t::default();
    let ret = unsafe {
        sgx_hmac_sha256_msg(
            src as *const _ as *const u8,
            size as i32,
            key as *const u8,
            SGX_HMAC256_KEY_SIZE as i32,
            &mut mac as *mut sgx_hmac_256bit_tag_t as *mut u8,
            SGX_HMAC256_MAC_SIZE as i32,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(mac),
        _ => Err(ret),
    }
}

pub fn rsgx_align_hmac_sha256_msg<T>(
    key: &sgx_hmac_256bit_key_t,
    src: &T,
) -> SgxResult<sgx_align_mac_256bit_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut align_mac = sgx_align_mac_256bit_t::default();
    let ret = unsafe {
        sgx_hmac_sha256_msg(
            src as *const _ as *const u8,
            size as i32,
            key as *const u8,
            SGX_HMAC256_KEY_SIZE as i32,
            &mut align_mac.mac as *mut sgx_hmac_256bit_tag_t as *mut u8,
            SGX_HMAC256_MAC_SIZE as i32,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(align_mac),
        _ => Err(ret),
    }
}

pub fn rsgx_hmac_sha256_slice<T>(
    key: &sgx_hmac_256bit_key_t,
    src: &[T],
) -> SgxResult<sgx_hmac_256bit_tag_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut mac = sgx_hmac_256bit_tag_t::default();
    let ret = unsafe {
        sgx_hmac_sha256_msg(
            src.as_ptr() as *const u8,
            size as i32,
            key as *const u8,
            SGX_HMAC256_KEY_SIZE as i32,
            &mut mac as *mut sgx_hmac_256bit_tag_t as *mut u8,
            SGX_HMAC256_MAC_SIZE as i32,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(mac),
        _ => Err(ret),
    }
}

pub fn rsgx_align_hmac_sha256_slice<T>(
    key: &sgx_hmac_256bit_key_t,
    src: &[T],
) -> SgxResult<sgx_align_mac_256bit_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut align_mac = sgx_align_mac_256bit_t::default();
    let ret = unsafe {
        sgx_hmac_sha256_msg(
            src.as_ptr() as *const u8,
            size as i32,
            key as *const u8,
            SGX_HMAC256_KEY_SIZE as i32,
            &mut align_mac.mac as *mut sgx_hmac_256bit_tag_t as *mut u8,
            SGX_HMAC256_MAC_SIZE as i32,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(align_mac),
        _ => Err(ret),
    }
}

fn rsgx_hmac256_init(
    key: &sgx_hmac_256bit_key_t,
    hmac_handle: &mut sgx_hmac_state_handle_t,
) -> sgx_status_t {
    unsafe {
        sgx_hmac256_init(
            key as *const sgx_hmac_256bit_key_t as *const u8,
            SGX_HMAC256_KEY_SIZE as i32,
            hmac_handle as *mut sgx_hmac_state_handle_t,
        )
    }
}

fn rsgx_hmac256_update_msg<T>(src: &T, hmac_handle: sgx_hmac_state_handle_t) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    unsafe { sgx_hmac256_update(src as *const _ as *const u8, size as i32, hmac_handle) }
}

fn rsgx_hmac256_update_slice<T>(src: &[T], hmac_handle: sgx_hmac_state_handle_t) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(src);
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    unsafe {
        sgx_hmac256_update(
            src.as_ptr() as *const _ as *const u8,
            size as i32,
            hmac_handle,
        )
    }
}

fn rsgx_hmac256_final(
    hmac_handle: sgx_hmac_state_handle_t,
    hash: &mut sgx_hmac_256bit_tag_t,
) -> sgx_status_t {
    unsafe {
        sgx_hmac256_final(
            hash as *mut sgx_hmac_256bit_tag_t as *mut u8,
            SGX_HMAC256_MAC_SIZE as i32,
            hmac_handle,
        )
    }
}

fn rsgx_hmac256_close(hmac_handle: sgx_hmac_state_handle_t) -> sgx_status_t {
    unsafe { sgx_hmac256_close(hmac_handle) }
}

pub struct SgxHmacHandle {
    handle: RefCell<sgx_hmac_state_handle_t>,
    initflag: Cell<bool>,
}

impl SgxHmacHandle {
    pub fn new() -> SgxHmacHandle {
        SgxHmacHandle {
            handle: RefCell::new(ptr::null_mut() as sgx_hmac_state_handle_t),
            initflag: Cell::new(false),
        }
    }

    pub fn init(&self, key: &sgx_hmac_256bit_key_t) -> SgxError {
        if self.initflag.get() {
            return Ok(());
        }

        let ret = rsgx_hmac256_init(key, self.handle.borrow_mut().deref_mut());
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(true);
                Ok(())
            }
            _ => Err(ret),
        }
    }

    pub fn update_msg<T>(&self, src: &T) -> SgxError
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_hmac256_update_msg(src, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    pub fn update_slice<T>(&self, src: &[T]) -> SgxError
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_hmac256_update_slice(src, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    pub fn get_hash(&self) -> SgxResult<sgx_hmac_256bit_tag_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut hash = sgx_hmac_256bit_tag_t::default();
        let ret = rsgx_hmac256_final(*self.handle.borrow(), &mut hash);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(hash),
            _ => Err(ret),
        }
    }

    pub fn get_align_hash(&self) -> SgxResult<sgx_align_mac_256bit_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut align_hash = sgx_align_mac_256bit_t::default();
        let ret = rsgx_hmac256_final(*self.handle.borrow(), &mut align_hash.mac);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(align_hash),
            _ => Err(ret),
        }
    }

    pub fn close(&self) -> SgxError {
        if !self.initflag.get() {
            return Ok(());
        }

        let ret = {
            let handle = *self.handle.borrow();
            if handle.is_null() {
                sgx_status_t::SGX_SUCCESS
            } else {
                rsgx_hmac256_close(handle)
            }
        };

        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(false);
                *self.handle.borrow_mut() = ptr::null_mut();
                Ok(())
            }
            _ => Err(ret),
        }
    }
}

impl Default for SgxHmacHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SgxHmacHandle {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub const SGX_AESCTR_CTR_SIZE: size_t = 16;
pub type sgx_aes_ctr_128bit_ctr_t = [uint8_t; SGX_AESCTR_CTR_SIZE];

///
/// rsgx_aes_ctr_encrypt performs a Rijndael AES-CTR encryption operation.
///
/// Only a 128bit key size is supported by this Intel(R) SGX SDK cryptography library.
///
/// # Description
///
/// This function encrypts the input data stream of a variable length according to
/// the CTR mode as specified in [NIST SP 800-38A]. The counter can be thought
/// of as an IV which increments on successive encryption or decryption calls. For
/// a given dataset or data stream, the incremented counter block should be used
/// on successive calls of the encryption process for that given stream. However,
/// for new or different datasets/streams, the same counter should not be reused,
/// instead initialize the counter for the new data set.
///
/// It is recommended that the source, destination and counter data buffers are
/// allocated within the enclave.
///
/// # Parameters
///
/// **key**
///
/// A pointer to key to be used in the AES-CTR encryption operation. The size must be 128 bits.
///
/// **src**
///
/// A pointer to the input data stream to be encrypted.
///
/// **ctr**
///
/// A pointer to the initialization vector to be used in the AES-CTR calculation.
///
/// **ctr_inc_bits**
///
/// Specifies the number of bits in the counter to be incremented.
///
/// **dst**
///
/// A pointer to the output encrypted data buffer. This buffer should be allocated by the calling code.
///
/// # Requirements
///
/// Library: libsgx_tcrypto.a
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// The pointer is invalid.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// An internal cryptography library failure occurred.
///
pub fn rsgx_aes_ctr_encrypt(
    key: &sgx_aes_ctr_128bit_key_t,
    src: &[u8],
    ctr: &mut sgx_aes_ctr_128bit_ctr_t,
    ctr_inc_bits: u32,
    dst: &mut [u8],
) -> SgxError {
    let src_len = src.len();
    if src_len > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if src_len < 1 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    let dst_len = dst.len();
    if dst_len > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if dst_len < src_len {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let ret = unsafe {
        sgx_aes_ctr_encrypt(
            key as *const sgx_aes_ctr_128bit_key_t,
            src.as_ptr(),
            src_len as u32,
            ctr as *mut sgx_aes_ctr_128bit_ctr_t as *mut u8,
            ctr_inc_bits,
            dst.as_mut_ptr(),
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

///
/// rsgx_aes_ctr_decrypt performs a Rijndael AES-CTR decryption operation.
///
/// Only a 128bit key size is supported by this Intel(R) SGX SDK cryptography library.
///
/// # Description
///
/// This function decrypts the input data stream of a variable length according to
/// the CTR mode as specified in [NIST SP 800-38A]. The counter can be thought
/// of as an IV which increments on successive encryption or decryption calls. For
/// a given dataset or data stream, the incremented counter block should be used
/// on successive calls of the decryption process for that given stream. However,
/// for new or different datasets/streams, the same counter should not be reused,
/// instead initialize the counter for the new data set.
///
/// It is recommended that the source, destination and counter data buffers are
/// allocated within the enclave.
///
/// # Parameters
///
/// **key**
///
/// A pointer to key to be used in the AES-CTR encryption operation. The size must be 128 bits.
///
/// **src**
///
/// A pointer to the input data stream to be decrypted.
///
/// **ctr**
///
/// A pointer to the initialization vector to be used in the AES-CTR calculation.
///
/// **ctr_inc_bits**
///
/// Specifies the number of bits in the counter to be incremented.
///
/// **dst**
///
/// A pointer to the output decrypted data buffer. This buffer should be allocated by the calling code.
///
/// # Requirements
///
/// Library: libsgx_tcrypto.a
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// The pointer is invalid.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// An internal cryptography library failure occurred.
///
pub fn rsgx_aes_ctr_decrypt(
    key: &sgx_aes_ctr_128bit_key_t,
    src: &[u8],
    ctr: &mut sgx_aes_ctr_128bit_ctr_t,
    ctr_inc_bits: u32,
    dst: &mut [u8],
) -> SgxError {
    let src_len = src.len();
    if src_len > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if src_len < 1 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    let dst_len = dst.len();
    if dst_len > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if dst_len < src_len {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let ret = unsafe {
        sgx_aes_ctr_decrypt(
            key as *const sgx_aes_ctr_128bit_key_t,
            src.as_ptr(),
            src.len() as u32,
            ctr as *mut sgx_aes_ctr_128bit_ctr_t as *mut u8,
            ctr_inc_bits,
            dst.as_mut_ptr(),
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

fn rsgx_ecc256_open_context(ecc_handle: &mut sgx_ecc_state_handle_t) -> sgx_status_t {
    unsafe { sgx_ecc256_open_context(ecc_handle as *mut _ as *mut sgx_ecc_state_handle_t) }
}

fn rsgx_ecc256_close_context(ecc_handle: sgx_ecc_state_handle_t) -> sgx_status_t {
    unsafe { sgx_ecc256_close_context(ecc_handle) }
}

fn rsgx_ecc256_create_key_pair(
    private: &mut sgx_ec256_private_t,
    public: &mut sgx_ec256_public_t,
    ecc_handle: sgx_ecc_state_handle_t,
) -> sgx_status_t {
    unsafe {
        sgx_ecc256_create_key_pair(
            private as *mut sgx_ec256_private_t,
            public as *mut sgx_ec256_public_t,
            ecc_handle,
        )
    }
}

fn rsgx_ecc256_check_point(
    point: &sgx_ec256_public_t,
    ecc_handle: sgx_ecc_state_handle_t,
    valid: &mut i32,
) -> sgx_status_t {
    unsafe {
        sgx_ecc256_check_point(
            point as *const sgx_ec256_public_t,
            ecc_handle,
            valid as *mut i32,
        )
    }
}

fn rsgx_ecc256_compute_shared_dhkey(
    private_b: &sgx_ec256_private_t,
    public_ga: &sgx_ec256_public_t,
    shared_key: &mut sgx_ec256_dh_shared_t,
    ecc_handle: sgx_ecc_state_handle_t,
) -> sgx_status_t {
    unsafe {
        sgx_ecc256_compute_shared_dhkey(
            private_b as *const sgx_ec256_private_t,
            public_ga as *const sgx_ec256_public_t,
            shared_key as *mut sgx_ec256_dh_shared_t,
            ecc_handle,
        )
    }
}

/* delete (intel sgx sdk 2.0)
fn rsgx_ecc256_compute_shared_dhkey512(
    private_b: &sgx_ec256_private_t,
    public_ga: &sgx_ec256_public_t,
    shared_key: &mut sgx_ec256_dh_shared512_t,
    ecc_handle: sgx_ecc_state_handle_t,
) -> sgx_status_t {
    unsafe {
        sgx_ecc256_compute_shared_dhkey512(
            private_b as *const _ as *mut sgx_ec256_private_t,
            public_ga as *const _ as *mut sgx_ec256_public_t,
            shared_key as *mut sgx_ec256_dh_shared512_t,
            ecc_handle,
        )
    }
}
*/

fn rsgx_ecdsa_sign_msg<T>(
    data: &T,
    private: &sgx_ec256_private_t,
    signature: &mut sgx_ec256_signature_t,
    ecc_handle: sgx_ecc_state_handle_t,
) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        sgx_ecdsa_sign(
            data as *const _ as *const u8,
            size as u32,
            private as *const sgx_ec256_private_t,
            signature as *mut sgx_ec256_signature_t,
            ecc_handle,
        )
    }
}

fn rsgx_ecdsa_sign_slice<T>(
    data: &[T],
    private: &sgx_ec256_private_t,
    signature: &mut sgx_ec256_signature_t,
    ecc_handle: sgx_ecc_state_handle_t,
) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(data);
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        sgx_ecdsa_sign(
            data.as_ptr() as *const _ as *const u8,
            size as u32,
            private as *const sgx_ec256_private_t,
            signature as *mut sgx_ec256_signature_t,
            ecc_handle,
        )
    }
}

fn rsgx_ecdsa_verify_msg<T>(
    data: &T,
    public: &sgx_ec256_public_t,
    signature: &sgx_ec256_signature_t,
    result: &mut sgx_generic_ecresult_t,
    ecc_handle: sgx_ecc_state_handle_t,
) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        let mut verify: u8 = 0;
        let ret = sgx_ecdsa_verify(
            data as *const _ as *const u8,
            size as u32,
            public as *const sgx_ec256_public_t,
            signature as *const sgx_ec256_signature_t,
            &mut verify as *mut u8,
            ecc_handle,
        );
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                let ecresult = sgx_generic_ecresult_t::from_repr(u32::from(verify));
                *result = ecresult.unwrap_or(sgx_generic_ecresult_t::SGX_EC_INVALID_SIGNATURE);
            }
            _ => {
                *result = sgx_generic_ecresult_t::SGX_EC_INVALID_SIGNATURE;
            }
        };
        ret
    }
}

fn rsgx_ecdsa_verify_slice<T>(
    data: &[T],
    public: &sgx_ec256_public_t,
    signature: &sgx_ec256_signature_t,
    result: &mut sgx_generic_ecresult_t,
    ecc_handle: sgx_ecc_state_handle_t,
) -> sgx_status_t
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(data);
    if size == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if size > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        let mut verify: u8 = 0;
        let ret = sgx_ecdsa_verify(
            data.as_ptr() as *const _ as *const u8,
            size as u32,
            public as *const sgx_ec256_public_t,
            signature as *const sgx_ec256_signature_t,
            &mut verify as *mut u8,
            ecc_handle,
        );
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                let ecresult = sgx_generic_ecresult_t::from_repr(u32::from(verify));
                *result = ecresult.unwrap_or(sgx_generic_ecresult_t::SGX_EC_INVALID_SIGNATURE);
            }
            _ => {
                *result = sgx_generic_ecresult_t::SGX_EC_INVALID_SIGNATURE;
            }
        };
        ret
    }
}

fn rsgx_ecdsa_verify_hash(
    hash: &sgx_sha256_hash_t,
    public: &sgx_ec256_public_t,
    signature: &sgx_ec256_signature_t,
    result: &mut sgx_generic_ecresult_t,
    ecc_handle: sgx_ecc_state_handle_t,
) -> sgx_status_t {
    unsafe {
        let mut verify: u8 = 0;
        let ret = sgx_ecdsa_verify_hash(
            hash as *const sgx_sha256_hash_t as *const u8,
            public as *const sgx_ec256_public_t,
            signature as *const sgx_ec256_signature_t,
            &mut verify as *mut u8,
            ecc_handle,
        );
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                let ecresult = sgx_generic_ecresult_t::from_repr(u32::from(verify));
                *result = ecresult.unwrap_or(sgx_generic_ecresult_t::SGX_EC_INVALID_SIGNATURE);
            }
            _ => {
                *result = sgx_generic_ecresult_t::SGX_EC_INVALID_SIGNATURE;
            }
        };
        ret
    }
}

///
/// ECC GF(p) context state.
///
/// This is a handle to the ECC GF(p) context state allocated and initialized used to perform
/// elliptic curve cryptosystem standard functions. The algorithm stores the intermediate results
/// of calculations performed using this context.
///
pub struct SgxEccHandle {
    handle: RefCell<sgx_ecc_state_handle_t>,
    initflag: Cell<bool>,
}

impl SgxEccHandle {
    ///
    /// Constructs a new, empty SgxEccHandle.
    ///
    pub fn new() -> SgxEccHandle {
        SgxEccHandle {
            handle: RefCell::new(ptr::null_mut() as sgx_ecc_state_handle_t),
            initflag: Cell::new(false),
        }
    }

    ///
    /// open returns an allocated and initialized context for the elliptic curve cryptosystem
    /// over a prime finite field, GF(p).
    ///
    /// This context must be created prior to calling create_key_pair or compute_shared_dhkey.
    /// When the calling code has completed its set of ECC operations, close should be called to
    /// cleanup and deallocate the ECC context.
    ///
    /// # Description
    ///
    /// open is utilized to allocate and initialize a 256-bit
    /// GF(p) cryptographic system. The caller does not allocate memory for the ECC
    /// state that this function returns. The state is specific to the implementation of
    /// the cryptography library and thus the allocation is performed by the library
    /// itself. If the ECC cryptographic function using this cryptographic system is completed
    /// or any error occurs, close should be called to free the state allocated by this algorithm.
    ///
    /// Public key cryptography successfully allows to solving problems of information
    /// safety by enabling trusted communication over insecure channels. Although
    /// elliptic curves are well studied as a branch of mathematics, an interest to the
    /// cryptographic schemes based on elliptic curves is constantly rising due to the
    /// advantages that the elliptic curve algorithms provide in the wireless communications:
    /// shorter processing time and key length.
    ///
    /// Elliptic curve cryptosystems (ECCs) implement a different way of creating public
    /// keys. As elliptic curve calculation is based on the addition of the rational
    /// points in the (x,y) plane and it is difficult to solve a discrete logarithm from
    /// these points, a higher level of safety is achieved through the cryptographic
    /// schemes that use the elliptic curves. The cryptographic systems that encrypt
    /// messages by using the properties of elliptic curves are hard to attack due to
    /// the extreme complexity of deciphering the private key.
    ///
    /// Using of elliptic curves allows shorter public key length and encourages cryptographers
    /// to create cryptosystems with the same or higher encryption
    /// strength as the RSA or DSA cryptosystems. Because of the relatively short key
    /// length, ECCs do encryption and decryption faster on the hardware that
    /// requires less computation processing volumes.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// The ECC context state was not initialized properly due to an internal cryptography library failure.
    ///
    pub fn open(&self) -> SgxError {
        if self.initflag.get() {
            return Ok(());
        }

        let ret = rsgx_ecc256_open_context(self.handle.borrow_mut().deref_mut());
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(true);
                Ok(())
            }
            _ => Err(ret),
        }
    }

    ///
    /// create_key_pair generates a private/public key pair on the ECC curve for the given
    /// cryptographic system.
    ///
    /// open must be called to allocate and initialize the ECC context prior to making this call.
    ///
    /// # Description
    ///
    /// This function populates private/public key pair. The calling code allocates
    /// memory for the private and public key pointers to be populated. The function
    /// generates a private key p_private and computes a public key p_public of
    /// the elliptic cryptosystem over a finite field GF(p).
    ///
    /// The private key p_private is a number that lies in the range of [1, n-1]
    /// where n is the order of the elliptic curve base point.
    /// The public key p_public is an elliptic curve point such that p_public =
    /// p_private *G, where G is the base point of the elliptic curve.
    /// The context of the point p_public as an elliptic curve point must be created
    /// by using the function open.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Return value
    ///
    /// **sgx_ec256_private_t**
    ///
    /// The private key which is a number that lies in the range of [1, n-1] where n is the order
    /// of the elliptic curve base point.
    ///
    /// **sgx_ec256_public_t**
    ///
    /// The public key which is an elliptic curve point such that:
    ///
    /// public key = private key * G, where G is the base point of the elliptic curve.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The ECC state is not initialized.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// The key creation process failed due to an internal cryptography library failure.
    ///
    pub fn create_key_pair(&self) -> SgxResult<(sgx_ec256_private_t, sgx_ec256_public_t)> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut private = sgx_ec256_private_t::default();
        let mut public = sgx_ec256_public_t::default();
        let ret = rsgx_ecc256_create_key_pair(&mut private, &mut public, *self.handle.borrow());

        match ret {
            sgx_status_t::SGX_SUCCESS => Ok((private, public)),
            _ => Err(ret),
        }
    }

    pub fn create_align_key_pair(
        &self,
    ) -> SgxResult<(sgx_align_ec256_private_t, sgx_ec256_public_t)> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut private = sgx_align_ec256_private_t::default();
        let mut public = sgx_ec256_public_t::default();
        let ret = rsgx_ecc256_create_key_pair(&mut private.key, &mut public, *self.handle.borrow());

        match ret {
            sgx_status_t::SGX_SUCCESS => Ok((private, public)),
            _ => Err(ret),
        }
    }

    ///
    /// check_point checks whether the input point is a valid point on the ECC curve for the given cryptographic system.
    ///
    /// open context must be called to allocate and initialize the ECC context prior to making this call.
    ///
    /// # Description
    ///
    /// check_point validates whether the input point is a valid point on the ECC curve for the given cryptographic system.
    ///
    /// # Parameters
    ///
    /// **point**
    ///
    /// A pointer to the point to perform validity check on.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Return value
    ///
    /// **true**
    ///
    /// The input point is valid
    ///
    /// **false**
    ///
    /// The input point is not valid
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The ECC state is not initialized.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// An internal cryptography library failure occurred.
    ///
    pub fn check_point(&self, point: &sgx_ec256_public_t) -> SgxResult<bool> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut valid: i32 = 0;
        let ret = rsgx_ecc256_check_point(point, *self.handle.borrow(), &mut valid);
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                if valid > 0 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Err(ret),
        }
    }

    ///
    /// compute_shared_dhkey generates a secret key shared between two participants of the cryptosystem.
    ///
    /// # Description
    ///
    /// This function computes the Diffie-Hellman shared key based on the enclave’s
    /// own (local) private key and remote enclave’s public Ga Key.
    ///
    /// The function computes a secret number sharedKey, which is a secret key
    /// shared between two participants of the cryptosystem.
    ///
    /// In cryptography, metasyntactic names such as Alice as Bob are normally used
    /// as examples and in discussions and stand for participant A and participant B.
    ///
    /// Both participants (Alice and Bob) use the cryptosystem for receiving a common
    /// secret point on the elliptic curve called a secret key (sharedKey). To
    /// receive a secret key, participants apply the Diffie-Hellman key-agreement
    /// scheme involving public key exchange. The value of the secret key entirely
    /// depends on participants.
    ///
    /// According to the scheme, Alice and Bob perform the following operations:
    ///
    /// 1. Alice calculates her own public key pubKeyA by using her private key
    /// privKeyA: pubKeyA = privKeyA * G, where G is the base point of the
    /// elliptic curve.
    ///
    /// 2. Alice passes the public key to Bob.
    ///
    /// 3. Bob calculates his own public key pubKeyB by using his private key
    /// privKeyB: pubKeyB = privKeyB * G, where G is a base point of the elliptic curve.
    ///
    /// 4. Bob passes the public key to Alice.
    ///
    /// 5. Alice gets Bob's public key and calculates the secret point shareKeyA. When
    /// calculating, she uses her own private key and Bob's public key and applies the
    /// following formula:
    ///
    /// shareKeyA = privKeyA * pubKeyB = privKeyA * privKeyB * G.
    ///
    /// 6. Bob gets Alice's public key and calculates the secret point shareKeyB. When
    /// calculating, he uses his own private key and Alice's public key and applies the
    /// following formula:
    ///
    /// shareKeyB = privKeyB * pubKeyA = privKeyB * privKeyA * G.
    ///
    /// As the following equation is true privKeyA * privKeyB * G =
    /// privKeyB * privKeyA * G, the result of both calculations is the same,
    /// that is, the equation shareKeyA = shareKeyB is true. The secret point serves as
    /// a secret key.
    ///
    /// Shared secret shareKey is an x-coordinate of the secret point on the elliptic
    /// curve. The elliptic curve domain parameters must be hitherto defined by the
    /// function: open.
    ///
    /// # Parameters
    ///
    /// **private_b**
    ///
    /// A pointer to the local private key.
    ///
    /// **public_ga**
    ///
    /// A pointer to the remote public key.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Return value
    ///
    /// The secret key generated by this function which is a common point on the elliptic curve.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The ECC state is not initialized.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// The key creation process failed due to an internal cryptography library failure.
    ///
    pub fn compute_shared_dhkey(
        &self,
        private_b: &sgx_ec256_private_t,
        public_ga: &sgx_ec256_public_t,
    ) -> SgxResult<sgx_ec256_dh_shared_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut shared_key = sgx_ec256_dh_shared_t::default();
        let ret = rsgx_ecc256_compute_shared_dhkey(
            private_b,
            public_ga,
            &mut shared_key,
            *self.handle.borrow(),
        );
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(shared_key),
            _ => Err(ret),
        }
    }

    pub fn compute_align_shared_dhkey(
        &self,
        private_b: &sgx_ec256_private_t,
        public_ga: &sgx_ec256_public_t,
    ) -> SgxResult<sgx_align_ec256_dh_shared_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut shared_key = sgx_align_ec256_dh_shared_t::default();
        let ret = rsgx_ecc256_compute_shared_dhkey(
            private_b,
            public_ga,
            &mut shared_key.key,
            *self.handle.borrow(),
        );
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(shared_key),
            _ => Err(ret),
        }
    }

    /* delete (intel sgx sdk 2.0)
    pub fn compute_shared_dhkey512(&self, private_b: &sgx_ec256_private_t, public_ga: &sgx_ec256_public_t) -> SgxResult<sgx_ec256_dh_shared512_t> {
        if self.initflag.get() == false {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut shared_key = sgx_ec256_dh_shared512_t::default();
        let ret = rsgx_ecc256_compute_shared_dhkey512(private_b, public_ga, &mut shared_key, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(shared_key),
            _ => Err(ret),
        }
    }
    */

    ///
    /// ecdsa_sign_msg computes a digital signature with a given private key over an input dataset.
    ///
    /// # Description
    ///
    /// This function computes a digital signature over the input dataset based on the
    /// put private key.
    ///
    /// A message digest is a fixed size number derived from the original message
    // with an applied hash function over the binary code of the message. (SHA256
    /// in this case)
    ///
    /// The signer's private key and the message digest are used to create a signature.
    ///
    /// A digital signature over a message consists of a pair of large numbers, 256-bits
    /// each, which the given function computes.
    ///
    /// The scheme used for computing a digital signature is of the ECDSA scheme, an
    /// elliptic curve of the DSA scheme.
    ///
    /// The keys can be generated and set up by the function: create_key_pair.
    ///
    /// The elliptic curve domain parameters must be created by function: open.
    ///
    /// # Parameters
    ///
    /// **data**
    ///
    /// A pointer to the data to calculate the signature over.
    ///
    /// **private**
    ///
    /// A pointer to the private key to be used in the calculation of the signature.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Return value
    ///
    /// The signature generated by this function.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The ECC state is not initialized.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// The signature generation process failed due to an internal cryptography library failure.
    ///
    pub fn ecdsa_sign_msg<T>(
        &self,
        data: &T,
        private: &sgx_ec256_private_t,
    ) -> SgxResult<sgx_ec256_signature_t>
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut signature = sgx_ec256_signature_t::default();
        let ret = rsgx_ecdsa_sign_msg(data, private, &mut signature, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(signature),
            _ => Err(ret),
        }
    }

    ///
    /// ecdsa_sign_slice computes a digital signature with a given private key over an input dataset.
    ///
    pub fn ecdsa_sign_slice<T>(
        &self,
        data: &[T],
        private: &sgx_ec256_private_t,
    ) -> SgxResult<sgx_ec256_signature_t>
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut signature = sgx_ec256_signature_t::default();
        let ret = rsgx_ecdsa_sign_slice(data, private, &mut signature, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(signature),
            _ => Err(ret),
        }
    }

    ///
    /// ecdsa_verify_msg verifies the input digital signature with a given public key over an input dataset.
    ///
    /// # Description
    ///
    /// This function verifies the signature for the given data set based on the input public key.
    ///
    /// A digital signature over a message consists of a pair of large numbers, 256-bits
    /// each, which could be created by function: sgx_ecdsa_sign. The scheme
    /// used for computing a digital signature is of the ECDSA scheme, an elliptic
    /// curve of the DSA scheme.
    ///
    /// The elliptic curve domain parameters must be created by function: open.
    ///
    /// # Parameters
    ///
    /// **data**
    ///
    /// A pointer to the signed dataset to verify.
    ///
    /// **public**
    ///
    /// A pointer to the public key to be used in the calculation of the signature.
    ///
    /// **signature**
    ///
    /// A pointer to the signature to be verified.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Return value
    ///
    /// **true**
    ///
    /// Digital signature is valid.
    ///
    /// **false**
    ///
    /// Digital signature is not valid.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The pointer is invalid.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The ECC state is not initialized.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// The verification process failed due to an internal cryptography library failure.
    ///
    pub fn ecdsa_verify_msg<T>(
        &self,
        data: &T,
        public: &sgx_ec256_public_t,
        signature: &sgx_ec256_signature_t,
    ) -> SgxResult<bool>
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut result = sgx_generic_ecresult_t::default();
        let ret =
            rsgx_ecdsa_verify_msg(data, public, signature, &mut result, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => match result {
                sgx_generic_ecresult_t::SGX_EC_VALID => Ok(true),
                _ => Ok(false),
            },
            _ => Err(ret),
        }
    }

    ///
    /// ecdsa_verify_slice verifies the input digital signature with a given public key over an input dataset.
    ///
    pub fn ecdsa_verify_slice<T>(
        &self,
        data: &[T],
        public: &sgx_ec256_public_t,
        signature: &sgx_ec256_signature_t,
    ) -> SgxResult<bool>
    where
        T: Copy + ContiguousMemory,
    {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut result = sgx_generic_ecresult_t::default();
        let ret =
            rsgx_ecdsa_verify_slice(data, public, signature, &mut result, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => match result {
                sgx_generic_ecresult_t::SGX_EC_VALID => Ok(true),
                _ => Ok(false),
            },
            _ => Err(ret),
        }
    }

    pub fn ecdsa_verify_hash(
        &self,
        hash: &sgx_sha256_hash_t,
        public: &sgx_ec256_public_t,
        signature: &sgx_ec256_signature_t,
    ) -> SgxResult<bool> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let mut result = sgx_generic_ecresult_t::default();
        let ret =
            rsgx_ecdsa_verify_hash(hash, public, signature, &mut result, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => match result {
                sgx_generic_ecresult_t::SGX_EC_VALID => Ok(true),
                _ => Ok(false),
            },
            _ => Err(ret),
        }
    }

    ///
    /// close cleans up and deallocates the ECC 256 GF(p) state that was allocated in function open.
    ///
    /// # Description
    ///
    /// close is used by calling code to deallocate memory used for storing the ECC 256 GF(p) state used
    /// in ECC cryptographic calculations.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tcrypto.a
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// The input handle is invalid.
    ///
    pub fn close(&self) -> SgxError {
        if !self.initflag.get() {
            return Ok(());
        }

        let ret = {
            let handle = *self.handle.borrow();
            if handle.is_null() {
                sgx_status_t::SGX_SUCCESS
            } else {
                rsgx_ecc256_close_context(handle)
            }
        };

        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(false);
                *self.handle.borrow_mut() = ptr::null_mut();
                Ok(())
            }
            _ => Err(ret),
        }
    }
}

impl Default for SgxEccHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SgxEccHandle {
    ///
    /// close cleans up and deallocates the ECC 256 GF(p) state that was allocated in function open.
    ///
    fn drop(&mut self) {
        let _ = self.close();
    }
}

///
/// The rsgx_rsa3072_sign_msg computes a digital signature for a given dataset based on RSA 3072 private key.
///
/// # Description
///
/// This function computes a digital signature over the input dataset based on the RSA 3072 private key.
///
/// A message digest is a fixed size number derived from the original message with an applied hash function
/// over the binary code of the message. (SHA256 in this case)
///
/// The signer's private key and the message digest are used to create a signature.
///
/// The scheme used for computing a digital signature is of the RSASSA-PKCS1-v1_5 scheme.
///
/// # Parameters
///
/// **data**
///
/// A pointer to the data to calculate the signature over.
///
/// **key**
///
/// A pointer to the RSA key.
///
/// # Requirements
///
/// Library: libsgx_tcrypto.a
///
/// # Return value
///
/// The signature generated by this function.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// The RSA key, data is NULL. Or the data size is 0.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// The signature generation process failed due to an internal cryptography library failure.
///
pub fn rsgx_rsa3072_sign_msg<T>(
    data: &T,
    key: &sgx_rsa3072_key_t,
) -> SgxResult<sgx_rsa3072_signature_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut sign = sgx_rsa3072_signature_t::default();
    let ret = unsafe {
        sgx_rsa3072_sign(
            data as *const _ as *const u8,
            size as u32,
            key as *const sgx_rsa3072_key_t,
            &mut sign as *mut sgx_rsa3072_signature_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(sign),
        _ => Err(ret),
    }
}

///
/// The rsgx_rsa3072_sign_slice computes a digital signature for a given dataset based on RSA 3072 private key.
///
pub fn rsgx_rsa3072_sign_slice<T>(
    data: &[T],
    key: &sgx_rsa3072_key_t,
) -> SgxResult<sgx_rsa3072_signature_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(data);
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut sign = sgx_rsa3072_signature_t::default();
    let ret = unsafe {
        sgx_rsa3072_sign(
            data.as_ptr() as *const _ as *const u8,
            size as u32,
            key as *const sgx_rsa3072_key_t,
            &mut sign as *mut sgx_rsa3072_signature_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(sign),
        _ => Err(ret),
    }
}

///
/// The rsgx_rsa3072_sign_msg_ex computes a digital signature for a given dataset based on RSA 3072 private key
/// and the optional corresponding RSA 3072 public key.
///
/// # Description
///
/// This function computes a digital signature over the input dataset based on the RSA 3072 private key.
///
/// A message digest is a fixed size number derived from the original message with an applied hash function
/// over the binary code of the message. (SHA256 in this case)
///
/// The signer's private key and the message digest are used to create a signature.
///
/// The scheme used for computing a digital signature is of the RSASSA-PKCS1-v1_5 scheme.
///
/// # Parameters
///
/// **data**
///
/// A pointer to the data to calculate the signature over.
///
/// **key**
///
/// A pointer to the RSA private key.
///
/// **public**
///
/// A pointer to the RSA public key. Can be None.
///
/// # Requirements
///
/// Library: libsgx_tcrypto.a
///
/// # Return value
///
/// The signature generated by this function.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// The private key, data is NULL. Or the data size is 0. Or the RSA private key and the public key do not match.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// The signature generation process failed due to an internal cryptography library failure.
///
pub fn rsgx_rsa3072_sign_msg_ex<T>(
    data: &T,
    key: &sgx_rsa3072_key_t,
    public: Option<&sgx_rsa3072_public_key_t>,
) -> SgxResult<sgx_rsa3072_signature_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut sign = sgx_rsa3072_signature_t::default();
    let ret = unsafe {
        sgx_rsa3072_sign_ex(
            data as *const _ as *const u8,
            size as u32,
            key as *const sgx_rsa3072_key_t,
            public.map_or(ptr::null(), |key| key as *const sgx_rsa3072_public_key_t),
            &mut sign as *mut sgx_rsa3072_signature_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(sign),
        _ => Err(ret),
    }
}

///
/// The rsgx_rsa3072_sign_slice_ex computes signature for a given data based on RSA 3072 private key
/// and the optional corresponding RSA 3072 public key.
///
pub fn rsgx_rsa3072_sign_slice_ex<T>(
    data: &[T],
    key: &sgx_rsa3072_key_t,
    public: Option<&sgx_rsa3072_public_key_t>,
) -> SgxResult<sgx_rsa3072_signature_t>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(data);
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut sign = sgx_rsa3072_signature_t::default();
    let ret = unsafe {
        sgx_rsa3072_sign_ex(
            data.as_ptr() as *const _ as *const u8,
            size as u32,
            key as *const sgx_rsa3072_key_t,
            public.map_or(ptr::null(), |key| key as *const sgx_rsa3072_public_key_t),
            &mut sign as *mut sgx_rsa3072_signature_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(sign),
        _ => Err(ret),
    }
}

///
/// rsgx_rsa3072_verify_msg verifies the input digital signature for the given data- set based on the RSA 3072 public key.
///
/// # Description
///
/// This function verifies the signature for the given data set based on the input RSA 3072 public key.
///
/// A digital signature over a message is a buffer of 384-bytes, which could be created by function: rsgx_rsa3072_sign.
/// The scheme used for computing a digital signature is of the RSASSA-PKCS1-v1_5 scheme.
///
/// # Parameters
///
/// **data**
///
/// A pointer to the signed dataset to be verified.
///
/// **public**
///
/// A pointer to the public key to be used in the calculation of the signature.
///
/// **signature**
///
/// A pointer to the signature to be verified.
///
/// # Requirements
///
/// Library: libsgx_tcrypto.a
///
/// # Return value
///
/// **true**
///
/// Digital signature is valid.
///
/// **false**
///
/// Digital signature is not valid.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// The private key, data is NULL. Or the data size is 0.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// The verification process failed due to an internal cryptography library failure.
///
pub fn rsgx_rsa3072_verify_msg<T>(
    data: &T,
    public: &sgx_rsa3072_public_key_t,
    signature: &sgx_rsa3072_signature_t,
) -> SgxResult<bool>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of::<T>();
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    unsafe {
        let mut verify = sgx_rsa_result_t::SGX_RSA_INVALID_SIGNATURE;
        let ret = sgx_rsa3072_verify(
            data as *const _ as *const u8,
            size as u32,
            public as *const sgx_rsa3072_public_key_t,
            signature as *const sgx_rsa3072_signature_t,
            &mut verify as *mut sgx_rsa_result_t,
        );
        match ret {
            sgx_status_t::SGX_SUCCESS => match verify {
                sgx_rsa_result_t::SGX_RSA_VALID => Ok(true),
                _ => Ok(false),
            },
            _ => Err(ret),
        }
    }
}

///
/// rsgx_rsa3072_verify_slice verifies the input digital signature for the given data- set based on the RSA 3072 public key.
///
pub fn rsgx_rsa3072_verify_slice<T>(
    data: &[T],
    public: &sgx_rsa3072_public_key_t,
    signature: &sgx_rsa3072_signature_t,
) -> SgxResult<bool>
where
    T: Copy + ContiguousMemory,
{
    let size = mem::size_of_val(data);
    if size == 0 {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if size > u32::MAX as usize {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    unsafe {
        let mut verify = sgx_rsa_result_t::SGX_RSA_INVALID_SIGNATURE;
        let ret = sgx_rsa3072_verify(
            data.as_ptr() as *const _ as *const u8,
            size as u32,
            public as *const sgx_rsa3072_public_key_t,
            signature as *const sgx_rsa3072_signature_t,
            &mut verify as *mut sgx_rsa_result_t,
        );
        match ret {
            sgx_status_t::SGX_SUCCESS => match verify {
                sgx_rsa_result_t::SGX_RSA_VALID => Ok(true),
                _ => Ok(false),
            },
            _ => Err(ret),
        }
    }
}

#[allow(clippy::many_single_char_names)]
pub fn rsgx_create_rsa_key_pair(
    n_byte_size: i32,
    e_byte_size: i32,
    n: &mut [u8],
    d: &mut [u8],
    e: &mut [u8],
    p: &mut [u8],
    q: &mut [u8],
    dmp1: &mut [u8],
    dmq1: &mut [u8],
    iqmp: &mut [u8],
) -> SgxError {
    if (n_byte_size <= 0) || (e_byte_size <= 0) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (n.is_empty()) || (n.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (d.is_empty()) || (d.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (e.is_empty()) || (e.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (p.is_empty()) || (p.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (q.is_empty()) || (q.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (dmp1.is_empty()) || (dmp1.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (dmq1.is_empty()) || (dmq1.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (iqmp.is_empty()) || (iqmp.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let ret = unsafe {
        sgx_create_rsa_key_pair(
            n_byte_size,
            e_byte_size,
            n.as_mut_ptr(),
            d.as_mut_ptr(),
            e.as_mut_ptr(),
            p.as_mut_ptr(),
            q.as_mut_ptr(),
            dmp1.as_mut_ptr(),
            dmq1.as_mut_ptr(),
            iqmp.as_mut_ptr(),
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

fn rsgx_create_rsa_priv2_key(
    mod_size: i32,
    exp_size: i32,
    e: &[u8],
    p: &[u8],
    q: &[u8],
    dmp1: &[u8],
    dmq1: &[u8],
    iqmp: &[u8],
    new_pri_key: &mut sgx_rsa_key_t,
) -> sgx_status_t {
    if (mod_size <= 0) || (exp_size <= 0) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (e.is_empty()) || (e.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (p.is_empty()) || (p.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (q.is_empty()) || (q.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (dmp1.is_empty()) || (dmp1.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (dmq1.is_empty()) || (dmq1.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (iqmp.is_empty()) || (iqmp.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        sgx_create_rsa_priv2_key(
            mod_size,
            exp_size,
            e.as_ptr(),
            p.as_ptr(),
            q.as_ptr(),
            dmp1.as_ptr(),
            dmq1.as_ptr(),
            iqmp.as_ptr(),
            new_pri_key as *mut sgx_rsa_key_t,
        )
    }
}

fn rsgx_create_rsa_priv1_key(
    n_size: i32,
    e_size: i32,
    d_size: i32,
    n: &[u8],
    e: &[u8],
    d: &[u8],
    new_pri_key: &mut sgx_rsa_key_t,
) -> sgx_status_t {
    if (n_size <= 0) || (e_size <= 0) || (d_size <= 0) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (n.is_empty()) || (n.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (e.is_empty()) || (e.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (d.is_empty()) || (d.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        sgx_create_rsa_priv1_key(
            n_size,
            e_size,
            d_size,
            n.as_ptr(),
            e.as_ptr(),
            d.as_ptr(),
            new_pri_key as *mut sgx_rsa_key_t,
        )
    }
}

fn rsgx_create_rsa_pub1_key(
    mod_size: i32,
    exp_size: i32,
    n: &[u8],
    e: &[u8],
    new_pub_key: &mut sgx_rsa_key_t,
) -> sgx_status_t {
    if (mod_size <= 0) || (exp_size <= 0) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (n.is_empty()) || (n.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if (e.is_empty()) || (e.len() > i32::MAX as usize) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        sgx_create_rsa_pub1_key(
            mod_size,
            exp_size,
            n.as_ptr(),
            e.as_ptr(),
            new_pub_key as *mut sgx_rsa_key_t,
        )
    }
}

fn rsgx_free_rsa_key(
    rsa_key: sgx_rsa_key_t,
    key_type: sgx_rsa_key_type_t,
    mod_size: i32,
    exp_size: i32,
) -> sgx_status_t {
    if (mod_size <= 0) || (exp_size <= 0) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    unsafe { sgx_free_rsa_key(rsa_key, key_type, mod_size, exp_size) }
}

fn rsgx_rsa_priv_decrypt_sha256(
    rsa_key: sgx_rsa_key_t,
    out_data: &mut [u8],
    out_len: &mut usize,
    in_data: &[u8],
) -> sgx_status_t {
    if in_data.is_empty() {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if *out_len != 0 && out_data.len() != *out_len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        let p_out_data: *mut u8 = if *out_len != 0 {
            out_data.as_mut_ptr()
        } else {
            ptr::null_mut()
        };
        sgx_rsa_priv_decrypt_sha256(
            rsa_key,
            p_out_data,
            out_len as *mut usize,
            in_data.as_ptr(),
            in_data.len(),
        )
    }
}

fn rsgx_rsa_pub_encrypt_sha256(
    rsa_key: sgx_rsa_key_t,
    out_data: &mut [u8],
    out_len: &mut usize,
    in_data: &[u8],
) -> sgx_status_t {
    if in_data.is_empty() {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if *out_len != 0 && out_data.len() != *out_len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        let p_out_data: *mut u8 = if *out_len != 0 {
            out_data.as_mut_ptr()
        } else {
            ptr::null_mut()
        };
        sgx_rsa_pub_encrypt_sha256(
            rsa_key,
            p_out_data,
            out_len as *mut usize,
            in_data.as_ptr(),
            in_data.len(),
        )
    }
}

pub struct SgxRsaPrivKey {
    key: RefCell<sgx_rsa_key_t>,
    mod_size: Cell<i32>,
    exp_size: Cell<i32>,
    createflag: Cell<bool>,
}

impl SgxRsaPrivKey {
    pub fn new() -> SgxRsaPrivKey {
        SgxRsaPrivKey {
            key: RefCell::new(ptr::null_mut() as sgx_rsa_key_t),
            mod_size: Cell::new(0),
            exp_size: Cell::new(0),
            createflag: Cell::new(false),
        }
    }

    #[inline]
    pub fn create(
        &self,
        mod_size: i32,
        exp_size: i32,
        e: &[u8],
        p: &[u8],
        q: &[u8],
        dmp1: &[u8],
        dmq1: &[u8],
        iqmp: &[u8],
    ) -> SgxError {
        self.create2(mod_size, exp_size, e, p, q, dmp1, dmq1, iqmp)
    }

    pub fn create2(
        &self,
        mod_size: i32,
        exp_size: i32,
        e: &[u8],
        p: &[u8],
        q: &[u8],
        dmp1: &[u8],
        dmq1: &[u8],
        iqmp: &[u8],
    ) -> SgxError {
        if self.createflag.get() {
            return Ok(());
        }

        let ret = rsgx_create_rsa_priv2_key(
            mod_size,
            exp_size,
            e,
            p,
            q,
            dmp1,
            dmq1,
            iqmp,
            self.key.borrow_mut().deref_mut(),
        );
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.mod_size.set(mod_size);
                self.exp_size.set(exp_size);
                self.createflag.set(true);
                Ok(())
            }
            _ => Err(ret),
        }
    }

    pub fn create1(
        &self,
        mod_size: i32,
        exp_size: i32,
        priv_exp_size: i32,
        n: &[u8],
        e: &[u8],
        d: &[u8],
    ) -> SgxError {
        if self.createflag.get() {
            return Ok(());
        }

        let ret = rsgx_create_rsa_priv1_key(
            mod_size,
            exp_size,
            priv_exp_size,
            n,
            e,
            d,
            self.key.borrow_mut().deref_mut(),
        );
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.mod_size.set(mod_size);
                self.exp_size.set(exp_size);
                self.createflag.set(true);
                Ok(())
            }
            _ => Err(ret),
        }
    }

    pub fn decrypt_sha256(
        &self,
        out_data: &mut [u8],
        out_len: &mut usize,
        in_data: &[u8],
    ) -> SgxError {
        if !self.createflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_rsa_priv_decrypt_sha256(*self.key.borrow(), out_data, out_len, in_data);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    pub fn free(&self) -> SgxError {
        if !self.createflag.get() {
            return Ok(());
        }

        let ret = {
            let key = *self.key.borrow();
            if key.is_null() {
                sgx_status_t::SGX_SUCCESS
            } else {
                rsgx_free_rsa_key(
                    key,
                    sgx_rsa_key_type_t::SGX_RSA_PRIVATE_KEY,
                    self.mod_size.get(),
                    self.exp_size.get(),
                )
            }
        };

        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.createflag.set(false);
                *self.key.borrow_mut() = ptr::null_mut();
                Ok(())
            }
            _ => Err(ret),
        }
    }
}

impl Default for SgxRsaPrivKey {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SgxRsaPrivKey {
    fn drop(&mut self) {
        let _ = self.free();
    }
}

pub struct SgxRsaPubKey {
    key: RefCell<sgx_rsa_key_t>,
    mod_size: Cell<i32>,
    exp_size: Cell<i32>,
    createflag: Cell<bool>,
}

impl SgxRsaPubKey {
    pub fn new() -> SgxRsaPubKey {
        SgxRsaPubKey {
            key: RefCell::new(ptr::null_mut() as sgx_rsa_key_t),
            mod_size: Cell::new(0),
            exp_size: Cell::new(0),
            createflag: Cell::new(false),
        }
    }

    pub fn create(&self, mod_size: i32, exp_size: i32, n: &[u8], e: &[u8]) -> SgxError {
        if self.createflag.get() {
            return Ok(());
        }

        let ret =
            rsgx_create_rsa_pub1_key(mod_size, exp_size, n, e, self.key.borrow_mut().deref_mut());
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.mod_size.set(mod_size);
                self.exp_size.set(exp_size);
                self.createflag.set(true);
                Ok(())
            }
            _ => Err(ret),
        }
    }

    pub fn encrypt_sha256(
        &self,
        out_data: &mut [u8],
        out_len: &mut usize,
        in_data: &[u8],
    ) -> SgxError {
        if !self.createflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ret = rsgx_rsa_pub_encrypt_sha256(*self.key.borrow(), out_data, out_len, in_data);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    pub fn free(&self) -> SgxError {
        if !self.createflag.get() {
            return Ok(());
        }

        let ret = {
            let key = *self.key.borrow();
            if key.is_null() {
                sgx_status_t::SGX_SUCCESS
            } else {
                rsgx_free_rsa_key(
                    key,
                    sgx_rsa_key_type_t::SGX_RSA_PUBLIC_KEY,
                    self.mod_size.get(),
                    self.exp_size.get(),
                )
            }
        };

        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.createflag.set(false);
                *self.key.borrow_mut() = ptr::null_mut();
                Ok(())
            }
            _ => Err(ret),
        }
    }
}

impl Default for SgxRsaPubKey {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SgxRsaPubKey {
    fn drop(&mut self) {
        let _ = self.free();
    }
}

///
/// rsgx_calculate_ecdsa_priv_key generates an ECDSA private key based on an input random seed.
///
/// # Description
///
/// This function generates an ECDSA private key based on an input random seed.
///
/// # Parameters
///
/// **hash_drg**
///
/// Pointer to the input random seed.
///
/// **sgx_nistp256_r_m1**
///
/// Pointer to the buffer for n-1 where n is order of the ECC group used.
///
/// **out_key**
///
/// Pointer to the generated ECDSA private key.
///
/// # Requirements
///
/// Library: libsgx_tcrypto.a
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Some of the pointers are NULL, or the input size is 0.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// Unexpected error occurred during the ECDSA private key generation.
///
pub fn rsgx_calculate_ecdsa_priv_key(
    hash_drg: &[u8],
    sgx_nistp256_r_m1: &[u8],
    out_key: &mut [u8],
) -> SgxError {
    if (hash_drg.is_empty()) || (hash_drg.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (sgx_nistp256_r_m1.is_empty()) || (sgx_nistp256_r_m1.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (out_key.is_empty()) || (out_key.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let ret = unsafe {
        sgx_calculate_ecdsa_priv_key(
            hash_drg.as_ptr(),
            hash_drg.len() as i32,
            sgx_nistp256_r_m1.as_ptr(),
            sgx_nistp256_r_m1.len() as i32,
            out_key.as_mut_ptr(),
            out_key.len() as i32,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

pub fn rsgx_ecc256_calculate_pub_from_priv(
    priv_key: &sgx_ec256_private_t,
    pub_key: &mut sgx_ec256_public_t,
) -> SgxError {
    let ret = unsafe {
        sgx_ecc256_calculate_pub_from_priv(
            priv_key as *const sgx_ec256_private_t,
            pub_key as *mut sgx_ec256_public_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

pub fn rsgx_ecc256_priv_key(
    hash_drg: &[u8],
    sgx_nistp256_r_m1: &[u8],
) -> SgxResult<sgx_ec256_private_t> {
    if (hash_drg.is_empty()) || (hash_drg.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (sgx_nistp256_r_m1.is_empty()) || (sgx_nistp256_r_m1.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut priv_key = sgx_ec256_private_t::default();
    let ret = unsafe {
        sgx_calculate_ecdsa_priv_key(
            hash_drg.as_ptr(),
            hash_drg.len() as i32,
            sgx_nistp256_r_m1.as_ptr(),
            sgx_nistp256_r_m1.len() as i32,
            &mut priv_key as *mut sgx_ec256_private_t as *mut u8,
            mem::size_of::<sgx_ec256_private_t>() as i32,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(priv_key),
        _ => Err(ret),
    }
}

pub fn rsgx_align_ecc256_priv_key(
    hash_drg: &[u8],
    sgx_nistp256_r_m1: &[u8],
) -> SgxResult<sgx_align_ec256_private_t> {
    if (hash_drg.is_empty()) || (hash_drg.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }
    if (sgx_nistp256_r_m1.is_empty()) || (sgx_nistp256_r_m1.len() > i32::MAX as usize) {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
    }

    let mut align_priv_key = sgx_align_ec256_private_t::default();
    let ret = unsafe {
        sgx_calculate_ecdsa_priv_key(
            hash_drg.as_ptr(),
            hash_drg.len() as i32,
            sgx_nistp256_r_m1.as_ptr(),
            sgx_nistp256_r_m1.len() as i32,
            &mut align_priv_key.key as *mut sgx_ec256_private_t as *mut u8,
            mem::size_of::<sgx_ec256_private_t>() as i32,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(align_priv_key),
        _ => Err(ret),
    }
}

pub fn rsgx_ecc256_pub_from_priv(priv_key: &sgx_ec256_private_t) -> SgxResult<sgx_ec256_public_t> {
    let mut pub_key = sgx_ec256_public_t::default();
    let ret = unsafe {
        sgx_ecc256_calculate_pub_from_priv(
            priv_key as *const sgx_ec256_private_t,
            &mut pub_key as *mut sgx_ec256_public_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(pub_key),
        _ => Err(ret),
    }
}

fn rsgx_aes_gcm128_enc_init(
    key: &sgx_aes_gcm_128bit_key_t,
    iv: &[u8],
    aad: &[u8],
    aes_gcm_state: &mut sgx_aes_state_handle_t,
) -> sgx_status_t {
    let iv_len = iv.len();
    if iv_len != SGX_AESGCM_IV_SIZE {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    let aad_len = aad.len();
    if aad_len > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        let p_aad = if !aad.is_empty() {
            aad.as_ptr()
        } else {
            ptr::null()
        };
        sgx_aes_gcm128_enc_init(
            key as *const sgx_aes_gcm_128bit_key_t as *const u8,
            iv.as_ptr(),
            iv_len as u32,
            p_aad,
            aad_len as u32,
            aes_gcm_state as *mut sgx_aes_state_handle_t,
        )
    }
}

fn rsgx_aes_gcm128_enc_update(
    src: &[u8],
    dst: &mut [u8],
    aes_gcm_state: sgx_aes_state_handle_t,
) -> sgx_status_t {
    let src_len = src.len();
    if src_len > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if src_len == 0 {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    let dst_len = dst.len();
    if dst_len > u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    if dst_len == 0 || dst_len < src_len {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    unsafe {
        sgx_aes_gcm128_enc_update(
            src.as_ptr(),
            src_len as u32,
            dst.as_mut_ptr(),
            aes_gcm_state,
        )
    }
}

fn rsgx_aes_gcm128_enc_get_mac(
    mac: &mut sgx_aes_gcm_128bit_tag_t,
    aes_gcm_state: sgx_aes_state_handle_t,
) -> sgx_status_t {
    unsafe {
        sgx_aes_gcm128_enc_get_mac(
            mac as *mut sgx_aes_gcm_128bit_tag_t as *mut u8,
            aes_gcm_state,
        )
    }
}

fn rsgx_aes_gcm_close(aes_gcm_state: sgx_aes_state_handle_t) -> sgx_status_t {
    unsafe { sgx_aes_gcm_close(aes_gcm_state) }
}

pub struct SgxAesHandle {
    handle: RefCell<sgx_aes_state_handle_t>,
    initflag: Cell<bool>,
}

impl SgxAesHandle {
    pub fn new() -> SgxAesHandle {
        SgxAesHandle {
            handle: RefCell::new(ptr::null_mut() as sgx_aes_state_handle_t),
            initflag: Cell::new(false),
        }
    }

    pub fn init(&self, key: &sgx_aes_gcm_128bit_key_t, iv: &[u8], aad: &[u8]) -> SgxError {
        if self.initflag.get() {
            return Ok(());
        }
        let ret = rsgx_aes_gcm128_enc_init(key, iv, aad, self.handle.borrow_mut().deref_mut());
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(true);
                Ok(())
            }
            _ => Err(ret),
        }
    }

    pub fn update(&self, src: &[u8], dst: &mut [u8]) -> SgxError {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }
        let ret = rsgx_aes_gcm128_enc_update(src, dst, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(()),
            _ => Err(ret),
        }
    }

    pub fn get_mac(&self) -> SgxResult<sgx_aes_gcm_128bit_tag_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }
        let mut mac = sgx_aes_gcm_128bit_tag_t::default();
        let ret = rsgx_aes_gcm128_enc_get_mac(&mut mac, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(mac),
            _ => Err(ret),
        }
    }

    pub fn get_align_mac(&self) -> SgxResult<sgx_align_mac_128bit_t> {
        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }
        let mut align_mac = sgx_align_mac_128bit_t::default();
        let ret = rsgx_aes_gcm128_enc_get_mac(&mut align_mac.mac, *self.handle.borrow());
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(align_mac),
            _ => Err(ret),
        }
    }

    pub fn close(&self) -> SgxError {
        if !self.initflag.get() {
            return Ok(());
        }

        let ret = {
            let handle = *self.handle.borrow();
            if handle.is_null() {
                sgx_status_t::SGX_SUCCESS
            } else {
                rsgx_aes_gcm_close(handle)
            }
        };
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                self.initflag.set(false);
                *self.handle.borrow_mut() = ptr::null_mut();
                Ok(())
            }
            _ => Err(ret),
        }
    }
}

impl Default for SgxAesHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SgxAesHandle {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
