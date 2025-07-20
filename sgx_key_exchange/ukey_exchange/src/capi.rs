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
// under the License.

use crate::Initiator;
use sgx_ra_msg::RaMsg2;
use sgx_types::error::SgxStatus;
use sgx_types::types::{
    AttKeyId, CRaMsg1, CRaMsg2, CRaMsg3, ECallGetGaFn, ECallGetMsg3Fn, ECallProcessMsg2Fn,
    EnclaveId, RaContext,
};
use std::mem::ManuallyDrop;
use std::slice;
use std::sync::LazyLock;
use std::sync::Mutex;

static INITIATOR: LazyLock<Mutex<Initiator>> = LazyLock::new(|| Mutex::new(Initiator::new(0, 0)));

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_get_msg1(
    rctx: RaContext,
    eid: EnclaveId,
    get_ga_fn: ECallGetGaFn,
    msg1: *mut CRaMsg1,
) -> SgxStatus {
    if msg1.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let mut initiator = Initiator::new(eid, rctx);
    let ra_msg1 = match initiator.generate_msg1(None, get_ga_fn) {
        Ok(msg) => msg,
        Err(e) => return e,
    };

    let mut guard = INITIATOR.lock().unwrap();
    guard.set_attkey_id(None);
    guard.set_qe_target(initiator.get_qe_target());
    drop(guard);

    *msg1 = ra_msg1.into();

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_proc_msg2(
    rctx: RaContext,
    eid: EnclaveId,
    process_msg2_fn: ECallProcessMsg2Fn,
    get_msg3_fn: ECallGetMsg3Fn,
    msg2: *const CRaMsg2,
    msg2_size: u32,
    msg3: *mut *mut CRaMsg3,
    msg3_size: *mut u32,
) -> SgxStatus {
    if msg2.is_null() || msg3.is_null() || msg3_size.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let guard = INITIATOR.lock().unwrap();
    if guard.get_attkey_id().is_some() {
        return SgxStatus::InvalidState;
    }
    let qe_target = guard.get_qe_target();
    drop(guard);

    let mut initiator = Initiator::new(eid, rctx);
    initiator.set_attkey_id(None);
    initiator.set_qe_target(qe_target);

    let msg2_slice = slice::from_raw_parts(msg2 as *const u8, msg2_size as usize);
    let ra_msg2 = match RaMsg2::from_slice(msg2_slice) {
        Ok(msg) => msg,
        Err(e) => return e,
    };

    let ra_msg3 = match initiator.process_msg2(&ra_msg2, process_msg2_fn, get_msg3_fn) {
        Ok(msg) => msg,
        Err(e) => return e,
    };

    let raw_msg3_size = match ra_msg3.get_raw_ize() {
        Some(size) => size,
        None => return SgxStatus::Unexpected,
    };

    let mut raw_msg3 = ManuallyDrop::new(vec![0_u8; raw_msg3_size as usize]);
    match ra_msg3.copy_to_slice(raw_msg3.as_mut_slice()) {
        Ok(_) => (),
        Err(e) => return e,
    };

    *msg3 = raw_msg3.as_mut_ptr() as *mut CRaMsg3;
    *msg3_size = raw_msg3_size;

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_get_msg1_ex(
    att_key_id: *const AttKeyId,
    rctx: RaContext,
    eid: EnclaveId,
    get_ga_fn: ECallGetGaFn,
    msg1: *mut CRaMsg1,
) -> SgxStatus {
    if att_key_id.is_null() || msg1.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let mut initiator = Initiator::new(eid, rctx);
    let ra_msg1 = match initiator.generate_msg1(Some(*att_key_id), get_ga_fn) {
        Ok(msg) => msg,
        Err(e) => return e,
    };

    let mut guard = INITIATOR.lock().unwrap();
    guard.set_attkey_id(initiator.get_attkey_id());
    guard.set_qe_target(initiator.get_qe_target());
    drop(guard);

    *msg1 = ra_msg1.into();

    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_proc_msg2_ex(
    att_key_id: *const AttKeyId,
    rctx: RaContext,
    eid: EnclaveId,
    process_msg2_fn: ECallProcessMsg2Fn,
    get_msg3_fn: ECallGetMsg3Fn,
    msg2: *const CRaMsg2,
    msg2_size: u32,
    msg3: *mut *mut CRaMsg3,
    msg3_size: *mut u32,
) -> SgxStatus {
    if att_key_id.is_null() || msg2.is_null() || msg3.is_null() || msg3_size.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let att_key_id = Some(*att_key_id);
    let guard = INITIATOR.lock().unwrap();
    if guard.get_attkey_id() != att_key_id {
        return SgxStatus::InvalidState;
    }
    let qe_target = guard.get_qe_target();
    drop(guard);

    let mut initiator = Initiator::new(eid, rctx);
    initiator.set_attkey_id(att_key_id);
    initiator.set_qe_target(qe_target);

    let msg2_slice = slice::from_raw_parts(msg2 as *const u8, msg2_size as usize);
    let ra_msg2 = match RaMsg2::from_slice(msg2_slice) {
        Ok(msg) => msg,
        Err(e) => return e,
    };

    let ra_msg3 = match initiator.process_msg2(&ra_msg2, process_msg2_fn, get_msg3_fn) {
        Ok(msg) => msg,
        Err(e) => return e,
    };

    let raw_msg3_size = match ra_msg3.get_raw_ize() {
        Some(size) => size,
        None => return SgxStatus::Unexpected,
    };

    let mut raw_msg3 = ManuallyDrop::new(vec![0_u8; raw_msg3_size as usize]);
    match ra_msg3.copy_to_slice(raw_msg3.as_mut_slice()) {
        Ok(_) => (),
        Err(e) => return e,
    };

    *msg3 = raw_msg3.as_mut_ptr() as *mut CRaMsg3;
    *msg3_size = raw_msg3_size;

    SgxStatus::Success
}
