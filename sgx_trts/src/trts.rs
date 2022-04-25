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

use crate::libc;
use core::arch::asm;
use core::mem;
use sgx_types::marker::ContiguousMemory;
use sgx_types::*;

///
/// rsgx_read_rand function is used to generate a random number inside the enclave.
///
/// # Description
///
/// The rsgx_read_rand function is provided to replace the standard pseudo-random sequence generation functions
/// inside the enclave, since these standard functions are not supported in the enclave, such as rand, srand, etc.
/// For HW mode, the function generates a real-random sequence; while in simulation mode, the function generates
/// a pseudo-random sequence.
///
/// # Parameters
///
/// **rand**
///
/// A pointer to the buffer that stores the generated random number. The rand buffer can be either within or outside the enclave,
/// but it is not allowed to be across the enclave boundary or wrapped around.
///
/// # Requirements
///
/// Library: libsgx_trts.a
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Invalid input parameters detected.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// Indicates an unexpected error occurs during the valid random number generation process.
///
pub fn rsgx_read_rand(rand: &mut [u8]) -> SgxError {
    let ret = unsafe { sgx_read_rand(rand.as_mut_ptr(), rand.len()) };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

///
/// rsgx_data_is_within_enclave checks whether a given address is within enclave memory.
///
#[inline]
pub fn rsgx_data_is_within_enclave<T: Copy + ContiguousMemory>(data: &T) -> bool {
    rsgx_raw_is_within_enclave(data as *const _ as *const u8, mem::size_of::<T>())
}

///
/// rsgx_slice_is_within_enclave checks whether a given address is within enclave memory.
///
#[inline]
pub fn rsgx_slice_is_within_enclave<T: Copy + ContiguousMemory>(data: &[T]) -> bool {
    rsgx_raw_is_within_enclave(data.as_ptr() as *const u8, mem::size_of_val(data))
}

///
/// rsgx_raw_is_within_enclave checks whether a given address is within enclave memory.
///
/// The rsgx_raw_is_within_enclave function checks that the buffer located at the pointer addr with its
/// length of size is an address that is strictly within the calling enclave address space.
///
/// # Description
///
/// rsgx_raw_is_within_enclave simply compares the start and end address of the buffer with the calling
/// enclave address space. It does not check the property of the address. Given a function pointer, you
/// sometimes need to confirm whether such a function is within the enclave. In this case, it is recommended
/// to use rsgx_raw_is_within_enclave with a size of 1.
///
/// # Parameters
///
/// **addr**
///
/// The start address of the buffer.
///
/// **size**
///
/// The size of the buffer.
///
/// # Requirements
///
/// Library: libsgx_trts.a
///
/// # Return value
///
/// **true**
///
/// The buffer is strictly within the enclave address space.
///
/// **false**
///
/// The whole buffer or part of the buffer is not within the enclave, or the buffer is wrapped around.
///
pub fn rsgx_raw_is_within_enclave(addr: *const u8, size: usize) -> bool {
    let ret = unsafe { sgx_is_within_enclave(addr as *const c_void, size) };
    ret != 0
}

///
/// rsgx_data_is_outside_enclave checks whether a given address is outside enclave memory.
///
#[inline]
pub fn rsgx_data_is_outside_enclave<T: Copy + ContiguousMemory>(data: &T) -> bool {
    rsgx_raw_is_outside_enclave(data as *const _ as *const u8, mem::size_of::<T>())
}

///
/// rsgx_slice_is_outside_enclave checks whether a given address is outside enclave memory.
///
#[inline]
pub fn rsgx_slice_is_outside_enclave<T: Copy + ContiguousMemory>(data: &[T]) -> bool {
    rsgx_raw_is_outside_enclave(data.as_ptr() as *const u8, mem::size_of_val(data))
}

///
/// rsgx_raw_is_outside_enclave checks whether a given address is outside enclave memory.
///
/// The rsgx_raw_is_outside_enclave function checks that the buffer located at the pointer addr with its
/// length of size is an address that is strictly outside the calling enclave address space.
///
/// # Description
///
/// rsgx_raw_is_outside_enclave simply compares the start and end address of the buffer with the calling
/// enclave address space. It does not check the property of the address.
///
/// # Parameters
///
/// **addr**
///
/// The start address of the buffer.
///
/// **size**
///
/// The size of the buffer.
///
/// # Requirements
///
/// Library: libsgx_trts.a
///
/// # Return value
///
/// **true**
///
/// The buffer is strictly outside the enclave address space.
///
/// **false**
///
/// The whole buffer or part of the buffer is not outside the enclave, or the buffer is wrapped around.
///
pub fn rsgx_raw_is_outside_enclave(addr: *const u8, size: usize) -> bool {
    let ret = unsafe { sgx_is_outside_enclave(addr as *const c_void, size) };
    ret != 0
}

pub fn rsgx_is_enclave_crashed() -> bool {
    let ret = unsafe { sgx_is_enclave_crashed() };
    ret != 0
}

pub use libc::exit_function_t;

pub fn rsgx_abort() -> ! {
    unsafe { libc::abort() }
}

pub fn rsgx_atexit(fun: exit_function_t) -> bool {
    let ret = unsafe { libc::atexit(fun) };
    ret >= 0
}

#[inline(always)]
pub fn rsgx_lfence() {
    unsafe {
        asm! {"lfence"};
    }
}

#[inline(always)]
pub fn rsgx_sfence() {
    unsafe {
        asm! {"sfence"};
    }
}

#[inline(always)]
pub fn rsgx_mfence() {
    unsafe {
        asm! {"mfence"};
    }
}

#[inline]
pub fn rsgx_rdpkru() -> Option<u32> {
    let mut val = 0_u32;
    let ret = unsafe { sgx_rdpkru(&mut val as *mut u32) };

    if ret == 1 {
        Some(val)
    } else {
        None
    }
}

#[inline]
pub fn rsgx_wrpkru(val: u32) -> bool {
    unsafe { sgx_wrpkru(val) == 1 }
}
