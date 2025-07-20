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

use super::DhResult;
use crate::message::{DhMsg1, DhMsg2, DhMsg3};
use sgx_crypto::ecc::{EcKeyPair, EcPublicKey, EcShareKey};
use sgx_trts::trts::EnclaveRange;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::types::{AlignKey128bit, DhSessionRole};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum SessionState {
    Error,
    #[default]
    WaitMsg1,
    WaitMsg3,
    Active,
}

#[derive(Debug)]
pub struct Initiator {
    role: DhSessionRole,
    state: SessionState,
    pub_key: EcPublicKey,
    peer_pub_key: EcPublicKey,
    shared_key: EcShareKey,
    smk: AlignKey128bit,
}

impl Default for Initiator {
    fn default() -> Initiator {
        Self::new()
    }
}

impl_struct_ContiguousMemory! {
    SessionState;
    Initiator;
}

impl Initiator {
    pub fn new() -> Initiator {
        Initiator {
            role: DhSessionRole::Initiator,
            state: SessionState::WaitMsg1,
            smk: AlignKey128bit::default(),
            pub_key: EcPublicKey::default(),
            peer_pub_key: EcPublicKey::default(),
            shared_key: EcShareKey::default(),
        }
    }

    #[inline]
    pub fn process_msg1_with_lav1(&mut self, msg1: &DhMsg1) -> SgxResult<DhMsg2> {
        self.process_msg1(msg1, DhMsg2::new_with_lav1)
    }

    #[inline]
    pub fn process_msg1_with_lav2(&mut self, msg1: &DhMsg1) -> SgxResult<DhMsg2> {
        self.process_msg1(msg1, DhMsg2::new_with_lav2)
    }

    #[inline]
    pub fn process_msg3_with_lav1(&mut self, msg3: &DhMsg3) -> SgxResult<DhResult> {
        self.process_msg3(msg3, DhMsg3::verify_with_lav1)
    }

    #[inline]
    pub fn process_msg3_with_lav2(&mut self, msg3: &DhMsg3) -> SgxResult<DhResult> {
        self.process_msg3(msg3, DhMsg3::verify_with_lav2)
    }

    fn process_msg1<F>(&mut self, msg1: &DhMsg1, generate_msg2: F) -> SgxResult<DhMsg2>
    where
        F: Fn(&DhMsg1, &EcPublicKey, &AlignKey128bit) -> SgxResult<DhMsg2>,
    {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(msg1.is_enclave_range(), SgxStatus::InvalidParameter);

        self.check(
            (self.role == DhSessionRole::Initiator) && (self.state == SessionState::WaitMsg1),
            SgxStatus::InvalidState,
        )?;

        let mut key_pair = EcKeyPair::create().map_err(|_| {
            self.clear();
            SgxStatus::Unexpected
        })?;

        let mut shared_key = key_pair.shared_key(&msg1.pub_key_a).map_err(|_| {
            self.clear();
            SgxStatus::Unexpected
        })?;

        let mut smk = shared_key.derive_key("SMK".as_bytes()).map_err(|_| {
            self.clear();
            SgxStatus::Unexpected
        })?;

        let pub_key = key_pair.public_key();
        key_pair.clear();

        let msg2 = generate_msg2(msg1, &pub_key, &smk).map_err(|err| {
            self.clear();
            err
        })?;

        self.pub_key = pub_key;
        self.peer_pub_key = msg1.pub_key_a;
        self.shared_key = shared_key;
        self.smk = smk;
        self.state = SessionState::WaitMsg3;

        shared_key.clear();
        smk.as_mut().fill(0);

        Ok(msg2)
    }

    fn process_msg3<F>(&mut self, msg3: &DhMsg3, verify_msg3: F) -> SgxResult<DhResult>
    where
        F: Fn(&DhMsg3, &EcPublicKey, &EcPublicKey, &AlignKey128bit) -> SgxResult,
    {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(msg3.is_enclave_range(), SgxStatus::InvalidParameter);

        self.check(
            (self.role == DhSessionRole::Initiator) && (self.state == SessionState::WaitMsg3),
            SgxStatus::InvalidState,
        )?;

        verify_msg3(msg3, &self.peer_pub_key, &self.pub_key, &self.smk).map_err(|err| {
            self.clear();
            err
        })?;

        let aek = self.shared_key.derive_key("AEK".as_bytes()).map_err(|_| {
            self.clear();
            SgxStatus::Unexpected
        })?;

        self.clear();
        self.state = SessionState::Active;

        Ok(DhResult {
            aek: aek.key,
            enclave_identity: From::from(&msg3.report),
        })
    }

    #[inline]
    fn check(&mut self, cond: bool, err: SgxStatus) -> SgxResult {
        if !cond {
            self.clear();
            return Err(err);
        }
        Ok(())
    }

    #[inline]
    fn clear(&mut self) {
        unsafe { (self as *mut Initiator).write_bytes(0, 1) }
        self.state = SessionState::Error;
    }
}
