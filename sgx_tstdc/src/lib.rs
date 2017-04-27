// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

//!
//! The library is named sgx_tstdc, provides the following functions:
//! 
//! * Mutex
//! * Condition
//! * Query CPUID inside Enclave
//! * Spin lock
//!
#![crate_name = "sgx_tstdc"]
#![crate_type = "rlib"]

#![cfg_attr(not(feature = "use_std"), no_std)]
#![cfg_attr(not(feature = "use_std"), feature(alloc, optin_builtin_traits))]

#![allow(non_camel_case_types)]

#[cfg(feature = "use_std")]
extern crate std as core;

#[cfg(not(feature = "use_std"))]
extern crate alloc;

extern crate sgx_types;
use sgx_types::*;

pub mod mutex;
pub use self::mutex::*;

pub mod cond;
pub use self::cond::*;

///
/// The rsgx_cpuid function performs the equivalent of a cpuid() function call or
/// intrinisic which executes the CPUID instruction to query the host processor for
/// the information about supported features.
///
/// **Note**
///
/// This function performs an OCALL to execute the CPUID instruction.
///
/// # Description
///
/// This function provides the equivalent of the cpuid() function or intrinsic. The
/// function executes the CPUID instruction for the given leaf (input). The CPUID
/// instruction provides processor feature and type information that is returned in
/// cpuinfo, an array of 4 integers to specify the values of EAX, EBX, ECX and EDX
/// registers. rsgx_cpuid performs an OCALL by invoking oc_cpuidex to get the
/// info from untrusted side because the CPUID instruction is an illegal instruction
/// in the enclave domain.
///
/// **Note**
///
/// As the CPUID instruction is executed by an OCALL, the results should not
/// be trusted. Code should verify the results and perform a threat evaluation
/// to determine the impact on trusted code if the results were
/// spoofed.
///
/// The implementation of this function performs an OCALL and therefore,
/// this function will not have the same serializing or fencing behavior of
/// executing a CPUID instruction in an untrusted domain code flow.
///
/// # Parameters
///
/// **leaf**
///
/// The leaf specified for retrieved CPU info.
///
/// # Requirements
///
/// Library: libsgx_tstdc.a
///
/// # Return value
///
/// The information returned in an array of four integers.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Indicates the parameter is invalid.
///
pub fn rsgx_cpuid(leaf: i32) -> SgxResult<sgx_cpuinfo_t> {
    
    let cpuinfo = [0_i32; 4];
    let ret = unsafe { sgx_cpuid(cpuinfo, leaf) };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(cpuinfo),
        _ => Err(ret),
    }
}

///
/// The rsgx_cpuidex function performs the equivalent of a cpuid_ex() function call or
/// intrinisic which executes the CPUID instruction to query the host processor for
/// the information about supported features.
///
/// **Note**
///
/// This function performs an OCALL to execute the CPUID instruction.
///
/// # Description
///
/// This function provides the equivalent of the cpuid_ex() function or intrinsic. The
/// function executes the CPUID instruction for the given leaf (input). The CPUID
/// instruction provides processor feature and type information that is returned in
/// cpuinfo, an array of 4 integers to specify the values of EAX, EBX, ECX and EDX
/// registers. rsgx_cpuidex performs an OCALL by invoking oc_cpuidex to get the
/// info from untrusted side because the CPUID instruction is an illegal instruction
/// in the enclave domain.
///
/// **Note**
///
/// As the CPUID instruction is executed by an OCALL, the results should not
/// be trusted. Code should verify the results and perform a threat evaluation
/// to determine the impact on trusted code if the results were
/// spoofed.
///
/// The implementation of this function performs an OCALL and therefore,
/// this function will not have the same serializing or fencing behavior of
/// executing a CPUID instruction in an untrusted domain code flow.
///
/// # Parameters
///
/// **leaf**
///
/// The leaf specified for retrieved CPU info.
///
/// **subleaf**
///
/// The sub-leaf specified for retrieved CPU info.
///
/// # Requirements
///
/// Library: libsgx_tstdc.a
///
/// # Return value
///
/// The information returned in an array of four integers.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Indicates the parameter is invalid.
///
pub fn rsgx_cpuidex(leaf: i32, subleaf: i32) -> SgxResult<sgx_cpuinfo_t> {

    let cpuinfo = [0_i32; 4];
    let ret = unsafe { sgx_cpuidex(cpuinfo, leaf, subleaf) };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(cpuinfo),
        _ => Err(ret),
    }
}

/// 
/// The rsgx_spin_lock function acquires a spin lock within the enclave.
///
/// # Description
///
/// rsgx_spin_lock modifies the value of the spin lock by using compiler atomic
/// operations. If the lock is not available to be acquired, the thread will always
/// wait on the lock until it can be acquired successfully.
///
/// # Parameters
///
/// **lock**
///
/// The trusted spin lock object to be acquired.
///
/// # Requirements
///
/// Library: libsgx_tstdc.a
///
pub fn rsgx_spin_lock(lock: &mut sgx_spinlock_t) {

    unsafe { sgx_spin_lock(lock as * mut sgx_spinlock_t); }
}

/// 
/// The rsgx_spin_unlock function releases a spin lock within the enclave.
///
/// # Description
///
/// rsgx_spin_unlock resets the value of the spin lock, regardless of its current
/// state. This function simply assigns a value of zero to the lock, which indicates
/// the lock is released.
///
/// # Parameters
///
/// **lock**
///
/// The trusted spin lock object to be released.
///
/// # Requirements
///
/// Library: libsgx_tstdc.a
///
pub fn rsgx_spin_unlock(lock: &mut sgx_spinlock_t) {

    unsafe { sgx_spin_unlock(lock as * mut sgx_spinlock_t); }
}

///
/// The rsgx_thread_self function returns the unique thread identification.
///
/// # Description
///
/// The function is a simple wrap of get_thread_data() provided in the tRTS,
/// which provides a trusted thread unique identifier.
///
/// # Requirements
///
/// Library: libsgx_tstdc.a
///
/// # Return value
///
/// The return value cannot be NULL and is always valid as long as it is invoked by a thread inside the enclave.
///
pub fn rsgx_thread_self() -> sgx_thread_t {
    
    unsafe { sgx_thread_self() }
}

///
/// The rsgx_thread_equal function compares two thread identifiers.
///
/// # Description
///
/// The function compares two thread identifiers provided by sgx_thread_
/// self to determine if the IDs refer to the same trusted thread.
///
/// # Requirements
///
/// Library: libsgx_tstdc.a
///
/// # Return value
///
/// **true**
///
/// The two thread IDs are equal.
///
pub fn rsgx_thread_equal(a: sgx_thread_t, b: sgx_thread_t) -> bool {
    a == b
}
