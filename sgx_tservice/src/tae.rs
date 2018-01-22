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

//! Trust Platform Service Functions
//!
//! The sgx_tservice library provides the following functions that allow an ISV
//! to use platform services and get platform services security property.
//!
use sgx_types::*;
use core::cell::Cell;
use core::cmp::Ordering;
use core::fmt;

///
/// rsgx_create_pse_session creates a session with the PSE.
///
/// # Description
///
/// An Intel(R) SGX enclave first calls rsgx_create_pse_session() in the process to request platform service.
///
/// It's suggested that the caller should wait (typically several seconds to tens of seconds) and retry
/// this API if SGX_ERROR_BUSY is returned.
///
/// # Requirements
///
/// Header: sgx_tae_service.edl
///
/// Library: libsgx_tservice.a
///
/// # Errors
///
/// **SGX_ERROR_SERVICE_UNAVAILABLE**
///
/// The AE service did not respond or the requested service is not supported.
///
/// **SGX_ERROR_SERVICE_TIMEOUT**
///
/// A request to the AE service timed out.
///
/// **SGX_ERROR_BUSY**
///
/// The requested service is temporarily not available.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation.
///
/// **SGX_ERROR_NETWORK_FAILURE**
///
/// Network connecting or proxy setting issue was encountered.
///
/// **SGX_ERROR_OUT_OF_EPC**
///
/// There is not enough EPC memory to load one of the Architecture Enclaves needed to complete this operation.
///
/// **SGX_ERROR_UPDATE_NEEDED**
///
/// Intel(R) SGX needs to be updated.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// Indicates an unexpected error occurred.
///
pub fn rsgx_create_pse_session() -> SgxError {

    let ret = unsafe { sgx_create_pse_session() };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

///
/// rsgx_close_pse_session closes a session created by rsgx_create_pse_ session.
///
/// # Description
///
/// An Intel(R) SGX enclave calls rsgx_close_pse_session() when there is no need to request platform service.
///
/// # Requirements
///
/// Header: sgx_tae_service.edl
///
/// Library: libsgx_tservice.a
///
/// # Errors
///
/// **SGX_ERROR_SERVICE_UNAVAILABLE**
///
/// The AE service did not respond or the requested service is not supported.
///
/// **SGX_ERROR_SERVICE_TIMEOUT**
///
/// A request to the AE service timed out.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// Indicates an unexpected error occurs.
///
pub fn rsgx_close_pse_session() -> SgxError {

    let ret = unsafe { sgx_close_pse_session() };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

///
/// rsgx_get_ps_sec_prop gets a data structure describing the security property of the platform service.
///
/// # Description
///
/// Gets a data structure that describes the security property of the platform service.
///
/// The caller should call rsgx_create_pse_session to establish a session with the platform service enclave
/// before calling this API.
///
/// # Requirements
///
/// Header: sgx_tae_service.edl
///
/// Library: libsgx_tservice.a
///
/// # Return value
///
/// The security property descriptor of the platform service
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Any of the pointers is invalid.
///
/// **SGX_ERROR_AE_SESSION_INVALID**
///
/// Session is not created or has been closed by architectural enclave service.
///
pub fn rsgx_get_ps_sec_prop() -> SgxResult<sgx_ps_sec_prop_desc_t> {

    let mut security_property: sgx_ps_sec_prop_desc_t = Default::default();
    let ret = unsafe { sgx_get_ps_sec_prop(&mut security_property as * mut sgx_ps_sec_prop_desc_t) };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(security_property),
        _ => Err(ret),
    }
}

///
/// rsgx_get_ps_sec_prop_ex gets a data structure describing the security property of the platform service.
///
/// # Description
///
/// Gets a data structure that describes the security property of the platform service.
///
/// The caller should call rsgx_create_pse_session to establish a session with the platform service enclave
/// before calling this API.
///
/// # Requirements
///
/// Header: sgx_tae_service.edl
///
/// Library: libsgx_tservice.a
///
/// # Return value
///
/// The security property descriptor of the platform service
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Any of the pointers is invalid.
///
/// **SGX_ERROR_AE_SESSION_INVALID**
///
/// Session is not created or has been closed by architectural enclave service.
///
pub fn rsgx_get_ps_sec_prop_ex() -> SgxResult<sgx_ps_sec_prop_desc_ex_t> {

    let mut security_property: sgx_ps_sec_prop_desc_ex_t = Default::default();

    let ret = unsafe { sgx_get_ps_sec_prop_ex(&mut security_property as * mut sgx_ps_sec_prop_desc_ex_t) };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(security_property),
        _ => Err(ret),
    }
}

///
/// rsgx_get_trusted_time gets trusted time from the AE service.
///
/// # Description
///
/// current_time contains time in seconds and time_source_nonce contains nonce associate with the time.
/// The caller should compare time_source_nonce against the value returned from the previous call of
/// this API if it needs to calculate the time passed between two readings of the Trusted Timer. If the
/// time_source_nonce of the two readings do not match, the difference between the two readings does not
/// necessarily reflect time passed.
///
/// The caller should call rsgx_create_pse_session to establish a session with the platform service enclave
/// before calling this API.
///
/// # Parameters
///
/// **current_time**
///
/// Trusted Time Stamp in seconds relative to a reference point. The reference point does not change as long as
/// the time_source_nonce has not changed.
///
/// **time_source_nonce**
///
/// A pointer to the buffer that receives the nonce which indicates time source.
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
fn rsgx_get_trusted_time(current_time: &mut sgx_time_t,
                         time_source_nonce: &mut sgx_time_source_nonce_t) -> sgx_status_t {

    unsafe { 
        sgx_get_trusted_time(current_time as * mut sgx_time_t, time_source_nonce as * mut sgx_time_source_nonce_t) 
    }
}

/// timestamp contains time in seconds and source_nonce contains nonce associate with the time.
#[derive(Copy, Clone, Debug, Default)]
pub struct SgxTime {
    timestamp: sgx_time_t,
    source_nonce: sgx_time_source_nonce_t,
}

pub type Duration = sgx_time_t;

pub enum SgxTimeError {
    TimeStamp(Duration),
    TimeSourceChanged,
    SgxStatus(sgx_status_t),
}

impl SgxTimeError {
    pub fn __description(&self) -> &str {
        match *self {
           SgxTimeError::TimeStamp(_) => "other time was not earlier than self",
           SgxTimeError::TimeSourceChanged => "time source is changed",
           SgxTimeError::SgxStatus(ref status) => status.__description(),
        }
    }
}

impl fmt::Display for SgxTimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
           SgxTimeError::TimeStamp(_) => write!(f, "second time provided was later than self"),
           SgxTimeError::TimeSourceChanged => write!(f, "time source does not match"),
           SgxTimeError::SgxStatus(status) => status.fmt(f),
        }
    }
}

impl PartialEq for SgxTime {
    fn eq(&self, other: &SgxTime) -> bool {
        self.timestamp == other.timestamp && self.source_nonce == other.source_nonce
    }
}

impl Eq for SgxTime {}

impl PartialOrd for SgxTime {

    fn partial_cmp(&self, other: &SgxTime) -> Option<Ordering> {
        
        if self.source_nonce == other.source_nonce {
            Some(self.timestamp.cmp(&other.timestamp))
        } else {
            None
        }
    }
}

impl SgxTime {

    pub fn now() -> Result<SgxTime, SgxTimeError> {

        let mut timestamp: sgx_time_t = 0;
        let mut source_nonce: sgx_time_source_nonce_t = Default::default();

        let ret = rsgx_get_trusted_time(&mut timestamp, &mut source_nonce);
        match ret {
            sgx_status_t::SGX_SUCCESS => Ok(SgxTime{ 
                                            timestamp: timestamp, 
                                            source_nonce: source_nonce 
                                         }),
            _ => Err(SgxTimeError::SgxStatus(ret)),
        }
    }

    pub fn duration_since(&self, earlier: &SgxTime) -> Result<Duration, SgxTimeError> {

        if self.source_nonce == earlier.source_nonce {

            if self.timestamp >= earlier.timestamp {
                Ok(self.timestamp - earlier.timestamp)
            } else {
                Err(SgxTimeError::TimeStamp(earlier.timestamp - self.timestamp))
            }
        } else {
            Err(SgxTimeError::TimeSourceChanged)
        }
    }

    pub fn elapsed(&self) -> Result<Duration, SgxTimeError> {

        SgxTime::now().and_then(|t| t.duration_since(self))
    }

    pub fn add_duration(&self, other: Duration) -> Option<SgxTime> {

        self.timestamp.checked_add(other).map(|secs|
            SgxTime{ timestamp: secs, source_nonce: self.source_nonce }
        )
    }

    pub fn sub_duration(&self, other: Duration) -> Option<SgxTime> {

        self.timestamp.checked_sub(other).map(|secs|
            SgxTime{ timestamp: secs, source_nonce: self.source_nonce }
        )
    }

    pub fn get_secs(&self) -> sgx_time_t { self.timestamp }

    pub fn get_source_nonce(&self) -> sgx_time_source_nonce_t { self.source_nonce }
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
                                            counter_uuid: counter_uuid, 
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
                                            counter_uuid: counter_uuid, 
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

        if self.initflag.get() == false {
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

        if self.initflag.get() == false {
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

        if self.initflag.get() == false {
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
