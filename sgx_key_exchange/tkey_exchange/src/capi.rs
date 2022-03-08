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

use crate::session::Initiator;
use core::mem::{self, ManuallyDrop};
use sgx_trts::trts::is_within_enclave;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::types::{
    AlignKey128bit, Ec256PublicKey, Key128bit, RaContext, RaDriveSecretKeyFn, RaKeyType,
};

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_init(
    sp_pub_key: *const Ec256PublicKey,
    _pse: i32,
    context: *mut RaContext,
) -> SgxStatus {
    if sp_pub_key.is_null() || context.is_null() {
        return SgxStatus::InvalidParameter;
    }

    if !is_within_enclave(sp_pub_key as *const u8, mem::size_of::<Ec256PublicKey>()) {
        return SgxStatus::InvalidParameter;
    }

    let pub_key = &*sp_pub_key;
    let initiator = match Initiator::new(&pub_key.into()) {
        Ok(initiator) => initiator,
        Err(e) => return e,
    };

    *context = initiator.into_raw();
    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_init_ex(
    sp_pub_key: *const Ec256PublicKey,
    _pse: i32,
    derive_key_cb: RaDriveSecretKeyFn,
    context: *mut RaContext,
) -> SgxStatus {
    if sp_pub_key.is_null() || context.is_null() {
        return SgxStatus::InvalidParameter;
    }

    if !is_within_enclave(sp_pub_key as *const u8, mem::size_of::<Ec256PublicKey>()) {
        return SgxStatus::InvalidParameter;
    }
    if !is_within_enclave(derive_key_cb as *const u8, 0) {
        return SgxStatus::InvalidParameter;
    }

    let pub_key = &*sp_pub_key;
    let initiator = match Initiator::new_with_derive_key(
        &pub_key.into(),
        move |share_key,
              kdf_id|
              -> SgxResult<(
            AlignKey128bit,
            AlignKey128bit,
            AlignKey128bit,
            AlignKey128bit,
        )> {
            let mut smk_key = Key128bit::default();
            let mut sk_key = Key128bit::default();
            let mut mk_key = Key128bit::default();
            let mut vk_key = Key128bit::default();

            let status = derive_key_cb(
                share_key,
                kdf_id,
                &mut smk_key,
                &mut sk_key,
                &mut mk_key,
                &mut vk_key,
            );
            let result = if status.is_success() {
                let smk = smk_key.into();
                let sk = sk_key.into();
                let mk = mk_key.into();
                let vk = vk_key.into();

                Ok((smk, sk, mk, vk))
            } else {
                Err(status)
            };

            smk_key.fill(0);
            sk_key.fill(0);
            mk_key.fill(0);
            vk_key.fill(0);
            result
        },
    ) {
        Ok(initiator) => initiator,
        Err(e) => return e,
    };

    *context = initiator.into_raw();
    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_get_keys(
    context: RaContext,
    key_type: RaKeyType,
    key: *mut Key128bit,
) -> SgxStatus {
    if key.is_null() {
        return SgxStatus::InvalidParameter;
    }

    if !is_within_enclave(key as *const u8, mem::size_of::<Key128bit>()) {
        return SgxStatus::InvalidParameter;
    }

    let initiator = ManuallyDrop::new(Initiator::from_raw(context));
    let ra_key = match initiator.get_keys(key_type) {
        Ok(key) => key.key,
        Err(e) => return e,
    };

    *key = ra_key;
    SgxStatus::Success
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_ra_close(context: RaContext) -> SgxStatus {
    let initiator = Initiator::from_raw(context);
    drop(initiator);
    SgxStatus::Success
}
