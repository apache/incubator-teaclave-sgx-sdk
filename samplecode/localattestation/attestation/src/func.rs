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

//use core::mem;
//use core::sync::atomic::{AtomicPtr, Ordering};

use sgx_types::*;
use sgx_trts::trts::{rsgx_raw_is_outside_enclave, rsgx_lfence};
use sgx_tdh::{SgxDhMsg1, SgxDhMsg2, SgxDhMsg3, SgxDhInitiator, SgxDhResponder};
use std::boxed::Box;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::mem;

use types::*;
use err::*;

extern {
    pub fn session_request_ocall(ret: *mut u32,
                                 src_enclave_id: sgx_enclave_id_t,
                                 dest_enclave_id: sgx_enclave_id_t,
                                 dh_msg1: *mut sgx_dh_msg1_t) -> sgx_status_t;

    pub fn exchange_report_ocall(ret: *mut u32,
                                 src_enclave_id: sgx_enclave_id_t,
                                 dest_enclave_id: sgx_enclave_id_t,
                                 dh_msg2: *mut sgx_dh_msg2_t,
                                 dh_msg3: *mut sgx_dh_msg3_t) -> sgx_status_t;

    pub fn end_session_ocall(ret: *mut u32,
                             src_enclave_id:sgx_enclave_id_t,
                             dest_enclave_id:sgx_enclave_id_t) -> sgx_status_t;
}

static CALLBACK_FN: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());

pub fn init(cb: Callback) {
    let ptr = CALLBACK_FN.load(Ordering::SeqCst);
    if ptr.is_null() {
        let ptr: * mut Callback = Box::into_raw(Box::new(cb));
        CALLBACK_FN.store(ptr as * mut (), Ordering::SeqCst);
    }
}

fn get_callback() -> Option<&'static Callback>{
    let ptr = CALLBACK_FN.load(Ordering::SeqCst) as *mut Callback;
    if ptr.is_null() {
         return None;
    }
    unsafe { Some( &* ptr ) }
}

pub fn create_session(src_enclave_id: sgx_enclave_id_t, dest_enclave_id: sgx_enclave_id_t) -> ATTESTATION_STATUS {

    let mut dh_msg1: SgxDhMsg1 = SgxDhMsg1::default(); //Diffie-Hellman Message 1
    let mut dh_msg2: SgxDhMsg2 = SgxDhMsg2::default(); //Diffie-Hellman Message 2
    let mut dh_aek: sgx_align_key_128bit_t = sgx_align_key_128bit_t::default(); // Session Key
    let mut responder_identity: sgx_dh_session_enclave_identity_t = sgx_dh_session_enclave_identity_t::default();
    let mut ret = 0;

    let mut initiator: SgxDhInitiator = SgxDhInitiator::init_session();

    let status = unsafe { session_request_ocall(&mut ret, src_enclave_id, dest_enclave_id, &mut dh_msg1) };
    if status != sgx_status_t::SGX_SUCCESS {
        return ATTESTATION_STATUS::ATTESTATION_SE_ERROR;
    }
    let err = ATTESTATION_STATUS::from_repr(ret).unwrap();
    if err != ATTESTATION_STATUS::SUCCESS{
        return err;
    }

    let status = initiator.proc_msg1(&dh_msg1, &mut dh_msg2);
    if status.is_err() {
        return ATTESTATION_STATUS::ATTESTATION_ERROR;
    }

    let mut dh_msg3_raw = sgx_dh_msg3_t::default();
    let status = unsafe { exchange_report_ocall(&mut ret, src_enclave_id, dest_enclave_id, &mut dh_msg2, &mut dh_msg3_raw as *mut sgx_dh_msg3_t) };
    if status != sgx_status_t::SGX_SUCCESS {
        return ATTESTATION_STATUS::ATTESTATION_SE_ERROR;
    }
    if ret != ATTESTATION_STATUS::SUCCESS as u32 {
        return ATTESTATION_STATUS::from_repr(ret).unwrap();
    }

    let dh_msg3_raw_len = mem::size_of::<sgx_dh_msg3_t>() as u32 + dh_msg3_raw.msg3_body.additional_prop_length;
    let dh_msg3 = unsafe{ SgxDhMsg3::from_raw_dh_msg3_t(&mut dh_msg3_raw, dh_msg3_raw_len ) };
    if dh_msg3.is_none() {
        return ATTESTATION_STATUS::ATTESTATION_SE_ERROR;
    }
    let dh_msg3 = dh_msg3.unwrap();

    let status = initiator.proc_msg3(&dh_msg3, &mut dh_aek.key, &mut responder_identity);
    if status.is_err() {
        return ATTESTATION_STATUS::ATTESTATION_ERROR;
    }

    let cb = get_callback();
    if cb.is_some() {
        let ret = (cb.unwrap().verify)(&responder_identity);
        if ret != ATTESTATION_STATUS::SUCCESS as u32{
            return ATTESTATION_STATUS::INVALID_SESSION;
        }
    }

    ATTESTATION_STATUS::SUCCESS
}

pub fn close_session(src_enclave_id: sgx_enclave_id_t, dest_enclave_id: sgx_enclave_id_t) -> ATTESTATION_STATUS {
    let mut ret = 0;
    let status = unsafe { end_session_ocall(&mut ret, src_enclave_id, dest_enclave_id) };
    if status != sgx_status_t::SGX_SUCCESS {
        return ATTESTATION_STATUS::ATTESTATION_SE_ERROR;
    }
    ATTESTATION_STATUS::from_repr(ret as u32).unwrap()
}

fn session_request_safe(src_enclave_id: sgx_enclave_id_t, dh_msg1: &mut sgx_dh_msg1_t, session_ptr: &mut usize) -> ATTESTATION_STATUS {

    let mut responder = SgxDhResponder::init_session();

    let status = responder.gen_msg1(dh_msg1);
    if status.is_err() {
        return ATTESTATION_STATUS::INVALID_SESSION;
    }

    let mut session_info = DhSessionInfo::default();
    session_info.enclave_id = src_enclave_id;
    session_info.session.session_status = DhSessionStatus::InProgress(responder);

    let ptr = Box::into_raw(Box::new(session_info));
    *session_ptr = ptr as * mut _ as usize;

    ATTESTATION_STATUS::SUCCESS
}

//Handle the request from Source Enclave for a session
#[no_mangle]
pub extern "C" fn session_request(src_enclave_id: sgx_enclave_id_t, dh_msg1: *mut sgx_dh_msg1_t, session_ptr: *mut usize) -> ATTESTATION_STATUS {
    unsafe {
        session_request_safe(src_enclave_id, &mut *dh_msg1, &mut *session_ptr)
    }
}

#[allow(unused_variables)]
fn exchange_report_safe(src_enclave_id: sgx_enclave_id_t, dh_msg2: &mut sgx_dh_msg2_t , dh_msg3: &mut sgx_dh_msg3_t, session_info: &mut DhSessionInfo) -> ATTESTATION_STATUS {

    let mut dh_aek = sgx_align_key_128bit_t::default() ;   // Session key
    let mut initiator_identity = sgx_dh_session_enclave_identity_t::default();

    let mut responder = match session_info.session.session_status {
        DhSessionStatus::InProgress(res) => {res},
        _ => {
            return ATTESTATION_STATUS::INVALID_SESSION;
        }
    };

    let mut dh_msg3_r = SgxDhMsg3::default();
    let status = responder.proc_msg2(dh_msg2, &mut dh_msg3_r, &mut dh_aek.key, &mut initiator_identity);
    if status.is_err() {
        return ATTESTATION_STATUS::ATTESTATION_ERROR;
    }

    unsafe{ dh_msg3_r.to_raw_dh_msg3_t(dh_msg3, (dh_msg3.msg3_body.additional_prop_length as usize + mem::size_of::<sgx_dh_msg3_t>() ) as u32); };

    let cb = get_callback();
    if cb.is_some() {
        let ret = (cb.unwrap().verify)(&initiator_identity);
        if ret != ATTESTATION_STATUS::SUCCESS as u32 {
            return ATTESTATION_STATUS::INVALID_SESSION;
        }
    }

    session_info.session.session_status = DhSessionStatus::Active(dh_aek);

    ATTESTATION_STATUS::SUCCESS
}
//Verify Message 2, generate Message3 and exchange Message 3 with Source Enclave
#[no_mangle]
pub extern "C" fn exchange_report(src_enclave_id: sgx_enclave_id_t, dh_msg2: *mut sgx_dh_msg2_t , dh_msg3: *mut sgx_dh_msg3_t, session_ptr: *mut usize) -> ATTESTATION_STATUS {

    if rsgx_raw_is_outside_enclave(session_ptr as * const u8, mem::size_of::<DhSessionInfo>()) {
        return ATTESTATION_STATUS::INVALID_PARAMETER;
    }
    rsgx_lfence();

    unsafe {
        exchange_report_safe(src_enclave_id, &mut *dh_msg2, &mut *dh_msg3, &mut *(session_ptr as *mut DhSessionInfo))
    }
}

//Respond to the request from the Source Enclave to close the session
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn end_session(src_enclave_id: sgx_enclave_id_t, session_ptr: *mut usize) -> ATTESTATION_STATUS {

    if rsgx_raw_is_outside_enclave(session_ptr as * const u8, mem::size_of::<DhSessionInfo>()) {
        return ATTESTATION_STATUS::INVALID_PARAMETER;
    }
    rsgx_lfence();

    let _ = unsafe { Box::from_raw(session_ptr as *mut DhSessionInfo) };
    ATTESTATION_STATUS::SUCCESS
}