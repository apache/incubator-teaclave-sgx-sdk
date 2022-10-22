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

use crate::message::{DhMsg1, DhMsg2, DhMsg3};
use crate::session::{DhResult, Initiator, Responder};
use core::mem;
use core::ptr;
use core::slice;
use sgx_trts::trts::{is_within_enclave, EnclaveRange};
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::types::{
    CDhMsg1, CDhMsg2, CDhMsg3, CDhSession, CEnclaveIdentity, DhSessionRole, Key128bit,
};

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_dh_init_session(
    role: DhSessionRole,
    session: *mut CDhSession,
) -> SgxStatus {
    if session.is_null() {
        return SgxStatus::InvalidParameter;
    }
    if !(*session).is_enclave_range() {
        return SgxStatus::InvalidParameter;
    }

    match role {
        DhSessionRole::Initiator => {
            let initiator = Initiator::new();
            ptr::copy_nonoverlapping(
                &initiator as *const _ as *const u8,
                session as *mut u8,
                mem::size_of::<Initiator>(),
            );
        }
        DhSessionRole::Responder => {
            let responder = Responder::new();
            ptr::copy_nonoverlapping(
                &responder as *const _ as *const u8,
                session as *mut u8,
                mem::size_of::<Responder>(),
            );
        }
    };
    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_dh_responder_gen_msg1(
    msg1: *mut CDhMsg1,
    session: *mut CDhSession,
) -> SgxStatus {
    if session.is_null() || msg1.is_null() {
        return SgxStatus::InvalidParameter;
    }
    if !(*session).is_enclave_range() {
        return SgxStatus::InvalidParameter;
    }

    let c_msg1 = &mut *msg1;
    if !c_msg1.is_enclave_range() {
        return SgxStatus::InvalidParameter;
    }

    let responder = &mut *(session as *mut Responder);
    let dh_msg1 = match responder.generate_msg1() {
        Ok(msg) => msg,
        Err(e) => return e,
    };

    c_msg1.g_a = dh_msg1.pub_key_a.into();
    c_msg1.target = dh_msg1.target;

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_dh_responder_proc_msg2(
    msg2: *const CDhMsg2,
    msg3: *mut CDhMsg3,
    session: *mut CDhSession,
    aek: *mut Key128bit,
    initiator_identity: *mut CEnclaveIdentity,
) -> SgxStatus {
    if session.is_null()
        || msg2.is_null()
        || msg3.is_null()
        || aek.is_null()
        || initiator_identity.is_null()
    {
        return SgxStatus::InvalidParameter;
    }
    if !(*session).is_enclave_range() {
        return SgxStatus::InvalidParameter;
    }

    let c_msg2 = &*msg2;
    let c_msg3 = &mut *msg3;
    let c_aek = &mut *aek;
    let c_initiator_identity = &mut *initiator_identity;
    if !c_msg2.is_enclave_range()
        || !c_aek.is_enclave_range()
        || !c_initiator_identity.is_enclave_range()
    {
        return SgxStatus::InvalidParameter;
    }
    if !is_within_enclave(
        msg3 as *const u8,
        mem::size_of::<CDhMsg3>() + c_msg3.msg_body.add_prop_len as usize,
    ) {
        return SgxStatus::InvalidParameter;
    }
    if c_msg3.msg_body.add_prop_len > u32::MAX - mem::size_of::<CDhMsg3>() as u32 {
        return SgxStatus::InvalidParameter;
    }

    let responder = &mut *(session as *mut Responder);
    let add_prop = if c_msg3.msg_body.add_prop_len > 0 {
        Some(slice::from_raw_parts(
            msg3.add(1) as *const u8,
            c_msg3.msg_body.add_prop_len as usize,
        ))
    } else {
        None
    };

    let dh_msg2: DhMsg2 = c_msg2.into();
    let (dh_msg3, dh_result) = match responder.process_msg2(&dh_msg2, add_prop) {
        Ok(result) => result,
        Err(e) => return e,
    };

    let meg3_vec = dh_msg3.into_bytes();
    ptr::copy_nonoverlapping(meg3_vec.as_ptr(), msg3 as *mut u8, meg3_vec.len());

    *c_aek = dh_result.aek;
    *c_initiator_identity = dh_result.enclave_identity.into();

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_LAv1_initiator_proc_msg1(
    msg1: *const CDhMsg1,
    msg2: *mut CDhMsg2,
    session: *mut CDhSession,
) -> SgxStatus {
    sgx_initiator_proc_msg1(msg1, msg2, session, Initiator::process_msg1_with_lav1)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_LAv2_initiator_proc_msg1(
    msg1: *const CDhMsg1,
    msg2: *mut CDhMsg2,
    session: *mut CDhSession,
) -> SgxStatus {
    sgx_initiator_proc_msg1(msg1, msg2, session, Initiator::process_msg1_with_lav2)
}

/// # Safety
unsafe fn sgx_initiator_proc_msg1<F>(
    msg1: *const CDhMsg1,
    msg2: *mut CDhMsg2,
    session: *mut CDhSession,
    process_msg1: F,
) -> SgxStatus
where
    F: Fn(&mut Initiator, &DhMsg1) -> SgxResult<DhMsg2>,
{
    if session.is_null() || msg1.is_null() || msg2.is_null() {
        return SgxStatus::InvalidParameter;
    }

    if !(*session).is_enclave_range() {
        return SgxStatus::InvalidParameter;
    }

    let c_msg1 = &*msg1;
    let c_msg2 = &mut *msg2;
    if !c_msg1.is_enclave_range() || !c_msg2.is_enclave_range() {
        return SgxStatus::InvalidParameter;
    }

    let initiator = &mut *(session as *mut Initiator);
    let dh_msg1: DhMsg1 = c_msg1.into();
    let dh_msg2 = match process_msg1(initiator, &dh_msg1) {
        Ok(msg) => msg,
        Err(e) => return e,
    };

    *c_msg2 = dh_msg2.into();

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_LAv1_initiator_proc_msg3(
    msg3: *const CDhMsg3,
    session: *mut CDhSession,
    aek: *mut Key128bit,
    responder_identity: *mut CEnclaveIdentity,
) -> SgxStatus {
    sgx_initiator_proc_msg3(
        msg3,
        session,
        aek,
        responder_identity,
        Initiator::process_msg3_with_lav1,
    )
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_LAv2_initiator_proc_msg3(
    msg3: *const CDhMsg3,
    session: *mut CDhSession,
    aek: *mut Key128bit,
    responder_identity: *mut CEnclaveIdentity,
) -> SgxStatus {
    sgx_initiator_proc_msg3(
        msg3,
        session,
        aek,
        responder_identity,
        Initiator::process_msg3_with_lav2,
    )
}

/// # Safety
unsafe fn sgx_initiator_proc_msg3<F>(
    msg3: *const CDhMsg3,
    session: *mut CDhSession,
    aek: *mut Key128bit,
    responder_identity: *mut CEnclaveIdentity,
    process_msg3: F,
) -> SgxStatus
where
    F: Fn(&mut Initiator, &DhMsg3) -> SgxResult<DhResult>,
{
    if session.is_null() || msg3.is_null() || aek.is_null() || responder_identity.is_null() {
        return SgxStatus::InvalidParameter;
    }
    if !(*session).is_enclave_range() {
        return SgxStatus::InvalidParameter;
    }

    let c_msg3 = &*msg3;
    let c_aek = &mut *aek;
    let c_responder_identity = &mut *responder_identity;
    if !c_aek.is_enclave_range() || !c_responder_identity.is_enclave_range() {
        return SgxStatus::InvalidParameter;
    }
    if !is_within_enclave(
        msg3 as *const u8,
        mem::size_of::<CDhMsg3>() + c_msg3.msg_body.add_prop_len as usize,
    ) {
        return SgxStatus::InvalidParameter;
    }

    let initiator = &mut *(session as *mut Initiator);
    let dh_msg3 = match DhMsg3::from_slice(c_msg3.as_ref()) {
        Ok(msg) => msg,
        Err(e) => return e,
    };

    let dh_result = match process_msg3(initiator, &dh_msg3) {
        Ok(result) => result,
        Err(e) => return e,
    };

    *c_aek = dh_result.aek;
    *c_responder_identity = dh_result.enclave_identity.into();

    SgxStatus::Success
}
