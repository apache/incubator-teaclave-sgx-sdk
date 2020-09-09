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

//! # Trusted Key Exchange Library
//!
//! The library allow an ISV to exchange secrets between its server and its enclaves. They are used in
//! concert with untrusted Key Exchange functions.
//!

#![no_std]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
use sgx_types::*;

///
/// The rsgx_ra_init function creates a context for the remote attestation and key exchange process.
///
/// # Description
///
/// This is the first API user should call for a key exchange process. The context returned from this
/// function is used as a handle for other APIs in the key exchange library.
///
/// # Parameters
///
/// **p_pub_key**
///
/// The EC public key of the service provider based on the NIST P-256 elliptic curve.
///
/// **b_pse**
///
/// Reserved for backward compatibility.
///
/// # Requirements
///
/// Header: sgx_tkey_exchange.edl
///
/// Library: libsgx_tkey_exchange.a
///
/// # Return value
///
/// The output context for the subsequent remote attestation and key exchange process, to be used in
/// sgx_ra_get_msg1 and sgx_ra_proc_msg2.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Indicates an error that the input parameters are invalid.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation, or contexts reach the limits.
///
/// **SGX_ERROR_AE_SESSION_INVALID**
///
/// The session is invalid or ended by the server.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// Indicates that an unexpected error occurred.
///
pub fn rsgx_ra_init(p_pub_key: &sgx_ec256_public_t, b_pse: i32) -> SgxResult<sgx_ra_context_t> {
    let mut context: sgx_ra_context_t = 0;
    let ret = unsafe {
        sgx_ra_init(
            p_pub_key as *const sgx_ec256_public_t,
            b_pse,
            &mut context as *mut sgx_ra_context_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(context),
        _ => Err(ret),
    }
}

///
/// The rsgx_ra_init_ex function creates a context for the remote attestation and key exchange process
/// while it allows the use of a custom defined Key Derivation Function (KDF).
///
/// # Description
///
/// This is the first API user should call for a key exchange process. The context returned from this
/// function is used as a handle for other APIs in the key exchange library.
///
/// # Parameters
///
/// **p_pub_key**
///
/// The EC public key of the service provider based on the NIST P-256 elliptic curve.
///
/// **b_pse**
///
/// Reserved for backward compatibility.
///
/// **derive_key_cb**
///
/// This a pointer to a call back routine matching the funtion prototype of sgx_ra_derive_secret_keys_t.
/// This function takes the Diffie-Hellman shared secret as input to allow the ISV enclave to generate
/// their own derived shared keys (SMK, SK, MK and VK).
///
/// # Requirements
///
/// Header: sgx_tkey_exchange.edl
///
/// Library: libsgx_tkey_exchange.a
///
/// # Return value
///
/// The output context for the subsequent remote attestation and key exchange process, to be used in
/// sgx_ra_get_msg1 and sgx_ra_proc_msg2.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Indicates an error that the input parameters are invalid.
///
/// **SGX_ERROR_OUT_OF_MEMORY**
///
/// Not enough memory is available to complete this operation, or contexts reach the limits.
///
/// **SGX_ERROR_AE_SESSION_INVALID**
///
/// The session is invalid or ended by the server.
///
/// **SGX_ERROR_UNEXPECTED**
///
/// Indicates that an unexpected error occurred.
///
pub fn rsgx_ra_init_ex(
    p_pub_key: &sgx_ec256_public_t,
    b_pse: i32,
    derive_key_cb: sgx_ra_derive_secret_keys_t,
) -> SgxResult<sgx_ra_context_t> {
    let mut context: sgx_ra_context_t = 0;
    let ret = unsafe {
        sgx_ra_init_ex(
            p_pub_key as *const sgx_ec256_public_t,
            b_pse,
            derive_key_cb,
            &mut context as *mut sgx_ra_context_t,
        )
    };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(context),
        _ => Err(ret),
    }
}

///
/// The sgx_ra_get_keys function is used to get the negotiated keys of a remote attestation and key exchange session.
///
/// This function should only be called after the service provider accepts the remote attestation and key exchange
/// protocol message 3 produced by sgx_ra_proc_msg2.
///
/// # Description
///
/// After a successful key exchange process, this API can be used in the enclave to get specific key associated
/// with this remote attestation and key exchange session.
///
/// # Parameters
///
/// **context**
///
/// Context returned by rsgx_ra_init.
///
/// **keytype**
///
/// The type of the keys, which can be SGX_RA_KEY_MK, SGX_RA_KEY_SK, or SGX_RA_VK.
///
/// # Requirements
///
/// Header: sgx_tkey_exchange.edl
///
/// Library: libsgx_tkey_exchange.a
///
/// # Return value
///
/// The key returned.
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Indicates an error that the input parameters are invalid.
///
/// **SGX_ERROR_INVALID_STATE**
///
/// Indicates this API is invoked in incorrect order, it can be called only after a success session has been established.
/// In other words, sgx_ra_proc_msg2 should have been called and no error returned.
///
pub fn rsgx_ra_get_keys(
    context: sgx_ra_context_t,
    keytype: sgx_ra_key_type_t,
) -> SgxResult<sgx_ra_key_128_t> {
    let mut key = sgx_ra_key_128_t::default();
    let ret = unsafe { sgx_ra_get_keys(context, keytype, &mut key as *mut sgx_ra_key_128_t) };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(key),
        _ => Err(ret),
    }
}

///
/// rsgx_ra_close release context created by rsgx_ra_init or rsgx_ra_init_ex.
///
/// Call the rsgx_ra_close function to release the remote attestation and key exchange context after
/// the process is done and the context isn’t needed anymore.
///
/// # Description
///
/// At the end of a key exchange process, the caller needs to use this API in an enclave to clear and
/// free memory associated with this remote attestation session.
///
/// # Parameters
///
/// **context**
///
/// Context returned by rsgx_ra_init.
///
/// # Requirements
///
/// Header: sgx_tkey_exchange.edl
///
/// Library: libsgx_tkey_exchange.a
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER**
///
/// Indicates an error that the input parameters are invalid.
///
pub fn rsgx_ra_close(context: sgx_ra_context_t) -> SgxError {
    let ret = unsafe { sgx_ra_close(context) };
    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}
