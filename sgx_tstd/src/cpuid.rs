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

    let mut cpuinfo = [0_i32; 4];
    let ret = unsafe { sgx_cpuid(&mut cpuinfo as * mut sgx_cpuinfo_t, leaf) };
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

    let mut cpuinfo = [0_i32; 4];
    let ret = unsafe { sgx_cpuidex(&mut cpuinfo as * mut sgx_cpuinfo_t, leaf, subleaf) };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(cpuinfo),
        _ => Err(ret),
    }
}