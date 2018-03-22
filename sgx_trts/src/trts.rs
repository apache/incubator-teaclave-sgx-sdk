// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use sgx_types::*;
use sgx_types::marker::ContiguousMemory;
use core::mem;

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

    rsgx_raw_is_within_enclave(data as * const _ as * const u8, mem::size_of::<T>())
}

///
/// rsgx_slice_is_within_enclave checks whether a given address is within enclave memory.
///
#[inline]
pub fn rsgx_slice_is_within_enclave<T: Copy + ContiguousMemory>(data: &[T]) -> bool {

    rsgx_raw_is_within_enclave(data.as_ptr() as * const u8, mem::size_of_val(data))
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
pub fn rsgx_raw_is_within_enclave(addr: * const u8, size: usize) -> bool {

    let ret = unsafe { sgx_is_within_enclave(addr as * const c_void, size) };
    if ret == 0 { false } else { true }
}

///
/// rsgx_data_is_outside_enclave checks whether a given address is outside enclave memory.
///
#[inline]
pub fn rsgx_data_is_outside_enclave<T: Copy + ContiguousMemory>(data: &T) -> bool {

    rsgx_raw_is_outside_enclave(data as * const _ as * const u8,  mem::size_of::<T>())
}

///
/// rsgx_slice_is_outside_enclave checks whether a given address is outside enclave memory.
///
#[inline]
pub fn rsgx_slice_is_outside_enclave<T: Copy + ContiguousMemory>(data: &[T]) -> bool {

    rsgx_raw_is_outside_enclave(data.as_ptr() as * const u8, mem::size_of_val(data))
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
pub fn rsgx_raw_is_outside_enclave(addr: * const u8, size: usize) -> bool {

    let ret = unsafe { sgx_is_outside_enclave(addr as * const c_void, size) };
    if ret == 0 { false } else { true }
}

pub fn rsgx_is_enclave_crashed() -> bool {

    let ret = unsafe { sgx_is_enclave_crashed() };
    if ret == 0 { false } else { true }
}

pub type exit_function_t = extern "C" fn();

#[link(name = "sgx_trts")]
extern {
    pub fn abort() -> !;
    pub fn atexit(fun: exit_function_t) -> c_int;
}

pub fn rsgx_abort() -> ! {
    unsafe { abort() }
}

pub fn rsgx_atexit(fun: exit_function_t) -> bool {

    let ret = unsafe { atexit(fun) };
    if ret < 0 {
        false
    } else {
        true
    }
}

#[inline(always)]
pub fn rsgx_lfence() {
    unsafe { asm!{"lfence"}; }
}

#[inline(always)]
pub fn rsgx_sfence() {
    unsafe { asm!{"sfence"}; }
}

#[inline(always)]
pub fn rsgx_mfence() {
    unsafe { asm!{"mfence"}; }
}
