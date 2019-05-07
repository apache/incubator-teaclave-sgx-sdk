// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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


