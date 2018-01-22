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

#![crate_name = "enclave1"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
use sgx_types::*;

extern crate attestation;
use attestation::types::*;
use attestation::err::*;
use attestation::func::*;

fn verify_peer_enclave_trust(peer_enclave_identity: &sgx_dh_session_enclave_identity_t )-> u32 {

    if peer_enclave_identity.isv_prod_id != 0 || peer_enclave_identity.attributes.flags & SGX_FLAGS_INITTED == 0 {
        // || peer_enclave_identity->attributes.xfrm !=3)// || peer_enclave_identity->mr_signer != xx //TODO: To be hardcoded with values to check
        ATTESTATION_STATUS::ENCLAVE_TRUST_ERROR as u32
    } else {
        ATTESTATION_STATUS::SUCCESS as u32
    }
}

#[no_mangle]
pub extern "C" fn test_enclave_init() {
    let cb = Callback{
        verify: verify_peer_enclave_trust,
    };
    init(cb);
}

#[no_mangle]
pub extern "C" fn test_create_session(src_enclave_id: sgx_enclave_id_t, dest_enclave_id: sgx_enclave_id_t) -> u32 {
    create_session(src_enclave_id, dest_enclave_id) as u32
}

#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn test_close_session(src_enclave_id: sgx_enclave_id_t, dest_enclave_id: sgx_enclave_id_t) -> u32 {
    close_session(src_enclave_id, dest_enclave_id) as u32
}