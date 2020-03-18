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
    let ret = unsafe { sgx_cpuid(&mut cpuinfo as *mut sgx_cpuinfo_t, leaf) };
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
    let ret = unsafe { sgx_cpuidex(&mut cpuinfo as *mut sgx_cpuinfo_t, leaf, subleaf) };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(cpuinfo),
        _ => Err(ret),
    }
}
