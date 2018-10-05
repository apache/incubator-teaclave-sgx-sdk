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
use core::cell::Cell;

/// Monotonic counter ID
pub struct SgxMonotonicCounter {
    counter_uuid: sgx_mc_uuid_t,
    initflag: Cell<bool>,
}

impl SgxMonotonicCounter {

    ///
    /// creates a monotonic counter with default owner policy and default user attribute mask.
    ///
    /// # Description
    ///
    /// Call new to create a monotonic counter with the default owner policy 0x1, which means enclaves
    /// with same signing key can access the monotonic counter and default owner_attribute_mask 0xFFFFFFFFFFFFFFCB.
    ///
    /// The caller should call rsgx_create_pse_session to establish a session with the platform service enclave
    /// before calling this API.
    ///
    /// Creating a monotonic counter (MC) involves writing to the non-volatile memory available in the platform.
    /// Repeated write operations could cause the memory to wear out during the normal lifecycle of the platform.
    /// Intel(R) SGX prevents this by limiting the rate at which MC operations can be performed. If you exceed
    /// the limit, the MC operation may return SGX_ERROR_BUSY for several minutes.
    ///
    /// Intel(R) SGX limits the number of MCs an enclave can create. To avoid exhausting the available quota,
    /// an SGX application should record the MC UUID that rsgx_create_monotonic_counter returns and destroy a MC
    /// when it is not needed any more. If an enclave reaches its quota and previously created MC UUIDs have not
    /// been recorded, you may restore the MC service after uninstalling the SGX PSW and installing it again.
    /// This procedure deletes all MCs created by any enclave in that system.
    ///
    /// # Parameters
    ///
    /// **counter_value**
    ///
    /// A pointer to the buffer that receives the monotonic counter value.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tae_service.edl
    ///
    /// Library: libsgx_tservice.a
    ///
    /// # Return value
    ///
    /// Monotonic counter ID
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Any of the pointers is invalid.
    ///
    /// **SGX_ERROR_BUSY**
    ///
    /// The requested service is temporarily not available.
    ///
    /// **SGX_ERROR_MC_OVER_QUOTA**
    ///
    /// The enclave has reached the quota of Monotonic Counters it can maintain.
    ///
    /// **SGX_ERROR_MC_USED_UP**
    ///
    /// Monotonic counters are used out.
    ///
    /// **SGX_ERROR_AE_SESSION_INVALID**
    ///
    /// Session is not created or has been closed by architectural enclave service.
    ///
    /// **SGX_ERROR_SERVICE_UNAVAILABLE**
    ///
    /// The AE service did not respond or the requested service is not supported.
    ///
    /// **SGX_ERROR_SERVICE_TIMEOUT**
    ///
    /// A request to the AE service timed out.
    ///
    /// **SGX_ERROR_NETWORK_FAILURE**
    ///
    /// Network connecting or proxy setting issue was encountered.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_OUT_OF_EPC**
    ///
    /// There is not enough EPC memory to load one of the Architecture Enclaves needed to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates an unexpected error occurs.
    ///
    pub fn new(counter_value: &mut u32) -> SgxResult<Self> {

        let mut counter_uuid = sgx_mc_uuid_t::default();
        let ret = rsgx_create_monotonic_counter(&mut counter_uuid, counter_value);

        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(SgxMonotonicCounter{
                                            counter_uuid,
                                            initflag: Cell::new(true),
                                         }),
            _ => Err(ret),
        }
    }

    ///
    /// creates a monotonic counter.
    ///
    /// # Description
    ///
    /// Call new_ex to create a monotonic counter with the given owner_policy and owner_attribute_mask.
    ///
    /// The caller should call rsgx_create_pse_session to establish a session with the platform service enclave
    /// before calling this API.
    ///
    /// Creating a monotonic counter (MC) involves writing to the non-volatile memory available in the platform.
    /// Repeated write operations could cause the memory to wear out during the normal lifecycle of the platform.
    /// Intel(R) SGX prevents this by limiting the rate at which MC operations can be performed. If you exceed
    /// the limit, the MC operation may return SGX_ERROR_BUSY for several minutes.
    ///
    /// Intel(R) SGX limits the number of MCs an enclave can create. To avoid exhausting the available quota,
    /// an SGX application should record the MC UUID that rsgx_create_monotonic_counter_ex returns and destroy a MC
    /// when it is not needed any more. If an enclave reaches its quota and previously created MC UUIDs have not
    /// been recorded, you may restore the MC service after uninstalling the SGX PSW and installing it again.
    /// This procedure deletes all MCs created by any enclave in that system.
    ///
    /// # Parameters
    ///
    /// **owner_policy**
    ///
    /// The owner policy of the monotonic counter.
    ///
    /// * 0x1 means enclave with same signing key can access the monotonic counter
    /// * 0x2 means enclave with same measurement can access the monotonic counter
    /// * 0x3 means enclave with same measurement as well as signing key can access the monotonic counter.
    /// * Owner policy values of 0x0 or any bits set beyond bits 0 and 1 will cause SGX_ERROR_INVALID_PARAMETER
    ///
    /// **owner_attribute_mask**
    ///
    /// Mask of owner attribute, in the format of sgx_attributes_t.
    ///
    /// **counter_value**
    ///
    /// A pointer to the buffer that receives the monotonic counter value.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tae_service.edl
    ///
    /// Library: libsgx_tservice.a
    ///
    /// # Return value
    ///
    /// Monotonic counter ID
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Any of the pointers is invalid.
    ///
    /// **SGX_ERROR_BUSY**
    ///
    /// The requested service is temporarily not available.
    ///
    /// **SGX_ERROR_MC_OVER_QUOTA**
    ///
    /// The enclave has reached the quota of Monotonic Counters it can maintain.
    ///
    /// **SGX_ERROR_MC_USED_UP**
    ///
    /// Monotonic counters are used out.
    ///
    /// **SGX_ERROR_AE_SESSION_INVALID**
    ///
    /// Session is not created or has been closed by architectural enclave service.
    ///
    /// **SGX_ERROR_SERVICE_UNAVAILABLE**
    ///
    /// The AE service did not respond or the requested service is not supported.
    ///
    /// **SGX_ERROR_SERVICE_TIMEOUT**
    ///
    /// A request to the AE service timed out.
    ///
    /// **SGX_ERROR_NETWORK_FAILURE**
    ///
    /// Network connecting or proxy setting issue was encountered.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_OUT_OF_EPC**
    ///
    /// There is not enough EPC memory to load one of the Architecture Enclaves needed to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates an unexpected error occurs.
    ///
    pub fn new_ex(owner_policy: u16, owner_attribute_mask: &sgx_attributes_t, counter_value: &mut u32) -> SgxResult<Self> {

        let mut counter_uuid = sgx_mc_uuid_t::default();
        let ret = rsgx_create_monotonic_counter_ex(owner_policy, owner_attribute_mask, &mut counter_uuid, counter_value);

        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(SgxMonotonicCounter{
                                            counter_uuid,
                                            initflag: Cell::new(true),
                                         }),
            _ => Err(ret),
        }
    }

    ///
    /// destroys a monotonic counter created by new or new_ex.
    ///
    /// # Description
    ///
    /// Calling destory after a monotonic counter is not needed anymore.
    ///
    /// The caller should call rsgx_create_pse_session to establish a session with the platform service enclave
    /// before calling this API.
    ///
    /// destory fails if the calling enclave does not match the owner policy and the attributes specified in the
    /// call that created the monotonic counter.
    ///
    /// Destroying a Monotonic Counter (MC) involves writing to the non-volatile memory available in the platform.
    /// Repeated write operations could cause the memory to wear out during the normal lifecycle of the platform.
    /// Intel(R) SGX prevents this by limiting the rate at which MC operations can be performed. If you exceed the
    /// limit, the MC operation may return SGX_ERROR_BUSY for several minutes.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tae_service.edl
    ///
    /// Library: libsgx_tservice.a
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Any of the pointers is invalid.
    ///
    /// **SGX_ERROR_BUSY**
    ///
    /// The requested service is temporarily not available.
    ///
    /// **SGX_ERROR_MC_NOT_FOUND**
    ///
    /// The Monotonic Counter does not exist or has been invalidated.
    ///
    /// **SGX_ERROR_MC_NO_ACCESS_RIGHT**
    ///
    /// The enclave doesn't have the access right to specified Monotonic Counter.
    ///
    /// **SGX_ERROR_AE_SESSION_INVALID**
    ///
    /// Session is not created or has been closed by architectural enclave service.
    ///
    /// **SGX_ERROR_SERVICE_UNAVAILABLE**
    ///
    /// The AE service did not respond or the requested service is not supported.
    ///
    /// **SGX_ERROR_SERVICE_TIMEOUT**
    ///
    /// A request to the AE service timed out.
    ///
    /// **SGX_ERROR_NETWORK_FAILURE**
    ///
    /// Network connecting or proxy setting issue was encountered.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_OUT_OF_EPC**
    ///
    /// There is not enough EPC memory to load one of the Architecture Enclaves needed to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates an unexpected error occurs.
    ///
    pub fn destory(&self) -> SgxError {

        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_MC_NOT_FOUND);
        }

        let ret = rsgx_destroy_monotonic_counter(&self.counter_uuid);
        if ret == sgx_status_t::SGX_SUCCESS {
            self.initflag.set(false);
            Ok(())
        } else {
            Err(ret)
        }
    }

    ///
    /// increments a monotonic counter value by 1.
    ///
    /// # Description
    ///
    /// Call increment to increase a monotonic counter value by 1.
    ///
    /// The caller should call rsgx_create_pse_session to establish a session with the platform service enclave
    /// before calling this API.
    ///
    /// increment fails if the calling enclave does not match the owner policy and the attributes specified in the
    /// call that created the monotonic counter.
    ///
    /// Incrementing a monotonic counter (MC) involves writing to the non-volatile memory available in the platform.
    /// Repeated write operations could cause the memory to wear out during the normal lifecycle of the platform.
    /// Intel(R) SGX prevents this by limiting the rate at which MC operations can be performed. If you exceed the limit,
    /// the MC operation may return SGX_ERROR_BUSY for several minutes.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tae_service.edl
    ///
    /// Library: libsgx_tservice.a
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Any of the pointers is invalid.
    ///
    /// **SGX_ERROR_BUSY**
    ///
    /// The requested service is temporarily not available.
    ///
    /// **SGX_ERROR_MC_NOT_FOUND**
    ///
    /// The Monotonic Counter does not exist or has been invalidated.
    ///
    /// **SGX_ERROR_MC_NO_ACCESS_RIGHT**
    ///
    /// The enclave doesn't have the access right to specified Monotonic Counter.
    ///
    /// **SGX_ERROR_AE_SESSION_INVALID**
    ///
    /// Session is not created or has been closed by architectural enclave service.
    ///
    /// **SGX_ERROR_SERVICE_UNAVAILABLE**
    ///
    /// The AE service did not respond or the requested service is not supported.
    ///
    /// **SGX_ERROR_SERVICE_TIMEOUT**
    ///
    /// A request to the AE service timed out.
    ///
    /// **SGX_ERROR_NETWORK_FAILURE**
    ///
    /// Network connecting or proxy setting issue was encountered.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_OUT_OF_EPC**
    ///
    /// There is not enough EPC memory to load one of the Architecture Enclaves needed to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates an unexpected error occurs.
    ///
    pub fn increment(&self) -> SgxResult<u32> {

        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_MC_NOT_FOUND);
        }

        let mut counter_value: u32 = 0;
        let ret = rsgx_increment_monotonic_counter(&self.counter_uuid, &mut counter_value);
         match ret {
            sgx_status_t::SGX_SUCCESS => Ok(counter_value),
            _ => Err(ret),
        }
    }

    ///
    /// returns the value of a monotonic counter.
    ///
    /// # Description
    ///
    /// Call read to read the value of a monotonic counter.
    ///
    /// The caller should call rsgx_create_pse_session to establish a session with the platform service enclave
    /// before calling this API.
    ///
    /// read fails if the calling enclave does not match the owner policy and the attributes specified in the
    /// call that created the monotonic counter.
    ///
    /// # Requirements
    ///
    /// Header: sgx_tae_service.edl
    ///
    /// Library: libsgx_tservice.a
    ///
    /// # Return value
    ///
    /// Monotonic counter value
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Any of the pointers is invalid.
    ///
    /// **SGX_ERROR_MC_NOT_FOUND**
    ///
    /// The Monotonic Counter does not exist or has been invalidated.
    ///
    /// **SGX_ERROR_AE_SESSION_INVALID**
    ///
    /// Session is not created or has been closed by architectural enclave service.
    ///
    /// **SGX_ERROR_SERVICE_UNAVAILABLE**
    ///
    /// The AE service did not respond or the requested service is not supported.
    ///
    /// **SGX_ERROR_SERVICE_TIMEOUT**
    ///
    /// A request to the AE service timed out.
    ///
    /// **SGX_ERROR_NETWORK_FAILURE**
    ///
    /// Network connecting or proxy setting issue was encountered.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// Not enough memory is available to complete this operation.
    ///
    /// **SGX_ERROR_OUT_OF_EPC**
    ///
    /// There is not enough EPC memory to load one of the Architecture Enclaves needed to complete this operation.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates an unexpected error occurs.
    ///
    pub fn read(&self) -> SgxResult<u32> {

        if !self.initflag.get() {
            return Err(sgx_status_t::SGX_ERROR_MC_NOT_FOUND);
        }

        let mut counter_value: u32 = 0;
        let ret = rsgx_read_monotonic_counter(&self.counter_uuid, &mut counter_value);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(counter_value),
            _ => Err(ret),
        }
    }
}

impl Drop for SgxMonotonicCounter {
    ///
    /// destroys a monotonic counter created by new or new_ex.
    ///
    fn drop(&mut self) {
        let _ = self.destory();
    }
}

fn rsgx_create_monotonic_counter_ex(owner_policy: u16,
                                    owner_attribute_mask: &sgx_attributes_t,
                                    counter_uuid: &mut sgx_mc_uuid_t,
                                    counter_value: &mut u32) -> sgx_status_t {

    unsafe {
        sgx_create_monotonic_counter_ex(owner_policy,
                                        owner_attribute_mask as * const sgx_attributes_t,
                                        counter_uuid as * mut sgx_mc_uuid_t,
                                        counter_value as * mut u32)
    }
}

fn rsgx_create_monotonic_counter(counter_uuid: &mut sgx_mc_uuid_t, counter_value: &mut u32) -> sgx_status_t {

    unsafe {
        sgx_create_monotonic_counter(counter_uuid as * mut sgx_mc_uuid_t, counter_value as * mut u32)
    }
}

fn rsgx_destroy_monotonic_counter(counter_uuid: &sgx_mc_uuid_t) -> sgx_status_t {

    unsafe {
        sgx_destroy_monotonic_counter(counter_uuid as * const sgx_mc_uuid_t)
    }
}

fn rsgx_increment_monotonic_counter(counter_uuid: &sgx_mc_uuid_t, counter_value: &mut u32) -> sgx_status_t {

    unsafe {
        sgx_increment_monotonic_counter(counter_uuid as * const sgx_mc_uuid_t, counter_value as * mut u32)
    }
}

fn rsgx_read_monotonic_counter(counter_uuid: &sgx_mc_uuid_t, counter_value: &mut u32) -> sgx_status_t {

    unsafe {
        sgx_read_monotonic_counter(counter_uuid as * const sgx_mc_uuid_t, counter_value as * mut u32)
    }
}
